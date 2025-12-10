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

use super::{Field, FieldError};
use std::ops::RangeInclusive;
use std::str::FromStr;

/// Parses a numeric field at a specific position with range validation.
fn parse_numeric_field(
    s: &str,
    field_name: &'static str,
    idx: usize,
    len: usize,
    range: RangeInclusive<u8>,
) -> Result<u8, FieldError> {
    if s.len() < idx + len {
        return Err(FieldError::invalid_length(field_name, idx, len));
    }
    let slice = &s[idx..idx + len];
    slice
        .parse()
        .map_err(|_| FieldError::not_a_number(field_name, idx, len).with_actual(slice))
        .and_then(|v| {
            range
                .contains(&v)
                .then_some(v)
                .ok_or_else(|| {
                    FieldError::number_out_of_range(field_name, idx, len).with_actual(slice)
                })
        })
}

#[derive(Debug, PartialEq)]
pub enum CardinalDirection {
    North,
    South,
    East,
    West,
}

/// Latitude field (ARINC 424 Spec §5.36).
///
/// Format: N/S + DD + MM + SS + cc (9 characters total)
/// - Position I: Cardinal direction (N or S)
/// - Position I+1..I+3: Degrees (00-90)
/// - Position I+3..I+5: Minutes (00-60)
/// - Position I+5..I+7: Seconds (00-60)
/// - Position I+7..I+9: Centiseconds (00-99)
#[derive(Debug, PartialEq)]
pub struct Latitude<const I: usize> {
    pub cardinal: CardinalDirection,
    pub degree: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub centiseconds: u8,
}

impl<const I: usize> Latitude<I> {
    /// The length of a latitude field in characters.
    pub const LENGTH: usize = 9;
}

impl<const I: usize> Field for Latitude<I> {}

impl<const I: usize> FromStr for Latitude<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("Latitude", I, Self::LENGTH));
        }

        let cardinal = match &s[I..I + 1] {
            "N" => Ok(CardinalDirection::North),
            "S" => Ok(CardinalDirection::South),
            c => Err(FieldError::unexpected_char(
                "Latitude",
                I,
                1,
                "expected N or S cardinal direction",
            )
            .with_actual(c)),
        }?;

        let degree = parse_numeric_field(s, "Latitude.degree", I + 1, 2, 0..=90)?;
        let minutes = parse_numeric_field(s, "Latitude.minutes", I + 3, 2, 0..=60)?;
        let seconds = parse_numeric_field(s, "Latitude.seconds", I + 5, 2, 0..=60)?;
        let centiseconds = parse_numeric_field(s, "Latitude.centiseconds", I + 7, 2, 0..=99)?;

        if degree == 90 && (minutes > 0 || seconds > 0 || centiseconds > 0) {
            Err(FieldError::number_out_of_range("Latitude", I, Self::LENGTH)
                .with_actual(&s[I..I + Self::LENGTH]))
        } else {
            Ok(Self {
                cardinal,
                degree,
                minutes,
                seconds,
                centiseconds,
            })
        }
    }
}

/// Longitude field (ARINC 424 Spec §5.37).
///
/// Format: E/W + DDD + MM + SS + cc (10 characters total)
/// - Position I: Cardinal direction (E or W)
/// - Position I+1..I+4: Degrees (000-180)
/// - Position I+4..I+6: Minutes (00-60)
/// - Position I+6..I+8: Seconds (00-60)
/// - Position I+8..I+10: Centiseconds (00-99)
#[derive(Debug, PartialEq)]
pub struct Longitude<const I: usize> {
    pub cardinal: CardinalDirection,
    pub degree: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub centiseconds: u8,
}

impl<const I: usize> Longitude<I> {
    /// The length of a longitude field in characters.
    pub const LENGTH: usize = 10;
}

impl<const I: usize> Field for Longitude<I> {}

impl<const I: usize> FromStr for Longitude<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("Longitude", I, Self::LENGTH));
        }

        let cardinal = match &s[I..I + 1] {
            "W" => Ok(CardinalDirection::West),
            "E" => Ok(CardinalDirection::East),
            c => Err(FieldError::unexpected_char(
                "Longitude",
                I,
                1,
                "expected E or W cardinal direction",
            )
            .with_actual(c)),
        }?;

        let degree = parse_numeric_field(s, "Longitude.degree", I + 1, 3, 0..=180)?;
        let minutes = parse_numeric_field(s, "Longitude.minutes", I + 4, 2, 0..=60)?;
        let seconds = parse_numeric_field(s, "Longitude.seconds", I + 6, 2, 0..=60)?;
        let centiseconds = parse_numeric_field(s, "Longitude.centiseconds", I + 8, 2, 0..=99)?;

        if degree == 180 && (minutes > 0 || seconds > 0 || centiseconds > 0) {
            Err(
                FieldError::number_out_of_range("Longitude", I, Self::LENGTH)
                    .with_actual(&s[I..I + Self::LENGTH]),
            )
        } else {
            Ok(Self {
                cardinal,
                degree,
                minutes,
                seconds,
                centiseconds,
            })
        }
    }
}
