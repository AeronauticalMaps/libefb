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
pub enum WaypointUsage {
    HiLoAltitude,
    HiAltitude,
    LoAltitude,
    TerminalOnly,
}

impl FixedField<'_> for WaypointUsage {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match &bytes[0] {
            b'B' => Ok(Self::HiLoAltitude),
            b'H' => Ok(Self::HiAltitude),
            b'L' => Ok(Self::LoAltitude),
            b' ' => Ok(Self::TerminalOnly),
            _ => Err(Error::InvalidVariant {
                field: "Waypoint Usage",
                bytes: Vec::from(bytes),
                expected: "according to ARINC 424-17 5.82",
            }),
        }
    }
}
