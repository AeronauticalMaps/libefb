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

/// 5.19 Level (LEVEL)
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Level {
    AllAltitudes,
    HighLevelAirwaysAltitudes,
    LowLevelAirwaysAltitudes,
}

impl FixedField<'_> for Level {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match bytes[0] {
            b'B' => Ok(Self::AllAltitudes),
            b'H' => Ok(Self::HighLevelAirwaysAltitudes),
            b'L' => Ok(Self::LowLevelAirwaysAltitudes),
            byte => Err(Error::InvalidCharacter {
                field: "Level",
                byte,
                expected: "B, H or L",
            }),
        }
    }
}
