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

use std::ops::Add;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::constants;
use super::{Measurement, PhysicalQuantity, UnitOfMeasure};
use crate::MagneticVariation;

/// Angle unit with _rad_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum AngleUnit {
    TrueNorth,
    MagneticNorth,
    Radian,
}

impl UnitOfMeasure<f32> for AngleUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Angle
    }

    fn si() -> Self {
        AngleUnit::Radian
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::TrueNorth => "°T",
            Self::MagneticNorth => "°M",
            Self::Radian => "rad",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::TrueNorth | Self::MagneticNorth => (if value.is_sign_negative() {
                constants::PI2 + (value % (constants::PI2))
            } else {
                value % constants::PI2
            })
            .to_degrees(),
            Self::Radian => value,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::TrueNorth | &Self::MagneticNorth => value.to_radians(),
            Self::Radian => *value,
        }
    }
}

pub type Angle = Measurement<f32, AngleUnit>;

impl Angle {
    pub fn t(value: f32) -> Angle {
        Measurement {
            value: Self::wrapped(value),
            unit: AngleUnit::TrueNorth,
        }
    }

    pub fn m(value: f32) -> Angle {
        Measurement {
            value: Self::wrapped(value),
            unit: AngleUnit::MagneticNorth,
        }
    }

    pub fn rad(value: f32) -> Angle {
        Measurement {
            value,
            unit: AngleUnit::Radian,
        }
    }

    /// Wraps the value into the range 0..360.
    fn wrapped(value: f32) -> f32 {
        if value.is_sign_negative() {
            360.0 + (value % 360.0)
        } else {
            value % 360.0
        }
    }
}

impl Add<MagneticVariation> for Angle {
    type Output = Self;

    /// Adds a magnetic variation to a true north angle.
    ///
    /// Magnetic variations to the east are subtracted while variations to the
    /// west are add to the angle.
    ///
    /// # Panics
    ///
    /// Panics if the magnetic variation is add to an angle that is not true north.
    fn add(self, rhs: MagneticVariation) -> Self::Output {
        let mag_var: f32 = match rhs {
            MagneticVariation::East(v) => -v,
            MagneticVariation::West(v) => v,
            MagneticVariation::OrientedToTrueNorth => 0.0,
        };

        match self.unit() {
            AngleUnit::TrueNorth => Self::m(self.value + mag_var),
            AngleUnit::MagneticNorth => panic!("Angle is already magnetic!"),
            AngleUnit::Radian => panic!("Magnetic variation can only be add to true north angles!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_north_from_si() {
        let west = Angle::t(270.0);
        assert_eq!(
            west,
            Angle::from_si(1.5 * std::f32::consts::PI, AngleUnit::TrueNorth)
        );
    }

    #[test]
    fn add_magnetic_variation() {
        let west = Angle::t(270.0);
        let mag_var_east = MagneticVariation::East(3.0);
        assert_eq!(west + mag_var_east, Angle::m(267.0));
    }

    #[test]
    fn wrap_angles() {
        let north = Angle::t(0.0);
        assert_eq!(north, Angle::t(360.0));

        let west = Angle::t(-90.0);
        assert_eq!(west, Angle::t(270.0));

        let south = Angle::rad(std::f32::consts::PI);
        assert_eq!(south, Angle::t(180.0));
    }
}
