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

/// 5.213 Controlled Airspace Type (ARSP TYPE)
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ArspType {
    ClassC,
    ControlArea,
    TerminalControlArea,
    RadarZone,
    ClassB,
    RadioMandatoryZone,
    TransponderMandatoryZone,
    ControlZone,
}

impl FixedField<'_> for ArspType {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match bytes[0] {
            b'A' => Ok(Self::ClassC),
            b'C' => Ok(Self::ControlArea),
            b'M' => Ok(Self::TerminalControlArea),
            b'R' => Ok(Self::RadarZone),
            b'T' => Ok(Self::ClassB),
            b'U' => Ok(Self::RadioMandatoryZone),
            b'V' => Ok(Self::TransponderMandatoryZone),
            b'Z' => Ok(Self::ControlZone),
            byte => Err(Error::InvalidCharacter {
                field: "Controlled Airspace Type",
                byte,
                expected: "ARSP TYPE according to ARINC 424-23 5.213",
            }),
        }
    }
}
