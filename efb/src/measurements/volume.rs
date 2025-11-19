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

use std::ops::Mul;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Density, DensityUnit, Mass, MassUnit, Measurement, PhysicalQuantity, UnitOfMeasure};

/// Volume with _m³_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum VolumeUnit {
    CubicMeters,
    Liter,
}

impl UnitOfMeasure<f32> for VolumeUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Volume
    }

    fn si() -> Self {
        Self::CubicMeters
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::CubicMeters => "m³",
            Self::Liter => "L",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::CubicMeters => value,
            Self::Liter => value * 1000.0,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::CubicMeters => *value,
            Self::Liter => value / 1000.0,
        }
    }
}

pub type Volume = Measurement<f32, VolumeUnit>;

impl Volume {
    pub fn cubic_m(value: f32) -> Self {
        Measurement {
            value,
            unit: VolumeUnit::CubicMeters,
        }
    }

    pub fn l(value: f32) -> Self {
        Measurement {
            value,
            unit: VolumeUnit::Liter,
        }
    }
}

impl Mul<Density> for Volume {
    type Output = Mass;

    fn mul(self, rhs: Density) -> Self::Output {
        let unit = match rhs.unit {
            DensityUnit::KilogramPerCubicMeter | DensityUnit::KilogramPerLiter => {
                MassUnit::Kilograms
            }
        };

        Mass::from_si(self.to_si() * rhs.to_si(), unit)
    }
}
