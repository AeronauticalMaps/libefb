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

pub type RunwayId<'a> = Alphanumeric<'a, 5>;

impl<'a> RunwayId<'a> {
    /// Returns the runway's designator.
    ///
    /// # Errors
    ///
    /// Returns an error if the field can not be parsed as number.
    pub fn designator(&self) -> Result<&str, Error> {
        match &self.0[..2] {
            b"RW" => match (&self.0[2], &self.0[3]) {
                // TODO: Improve error handling.
                (b'0'..=b'2', b'0'..=b'9') | (b'3', b'0'..=b'6') => {
                    Ok(str::from_utf8(&self.0[2..]).unwrap_or("").trim())
                }
                _ => Err(Error::InvalidVariant {
                    field: "Runway Identifier",
                    bytes: Vec::from(&self.0[2..]),
                    expected: "two digits in the range from 00 to 36",
                }),
            },
            // there are runways with designator just being N, S, etc.
            _ => Ok(str::from_utf8(self.0).unwrap_or("").trim()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedField;

    use super::*;

    #[test]
    #[should_panic(expected = "InvalidVariant")]
    fn fail_on_invalid_designator() {
        let rwy = RunwayId::from_bytes(b"RW39L".as_slice()).expect("runway should parse");
        rwy.designator().unwrap();
    }

    #[test]
    fn parses_designator_range() {
        let rwy = RunwayId::from_bytes(b"RW36L".as_slice()).expect("runway should parse");
        assert_eq!(rwy.designator(), Ok("36L"));

        let rwy = RunwayId::from_bytes(b"RW29R".as_slice()).expect("runway should parse");
        assert_eq!(rwy.designator(), Ok("29R"));
    }
}
