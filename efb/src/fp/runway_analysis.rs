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

use std::cmp::{Ordering, PartialOrd};
use std::fmt;

use log::{debug, warn};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::measurements::{Length, Speed, Temperature};
use crate::nd::{Runway, RunwayConditionCode};
use crate::Wind;

use super::{AlteringFactors, Influences, MassAndBalance, TakeoffLandingPerformance};

/// Analysis the runway length and direction to wind.
///
/// The analysis provides the headwind and crosswind components in runway
/// direction and the ground roll with distance over a 50ft obstacle for [`takeoff`]
/// and [`landing`].
///
/// [`takeoff`]: RunwayAnalysis::takeoff
/// [`landing`]: RunwayAnalysis::landing
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug, Default)]
pub struct RunwayAnalysis {
    headwind: Speed,
    crosswind: Speed,
    ground_roll: Length,
    clear_obstacle: Length,
    margin: Length,
    pct_margin: f32,
    notes: Vec<RunwayAnalysisNote>,
}

/// An input of a runway analysis that is unknown, so that the analysis is an
/// estimate biased by it.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RunwayAnalysisNote {
    /// The TORA is unknown and estimated by the runway length.
    ToraEstimated,
    /// The LDA is unknown and estimated by the runway length less the
    /// displaced threshold.
    LdaEstimated,
    /// The surface is unknown, and only factors for any surface apply.
    SurfaceUnknown,
    /// The slope is unknown and not corrected.
    SlopeUnknown,
}

impl fmt::Display for RunwayAnalysisNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToraEstimated => write!(f, "TORA ESTIMATED FROM RWY LENGTH"),
            Self::LdaEstimated => write!(f, "LDA ESTIMATED FROM RWY LENGTH"),
            Self::SurfaceUnknown => write!(f, "SURFACE UNKNOWN"),
            Self::SlopeUnknown => write!(f, "SLOPE UNKNOWN"),
        }
    }
}

impl RunwayAnalysis {
    /// Creates a new runway analysis for takeoff.
    pub fn takeoff(
        rwy: &Runway,
        rwycc: RunwayConditionCode,
        wind: &Wind,
        temperature: Temperature,
        mb: &MassAndBalance,
        perf: &TakeoffLandingPerformance,
        factors: Option<&AlteringFactors>,
    ) -> Self {
        let influences = Influences::new(*mb.mass_on_ramp(), rwy, wind, temperature, rwycc);
        let min_distance = perf.min_distance(&influences);

        let ground_roll = *min_distance.ground_roll()
            * factors.map_or(1.0, |f| f.ground_roll_factor(&influences));

        let clear_obstacle = *min_distance.clear_obstacle()
            * factors.map_or(1.0, |f| f.clear_obstacle_factor(&influences));

        let tora = rwy.tora_or_length();
        let margin = tora - ground_roll;
        let pct_margin = margin / tora;

        let mut notes = Vec::new();

        if rwy.tora.is_none() {
            notes.push(RunwayAnalysisNote::ToraEstimated);
        }

        if rwy.surface.is_none() {
            notes.push(RunwayAnalysisNote::SurfaceUnknown);
        }

        if rwy.slope.is_none() {
            notes.push(RunwayAnalysisNote::SlopeUnknown);
        }

        debug!(
            "takeoff analysis rwy {}: ground_roll={:?}, margin={:?} ({:.0}%), notes={:?}",
            rwy.designator,
            ground_roll,
            margin,
            pct_margin * 100.0,
            notes
        );

        if pct_margin < 0.0 {
            warn!(
                "takeoff ground roll exceeds available runway length on rwy {}",
                rwy.designator
            );
        }

        Self {
            headwind: wind.headwind(&rwy.bearing),
            crosswind: wind.crosswind(&rwy.bearing),
            ground_roll,
            clear_obstacle,
            margin,
            pct_margin,
            notes,
        }
    }

    /// Creates a new runway analysis for landing.
    pub fn landing(
        rwy: &Runway,
        rwycc: RunwayConditionCode,
        wind: &Wind,
        temperature: Temperature,
        mb: &MassAndBalance,
        perf: &TakeoffLandingPerformance,
        factors: Option<&AlteringFactors>,
    ) -> Self {
        let influences = Influences::new(*mb.mass_after_landing(), rwy, wind, temperature, rwycc);
        let min_distance = perf.min_distance(&influences);

        let ground_roll = *min_distance.ground_roll()
            * factors.map_or(1.0, |f| f.ground_roll_factor(&influences));

        let clear_obstacle = *min_distance.clear_obstacle()
            * factors.map_or(1.0, |f| f.clear_obstacle_factor(&influences));

        let lda = rwy.lda_or_length();
        let margin = lda - ground_roll;
        let pct_margin = margin / lda;

        let mut notes = Vec::new();

        if rwy.lda.is_none() {
            notes.push(RunwayAnalysisNote::LdaEstimated);
        }

        if rwy.surface.is_none() {
            notes.push(RunwayAnalysisNote::SurfaceUnknown);
        }

        if rwy.slope.is_none() {
            notes.push(RunwayAnalysisNote::SlopeUnknown);
        }

        debug!(
            "landing analysis rwy {}: ground_roll={:?}, margin={:?} ({:.0}%), notes={:?}",
            rwy.designator,
            ground_roll,
            margin,
            pct_margin * 100.0,
            notes
        );

        if pct_margin < 0.0 {
            warn!(
                "landing ground roll exceeds available runway length on rwy {}",
                rwy.designator
            );
        }

        Self {
            headwind: wind.headwind(&rwy.bearing),
            crosswind: wind.crosswind(&rwy.bearing),
            ground_roll,
            clear_obstacle,
            margin,
            pct_margin,
            notes,
        }
    }

    /// The headwind component relative to the runway direction.
    pub fn headwind(&self) -> &Speed {
        &self.headwind
    }

    /// The crosswind component relative to the runway direction.
    pub fn crosswind(&self) -> &Speed {
        &self.crosswind
    }

    /// The minimal predicted ground roll.
    pub fn ground_roll(&self) -> &Length {
        &self.ground_roll
    }

    /// The minimal predicted distance to clear a 50ft obstacle
    pub fn clear_obstacle(&self) -> &Length {
        &self.clear_obstacle
    }

    /// Returns the runway length's margin.
    pub fn margin(&self) -> &Length {
        &self.margin
    }

    /// Returns the margin in percent.
    pub fn pct_margin(&self) -> &f32 {
        &self.pct_margin
    }

    /// Returns the inputs that are unknown and bias the analysis.
    pub fn notes(&self) -> &[RunwayAnalysisNote] {
        &self.notes
    }
}

impl PartialOrd for RunwayAnalysis {
    /// Order runway analysis by margin.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.margin.partial_cmp(&other.margin)
    }
}
