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

use crate::{Error, FixedField};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum MagVar {
    /// The variation is east of true north.
    East(f32),
    /// The variation is west of true north.
    West(f32),
    /// The point is oriented to true north.
    OrientedToTrueNorth,
}

impl FixedField<'_> for MagVar {
    const LENGTH: usize = 5;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        let code = bytes[0];
        let deg = || -> Result<f32, Error> {
            Ok(parse_numeric!(4, u32, &bytes[1..5])? as f32 / 100.0) // includes centidegree;
        };

        match code {
            b'E' => Ok(Self::East(deg()?)),
            b'W' => Ok(Self::West(deg()?)),
            b'T' => Ok(Self::OrientedToTrueNorth),
            _ => Err(Error::InvalidCharacter {
                field: "Magnetic Variation",
                byte: code,
                expected: "E, W or T as variation direction",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mag_var() {
        let mag_var = MagVar::from_bytes(b"E0140");
        assert_eq!(mag_var, Ok(MagVar::East(1.4)));

        let mag_var = MagVar::from_bytes(b"W0410");
        assert_eq!(mag_var, Ok(MagVar::West(4.1)));

        let mag_var = MagVar::from_bytes(b"T0000");
        assert_eq!(mag_var, Ok(MagVar::OrientedToTrueNorth));
    }
}
