// SPDX-License-Identifier: Apache-2.0
// Copyright 2025, 2026 Joe Pearson
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

//! ICAO Location Indicators according to ICAO Doc 7910.
//!
//! An airport is identified by its four-letter [`LocationIndicator`], e.g.
//! EDDH for Hamburg. Its first letters, the [`TerritoryLetters`], tell in
//! which country or territory the airport is.

use std::fmt;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::Error;

/////////////////////////////////////////////////////////////////////////////
// Location Indicator
/////////////////////////////////////////////////////////////////////////////

/// ICAO location indicator.
///
/// The four letters that identify an airport, e.g. EDDH for Hamburg.
///
/// # Examples
///
/// ```
/// # use efb::nd::LocationIndicator;
/// let eddh = LocationIndicator::new("EDDH").expect("EDDH should be a location indicator");
/// assert_eq!(eddh.as_str(), "EDDH");
/// assert_eq!(eddh.territory().as_str(), "ED");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LocationIndicator([u8; 4]);

impl LocationIndicator {
    /// Creates a location indicator from four letters.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownLocationIndicator`] if `s` isn't four letters
    /// from A to Z.
    pub fn new(s: &str) -> Result<Self, Error> {
        Self::from_str(s)
    }

    /// Returns the country or territory of the location.
    ///
    /// # Examples
    ///
    /// ```
    /// # use efb::nd::LocationIndicator;
    /// let territory = |ident| {
    ///     LocationIndicator::new(ident)
    ///         .expect("ident should be a location indicator")
    ///         .territory()
    /// };
    ///
    /// // the contiguous US has one letter
    /// assert_eq!(territory("KJFK").as_str(), "K");
    ///
    /// // Germany and Alaska have two
    /// assert_eq!(territory("EDDH").as_str(), "ED");
    /// assert_eq!(territory("PAJN").as_str(), "PA");
    /// ```
    pub fn territory(&self) -> TerritoryLetters {
        TerritoryLetters::try_from([self.0[0], self.0[1]])
            .expect("letters of a location indicator should be territory letters")
    }

    /// Returns the location indicator as string.
    pub fn as_str(&self) -> &str {
        // SAFETY: the constructors accept only the letters A to Z.
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl fmt::Display for LocationIndicator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<[u8; 4]> for LocationIndicator {
    type Error = Error;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        if value.iter().all(|b| matches!(b, b'A'..=b'Z')) {
            Ok(Self(value))
        } else {
            Err(Error::UnknownLocationIndicator(format_bytes(&value)))
        }
    }
}

impl FromStr for LocationIndicator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; 4] = s
            .as_bytes()
            .try_into()
            .map_err(|_| Error::UnknownLocationIndicator(s.to_string()))?;
        Self::try_from(bytes)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Territory Letters
/////////////////////////////////////////////////////////////////////////////

/// The letters of a country or territory.
///
/// These are the first one or two letters of a location indicator, e.g.
/// `ED` for Germany.
///
/// # Examples
///
/// ```
/// # use efb::nd::TerritoryLetters;
/// let ed: TerritoryLetters = "ED".parse().expect("ED should be territory letters");
/// assert_eq!(ed.as_str(), "ED");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TerritoryLetters([u8; 2]);

impl TerritoryLetters {
    /// Returns the territory letters as string.
    pub fn as_str(&self) -> &str {
        // SAFETY: the constructor accepts only the letters A to Z and a
        // space after a single letter.
        unsafe { std::str::from_utf8_unchecked(&self.0) }.trim_end()
    }
}

impl fmt::Display for TerritoryLetters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<[u8; 2]> for TerritoryLetters {
    type Error = Error;

    fn try_from([first, second]: [u8; 2]) -> Result<Self, Self::Error> {
        match (first, second) {
            // The following territories have only one letter:
            // - C: Canada
            // - K: USA
            // - Y: Australia
            (b'C' | b'K' | b'Y', _) => Ok(Self([first, b' '])),
            (b'A'..=b'Z', b'A'..=b'Z') => Ok(Self([first, second])),
            _ => Err(Error::UnknownTerritoryLetters(format_bytes(&[
                first, second,
            ]))),
        }
    }
}

impl FromStr for TerritoryLetters {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = match *s.as_bytes() {
            [first] => [first, b' '],
            [first, second] => [first, second],
            _ => return Err(Error::UnknownTerritoryLetters(s.to_string())),
        };
        Self::try_from(bytes)
    }
}

fn format_bytes<const N: usize>(bytes: &[u8; N]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| format!("{bytes:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn territory(ident: &str) -> String {
        LocationIndicator::new(ident)
            .expect("ident should be a location indicator")
            .territory()
            .to_string()
    }

    #[test]
    fn takes_territory_of_location_indicator() {
        assert_eq!(territory("EDDH"), "ED");
        assert_eq!(territory("PAJN"), "PA");
        assert_eq!(territory("KJFK"), "K");
        assert_eq!(territory("CYVR"), "C");
        assert_eq!(territory("YSSY"), "Y");
    }

    #[test]
    fn reduces_us_region_digit_to_territory() {
        let k6 = TerritoryLetters::try_from(*b"K6").expect("K6 should be territory letters");
        assert_eq!(k6.as_str(), "K");
    }

    #[test]
    fn parses_single_territory_letter() {
        let k: TerritoryLetters = "K".parse().expect("K should be territory letters");
        assert_eq!(k.as_str(), "K");
    }

    #[test]
    fn rejects_non_letters_in_location_indicator() {
        for ident in ["ED1H", "eddh", "EDD", "EDDHH", "🛫"] {
            assert!(LocationIndicator::new(ident).is_err(), "{ident}");
        }
    }

    #[test]
    fn rejects_non_letters_in_territory_letters() {
        for letters in ["E", "E1", "ed", "EDD", ""] {
            assert!(letters.parse::<TerritoryLetters>().is_err(), "{letters}");
        }
    }
}
