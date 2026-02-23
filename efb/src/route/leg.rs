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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use geo::{Bearing, Distance, Geodesic};

use crate::fp::Performance;
use crate::measurements::{Angle, AngleUnit, Duration, Length, LengthUnit, Speed};
use crate::nd::{Fix, NavAid};
use crate::{Fuel, VerticalDistance, Wind};

/// A leg `from` one point `to` another.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Leg {
    from: NavAid,
    to: NavAid,
    level: Option<VerticalDistance>,
    tas: Option<Speed>,
    wind: Option<Wind>,
    heading: Option<Angle>,
    mh: Option<Angle>,
    bearing: Angle,
    mc: Angle,
    dist: Length,
    gs: Option<Speed>,
    wca: Option<Angle>,
    ete: Option<Duration>,
}

impl Leg {
    pub fn new(
        from: NavAid,
        to: NavAid,
        level: Option<VerticalDistance>,
        tas: Option<Speed>,
        wind: Option<Wind>,
    ) -> Leg {
        let from_coord = from.coordinate();
        let to_coord = to.coordinate();

        // Use geo's Geodesic for bearing and distance calculations
        let bearing_deg = Geodesic.bearing(from_coord, to_coord);
        let bearing = Angle::t(bearing_deg as f32);
        let mc = bearing + from.mag_var();

        let distance_m = Geodesic.distance(from_coord, to_coord);
        let dist = Length::m(distance_m as f32).convert_to(LengthUnit::NauticalMiles);

        let (gs, wca) = {
            match (tas, wind) {
                (Some(tas), Some(wind)) => {
                    let wca = wind_correction_angle(&wind, &tas, &bearing);
                    let gs = ground_speed(&tas, &wind, &wca, &bearing);

                    (Some(gs), Some(wca))
                }
                _ => (None, None),
            }
        };

        let heading = wca.map(|wca| bearing + wca);
        let mh = heading.map(|heading| heading + from.mag_var());
        let ete = gs.map(|gs| dist / gs);

        Self {
            from,
            to,
            level,
            tas,
            wind,
            heading,
            mh,
            bearing,
            mc,
            dist,
            gs,
            wca,
            ete,
        }
    }

    /// The point from which the leg starts.
    pub fn from(&self) -> &NavAid {
        &self.from
    }

    /// The point to which the leg is going.
    pub fn to(&self) -> &NavAid {
        &self.to
    }

    /// The level of the leg.
    pub fn level(&self) -> Option<&VerticalDistance> {
        self.level.as_ref()
    }

    /// The desired true airspeed (TAS).
    pub fn tas(&self) -> Option<&Speed> {
        self.tas.as_ref()
    }

    /// The wind to take into account.
    pub fn wind(&self) -> Option<&Wind> {
        self.wind.as_ref()
    }

    /// The true heading considering the wind correction angle (WCA).
    pub fn heading(&self) -> Option<&Angle> {
        self.heading.as_ref()
    }

    /// The magnetic heading considering the variation at the start of the leg.
    pub fn mh(&self) -> Option<&Angle> {
        self.mh.as_ref()
    }

    /// The bearing between the two points.
    pub fn bearing(&self) -> &Angle {
        &self.bearing
    }

    /// The magnetic course taking the magnetic variation from the starting
    /// point into consideration.
    pub fn mc(&self) -> &Angle {
        &self.mc
    }

    /// The distance between the leg's two points.
    pub fn dist(&self) -> &Length {
        &self.dist
    }

    // TODO add test to verify calculation
    /// The ground speed in knots.
    pub fn gs(&self) -> Option<&Speed> {
        self.gs.as_ref()
    }

    /// The wind correction angle based on the wind.
    pub fn wca(&self) -> Option<&Angle> {
        self.wca.as_ref()
    }

    // TODO add test to verify calculation
    /// The estimated time enroute the leg.
    pub fn ete(&self) -> Option<&Duration> {
        self.ete.as_ref()
    }

    /// The [Fuel] consumed on the leg with the given [Performance].
    pub fn fuel(&self, perf: &Performance) -> Option<Fuel> {
        match (self.level, self.ete) {
            (Some(level), Some(ete)) => Some(perf.ff(&level) * ete),
            _ => None,
        }
    }
}

fn wind_correction_angle(wind: &Wind, tas: &Speed, bearing: &Angle) -> Angle {
    let wind_azimuth = wind.direction + Angle::t(180.0);
    // the angle between the wind direction and bearing
    let wind_angle = *bearing - wind_azimuth;

    // The law of sines gives us
    //
    //   sin(wca) / ws = sin(wind_angle) / tas
    //
    // from which we get the wca as following:
    Angle::from_si(
        (wind.speed / *tas * wind_angle.to_si().sin()).asin(),
        AngleUnit::TrueNorth,
    )
}

fn ground_speed(tas: &Speed, wind: &Wind, wca: &Angle, bearing: &Angle) -> Speed {
    Speed::from_si(
        (*tas * *tas + wind.speed * wind.speed
            - ((*tas * wind.speed * 2.0) * (*bearing - wind.direction + *wca).to_si().cos()))
        .to_si()
        .sqrt(),
        *tas.unit(),
    )
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn wind_correction_angle_left() {
        let wca = wind_correction_angle(
            &Wind::from_str("18050KT").unwrap(),
            &Speed::from_str("N0100").unwrap(),
            &Angle::t(90.0),
        );

        assert_eq!(wca.value().round(), 30.0);
    }

    #[test]
    fn wind_correction_angle_right() {
        let wca = wind_correction_angle(
            &Wind::from_str("00050KT").unwrap(),
            &Speed::from_str("N0100").unwrap(),
            &Angle::t(90.0),
        );

        // negative angles are wrapped: 360 - 30 = 330
        assert_eq!(wca.value().round(), 330.0);
    }
}
