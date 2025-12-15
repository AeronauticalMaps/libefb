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

use crate::geom::Coordinate;
use crate::VerticalDistance;

use super::*;

#[derive(Clone, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Airport {
    pub(crate) icao_ident: String,
    pub(crate) iata_designator: String,
    pub(crate) name: String,
    pub(crate) coordinate: Coordinate,
    pub(crate) mag_var: MagneticVariation,
    pub(crate) elevation: VerticalDistance,
    pub(crate) runways: Vec<Runway>,
    pub(crate) location: Option<LocationIndicator>,
    pub(crate) cycle: Option<AiracCycle>,
}

impl Fix for Airport {
    fn ident(&self) -> String {
        self.icao_ident.clone()
    }

    fn coordinate(&self) -> Coordinate {
        self.coordinate
    }
}
