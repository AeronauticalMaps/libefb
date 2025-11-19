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

use super::{
    constants, Density, DensityUnit, Measurement, PhysicalQuantity, UnitOfMeasure, Volume,
    VolumeUnit,
};

/// Mass unit with _kg_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum MassUnit {
    Kilograms,
    Pounds,
}

impl UnitOfMeasure<f32> for MassUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Mass
    }

    fn si() -> Self {
        Self::Kilograms
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::Kilograms => "kg",
            Self::Pounds => "lb",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::Kilograms => value,
            Self::Pounds => value / constants::POUNDS_IN_KILOGRAMS,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::Kilograms => *value,
            Self::Pounds => value * constants::POUNDS_IN_KILOGRAMS,
        }
    }
}

pub type Mass = Measurement<f32, MassUnit>;

impl Mass {
    pub fn kg(value: f32) -> Self {
        Mass {
            value,
            unit: MassUnit::Kilograms,
        }
    }

    pub fn lb(value: f32) -> Self {
        Mass {
            value,
            unit: MassUnit::Pounds,
        }
    }
}

impl Div<Density> for Mass {
    type Output = Volume;

    fn div(self, rhs: Density) -> Self::Output {
        let unit = match rhs.unit {
            DensityUnit::KilogramPerCubicMeter => VolumeUnit::CubicMeters,
            DensityUnit::KilogramPerLiter => VolumeUnit::Liter,
        };

        Volume::from_si(self.to_si() / rhs.to_si(), unit)
    }
}
