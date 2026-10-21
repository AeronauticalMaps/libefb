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

use crate::measurements::{Length, Temperature};
use crate::VerticalDistance;

mod altering_factors;
mod builder;
mod influences;

pub use altering_factors::*;
pub use builder::*;
pub use influences::*;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Debug, Default)]
pub struct TakeoffLandingDistance {
    ground_roll: Length,
    clear_obstacle: Length,
}

impl TakeoffLandingDistance {
    pub fn ground_roll(&self) -> &Length {
        &self.ground_roll
    }

    pub fn clear_obstacle(&self) -> &Length {
        &self.clear_obstacle
    }
}

/// The takeoff or landing performance.
///
/// This takeoff or landing performance provides the minimal estimated ground
/// roll or distance to clear a 50ft obstacle. The performance is returned based
/// on some [`Influences`] affecting the takeoff or landing.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TakeoffLandingPerformance {
    table: Vec<(VerticalDistance, Temperature, Length, Length)>,
    factors: Option<AlteringFactors>,
    notes: Option<String>,
}

impl TakeoffLandingPerformance {
    pub fn new(
        table: Vec<(VerticalDistance, Temperature, Length, Length)>,
        factors: Option<AlteringFactors>,
        notes: Option<String>,
    ) -> Self {
        Self {
            table,
            factors,
            notes,
        }
    }

    /// Returns a builder to construct the performance.
    pub fn builder<I>(table: I) -> TakeoffLandingPerformanceBuilder
    where
        I: IntoIterator<Item = (VerticalDistance, Temperature, Length, Length)>,
    {
        TakeoffLandingPerformanceBuilder::new(table)
    }

    /// Minimal predicted takeoff or landing distance.
    pub fn min_distance(&self, influences: &Influences) -> TakeoffLandingDistance {
        let distance = self.distance(influences.temperature(), influences.level());

        TakeoffLandingDistance {
            ground_roll: distance.ground_roll
                * self
                    .factors
                    .as_ref()
                    .map_or(1.0, |f| f.ground_roll_factor(influences)),
            clear_obstacle: distance.clear_obstacle
                * self
                    .factors
                    .as_ref()
                    .map_or(1.0, |f| f.clear_obstacle_factor(influences)),
        }
    }

    pub fn distance(
        &self,
        temperature: &Temperature,
        pa: &VerticalDistance,
    ) -> TakeoffLandingDistance {
        // Since the ground roll and distance to clear an obstacle
        // increase with pressure altitude and temperature, the
        // conservative value is provided by the next higher PA or
        // temperature.
        let closest = self
            .table
            .iter()
            .reduce(|closest, row| {
                // 1. Find the rows that have the PA that is the closest
                //    to the influence's level and equal or above.
                let closest_level = if row.0 >= *pa && closest.0 >= *pa {
                    if row.0 < closest.0 {
                        row
                    } else {
                        closest
                    }
                } else if row.0 >= *pa {
                    row
                } else {
                    closest
                };

                // 2. Find on the rows filtered in the first step the
                //    row with the closest temperature which is equal or
                //    above the influence's temperature.
                if row.1 >= *temperature && closest_level.1 >= *temperature {
                    // row and closest are both above the influence so
                    // we take the smaller of both
                    if row.1 < closest_level.1 {
                        row
                    } else {
                        closest_level
                    }
                } else if row.1 >= *temperature {
                    // only row is above or equal so that's not the closest
                    row
                } else {
                    closest_level
                }
            })
            .expect("table should not be empty");

        TakeoffLandingDistance {
            ground_roll: closest.2,
            clear_obstacle: closest.3,
        }
    }

    /// Notes regarding the performance.
    ///
    /// Use the notes to e.g. keep track of any specific conditions for which
    /// the performance might only be valid.
    pub fn notes(&self) -> Option<&String> {
        self.notes.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_closest_distance_from_table() {
        let perf = TakeoffLandingPerformance::new(
            vec![
                (
                    VerticalDistance::PressureAltitude(0),
                    Temperature::c(0.0),
                    Length::ft(800.0),
                    Length::ft(1500.0),
                ),
                (
                    VerticalDistance::PressureAltitude(0),
                    Temperature::c(40.0),
                    Length::ft(1000.0),
                    Length::ft(2000.0),
                ),
                (
                    VerticalDistance::PressureAltitude(8000),
                    Temperature::c(0.0),
                    Length::ft(1800.0),
                    Length::ft(3600.0),
                ),
                (
                    VerticalDistance::PressureAltitude(8000),
                    Temperature::c(30.0),
                    Length::ft(2300.0),
                    Length::ft(4800.0),
                ),
            ],
            None,
            None,
        );

        assert_eq!(
            perf.distance(&Temperature::c(-10.0), &VerticalDistance::Gnd)
                .ground_roll,
            Length::ft(800.0)
        );

        assert_eq!(
            perf.distance(
                &Temperature::c(20.0),
                &VerticalDistance::PressureAltitude(1000)
            )
            .ground_roll,
            Length::ft(2300.0)
        );
    }
}
