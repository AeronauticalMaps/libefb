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

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MagTrueInd {
    Magnetic,
    TrueNorth,
    Mixed,
}

impl FixedField<'_> for MagTrueInd {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match bytes[0] {
            b'M' => Ok(Self::Magnetic),
            b'T' => Ok(Self::TrueNorth),
            b' ' => Ok(Self::Mixed),
            byte => Err(Error::InvalidCharacter {
                field: "Magnetic/True Indicator",
                byte,
                expected: "M, T or blank",
            }),
        }
    }
}
