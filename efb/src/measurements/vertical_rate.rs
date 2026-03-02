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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Measurement, PhysicalQuantity, UnitOfMeasure};

/// Vertical rate unit with _m/s_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum VerticalRateUnit {
    MetersPerSecond,
    FeetPerMinute,
}

mod constants {
    /// 1 ft/min in m/s: 0.3048 m/ft ÷ 60 s/min
    pub const FEET_PER_MINUTE_IN_METERS_PER_SECOND: f32 = 0.3048 / 60.0;
}

impl UnitOfMeasure<f32> for VerticalRateUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Speed
    }

    fn si() -> Self {
        Self::MetersPerSecond
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::MetersPerSecond => "m/s",
            Self::FeetPerMinute => "fpm",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::MetersPerSecond => value,
            Self::FeetPerMinute => value / constants::FEET_PER_MINUTE_IN_METERS_PER_SECOND,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::MetersPerSecond => *value,
            Self::FeetPerMinute => value * constants::FEET_PER_MINUTE_IN_METERS_PER_SECOND,
        }
    }
}

pub type VerticalRate = Measurement<f32, VerticalRateUnit>;

impl VerticalRate {
    /// Creates a vertical rate in meters per second.
    pub fn mps(value: f32) -> Self {
        Self {
            value,
            unit: VerticalRateUnit::MetersPerSecond,
        }
    }

    /// Creates a vertical rate in feet per minute.
    pub fn fpm(value: f32) -> Self {
        Self {
            value,
            unit: VerticalRateUnit::FeetPerMinute,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fpm_to_si() {
        // 1 fpm should be 0.00508 m/s
        let rate = VerticalRate::fpm(1.0);
        let si = rate.to_si();
        assert!((si - 0.00508).abs() < 1e-5, "1 fpm = {si} m/s");
    }

    #[test]
    fn convert_fpm_to_mps() {
        let rate = VerticalRate::fpm(1000.0);
        let as_mps = rate.convert_to(VerticalRateUnit::MetersPerSecond);
        // 1000 fpm ≈ 5.08 m/s
        assert!((*as_mps.value() - 5.08).abs() < 0.01);
    }

    #[test]
    fn convert_mps_to_fpm() {
        let rate = VerticalRate::mps(5.08);
        let as_fpm = rate.convert_to(VerticalRateUnit::FeetPerMinute);
        assert!((*as_fpm.value() - 1000.0).abs() < 1.0);
    }
}
