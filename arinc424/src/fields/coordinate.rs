// SPDX-License-Identifier: Apache-2.0
// Copyright 2024, 2026 Joe Pearson
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

use crate::{Alphanumeric, Error};

pub type Latitude<'a> = Alphanumeric<'a, 9>;

impl<'a> Latitude<'a> {
    /// Returns the latitude as decimal in the range -90.0 (south) to 90.0 (north).
    ///
    /// # Errors
    ///
    /// Returns an error if blank or if the hemisphere is neither `N` nor `S`.
    pub fn as_decimal(&self) -> Result<f64, Error> {
        let hem = self.first();
        let deg = parse_numeric!(2, u8, &self.0[1..3])? as f64;
        let min = parse_numeric!(2, u8, &self.0[3..5])? as f64;
        let sec = parse_numeric!(4, u32, &self.0[5..9])? as f64 / 100.0; // includes centiseconds

        let decimal = deg + min / 60.0 + sec / 3600.0;

        match hem {
            b'N' => Ok(decimal),
            b'S' => Ok(-decimal),
            _ => Err(Error::InvalidCharacter {
                field: "Latitude",
                byte: hem,
                expected: "N or S",
            }),
        }
    }
}

pub type Longitude<'a> = Alphanumeric<'a, 10>;

impl<'a> Longitude<'a> {
    /// Returns the longitude as decimal in the range -180.0 (west) to 180.0 (east).
    ///
    /// # Errors
    ///
    /// Returns an error if blank or if the hemisphere is neither `W` nor `E`.
    pub fn as_decimal(&self) -> Result<f64, Error> {
        let hem = self.first();
        let deg = parse_numeric!(3, u8, &self.0[1..4])? as f64;
        let min = parse_numeric!(2, u8, &self.0[4..6])? as f64;
        let sec = parse_numeric!(4, u32, &self.0[6..10])? as f64 / 100.0; // includes centiseconds

        let decimal = deg + min / 60.0 + sec / 3600.0;

        match hem {
            b'E' => Ok(decimal),
            b'W' => Ok(-decimal),
            _ => Err(Error::InvalidCharacter {
                field: "Longitude",
                byte: hem,
                expected: "E or W",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedField;

    use super::*;

    #[test]
    fn parses_latitude() {
        let lat = Latitude::from_bytes(b"N40394857").expect("latitude should parse");
        assert_eq!(lat.as_decimal(), Ok(40.663491666666665));
    }

    #[test]
    fn parses_longitude() {
        let long = Longitude::from_bytes(b"W0741444230").expect("longitude should parse");
        assert_eq!(long.as_decimal(), Ok(-74.24561944444444));
    }
}
