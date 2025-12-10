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

/// Waypoint Usage field (ARINC 424 Spec §5.82).
///
/// Position: 29-31 (2 characters)
/// Indicates the usage classification of the waypoint.
#[derive(Debug, PartialEq)]
pub enum WaypointUsage {
    HiLoAltitude,
    HiAltitude,
    LoAltitude,
    TerminalOnly,
    RNAV,
}

impl WaypointUsage {
    /// The position of the waypoint usage field.
    pub const POSITION: usize = 29;
    /// The length of the waypoint usage field.
    pub const LENGTH: usize = 2;
}

impl Field for WaypointUsage {}

impl FromStr for WaypointUsage {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < Self::POSITION + Self::LENGTH {
            return Err(FieldError::invalid_length(
                "WaypointUsage",
                Self::POSITION,
                Self::LENGTH,
            ));
        }

        let usage = &s[29..31];
        match usage {
            " B" => Ok(Self::HiLoAltitude),
            " H" => Ok(Self::HiAltitude),
            " L" => Ok(Self::LoAltitude),
            "  " => Ok(Self::TerminalOnly),
            "R " => Ok(Self::RNAV),
            c => Err(FieldError::invalid_value(
                "WaypointUsage",
                Self::POSITION,
                Self::LENGTH,
                "unknown waypoint usage",
            )
            .with_actual(c)),
        }
    }
}
