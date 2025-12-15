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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::*;
use crate::geom::Coordinate;

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

#[derive(Clone, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Waypoint {
    pub(crate) fix_ident: String,
    pub(crate) desc: String,
    pub(crate) usage: WaypointUsage,
    pub(crate) coordinate: Coordinate,
    pub(crate) mag_var: MagneticVariation,
    pub(crate) region: Region,
    pub(crate) location: Option<LocationIndicator>,
    pub(crate) cycle: Option<AiracCycle>,
}

impl Fix for Waypoint {
    /// Returns the identifier used in routes. This is the `fix_ident` with a
    /// region prefix in case the waypoint is within a terminal area. In that
    /// case, the last two character of the airport ident are prefixed. For
    /// example, a waypoint with ident `W1` at the Hamburg terminal area `EDDH`
    /// would be `DHW1`.
    fn ident(&self) -> String {
        let region_prefix: String = match self.region {
            Region::Enroute => String::default(),
            Region::TerminalArea(airport_ident) => {
                String::from_utf8(vec![airport_ident[2], airport_ident[3]]).unwrap_or_default()
            }
        };

        region_prefix + &self.fix_ident
    }

    fn coordinate(&self) -> Coordinate {
        self.coordinate
    }
}
