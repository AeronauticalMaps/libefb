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

use crate::{Error, FixedField, Numeric};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Cycle<'a> {
    year: Numeric<'a, 2>,
    cycle: Numeric<'a, 2>,
}

impl<'a> Cycle<'a> {
    /// The last two digits of the cycle's year.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is not a number.
    pub fn year(&self) -> Result<u8, Error> {
        self.year.as_u8()
    }

    /// The numeric identity of the 28-day data update cycle.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is not a number.
    pub fn cycle(&self) -> Result<u8, Error> {
        self.cycle.as_u8()
    }
}

impl<'a> FixedField<'a> for Cycle<'a> {
    const LENGTH: usize = 4;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        Ok(Self {
            year: Numeric::from_bytes(&bytes[0..2])?,
            cycle: Numeric::from_bytes(&bytes[2..4])?,
        })
    }
}
