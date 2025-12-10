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

/// Runway Identifier field (ARINC 424 Spec §5.46).
///
/// Position I (5 characters):
/// - Characters 1-2: "RW" prefix
/// - Characters 3-4: Runway number (01-36)
/// - Character 5: Designator suffix (L, R, C, W, G, U, or space)
pub struct RunwayId<const I: usize> {
    pub designator: String,
}

impl<const I: usize> RunwayId<I> {
    /// The length of the runway identifier field.
    pub const LENGTH: usize = 5;
}

impl<const I: usize> Field for RunwayId<I> {}

impl<const I: usize> FromStr for RunwayId<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("RunwayId", I, Self::LENGTH));
        }

        let suffix = &s[I + 4..I + 5];
        match suffix {
            " " | "C" | "L" | "R" | "W" | "G" | "U" => {
                let designator = s[I + 2..I + 5].trim_end().to_string();
                Ok(Self { designator })
            }
            c => Err(FieldError::unexpected_char(
                "RunwayId.suffix",
                I + 4,
                1,
                "expected L, R, C, W, G, U or space",
            )
            .with_actual(c)),
        }
    }
}
