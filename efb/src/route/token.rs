// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Joe Pearson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Route prompt tokenization.
//!
//! This module implements two-phase parsing of ICAO route strings into semantic tokens.
//! The parsing flow is: **Input String → Lexer → Words → Tokenizer → Tokens**.
//!
//! #  Lexing (Context-Free)
//!
//! The [`Lexer`] performs simple pattern matching to split the input string into [`Word`]
//! variants without any semantic context. Each space-separated element is classified based
//! solely on its format:
//!
//! - `"N0107"` → `WordKind::Speed` (try different parser)
//! - `"EDDH"` → `WordKind::Airport` (found in navigation data)
//! - `"EDDH33"` → `WordKind::Airport` (found after splitting and matching runway)
//! - `"W"` → `WordKind::VFRWaypoint` (not in navigation data)
//! - `"DCT"` → `WordKind::Via(Via::Direct)`
//!
//! # Tokenization (Context-Aware)
//!
//! The tokenizer (`Tokens::tokenize`) converts [`Word`]s into [`Token`]s by
//! resolving semantic meaning using context from the navigation data and
//! surrounding words. This includes resolving VFR waypoints within a terminal
//! area.

use std::ops::Range;
use std::rc::Rc;

use crate::error::Error;
use crate::measurements::Speed;
use crate::nd::*;
use crate::{VerticalDistance, Wind};

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    range: Range<usize>,
    kind: TokenKind,
}

impl Token {
    pub fn range(&self) -> &Range<usize> {
        &self.range
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

/// Semantic token representing a resolved route element.
///
/// Tokens contain fully resolved references to navigation data objects.
/// All context-dependent resolution (e.g., which airport a VFR waypoint belongs to)
/// has been completed during tokenization.
#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    /// True airspeed (TAS) for subsequent legs.
    Speed(Speed),
    /// Flight level or altitude for subsequent legs.
    Level(VerticalDistance),
    /// Wind conditions for subsequent legs.
    Wind(Wind),
    /// Airport with optional runway specification.
    Airport {
        aprt: Rc<Airport>,
        rwy: Option<Runway>,
    },
    /// Navigation aid (waypoint, VOR, NDB, etc.) - but NOT airports.
    NavAid(NavAid),
    /// Route connection type.
    Via(Via),
}

/// Route connection type between waypoints.
#[derive(Debug, Clone, PartialEq)]
pub enum Via {
    /// Direct connection between waypoints.
    Direct,
    // Airway(String),
}

/// Collection of semantic tokens parsed from a route string.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Tokens {
    tokens: Vec<Token>,
}

impl Tokens {
    pub fn try_new(s: &str, nd: &NavigationData) -> Result<Self, Error> {
        let words = Lexer::lex(s, nd)?;
        let tokens = Self::tokenize(words, nd)?;
        Ok(Self { tokens })
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
    }

    fn tokenize(words: Vec<Word>, nd: &NavigationData) -> Result<Vec<Token>, Error> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut terminal: Option<Rc<Airport>> = None;
        let mut i = 0;

        while i < words.len() {
            let kind = match &words[i].kind {
                WordKind::Speed(speed) => Some(TokenKind::Speed(*speed)),
                WordKind::Level(level) => Some(TokenKind::Level(*level)),
                WordKind::Wind(wind) => Some(TokenKind::Wind(*wind)),

                WordKind::Via(via) => {
                    terminal = None;
                    Some(TokenKind::Via(via.clone()))
                }

                WordKind::Airport { aprt, rwy } => {
                    // Each airport sets a new terminal scope
                    terminal = Some(Rc::clone(aprt));

                    if i == 0 {
                        // First airport always gets added
                        Some(TokenKind::Airport {
                            aprt: Rc::clone(aprt),
                            rwy: rwy.clone(),
                        })
                    } else {
                        // If we go direct to this airport (previous is DCT) and
                        // the next word is a terminal waypoint, we don't add
                        // the airport since it is used only to open the
                        // terminal scope.
                        match (words.get(i - 1), words.get(i + 1)) {
                            (
                                Some(Word {
                                    kind: WordKind::Via(Via::Direct),
                                    ..
                                }),
                                Some(Word {
                                    kind: WordKind::VFRWaypoint { .. },
                                    ..
                                }),
                            ) => None,
                            _ => Some(TokenKind::Airport {
                                aprt: Rc::clone(aprt),
                                rwy: rwy.clone(),
                            }),
                        }
                    }
                }

                WordKind::NavAid(navaid) => Some(TokenKind::NavAid(navaid.clone())),

                WordKind::VFRWaypoint { ident, wp } => {
                    // Check for out- or inbound terminal areas and if any of
                    // them includes this point. If we find two different
                    // terminal areas and both have a matching waypoint, the
                    // prompt is ambiguous and can't be resolved!
                    match Self::resolve_in_terminal_areas(
                        terminal.as_ref(),
                        Self::lookahead_terminal_area(&words[i + 1..]).as_ref(),
                        &ident,
                        nd,
                    ) {
                        (Some(wp), None) | (None, Some(wp)) => Some(TokenKind::NavAid(wp)),

                        (Some(NavAid::Waypoint(a)), Some(NavAid::Waypoint(b))) => {
                            // we are in the same terminal area
                            if a == b {
                                Some(TokenKind::NavAid(NavAid::Waypoint(a)))
                            } else {
                                return Err(Error::AmbiguousTerminalArea {
                                    wp: ident.to_string(),
                                    a: a.terminal_area().unwrap_or("ZZZZ").to_string(),
                                    b: b.terminal_area().unwrap_or("ZZZZ").to_string(),
                                });
                            }
                        }

                        _ => {
                            if let Some(wp) = wp {
                                // TODO: VFR enroute waypoints are highly
                                //       ambiguous since they don't belong to
                                //       any terminal areas and can be named
                                //       e.g. WHISKEY. For now we just take the
                                //       first name matching point, but for the
                                //       future we should resolve the point in
                                //       relation to neighboring waypoints.
                                Some(TokenKind::NavAid(NavAid::Waypoint(wp.clone())))
                            } else {
                                return Err(Error::UnexpectedRouteToken(ident.clone()));
                            }
                        }
                    }
                }
            };

            if let Some(kind) = kind {
                tokens.push(Token {
                    range: words[i].range.clone(),
                    kind,
                });
            }

            i += 1;
        }

        Ok(tokens)
    }

    fn resolve_in_terminal_areas(
        current: Option<&Rc<Airport>>,
        next: Option<&Rc<Airport>>,
        ident: &str,
        nd: &NavigationData,
    ) -> (Option<NavAid>, Option<NavAid>) {
        match (current, next) {
            (Some(a), None) => (nd.find_terminal_waypoint(&a.ident(), ident), None),
            (None, Some(b)) => (None, nd.find_terminal_waypoint(&b.ident(), ident)),
            (Some(a), Some(b)) => (
                nd.find_terminal_waypoint(&a.ident(), ident),
                nd.find_terminal_waypoint(&b.ident(), ident),
            ),
            (None, None) => (None, None),
        }
    }

    /// Looks ahead in the word stream to find the next airport.
    fn lookahead_terminal_area(words: &[Word]) -> Option<Rc<Airport>> {
        for word in words {
            match &word.kind {
                WordKind::Airport { aprt, .. } => return Some(aprt.clone()),
                // next direct terminates any terminal area we would be looking in
                WordKind::Via(Via::Direct) => return None,
                _ => continue,
            }
        }
        None
    }
}

impl IntoIterator for Tokens {
    type Item = Token;
    type IntoIter = std::vec::IntoIter<Token>;

    fn into_iter(self) -> Self::IntoIter {
        self.tokens.into_iter()
    }
}

impl<'a> IntoIterator for &'a Tokens {
    type Item = &'a Token;
    type IntoIter = std::slice::Iter<'a, Token>;

    fn into_iter(self) -> Self::IntoIter {
        self.tokens.iter()
    }
}

impl<'a> IntoIterator for &'a mut Tokens {
    type Item = &'a mut Token;
    type IntoIter = std::slice::IterMut<'a, Token>;

    fn into_iter(self) -> Self::IntoIter {
        self.tokens.iter_mut()
    }
}

/////////////////////////////////////////////////////////////////////////////
// Lexer
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
struct Word {
    range: Range<usize>,
    kind: WordKind,
}

#[derive(Debug, Clone, PartialEq)]
enum WordKind {
    Via(Via),
    Speed(Speed),
    Level(VerticalDistance),
    Wind(Wind),
    Airport {
        aprt: Rc<Airport>,
        rwy: Option<Runway>,
    },
    NavAid(NavAid),
    VFRWaypoint {
        ident: String,
        wp: Option<Rc<Waypoint>>,
    },
}

struct Lexer;

impl Lexer {
    fn lex(prompt: &str, nd: &NavigationData) -> Result<Vec<Word>, Error> {
        let upper = prompt.to_uppercase();
        let base = upper.as_ptr() as usize;

        upper
            .split_whitespace()
            .map(|s| {
                let start = s.as_ptr() as usize - base;
                Ok(Word {
                    range: start..start + s.len(),
                    kind: Self::classify(s, nd)?,
                })
            })
            .collect()
    }

    fn classify(s: &str, nd: &NavigationData) -> Result<WordKind, Error> {
        // Check for special keywords first
        if s == "DCT" {
            return Ok(WordKind::Via(Via::Direct));
        }

        // Try navaids or airports
        if let Some(navaid) = nd.find(s) {
            return match navaid {
                NavAid::Waypoint(wp) if wp.usage == WaypointUsage::VFROnly => {
                    Ok(WordKind::VFRWaypoint {
                        ident: wp.fix_ident.clone(),
                        wp: Some(wp),
                    })
                }
                NavAid::Waypoint(_) => Ok(WordKind::NavAid(navaid)),
                NavAid::Airport(aprt) => Ok(WordKind::Airport { aprt, rwy: None }),
            };
        }

        // Try parsing as performance elements
        if let Ok(speed) = s.parse::<Speed>() {
            return Ok(WordKind::Speed(speed));
        }

        if let Ok(level) = s.parse::<VerticalDistance>() {
            return Ok(WordKind::Level(level));
        }

        if let Ok(wind) = s.parse::<Wind>() {
            return Ok(WordKind::Wind(wind));
        }

        // try airport with runway
        if let Some((ident, rwy_designator)) = s.split_at_checked(4) {
            if let Some(NavAid::Airport(aprt)) = nd.find(ident) {
                let rwy = aprt
                    .runways
                    .iter()
                    .find(|rwy| rwy.designator == rwy_designator)
                    .cloned()
                    .ok_or(Error::UnknownRunwayInRoute {
                        aprt: aprt.ident(),
                        rwy: rwy_designator.to_string(),
                    })?;

                return Ok(WordKind::Airport {
                    aprt,
                    rwy: Some(rwy),
                });
            }
        }

        // Fallback: treat as potential VFR waypoint
        Ok(WordKind::VFRWaypoint {
            ident: s.to_string(),
            wp: None,
        })
    }
}

/////////////////////////////////////////////////////////////////////////////
// Unit tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    // - Hamburg     (EDDH) with VRP November 1 & 2
    // - Luebeck     (EDHL) with VRP Whiskey and in close proximity to EDDH
    // - Heringsdorf (EDAH) with VRP Whiskey too
    const ARINC_424_RECORDS: &'static str = r#"SEURP EDDHEDA        0        N N53374900E009591762E002000053                   P    MWGE    HAMBURG                       356462409
SEURPCEDDHED N1    ED0    V     N53482105E010015451                                 WGE           NOVEMBER1                359892409
SEURPCEDDHED N2    ED0    V     N53405701E010000576                                 WGE           NOVEMBER2                359902409
SEURP EDHLEDA        0        N N53481800E010430400E002000055                   P    MWGE    LUBECK-BLANKENSEE             385832513
SEURP EDHLEDGRW07    0068960720 N53480876E010421519                          197                                           141222513
SEURPCEDHLED W     ED0    V     N53495526E010331676                                 WGE           WHISKEY                  380672513
SEURP EDAHEDA        0        N N53524334E014090845E004000094                   P    MWGE    HERINGSDORF                   480342513
SEURPCEDAHED W     ED0    V     N53505381E013552347                                 WGE           WHISKEY                  476562513
"#;

    struct TestData {
        nd: NavigationData,
    }

    impl TestData {
        fn new() -> Self {
            Self {
                nd: NavigationData::try_from_arinc424(ARINC_424_RECORDS)
                    .expect("records should be valid"),
            }
        }

        fn airport(&self, ident: &str) -> Rc<Airport> {
            match self.nd.find(ident) {
                Some(NavAid::Airport(aprt)) => aprt,
                _ => panic!("should find airport {ident}"),
            }
        }

        fn vrp(&self, airport_ident: &str, fix_ident: &str) -> NavAid {
            match self.nd.find_terminal_waypoint(airport_ident, fix_ident) {
                Some(NavAid::Waypoint(wp)) => NavAid::Waypoint(wp),
                _ => panic!("should find visual reporting point {fix_ident} in {airport_ident}"),
            }
        }
    }

    #[test]
    fn lexes_words() {
        let data = TestData::new();
        let words =
            Lexer::lex("N0107 A0250 EDDH D DCT EDHL07", &data.nd).expect("should lex words");

        let edhl = data.airport("EDHL");
        let rwy07 = edhl.runways.iter().find(|r| r.designator == "07").cloned();

        assert_eq!(
            words,
            vec![
                Word {
                    range: 0..5,
                    kind: WordKind::Speed(Speed::kt(107.0)),
                },
                Word {
                    range: 6..11,
                    kind: WordKind::Level(VerticalDistance::Altitude(2500)),
                },
                Word {
                    range: 12..16,
                    kind: WordKind::Airport {
                        aprt: data.airport("EDDH"),
                        rwy: None
                    },
                },
                Word {
                    range: 17..18,
                    kind: WordKind::VFRWaypoint {
                        ident: "D".to_string(),
                        wp: None
                    },
                },
                Word {
                    range: 19..22,
                    kind: WordKind::Via(Via::Direct),
                },
                Word {
                    range: 23..29,
                    kind: WordKind::Airport {
                        aprt: edhl,
                        rwy: rwy07
                    },
                },
            ]
        );
    }

    #[test]
    fn tokenizes_prompt() {
        let data = TestData::new();

        let prompt = "N0107 A0250 EDDH N2 N1 DCT EDHL W DCT W EDAH";
        let tokens: Vec<TokenKind> = Tokens::try_new(prompt, &data.nd)
            .expect("should tokenize prompt")
            .tokens
            .into_iter()
            .map(|token| token.kind)
            .collect();

        assert_eq!(
            tokens,
            vec![
                TokenKind::Speed(Speed::kt(107.0)),
                TokenKind::Level(VerticalDistance::Altitude(2500)),
                TokenKind::Airport {
                    aprt: data.airport("EDDH"),
                    rwy: None
                },
                TokenKind::NavAid(data.vrp("EDDH", "N2")),
                TokenKind::NavAid(data.vrp("EDDH", "N1")),
                TokenKind::Via(Via::Direct),
                // EDHL should be parsed out since it only opens the terminal scope
                TokenKind::NavAid(data.vrp("EDHL", "W")),
                TokenKind::Via(Via::Direct),
                TokenKind::NavAid(data.vrp("EDAH", "W")),
                TokenKind::Airport {
                    aprt: data.airport("EDAH"),
                    rwy: None
                },
            ]
        );
    }

    #[test]
    fn tokenizes_implicit_prompt() {
        let data = TestData::new();

        let prompt = "EDDH N2 N1 W EDHL";
        let tokens: Vec<TokenKind> = Tokens::try_new(prompt, &data.nd)
            .expect("should tokenize prompt")
            .tokens
            .into_iter()
            .map(|token| token.kind)
            .collect();

        assert_eq!(
            tokens,
            vec![
                TokenKind::Airport {
                    aprt: data.airport("EDDH"),
                    rwy: None
                },
                TokenKind::NavAid(data.vrp("EDDH", "N2")),
                TokenKind::NavAid(data.vrp("EDDH", "N1")),
                TokenKind::NavAid(data.vrp("EDHL", "W")),
                TokenKind::Airport {
                    aprt: data.airport("EDHL"),
                    rwy: None
                },
            ]
        );
    }

    #[test]
    #[should_panic(expected = "AmbiguousTerminalArea")]
    fn fails_tokenize_on_ambiguous_prompt() {
        let data = TestData::new();
        let prompt = "EDAH W W EDHL";
        let _ = Tokens::try_new(prompt, &data.nd).unwrap();
    }
}
