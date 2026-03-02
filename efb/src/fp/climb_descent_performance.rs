// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Joe Pearson
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

use crate::measurements::{Duration, Length, LengthUnit, Speed, VerticalRate, VerticalRateUnit};
use crate::{Fuel, FuelFlow, VerticalDistance};

/// One row of a climb or descent performance table.
///
/// Each row describes the aircraft performance within the altitude band that
/// ends at `level`. The band starts at the previous row's level, or at ground
/// for the first row.
///
/// The same struct is used for both climb and descent tables. In a climb table
/// `vertical_rate` is the rate of climb; in a descent table it is the rate of
/// descent (both expressed as a positive value).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ClimbDescentPerformanceRow {
    /// Upper bound of this altitude band.
    pub level: VerticalDistance,
    /// True airspeed during this band, used to compute horizontal distance.
    pub tas: Speed,
    /// Rate of altitude change (always positive; direction is implied by
    /// whether this is a climb or descent table).
    pub vertical_rate: VerticalRate,
    /// Fuel flow during this phase.
    pub ff: FuelFlow,
}

/// A table of climb or descent performance rows, sorted in ascending level
/// order.
pub type ClimbDescentPerformanceTable = Vec<ClimbDescentPerformanceRow>;

/// Aircraft climb or descent performance data.
///
/// Stores a table of performance rows that describe how the aircraft behaves in
/// each altitude band. The same type is used for both climb and descent
/// performance; the caller decides which applies.
///
/// # Usage
///
/// ```
/// use efb::fp::ClimbDescentPerformance;
/// use efb::measurements::{Speed, VerticalRate};
/// use efb::{FuelFlow, FuelType, Fuel, VerticalDistance};
/// use efb::measurements::Mass;
///
/// let fuel_flow = FuelFlow::PerHour(Fuel::new(Mass::kg(20.0), FuelType::AvGas));
///
/// // Build a simple two-band climb table
/// let climb = ClimbDescentPerformance::from_fn(
///     |_level| (Speed::kt(80.0), VerticalRate::fpm(700.0), fuel_flow),
///     VerticalDistance::Altitude(10_000),
/// );
/// ```
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ClimbDescentPerformance {
    table: ClimbDescentPerformanceTable,
}

/// The result of a climb or descent computation.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ClimbDescentResult {
    /// Time spent climbing or descending.
    pub time: Duration,
    /// Fuel burned during the climb or descent.
    pub fuel: Fuel,
    /// Horizontal distance covered at TAS, without any wind correction.
    ///
    /// Use [`with_wind`] to apply a headwind or tailwind component.
    ///
    /// [`with_wind`]: ClimbDescentResult::with_wind
    pub horizontal_distance_tas: Length,
}

impl ClimbDescentResult {
    /// Returns a copy with the horizontal distance corrected for wind.
    ///
    /// `headwind` is the headwind component on the leg (positive = headwind,
    /// which reduces ground speed; negative = tailwind, which increases it).
    /// Use [`Wind::headwind`] with the leg bearing to derive this value.
    ///
    /// [`Wind::headwind`]: crate::Wind::headwind
    pub fn with_wind(self, headwind: Speed) -> Self {
        // ground_distance = (TAS - headwind) * time
        //                 = TAS * time - headwind * time
        //                 = horizontal_distance_tas - headwind * time
        let wind_correction = headwind * self.time;
        let ground_dist = self.horizontal_distance_tas - wind_correction;
        // Clamp to zero for extreme headwind situations (headwind >= TAS)
        let ground_dist = if *ground_dist.value() < 0.0 {
            Length::nm(0.0)
        } else {
            ground_dist
        };
        Self {
            horizontal_distance_tas: ground_dist,
            ..self
        }
    }
}

/// Converts a `VerticalDistance` to feet as an `f32`.
///
/// Returns `None` for variants that cannot be safely expressed in feet without
/// an external reference datum (`Agl`, `PressureAltitude`, `Unlimited`).
fn vd_to_ft(vd: &VerticalDistance) -> Option<f32> {
    match vd {
        VerticalDistance::Gnd => Some(0.0),
        VerticalDistance::Altitude(n) => Some(*n as f32),
        VerticalDistance::Fl(n) => Some(*n as f32 * 100.0),
        VerticalDistance::Msl(n) => Some(*n as f32),
        // These variants require an external reference and cannot be converted
        _ => None,
    }
}

impl ClimbDescentPerformance {
    /// Creates a performance table from a pre-built vector of rows.
    ///
    /// The rows must be sorted in ascending order by `level`.
    pub fn new(table: ClimbDescentPerformanceTable) -> Self {
        Self { table }
    }

    /// Builds a performance table by calling `f` in 1 000 ft steps up to
    /// `ceiling`.
    ///
    /// The closure receives the level of each band boundary and returns `(tas,
    /// vertical_rate, fuel_flow)` for that band.
    pub fn from_fn<F>(f: F, ceiling: VerticalDistance) -> Self
    where
        F: Fn(&VerticalDistance) -> (Speed, VerticalRate, FuelFlow),
    {
        let mut table: ClimbDescentPerformanceTable = Vec::new();
        let mut vd = VerticalDistance::Gnd;
        let mut alt = 0u16;

        while vd <= ceiling {
            let (tas, vertical_rate, ff) = f(&vd);
            table.push(ClimbDescentPerformanceRow {
                level: vd,
                tas,
                vertical_rate,
                ff,
            });

            alt += 1000;
            vd = VerticalDistance::Altitude(alt);
        }

        Self { table }
    }

    /// Returns the performance row applicable at `level`.
    ///
    /// Uses a reverse-find to return the row with the highest level that is
    /// less than or equal to the target. Does not interpolate.
    ///
    /// # Panics
    ///
    /// Panics if the table is empty.
    fn at_level(&self, level: &VerticalDistance) -> &ClimbDescentPerformanceRow {
        self.table
            .iter()
            .rfind(|row| &row.level <= level)
            .expect("climb/descent performance table must not be empty")
    }

    /// Computes the time, fuel, and horizontal distance for a climb or descent
    /// between two altitude levels.
    ///
    /// The computation walks the performance table to apply the correct row for
    /// each altitude band within the range `[from_level, to_level]`.
    ///
    /// Returns `None` if:
    /// - `from_level >= to_level` (no altitude change),
    /// - the table is empty, or
    /// - either level uses a `VerticalDistance` variant that cannot be
    ///   expressed in feet (e.g. `Agl`, `PressureAltitude`, `Unlimited`).
    pub fn compute(
        &self,
        from_level: &VerticalDistance,
        to_level: &VerticalDistance,
    ) -> Option<ClimbDescentResult> {
        if from_level >= to_level || self.table.is_empty() {
            return None;
        }

        let from_ft = vd_to_ft(from_level)?;
        let to_ft = vd_to_ft(to_level)?;

        let mut band_floor_ft = from_ft;
        let mut total_time_s: f32 = 0.0;
        let mut accumulated_fuel: Option<Fuel> = None;
        let mut total_dist_m: f32 = 0.0;

        let mut accumulate = |row: &ClimbDescentPerformanceRow, delta_ft: f32| {
            let rate_fpm = *row
                .vertical_rate
                .convert_to(VerticalRateUnit::FeetPerMinute)
                .value();
            let time_s = (delta_ft / rate_fpm * 60.0).round() as u32;
            let time = Duration::s(time_s);

            let fuel = row.ff * time;
            // Speed (m/s) * Duration (s) = Length (m); convert to metres
            let dist_m = *(row.tas * time)
                .convert_to(LengthUnit::Meters)
                .value();

            total_time_s += time_s as f32;
            accumulated_fuel = Some(match accumulated_fuel {
                Some(f) => f + fuel,
                None => fuel,
            });
            total_dist_m += dist_m;
        };

        // Walk the table to cover all bands within (from_ft, to_ft]
        for row in &self.table {
            let row_ft = vd_to_ft(&row.level)?;

            if row_ft <= band_floor_ft {
                continue; // this band is below our starting altitude
            }

            let band_ceiling_ft = row_ft.min(to_ft);
            let delta_ft = band_ceiling_ft - band_floor_ft;

            accumulate(row, delta_ft);
            band_floor_ft = band_ceiling_ft;

            if band_floor_ft >= to_ft {
                break;
            }
        }

        // If to_ft exceeds all table rows, apply the highest row for the tail
        if band_floor_ft < to_ft {
            let row = self.at_level(to_level);
            let delta_ft = to_ft - band_floor_ft;
            accumulate(row, delta_ft);
        }

        let fuel = accumulated_fuel?;

        Some(ClimbDescentResult {
            time: Duration::s(total_time_s as u32),
            fuel,
            horizontal_distance_tas: Length::from_si(total_dist_m, LengthUnit::NauticalMiles),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurements::Mass;
    use crate::{FuelType};

    fn avgas_ff(lph: f32) -> FuelFlow {
        FuelFlow::PerHour(Fuel::new(
            Mass::kg(lph * 0.75), // AvGas density at ISA
            FuelType::AvGas,
        ))
    }

    fn simple_table() -> ClimbDescentPerformance {
        // Two-band table: ground→2000 ft and 2000→4000 ft
        ClimbDescentPerformance::new(vec![
            ClimbDescentPerformanceRow {
                level: VerticalDistance::Altitude(2000),
                tas: Speed::kt(80.0),
                vertical_rate: VerticalRate::fpm(800.0),
                ff: avgas_ff(20.0),
            },
            ClimbDescentPerformanceRow {
                level: VerticalDistance::Altitude(4000),
                tas: Speed::kt(85.0),
                vertical_rate: VerticalRate::fpm(600.0),
                ff: avgas_ff(18.0),
            },
        ])
    }

    #[test]
    fn compute_single_band() {
        let perf = simple_table();
        // Climb from 0 to 2000 ft: 2000 ft / 800 fpm = 2.5 min = 150 s
        let result = perf
            .compute(
                &VerticalDistance::Gnd,
                &VerticalDistance::Altitude(2000),
            )
            .expect("should produce a result");

        assert_eq!(*result.time.value(), 150, "time should be 150 s");
        // Horizontal distance: 80 kt * (2.5/60) h ≈ 3.33 NM
        let dist_nm = *result
            .horizontal_distance_tas
            .convert_to(LengthUnit::NauticalMiles)
            .value();
        assert!((dist_nm - 3.333).abs() < 0.05, "distance ~3.33 NM, got {dist_nm}");
    }

    #[test]
    fn compute_two_bands() {
        let perf = simple_table();
        // Climb from 0 to 4000 ft:
        //   Band 0→2000: 2000 / 800 = 2.5 min = 150 s
        //   Band 2000→4000: 2000 / 600 = 3.33 min = 200 s
        //   Total: 350 s
        let result = perf
            .compute(
                &VerticalDistance::Gnd,
                &VerticalDistance::Altitude(4000),
            )
            .expect("should produce a result");

        assert_eq!(*result.time.value(), 350, "time should be 350 s");
    }

    #[test]
    fn compute_from_to_equal_returns_none() {
        let perf = simple_table();
        let result = perf.compute(
            &VerticalDistance::Altitude(2000),
            &VerticalDistance::Altitude(2000),
        );
        assert!(result.is_none());
    }

    #[test]
    fn compute_from_above_to_returns_none() {
        let perf = simple_table();
        let result = perf.compute(
            &VerticalDistance::Altitude(3000),
            &VerticalDistance::Altitude(1000),
        );
        assert!(result.is_none());
    }

    #[test]
    fn with_wind_reduces_distance_for_headwind() {
        let perf = simple_table();
        let result = perf
            .compute(
                &VerticalDistance::Gnd,
                &VerticalDistance::Altitude(2000),
            )
            .unwrap();
        let no_wind_dist = *result
            .horizontal_distance_tas
            .convert_to(LengthUnit::NauticalMiles)
            .value();
        let with_headwind = result.with_wind(Speed::kt(20.0));
        let headwind_dist = *with_headwind
            .horizontal_distance_tas
            .convert_to(LengthUnit::NauticalMiles)
            .value();
        assert!(
            headwind_dist < no_wind_dist,
            "headwind should reduce ground distance: {headwind_dist} >= {no_wind_dist}"
        );
    }

    #[test]
    fn from_fn_builds_table() {
        let ff = avgas_ff(20.0);
        let perf = ClimbDescentPerformance::from_fn(
            |_| (Speed::kt(80.0), VerticalRate::fpm(700.0), ff),
            VerticalDistance::Altitude(4000),
        );
        // from_fn generates rows at Gnd, 1000, 2000, 3000, 4000
        assert_eq!(perf.table.len(), 5);
    }
}
