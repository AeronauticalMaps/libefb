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
//! - `"N0107"` → `Word::Speed` (try different parser)
//! - `"EDDH"` → `Word::Airport` (found in navigation data)
//! - `"EDDH33"` → `Word::Airport` (found after splitting and matching runway)
//! - `"W"` → `Word::VFRWaypoint` (not in navigation data)
//! - `"DCT"` → `Word::Via(Via::Direct)`
//!
//! # Tokenization (Context-Aware)
//!
//! The tokenizer (`Tokens::tokenize`) converts [`Word`]s into [`Token`]s by
//! resolving semantic meaning using context from the navigation data and
//! surrounding words. This includes resolving VFR waypoints within a terminal
//! area.

use std::rc::Rc;

use crate::error::Error;
use crate::measurements::Speed;
use crate::nd::*;
use crate::{VerticalDistance, Wind};

/// Semantic token representing a resolved route element.
///
/// Tokens contain fully resolved references to navigation data objects.
/// All context-dependent resolution (e.g., which airport a VFR waypoint belongs to)
/// has been completed during tokenization.
#[derive(Clone, PartialEq, Debug)]
pub enum Token {
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
            match &words[i] {
                Word::Speed(speed) => tokens.push(Token::Speed(*speed)),
                Word::Level(level) => tokens.push(Token::Level(*level)),
                Word::Wind(wind) => tokens.push(Token::Wind(*wind)),

                Word::Via(via) => {
                    terminal = None;
                    tokens.push(Token::Via(via.clone()));
                }

                Word::Airport { aprt, rwy } => {
                    // Each airport sets a new terminal scope
                    terminal = Some(Rc::clone(aprt));

                    if i == 0 {
                        // First airport always gets added
                        tokens.push(Token::Airport {
                            aprt: Rc::clone(aprt),
                            rwy: rwy.clone(),
                        });
                    } else {
                        // If we go direct to this airport (previous is DCT) and
                        // the next word is a terminal waypoint, we don't add
                        // the airport since it is used only to open the
                        // terminal scope.
                        match (words.get(i - 1), words.get(i + 1)) {
                            (Some(Word::Via(Via::Direct)), Some(Word::VFRWaypoint(_))) => (),
                            _ => tokens.push(Token::Airport {
                                aprt: Rc::clone(aprt),
                                rwy: rwy.clone(),
                            }),
                        };
                    }
                }

                Word::NavAid(navaid) => {
                    // Waypoint, VOR, NDB, or other navaid - just pass through
                    tokens.push(Token::NavAid(navaid.clone()));
                }

                Word::VFRWaypoint(fix) => {
                    // Set the current terminal area scope. There should be only one
                    // explicit terminal area. If we are already in one and we
                    // find another looking ahead, this fix is ambiguous and
                    // can't be resolved! If there is no terminal area at all,
                    // something went wrong too.
                    terminal = match (terminal, Self::lookahead_terminal_area(&words[i + 1..])) {
                        (Some(current_terminal), None) => Ok(current_terminal),
                        (None, Some(next_terminal)) => Ok(next_terminal),
                        (Some(a), Some(b)) => {
                            // we have multiple waypoints in the same terminal area going inbound
                            if a == b {
                                Ok(a)
                            } else {
                                Err(Error::AmbiguousTerminalArea {
                                    wp: fix.clone(),
                                    a: a.ident(),
                                    b: b.ident(),
                                })
                            }
                        }
                        // TODO: This might actually be a valid VFR enroute
                        //       waypoint. We would need to find all points for
                        //       this ident and pick the closest.
                        (None, None) => Err(Error::UnexpectedRouteElement(fix.clone())),
                    }?
                    .into();

                    if let Some(ref terminal) = terminal {
                        // We have a terminal scope - try to resolve as VRP
                        if let Some(navaid) = nd.find_terminal_waypoint(&terminal.ident(), fix) {
                            tokens.push(Token::NavAid(navaid));
                        } else {
                            return Err(Error::UnknownIdent(fix.clone()));
                        }
                    }
                }
            }

            i += 1;
        }

        Ok(tokens)
    }

    /// Looks ahead in the word stream to find the next airport.
    fn lookahead_terminal_area(words: &[Word]) -> Option<Rc<Airport>> {
        for word in words {
            match word {
                Word::Airport { aprt, .. } => return Some(Rc::clone(aprt)),
                // next direct terminates any terminal area we would be looking in
                Word::Via(Via::Direct) => return None,
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
enum Word {
    Via(Via),
    Speed(Speed),
    Level(VerticalDistance),
    Wind(Wind),
    Airport {
        aprt: Rc<Airport>,
        rwy: Option<Runway>,
    },
    NavAid(NavAid),
    VFRWaypoint(String),
}

struct Lexer;

impl Lexer {
    fn lex(s: &str, nd: &NavigationData) -> Result<Vec<Word>, Error> {
        s.to_uppercase()
            .split_whitespace()
            .map(|s| Self::classify(s, nd))
            .collect()
    }

    fn classify(s: &str, nd: &NavigationData) -> Result<Word, Error> {
        // Check for special keywords first
        if s == "DCT" {
            return Ok(Word::Via(Via::Direct));
        }

        // Try navaids or airports
        if let Some(navaid) = nd.find(s) {
            return match navaid {
                NavAid::Waypoint(wp) => match &wp.usage {
                    WaypointUsage::VFROnly => Ok(Word::VFRWaypoint(wp.fix_ident.clone())),
                    _ => Ok(Word::NavAid(NavAid::Waypoint(wp))),
                },
                NavAid::Airport(aprt) => Ok(Word::Airport { aprt, rwy: None }),
            };
        }

        // Try parsing as performance elements
        if let Ok(speed) = s.parse::<Speed>() {
            return Ok(Word::Speed(speed));
        }

        if let Ok(level) = s.parse::<VerticalDistance>() {
            return Ok(Word::Level(level));
        }

        if let Ok(wind) = s.parse::<Wind>() {
            return Ok(Word::Wind(wind));
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

                return Ok(Word::Airport {
                    aprt,
                    rwy: Some(rwy),
                });
            }
        }

        // Fallback: treat as potential VFR waypoint
        Ok(Word::VFRWaypoint(s.to_string()))
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
                Word::Speed(Speed::kt(107.0)),
                Word::Level(VerticalDistance::Altitude(2500)),
                Word::Airport {
                    aprt: data.airport("EDDH"),
                    rwy: None
                },
                Word::VFRWaypoint("D".to_string()),
                Word::Via(Via::Direct),
                Word::Airport {
                    aprt: edhl,
                    rwy: rwy07
                }
            ]
        );
    }

    #[test]
    fn tokenizes_prompt() {
        let data = TestData::new();

        let prompt = "N0107 A0250 EDDH N2 N1 DCT EDHL W DCT W EDAH";
        let tokens = Tokens::try_new(prompt, &data.nd).expect("should tokenize prompt");

        assert_eq!(
            tokens.tokens,
            vec![
                Token::Speed(Speed::kt(107.0)),
                Token::Level(VerticalDistance::Altitude(2500)),
                Token::Airport {
                    aprt: data.airport("EDDH"),
                    rwy: None
                },
                Token::NavAid(data.vrp("EDDH", "N2")),
                Token::NavAid(data.vrp("EDDH", "N1")),
                Token::Via(Via::Direct),
                // EDHL should be parsed out since it only opens the terminal scope
                Token::NavAid(data.vrp("EDHL", "W")),
                Token::Via(Via::Direct),
                Token::NavAid(data.vrp("EDAH", "W")),
                Token::Airport {
                    aprt: data.airport("EDAH"),
                    rwy: None
                },
            ]
        );
    }

    #[test]
    #[should_panic(expected = "AmbiguousTerminalArea")]
    fn fails_tokenize_on_invalid_prompt() {
        let data = TestData::new();
        let prompt = "EDAH W W EDHL";
        let _ = Tokens::try_new(prompt, &data.nd).unwrap();
    }
}
