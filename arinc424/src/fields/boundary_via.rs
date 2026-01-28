// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Joe Pearson
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

/// 5.118 Boundary Via
///
/// The "Boundary VIA" field defines the path of the boundary from the position
/// identified in the record to the next defined position.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct BoundaryVia {
    /// The boundary path type.
    pub path: BoundaryPath,
    /// Whether this is the last record in the boundary description.
    pub return_to_origin: bool,
}

/// The path type for a boundary segment.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BoundaryPath {
    /// Circle defined by center point and radius.
    Circle,
    /// Great circle path to next point.
    GreatCircle,
    /// Rhumb line (constant bearing) to next point.
    RhumbLine,
    /// Counter-clockwise arc around arc origin.
    CounterClockwiseArc,
    /// Clockwise arc around arc origin.
    ClockwiseArc,
}

impl FixedField<'_> for BoundaryVia {
    const LENGTH: usize = 2;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        let path = match bytes[0] {
            b'C' => BoundaryPath::Circle,
            b'G' => BoundaryPath::GreatCircle,
            b'H' => BoundaryPath::RhumbLine,
            b'L' => BoundaryPath::CounterClockwiseArc,
            b'R' => BoundaryPath::ClockwiseArc,
            byte => {
                return Err(Error::InvalidCharacter {
                    field: "Boundary Via",
                    byte,
                    expected: "C, G, H, L or R",
                })
            }
        };

        let return_to_origin = bytes[1] == b'E';

        Ok(BoundaryVia {
            path,
            return_to_origin,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clockwise_arc() {
        let via = BoundaryVia::from_bytes(b"R ").expect("should parse");
        assert_eq!(via.path, BoundaryPath::ClockwiseArc);
        assert!(!via.return_to_origin);
    }

    #[test]
    fn parses_with_end_indicator() {
        let via = BoundaryVia::from_bytes(b"GE").expect("should parse");
        assert_eq!(via.path, BoundaryPath::GreatCircle);
        assert!(via.return_to_origin);
    }
}
