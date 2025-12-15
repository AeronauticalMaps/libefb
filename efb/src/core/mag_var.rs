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

use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use time::OffsetDateTime;
use world_magnetic_model::uom::si::{
    angle::degree, angle::radian, f32::Angle, f32::Length, length::meter,
};
use world_magnetic_model::GeomagneticField;

use crate::geom::Coordinate;

/// The magnetic variation (declination) of a point.
///
/// Any [Coordinate] can be converted into a declination.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum MagneticVariation {
    /// The declination is towards the east.
    East(f32),
    /// The declination is towards the west.
    West(f32),
    /// The point is oriented to true north.
    OrientedToTrueNorth,
}

impl Hash for MagneticVariation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MagneticVariation::East(v) => {
                0u8.hash(state);
                v.to_bits().hash(state);
            }
            MagneticVariation::West(v) => {
                1u8.hash(state);
                v.to_bits().hash(state);
            }
            MagneticVariation::OrientedToTrueNorth => {
                2u8.hash(state);
            }
        }
    }
}

impl From<Coordinate> for MagneticVariation {
    fn from(value: Coordinate) -> Self {
        let mag_var = match GeomagneticField::new(
            Length::new::<meter>(0.0),
            Angle::new::<radian>(value.latitude.to_radians()),
            Angle::new::<radian>(value.longitude.to_radians()),
            OffsetDateTime::now_utc().date(),
        ) {
            Ok(field) => field.declination().get::<degree>(),
            Err(_) => todo!("implement TryFrom to handle unavailable variation!"),
        };

        if mag_var.is_sign_negative() {
            Self::West(mag_var.abs())
        } else {
            Self::East(mag_var)
        }
    }
}

impl Display for MagneticVariation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::East(value) => write!(f, "{value:.1}° E"),
            Self::West(value) => write!(f, "{value:.1}° W"),
            Self::OrientedToTrueNorth => write!(f, "T"),
        }
    }
}
