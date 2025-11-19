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

use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::constants;
use super::{Measurement, PhysicalQuantity, UnitOfMeasure};
use crate::error::Error;

/// Speed unit with _m/s_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum SpeedUnit {
    MetersPerSecond,
    Knots,
    Mach,
}

impl UnitOfMeasure<f32> for SpeedUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Speed
    }

    fn si() -> Self {
        Self::MetersPerSecond
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::MetersPerSecond => "mps",
            Self::Knots => "kt",
            Self::Mach => "mach",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::MetersPerSecond => value,
            Self::Knots => value * constants::METER_PER_SECONDS_IN_KNOTS,
            Self::Mach => unimplemented!(),
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::MetersPerSecond => *value,
            Self::Knots => value / constants::METER_PER_SECONDS_IN_KNOTS,
            Self::Mach => unimplemented!(),
        }
    }
}

pub type Speed = Measurement<f32, SpeedUnit>;

impl Speed {
    pub fn mps(value: f32) -> Self {
        Measurement {
            value,
            unit: SpeedUnit::MetersPerSecond,
        }
    }

    pub fn kt(value: f32) -> Self {
        Measurement {
            value,
            unit: SpeedUnit::Knots,
        }
    }

    pub fn mach(value: f32) -> Self {
        Measurement {
            value,
            unit: SpeedUnit::Mach,
        }
    }
}

impl FromStr for Speed {
    type Err = Error;

    /// Parses a string `s` to return Speed.
    ///
    /// The string should be according to ICAO Doc. 4444 Annex 2:
    /// - Kilometers per hour, expressed as K followed by 4 figures e.g. `K0830`
    /// - Knots, expressed as N followed by 4 figures e.g. `N0485`
    /// - Mach, expressed as M followed by 3 figures to the nearest hundredth of
    ///   unit Mach e.g. `M082`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! value {
            ($s:expr, $index:expr) => {
                $s.get($index)
                    .and_then(|s| s.parse::<u16>().ok()) // ensure that no dot is within the digit
                    .map(|value| value as f32)
                    .ok_or(Error::UnexpectedString)
            };
        }

        match s.get(0..1).unwrap_or_default() {
            // first character is the unit
            "K" => Ok(Speed::mps(value!(s, 1..5)? / 3.6)), // value in km/h
            "N" => Ok(Speed::kt(value!(s, 1..5)?)),
            "M" => Ok(Speed::mach(value!(s, 1..4)? / 100.0)), // value in hundredth of unit Mach
            _ => Err(Error::UnexpectedString),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_icao_4444_2_str() {
        assert_eq!("K0360".parse::<Speed>(), Ok(Speed::mps(100.0)));
        assert_eq!("N0485".parse::<Speed>(), Ok(Speed::kt(485.0)));
        // TODO: Implement conversion of Mach to SI.
        // assert_eq!("M082".parse::<Speed>(), Ok(Speed::mach(0.82)));
        assert_eq!("M08".parse::<Speed>(), Err(Error::UnexpectedString));
    }
}
