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

/// AIRAC Cycle field (ARINC 424 Spec §5.31).
///
/// Position: 128-132 (4 characters)
/// Format: YY + CC (2-digit year + 2-digit cycle number)
#[derive(Debug, PartialEq)]
pub struct Cycle {
    pub year: u8,
    pub cycle: u8,
}

impl Cycle {
    /// The starting position of the cycle field.
    pub const POSITION: usize = 128;
    /// The length of the cycle field.
    pub const LENGTH: usize = 4;
}

impl Field for Cycle {}

impl FromStr for Cycle {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < Self::POSITION + Self::LENGTH {
            return Err(FieldError::invalid_length(
                "Cycle",
                Self::POSITION,
                Self::LENGTH,
            ));
        }

        let year_slice = &s[128..130];
        let cycle_slice = &s[130..132];

        let year: u8 = year_slice
            .parse()
            .map_err(|_| FieldError::not_a_number("Cycle.year", 128, 2).with_actual(year_slice))?;

        let cycle: u8 = cycle_slice.parse().map_err(|_| {
            FieldError::not_a_number("Cycle.cycle", 130, 2).with_actual(cycle_slice)
        })?;

        Ok(Self { year, cycle })
    }
}
