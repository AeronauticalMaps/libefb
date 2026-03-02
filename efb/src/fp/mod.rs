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

//! Flight Planning.
//!
//! This module contains tools to help planning a flight. The following aspects
//! of the flight planning are covered by this module:
//!
//! - [`ClimbDescentPerformance`] to model aircraft performance during climbs
//!   and descents, computing time, fuel, and horizontal distance per altitude
//!   band
//! - [`FuelPlanning`] to estimate the fuel required for the trip including any
//!   safety reserves
//! - [`MassAndBalance`] to check if the mass and CG are within the aircraft's
//!   bounds
//! - [`RunwayAnalysis`] to estimate the ground roll and distance to clear a
//!   50ft obstacle on takeoff or landing

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod builder;
mod climb_descent_performance;
mod fuel_planning;
mod mb;
mod perf;
mod runway_analysis;
mod takeoff_landing_performance;

pub use builder::*;
pub use climb_descent_performance::{
    ClimbDescentBand, ClimbDescentPerformance, ClimbDescentResult, CumulativeClimbDescentEntry,
};
pub use fuel_planning::*;
pub use mb::MassAndBalance;
pub use perf::{Performance, PerformanceTable, PerformanceTableRow};
pub use runway_analysis::*;
pub use takeoff_landing_performance::*;

use crate::aircraft::Aircraft;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct FlightPlanning {
    aircraft: Option<Aircraft>,
    fuel_planning: Option<FuelPlanning>,
    mb: Option<MassAndBalance>,
    is_balanced: Option<bool>,
    takeoff_rwy_analysis: Option<RunwayAnalysis>,
    landing_rwy_analysis: Option<RunwayAnalysis>,
}

impl FlightPlanning {
    pub fn builder() -> FlightPlanningBuilder {
        FlightPlanningBuilder::new()
    }

    pub fn fuel_planning(&self) -> Option<&FuelPlanning> {
        self.fuel_planning.as_ref()
    }

    pub fn mb(&self) -> Option<&MassAndBalance> {
        self.mb.as_ref()
    }

    pub fn is_balanced(&self) -> Option<bool> {
        self.is_balanced
    }

    pub fn takeoff_rwy_analysis(&self) -> Option<&RunwayAnalysis> {
        self.takeoff_rwy_analysis.as_ref()
    }

    pub fn landing_rwy_analysis(&self) -> Option<&RunwayAnalysis> {
        self.landing_rwy_analysis.as_ref()
    }
}
