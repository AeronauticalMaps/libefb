// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Joe Pearson
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

use std::str::FromStr;

use super::{Field, FieldError};

/// Runway Bearing field (ARINC 424 Spec §5.51).
///
/// Position I (4 characters):
/// - Characters 1-3: Bearing in degrees (000-360)
/// - Character 4: "T" for True North, or digit for tenths (Magnetic North)
#[derive(Debug, PartialEq)]
pub enum RwyBrg<const I: usize> {
    MagneticNorth(f32),
    TrueNorth(u32),
}

impl<const I: usize> RwyBrg<I> {
    /// The length of the runway bearing field.
    pub const LENGTH: usize = 4;
}

impl<const I: usize> Field for RwyBrg<I> {}

impl<const I: usize> FromStr for RwyBrg<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("RwyBrg", I, Self::LENGTH));
        }

        let degree_slice = &s[I..I + 3];
        let fourth_char = &s[I + 3..I + 4];

        match fourth_char {
            "T" => {
                let degree = degree_slice.parse::<u32>().map_err(|_| {
                    FieldError::not_a_number("RwyBrg.degree", I, 3).with_actual(degree_slice)
                })?;

                Ok(Self::TrueNorth(degree))
            }
            _ => {
                let degree = degree_slice.parse::<u32>().map_err(|_| {
                    FieldError::not_a_number("RwyBrg.degree", I, 3).with_actual(degree_slice)
                })?;

                let decimal = fourth_char.parse::<u32>().map_err(|_| {
                    FieldError::not_a_number("RwyBrg.decimal", I + 3, 1).with_actual(fourth_char)
                })?;

                Ok(Self::MagneticNorth(degree as f32 + decimal as f32 / 10.0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_true_north() {
        assert_eq!("347T".parse::<RwyBrg<0>>(), Ok(RwyBrg::TrueNorth(347)));
    }

    #[test]
    fn parse_magnetic_north() {
        assert_eq!(
            "2302".parse::<RwyBrg<0>>(),
            Ok(RwyBrg::MagneticNorth(230.2))
        );
    }
}
