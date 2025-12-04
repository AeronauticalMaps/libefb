// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Joe Pearson
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

use super::{constants, SpeedUnit};
use super::{Duration, DurationUnit, Measurement, PhysicalQuantity, Speed, UnitOfMeasure};

/// Length unit with _m_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum LengthUnit {
    Centimeters,
    Meters,
    Kilometers,
    Inches,
    Feet,
    NauticalMiles,
    StatuteMiles,
}

impl UnitOfMeasure<f32> for LengthUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Length
    }

    fn si() -> Self {
        Self::Meters
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::Centimeters => "cm",
            Self::Meters => "m",
            Self::Kilometers => "km",
            Self::Inches => "in",
            Self::Feet => "ft",
            Self::NauticalMiles => "NM",
            Self::StatuteMiles => "SM",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::Centimeters => value * 100.0,
            Self::Meters => value,
            Self::Kilometers => value / 1000.0,
            Self::Inches => value / constants::INCH_IN_METER,
            Self::Feet => value / constants::FEET_IN_METER,
            Self::NauticalMiles => value / constants::NAUTICAL_MILE_IN_METER,
            Self::StatuteMiles => value / constants::STATUTE_MILE_IN_METER,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::Centimeters => value / 100.0,
            Self::Meters => *value,
            Self::Kilometers => value * 1000.0,
            Self::Inches => value * constants::INCH_IN_METER,
            Self::Feet => value * constants::FEET_IN_METER,
            Self::NauticalMiles => value * constants::NAUTICAL_MILE_IN_METER,
            Self::StatuteMiles => value * constants::STATUTE_MILE_IN_METER,
        }
    }
}

pub type Length = Measurement<f32, LengthUnit>;

impl Length {
    pub fn cm(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::Centimeters,
        }
    }

    pub fn m(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::Meters,
        }
    }

    pub fn km(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::Kilometers,
        }
    }

    pub fn inch(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::Inches,
        }
    }

    pub fn ft(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::Feet,
        }
    }

    pub fn nm(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::NauticalMiles,
        }
    }

    pub fn sm(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::StatuteMiles,
        }
    }
}

impl Div<Speed> for Length {
    type Output = Duration;

    fn div(self, rhs: Speed) -> Self::Output {
        let s = self.to_si() / rhs.to_si();
        Duration::from_si(s.round() as u32, DurationUnit::Seconds)
    }
}

impl Div<Duration> for Length {
    type Output = Speed;

    fn div(self, rhs: Duration) -> Self::Output {
        let unit = match self.unit {
            LengthUnit::NauticalMiles => SpeedUnit::Knots,
            _ => SpeedUnit::MetersPerSecond,
        };

        let value = self.to_si() / rhs.to_si() as f32;
        Speed::from_si(value, unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_shows_symbol() {
        let m = Length::m(0.0);
        let nm = Length::nm(0.0);

        assert_eq!(m.symbol(), "m");
        assert_eq!(nm.symbol(), "NM");
    }

    #[test]
    fn length_to_si() {
        let nm = Length::nm(1.0);
        let inch = Length::inch(1.0);
        assert_eq!(nm.to_si(), constants::NAUTICAL_MILE_IN_METER);
        assert_eq!(inch.to_si(), constants::INCH_IN_METER);
    }

    #[test]
    fn convert_length() {
        assert_eq!(Length::m(0.0254), Length::inch(1.0));
    }

    #[test]
    fn convert_nm_to_m() {
        assert_eq!(Length::nm(1.0), Length::m(1852.0));
    }

    #[test]
    fn div_length_by_speed() {
        let time = Length::nm(1.0) / Speed::kt(1.0);
        assert_eq!(time, Duration::s(3600));
    }
}
