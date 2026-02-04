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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::*;
use geo::Point;

pub type Waypoints = Vec<Waypoint>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WaypointUsage {
    VFROnly,
    Unknown,
}

/// The region where the waypoint is located. This can be either a terminal area
/// or enroute if the holding fix is an enroute waypoint or enroute Navaid.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Region {
    /// An enroute fix or Navaid.
    Enroute,
    /// The terminal area to which the fix belongs with the airport ident as
    /// value.
    TerminalArea([u8; 4]),
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Waypoint {
    // TODO: Make all fields private and add getter methods.
    pub(crate) fix_ident: String,
    pub(crate) desc: String,
    pub(crate) usage: WaypointUsage,
    pub(crate) coordinate: Point<f64>,
    pub(crate) mag_var: Option<MagneticVariation>,
    pub(crate) region: Region,
    pub(crate) location: Option<LocationIndicator>,
    pub(crate) cycle: Option<AiracCycle>,
}

impl Waypoint {
    /// The terminal area of the waypoint.
    ///
    /// Returns `None` if the waypoint is not within a terminal area.
    pub(crate) fn terminal_area(&self) -> Option<&str> {
        match self.region {
            Region::TerminalArea(ref ident) => {
                Some(str::from_utf8(ident).expect("ident should be valid UTF-8"))
            }
            _ => None,
        }
    }
}

impl Fix for Waypoint {
    fn ident(&self) -> String {
        self.fix_ident.clone()
    }

    fn coordinate(&self) -> Point<f64> {
        self.coordinate
    }
}
