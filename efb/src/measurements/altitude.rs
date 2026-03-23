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

use std::ops::Div;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{
    constants, Duration, DurationUnit, Measurement, PhysicalQuantity, UnitOfMeasure, VerticalRate,
    VerticalRateUnit,
};

/// Altitude unit with _m_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum AltitudeUnit {
    Feet,
    Meters,
}

impl UnitOfMeasure<f32> for AltitudeUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Length
    }

    fn si() -> Self {
        Self::Meters
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::Feet => "ft",
            Self::Meters => "m",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::Meters => value,
            Self::Feet => value / constants::FEET_IN_METER,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::Meters => *value,
            Self::Feet => value * constants::FEET_IN_METER,
        }
    }
}

/// Altitude above mean sea level (MSL).
///
/// Represents a vertical position resolved to a common MSL reference under
/// explicit atmospheric assumptions. Distinct from [`VerticalDistance`], which
/// preserves the original reference (FL, AGL, MSL, etc.) as stored in
/// navigation data.
///
/// Typically obtained by calling [`VerticalDistance::to_msl`].
///
/// [`VerticalDistance`]: crate::VerticalDistance
/// [`VerticalDistance::to_msl`]: crate::VerticalDistance::to_msl
pub type Altitude = Measurement<f32, AltitudeUnit>;

impl Altitude {
    /// Creates an altitude in feet above MSL.
    pub fn ft(value: f32) -> Self {
        Self {
            value,
            unit: AltitudeUnit::Feet,
        }
    }

    /// Creates an altitude in meters above MSL.
    pub fn m(value: f32) -> Self {
        Self {
            value,
            unit: AltitudeUnit::Meters,
        }
    }
}

impl Div<Duration> for Altitude {
    type Output = VerticalRate;

    /// Divides altitude by time to produce a vertical rate.
    ///
    /// Uses SI units internally: altitude in m / time in s = rate in m/s.
    fn div(self, rhs: Duration) -> Self::Output {
        let mps = self.to_si() / rhs.to_si() as f32;
        VerticalRate::from_si(mps, VerticalRateUnit::FeetPerMinute)
    }
}

impl Div<VerticalRate> for Altitude {
    type Output = Duration;

    /// Divides altitude by vertical rate to produce a duration.
    ///
    /// Uses SI units internally: altitude in m / rate in m/s = time in s.
    fn div(self, rhs: VerticalRate) -> Self::Output {
        let s = self.to_si() / rhs.to_si();
        Duration::from_si(s.round() as u32, DurationUnit::Seconds)
    }
}
