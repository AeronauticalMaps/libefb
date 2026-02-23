// SPDX-License-Identifier: Apache-2.0
// Copyright 2025, 2026 Joe Pearson
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

use std::fmt;
use std::rc::Rc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::MagneticVariation;
use geo::Point;

use super::AiracCycle;
use super::Airport;
use super::Fix;
use super::LocationIndicator;
use super::Waypoint;

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NavAid {
    Airport(Rc<Airport>),
    Waypoint(Rc<Waypoint>),
}

impl NavAid {
    pub fn location(&self) -> Option<LocationIndicator> {
        match self {
            Self::Airport(arpt) => arpt.location,
            Self::Waypoint(wp) => wp.location,
        }
    }

    pub fn cycle(&self) -> Option<AiracCycle> {
        match self {
            Self::Airport(arpt) => arpt.cycle,
            Self::Waypoint(wp) => wp.cycle,
        }
    }
}

impl Fix for NavAid {
    fn ident(&self) -> String {
        match self {
            Self::Airport(arpt) => arpt.ident(),
            Self::Waypoint(wp) => wp.ident(),
        }
    }

    fn coordinate(&self) -> Point<f64> {
        match self {
            Self::Airport(arpt) => arpt.coordinate(),
            Self::Waypoint(wp) => wp.coordinate(),
        }
    }

    fn mag_var(&self) -> MagneticVariation {
        match self {
            Self::Airport(arpt) => arpt.mag_var(),
            Self::Waypoint(wp) => wp.mag_var(),
        }
    }
}

impl fmt::Display for NavAid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident())
    }
}
