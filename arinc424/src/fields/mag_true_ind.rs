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
use std::str::FromStr;

/// Magnetic/True Indicator field (ARINC 424 Spec §5.165).
///
/// Position I (1 character):
/// - "M": Magnetic bearings
/// - "T": True bearings
/// - " ": Mixed bearings
#[derive(Debug, PartialEq)]
pub enum MagTrueInd<const I: usize> {
    Magnetic,
    TrueNorth,
    Mixed,
}

impl<const I: usize> MagTrueInd<I> {
    /// The length of the magnetic/true indicator field.
    pub const LENGTH: usize = 1;
}

impl<const I: usize> Field for MagTrueInd<I> {}

impl<const I: usize> FromStr for MagTrueInd<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("MagTrueInd", I, Self::LENGTH));
        }

        let indicator = &s[I..I + 1];
        match indicator {
            "M" => Ok(Self::Magnetic),
            "T" => Ok(Self::TrueNorth),
            " " => Ok(Self::Mixed),
            c => Err(FieldError::unexpected_char(
                "MagTrueInd",
                I,
                Self::LENGTH,
                "expected M, T or space",
            )
            .with_actual(c)),
        }
    }
}
