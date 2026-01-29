// SPDX-License-Identifier: Apache-2.0
// Copyright 2024, 2026 Joe Pearson
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

use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use geo::{Bearing, Distance, Geodesic};

use crate::fc;
use crate::measurements::{Angle, Length};

/// Coordinate value.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub struct Coordinate {
    /// Latitude in the range from -90° (south) to 90° (north).
    pub latitude: f64,

    /// Longitude in the range from -180° (west) to 180° (east).
    pub longitude: f64,
}

impl Hash for Coordinate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.latitude.to_bits().hash(state);
        self.longitude.to_bits().hash(state);
    }
}

impl From<Coordinate> for geo::Coord<f64> {
    fn from(c: Coordinate) -> Self {
        geo::Coord {
            x: c.longitude,
            y: c.latitude,
        }
    }
}

impl From<geo::Coord<f64>> for Coordinate {
    fn from(c: geo::Coord<f64>) -> Self {
        Self {
            latitude: c.y,
            longitude: c.x,
        }
    }
}

impl From<Coordinate> for geo::Point<f64> {
    fn from(c: Coordinate) -> Self {
        geo::Point::new(c.longitude, c.latitude)
    }
}

impl From<geo::Point<f64>> for Coordinate {
    fn from(p: geo::Point<f64>) -> Self {
        Self {
            latitude: p.y(),
            longitude: p.x(),
        }
    }
}

impl Coordinate {
    /// Creates a new coordinate.
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    /// Returns the bearing between this point and the `other`.
    ///
    /// Uses geodesic calculation on the WGS84 ellipsoid.
    pub fn bearing(&self, other: &Coordinate) -> Angle {
        let bearing = Geodesic.bearing((*self).into(), (*other).into());
        Angle::t(bearing as f32)
    }

    /// Returns the distance from this point to the `other`.
    ///
    /// Uses geodesic calculation on the WGS84 ellipsoid.
    pub fn dist(&self, other: &Coordinate) -> Length {
        let distance_m = Geodesic.distance((*self).into(), (*other).into());
        Length::m(distance_m as f32)
    }

    pub fn from_dms(latitude: (i8, u8, u8), longitude: (i16, u8, u8)) -> Self {
        Self {
            latitude: latitude.0.signum() as f64
                * fc::dms_to_decimal(latitude.0 as u8, latitude.1, latitude.2),
            longitude: longitude.0.signum() as f64
                * fc::dms_to_decimal(longitude.0 as u8, longitude.1, longitude.2),
        }
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({0}, {1})", self.latitude, self.longitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurements::LengthUnit;

    // As benchmark for our testing we use the directions to an airfield as
    // published in the German AIP. The airfield Hungriger Wolf in Itzehoe
    // (EDHF) has two directions from two VOR published in its visual operation
    // chart (25 JUL 2024).

    // Helgoland VOR
    const DHE: Coordinate = coord!(54.18568611, 7.91070000);
    // Itzehoe Hungriger Wolf
    const EDHF: Coordinate = coord!(53.99250000, 9.57666667);

    #[test]
    fn bearing() {
        // From the AIP we get a magnetic heading from the Helgoland VOR (DHE)
        // to EDHF of 97°. With a magnetic variation of 4° east in EDHF, we get
        // a true bearing of approximately 101°. The geodesic calculation on the
        // WGS84 ellipsoid gives a more precise result of ~100°.
        assert_eq!(DHE.bearing(&EDHF).value().round(), 100.0);
    }

    #[test]
    fn dist() {
        // the AIP provides only rounded values
        assert_eq!(
            DHE.dist(&EDHF)
                .convert_to(LengthUnit::NauticalMiles)
                .value()
                .round(),
            60.0
        );
    }
}
