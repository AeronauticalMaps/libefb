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

/// 5.340 Unmanned Aerial Vhicle (UAV) Only
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UAV(bool);

impl FixedField<'_> for UAV {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match bytes[0] {
            b'Y' => Ok(Self(true)),
            b' ' => Ok(Self(false)),
            byte => Err(Error::InvalidCharacter {
                field: "UAV",
                byte,
                expected: "Y or blank",
            }),
        }
    }
}

impl From<UAV> for bool {
    fn from(value: UAV) -> Self {
        value.0
    }
}
