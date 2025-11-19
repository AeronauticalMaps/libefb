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

use super::constants;
use super::{Measurement, PhysicalQuantity, UnitOfMeasure};

/// Temperature with _K_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum TemperatureUnit {
    Kelvin,
    Celsius,
    Fahrenheit,
}

impl UnitOfMeasure<f32> for TemperatureUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Temperature
    }

    fn si() -> Self {
        Self::Kelvin
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::Kelvin => "K",
            Self::Celsius => "°C",
            Self::Fahrenheit => "°F",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::Kelvin => value,
            Self::Celsius => value - constants::KELVIN_IN_CELSIUS,
            Self::Fahrenheit => ((value - constants::KELVIN_IN_CELSIUS) * 1.8) + 32.0,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::Kelvin => *value,
            Self::Celsius => value + constants::KELVIN_IN_CELSIUS,
            Self::Fahrenheit => ((value - 32.0) / 1.8) + constants::KELVIN_IN_CELSIUS,
        }
    }
}

pub type Temperature = Measurement<f32, TemperatureUnit>;

impl Temperature {
    pub fn k(value: f32) -> Self {
        Measurement {
            value,
            unit: TemperatureUnit::Kelvin,
        }
    }

    pub fn c(value: f32) -> Self {
        Measurement {
            value,
            unit: TemperatureUnit::Celsius,
        }
    }

    pub fn f(value: f32) -> Self {
        Measurement {
            value,
            unit: TemperatureUnit::Fahrenheit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_shows_symbol() {
        let k = Temperature::k(0.0);
        let c = Temperature::c(0.0);
        let f = Temperature::f(0.0);

        assert_eq!(k.symbol(), "K");
        assert_eq!(c.symbol(), "°C");
        assert_eq!(f.symbol(), "°F");
    }

    #[test]
    fn convert_k_to_c() {
        assert_eq!(Temperature::k(0.0), Temperature::c(-273.15));
    }

    #[test]
    fn convert_c_to_f() {
        assert_eq!(Temperature::c(15.0), Temperature::f(59.0));
    }
}
