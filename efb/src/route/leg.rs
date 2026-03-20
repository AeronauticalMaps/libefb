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

use log::trace;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use geo::{Bearing, Distance, Geodesic};

use crate::fp::LegPerformance;
use crate::measurements::{Angle, AngleUnit, Duration, Length, LengthUnit, Speed};
use crate::nd::{Fix, NavAid};
use crate::{Fuel, VerticalDistance, Wind};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClimbDescentAlongLeg {
    /// Level the aircraft is at when entering this leg (before any transitions).
    from: Option<VerticalDistance>,
    /// Target level of a transition starting at the FROM fix.
    to: Option<VerticalDistance>,
    /// Target level that must be reached by the TO fix.
    reach_at: Option<VerticalDistance>,
}

impl ClimbDescentAlongLeg {
    /// Level the aircraft is at when entering this leg.
    pub fn from(&self) -> Option<&VerticalDistance> {
        self.from.as_ref()
    }

    /// Target level of a transition starting at the FROM fix.
    pub fn to(&self) -> Option<&VerticalDistance> {
        self.to.as_ref()
    }

    /// Target level that must be reached by the TO fix.
    pub fn reach_at(&self) -> Option<&VerticalDistance> {
        self.reach_at.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct LegBuilder {
    level: Option<VerticalDistance>,
    climb_descent: ClimbDescentAlongLeg,
    tas: Option<Speed>,
    wind: Option<Wind>,
}

impl LegBuilder {
    /// Builds a new leg `from` → `to`.
    ///
    /// The builder consumes all level changes and updates the level according
    /// to the changes. Subsequent builds retain the latest cruise level in case
    /// no further level changes occur.
    pub fn build(&mut self, from: NavAid, to: NavAid) -> Leg {
        // Seed `from` level from the builder's current level (before any
        // transitions on this leg). If still None and the FROM fix is an
        // airport, use its elevation.
        self.climb_descent.from = self.level.or_else(|| match &from {
            NavAid::Airport(arpt) => Some(arpt.elevation),
            _ => None,
        });

        self.climb_descent
            .to
            .inspect(|level| trace!("climb/descent to {level} from {from}"));
        self.climb_descent
            .reach_at
            .inspect(|level| trace!("reach {to} on {level}"));

        // The leg's cruise level is the level after the `to` transition (if
        // any), otherwise the previous level.
        let level = self.climb_descent.to.or(self.level);

        let leg = Leg::new(from, to, self.climb_descent, level, self.tas, self.wind);

        // Update the level for subsequent legs: the last transition reached
        // is the new cruise level.
        if self.climb_descent.reach_at.is_some() {
            self.level = self.climb_descent.reach_at.take();
        } else if self.climb_descent.to.is_some() {
            self.level = self.climb_descent.to.take();
        }
        let _ = self.climb_descent.to.take();

        leg
    }

    pub fn cruise(self: &mut Self, level: VerticalDistance) {
        // Since we can't teleport to the new level, we need to climb/descent to
        // it starting at the "from" fix. Don't update self.level here — build()
        // needs it to set climb_descent.from to the *previous* level first.
        self.climb_descent.to = Some(level);
    }

    pub fn level_at_fix(self: &mut Self, level: VerticalDistance) {
        // Reaching a new level at a fix along the leg is only possible for the
        // "to" fix.
        self.climb_descent.reach_at = Some(level);
    }

    pub fn tas(self: &mut Self, tas: Speed) {
        self.tas = Some(tas);
        trace!("cruise speed set to {tas}");
    }

    pub fn wind(self: &mut Self, wind: Wind) {
        self.wind = Some(wind);
        trace!("wind set to {wind}");
    }
}

/// A leg `from` one point `to` another.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Leg {
    from: NavAid,
    to: NavAid,
    climb_descent: ClimbDescentAlongLeg,
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
    pub(super) fn builder() -> LegBuilder {
        LegBuilder::default()
    }

    pub fn divert(&self, alternate: NavAid) -> Leg {
        Leg::new(
            self.from.clone(),
            alternate,
            self.climb_descent,
            self.level,
            self.tas,
            self.wind,
        )
    }

    fn new(
        from: NavAid,
        to: NavAid,
        climb_descent: ClimbDescentAlongLeg,
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

        trace!(
            "leg {} -> {}: dist={:.1}, bearing={:.1}, gs={:?}, ete={:?}",
            from.ident(),
            to.ident(),
            dist,
            bearing,
            gs,
            ete
        );

        Self {
            from,
            to,
            climb_descent,
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

    pub fn climb_descent(&self) -> &ClimbDescentAlongLeg {
        &self.climb_descent
    }

    /// The desired true airspeed (TAS).
    pub fn tas(&self) -> Option<&Speed> {
        self.tas.as_ref()
    }

    /// The wind to take into account.
    pub fn wind(&self) -> Option<&Wind> {
        self.wind.as_ref()
    }

    /// The headwind component along this leg's bearing.
    pub fn headwind(&self) -> Option<Speed> {
        self.wind.map(|w| w.headwind(&self.bearing))
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

    /// The [Fuel] consumed on the leg with the given [LegPerformance].
    ///
    /// When climb or descent performance is available, climb/descent fuel is
    /// computed for any level transitions on the leg and the cruise time is
    /// reduced accordingly. Falls back to pure cruise when no transitions
    /// exist or no climb/descent performance is provided.
    pub fn fuel(&self, perf: &LegPerformance) -> Option<Fuel> {
        let ete = self.ete?;
        let level = self.level?;

        let from_level = self.climb_descent.from;
        let to_level = self.climb_descent.to;
        let reach_at = self.climb_descent.reach_at;
        let hw = self.headwind().unwrap_or(Speed::kt(0.0));

        let mut climb_descent_time = Duration::s(0);
        let mut climb_descent_fuel: Option<Fuel> = None;

        let mut add_transition =
            |current: &VerticalDistance, target: &VerticalDistance| -> Option<()> {
                let (lo, hi) = if target > current {
                    (current, target)
                } else {
                    (target, current)
                };
                let is_climb = target > current;
                let cdp = if is_climb {
                    perf.climb()?
                } else {
                    perf.descent()?
                };
                let result = cdp.between(lo, hi)?.with_wind(hw);
                climb_descent_time = climb_descent_time + result.time;
                climb_descent_fuel = Some(match climb_descent_fuel {
                    Some(f) => f + result.fuel,
                    None => result.fuel,
                });
                Some(())
            };

        // Transition at FROM fix (to_level)
        let mut current = from_level;
        if let (Some(from), Some(to)) = (current, to_level) {
            add_transition(&from, &to);
            current = Some(to);
        }

        // Transition reaching TO fix (reach_at)
        if let (Some(from), Some(ra)) = (current.or(from_level), reach_at) {
            add_transition(&from, &ra);
        }

        // Cruise for the remaining time
        let cruise_time = if climb_descent_time < ete {
            ete - climb_descent_time
        } else {
            Duration::s(0)
        };
        let cruise_fuel = perf.cruise()?.ff(&level) * cruise_time;

        Some(match climb_descent_fuel {
            Some(cd) => cruise_fuel + cd,
            None => cruise_fuel,
        })
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
