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

/// 5.128 Restrictive Airspace Type
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum RestrictiveType {
    Alert,
    Caution,
    Danger,
    LongTermTFR,
    MOA,
    NationalSecurityArea,
    Prohibited,
    Restricted,
    Training,
    Warning,
    UnspecifiedOrUnknown,
}

impl FixedField<'_> for RestrictiveType {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match bytes[0] {
            b'A' => Ok(Self::Alert),
            b'C' => Ok(Self::Caution),
            b'D' => Ok(Self::Danger),
            b'L' => Ok(Self::LongTermTFR),
            b'M' => Ok(Self::MOA),
            b'N' => Ok(Self::NationalSecurityArea),
            b'P' => Ok(Self::Prohibited),
            b'R' => Ok(Self::Restricted),
            b'T' => Ok(Self::Training),
            b'W' => Ok(Self::Warning),
            b'U' => Ok(Self::UnspecifiedOrUnknown),

            // NOTE: The following type is only for EuroNav 7 compatibility and
            //       is NOT defined by ARINC 424!
            b'G' => Ok(Self::Restricted),

            byte => Err(Error::InvalidCharacter {
                field: "Restrictive Airspace Type",
                byte,
                expected: "RESTRICTIVE TYPE according to ARINC 424-23 5.128",
            }),
        }
    }
}
