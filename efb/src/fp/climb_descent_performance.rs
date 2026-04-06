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

//! Climb and descent performance modelling.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::measurements::{Altitude, Duration, Length, Pressure, Speed, VerticalRate, Volume};
use crate::{Fuel, FuelFlow, FuelType, VerticalDistance};

/// One row of a climb or descent performance table.
///
/// Each row describes the aircraft performance within the altitude band that
/// ends at [level]. The band starts at the previous row's level, or at
/// ground for the first row.
///
/// In a climb table, the [vertical rate] is the rate of climb; in a descent table
/// it is the rate of descent. The value is always positive — direction is
/// implied by the context in which the table is used.
///
/// [level]: ClimbDescentBand::level
/// [vertical rate]: ClimbDescentBand::vertical_rate
///
/// # Example
///
/// ```
/// use efb::prelude::*;
///
/// let row = ClimbDescentBand {
///     level: VerticalDistance::Altitude(5_000),
///     tas: Speed::kt(85.0),
///     vertical_rate: VerticalRate::fpm(650.0),
///     ff: FuelFlow::PerHour(Fuel::new(Mass::kg(15.0), FuelType::AvGas)),
/// };
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ClimbDescentBand {
    /// Upper bound of this altitude band (e.g. `Altitude(5000)` for a band
    /// that covers everything up to 5 000 ft).
    pub level: VerticalDistance,

    /// True airspeed during this band, used to compute horizontal distance
    /// covered while climbing or descending.
    pub tas: Speed,

    /// Rate of altitude change (always positive; direction is implied by
    /// whether this is a climb or descent table).
    pub vertical_rate: VerticalRate,

    /// Fuel flow during this band.
    pub ff: FuelFlow,
}

/// One row of a cumulative "time, fuel, and distance to climb" table as found
/// in most POH / AFM documents.
///
/// Each entry represents the **cumulative** time, fuel, and distance from
/// sea level (or ground) to the given [`level`]. The first entry should be
/// the baseline (typically sea level with all values at zero).
///
/// Use [`ClimbDescentPerformance::from_cumulative`] to convert a slice of
/// these entries into a performance table.
///
/// [`level`]: CumulativeClimbDescentEntry::level
///
/// # Reading from a POH
///
/// A typical PA-28-181 Archer II POH climb table (ISA, gross weight) looks
/// like:
///
/// | Press. Alt | Time  | Fuel Used | Distance |
/// |------------|-------|-----------|----------|
/// | SL         | 0 min | 0 gal     | 0 NM     |
/// | 2 000 ft   | 4 min | 0.9 gal   | 5 NM     |
/// | 4 000 ft   | 8 min | 1.8 gal   | 11 NM    |
/// | …          | …     | …         | …        |
///
/// Each row of that table maps to one `CumulativeClimbDescentEntry`. Enter
/// the values exactly as printed — they are cumulative from sea level.
/// POH tables are indexed by pressure altitude, so use
/// [`VerticalDistance::PressureAltitude`] for the level. For the sea-level
/// baseline, either [`VerticalDistance::Gnd`] or `PressureAltitude(0)` may
/// be used.
///
/// # Example
///
/// ```
/// use efb::prelude::*;
///
/// // Sea-level baseline
/// let sl = CumulativeClimbDescentEntry {
///     level: VerticalDistance::Gnd,
///     time: Duration::m(0),
///     fuel: Volume::gal(0.0),
///     distance: Length::nm(0.0),
/// };
///
/// // At 2 000 ft pressure altitude
/// let entry = CumulativeClimbDescentEntry {
///     level: VerticalDistance::PressureAltitude(2_000),
///     time: Duration::m(4),
///     fuel: Volume::gal(0.9),
///     distance: Length::nm(5.0),
/// };
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CumulativeClimbDescentEntry {
    /// Altitude for this row, typically
    /// [`VerticalDistance::PressureAltitude`] as printed in the POH, or
    /// [`VerticalDistance::Gnd`] for the sea-level baseline.
    pub level: VerticalDistance,

    /// Cumulative time from the baseline altitude.
    pub time: Duration,

    /// Cumulative fuel consumed from the baseline altitude.
    ///
    /// Enter the volume as printed in the POH (e.g. `Volume::gal(0.9)`).
    /// The conversion to mass is handled by [`from_cumulative`] using the
    /// supplied [`FuelType`].
    ///
    /// [`from_cumulative`]: ClimbDescentPerformance::from_cumulative
    pub fuel: Volume,

    /// Cumulative still-air distance from the baseline altitude.
    pub distance: Length,
}

/// Aircraft climb or descent performance data.
///
/// Stores a table of [`ClimbDescentBand`]s describing aircraft performance
/// across altitude bands. The same type is used for both climb and descent;
/// typically an application creates two separate instances and passes them
/// into the [`FlightPlanningBuilder`].
///
/// Construct with [`new`](Self::new), [`from_fn`](Self::from_fn), or
/// [`from_cumulative`](Self::from_cumulative), then call
/// [`between`](Self::between) to obtain a [`ClimbDescentResult`].
/// Optionally correct the horizontal distance for wind with
/// [`ClimbDescentResult::with_wind`].
///
/// [`FlightPlanningBuilder`]: crate::fp::FlightPlanningBuilder
///
/// # Examples
///
/// Climb from sea level to FL 100 with wind correction:
///
/// ```
/// use efb::prelude::*;
///
/// let ff = FuelFlow::PerHour(Fuel::new(Mass::kg(15.0), FuelType::AvGas));
///
/// let climb = ClimbDescentPerformance::new(vec![
///     ClimbDescentBand {
///         level: VerticalDistance::Altitude(5_000),
///         tas: Speed::kt(80.0),
///         vertical_rate: VerticalRate::fpm(700.0),
///         ff,
///     },
///     ClimbDescentBand {
///         level: VerticalDistance::Altitude(10_000),
///         tas: Speed::kt(90.0),
///         vertical_rate: VerticalRate::fpm(500.0),
///         ff,
///     },
/// ]);
///
/// let result = climb
///     .between(&VerticalDistance::Gnd, &VerticalDistance::Fl(100))
///     .expect("valid altitude range");
///
/// // Apply 15 kt headwind to get ground distance
/// let corrected = result.with_wind(Speed::kt(15.0));
/// ```
///
/// Descent from FL 080 to 500 ft using [`from_fn`](Self::from_fn):
///
/// ```
/// # use efb::prelude::*;
/// let ff = FuelFlow::PerHour(Fuel::new(Mass::kg(10.0), FuelType::AvGas));
///
/// let descent = ClimbDescentPerformance::from_fn(
///     |_level| (Speed::kt(100.0), VerticalRate::fpm(500.0), ff),
///     VerticalDistance::Altitude(10_000),
/// );
///
/// // Destination at 500 ft, cruise at FL 080
/// let result = descent
///     .between(&VerticalDistance::Altitude(500), &VerticalDistance::Fl(80))
///     .unwrap();
/// ```
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ClimbDescentPerformance {
    table: Vec<ClimbDescentBand>,
}

/// The result of a [`ClimbDescentPerformance::between`] call.
///
/// Contains the aggregated time, fuel, and horizontal distance for the
/// altitude change. The horizontal distance is initially based on TAS alone;
/// call [`with_wind`] to obtain a wind-corrected copy.
///
/// [`with_wind`]: ClimbDescentResult::with_wind
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ClimbDescentResult {
    /// Total time spent climbing or descending.
    pub time: Duration,

    /// Total fuel burned during the climb or descent.
    pub fuel: Fuel,

    /// Horizontal distance covered during the climb or descent.
    ///
    /// Initially computed from TAS (still-air distance). After calling
    /// [`with_wind`], this is the wind-corrected ground distance.
    ///
    /// [`with_wind`]: ClimbDescentResult::with_wind
    pub horizontal_distance: Length,
}

impl ClimbDescentResult {
    /// Returns the result with wind corrected [horizontal distance].
    ///
    /// The ground distance is computed as `(TAS − headwind) × time`. If the
    /// headwind exceeds TAS (extreme case), the distance is clamped to zero.
    ///
    /// Headwind is the headwind component on the leg:
    /// - **positive** = headwind (reduces ground speed),
    /// - **negative** = tailwind (increases ground speed).
    ///
    /// [horizontal distance]: ClimbDescentResult::horizontal_distance
    pub fn with_wind(self, headwind: Speed) -> Self {
        // ground_distance = (TAS - headwind) * time
        //                 = TAS * time - headwind * time
        //                 = horizontal_distance - headwind * time
        let wind_correction = headwind * self.time;
        let ground_dist = self.horizontal_distance - wind_correction;
        // Clamp to zero for extreme headwind situations (headwind >= TAS)
        let ground_dist = if ground_dist < Length::nm(0.0) {
            Length::nm(0.0)
        } else {
            ground_dist
        };
        Self {
            horizontal_distance: ground_dist,
            ..self
        }
    }
}

/// Resolves a [`VerticalDistance`] to an [`Altitude`] at standard pressure and
/// sea-level elevation.
///
/// Returns `None` for [`VerticalDistance::Unlimited`].
fn to_altitude(vd: &VerticalDistance) -> Option<Altitude> {
    vd.to_msl(Pressure::STD, Length::ft(0.0))
}

impl ClimbDescentPerformance {
    /// Creates a performance table from a pre-built vector of rows.
    ///
    /// The rows **must** be sorted in ascending order by
    /// [`level`](ClimbDescentBand::level). No validation is
    /// performed; an unsorted table will produce incorrect results from
    /// [`between`](Self::between).
    pub fn new(table: Vec<ClimbDescentBand>) -> Self {
        Self { table }
    }

    /// Builds a performance table by sampling `f` in 1000 ft steps from
    /// ground up to `ceiling` (inclusive).
    ///
    /// The closure receives the [`VerticalDistance`] of each band boundary and
    /// returns `(tas, vertical_rate, fuel_flow)` for that band. The first
    /// invocation receives [GND]; subsequent ones receive
    /// 1000 ft, 2000 ft, etc.
    ///
    /// This is a convenience constructor for tables with uniform altitude
    /// spacing. For irregularly spaced bands or data taken directly from a
    /// POH, use [`new`](Self::new) instead.
    ///
    /// [GND]: `VerticalDistance::Gnd`
    pub fn from_fn<F>(f: F, ceiling: VerticalDistance) -> Self
    where
        F: Fn(&VerticalDistance) -> (Speed, VerticalRate, FuelFlow),
    {
        let mut table: Vec<ClimbDescentBand> = Vec::new();
        let mut vd = VerticalDistance::Gnd;
        let mut alt = 0u16;

        while vd <= ceiling {
            let (tas, vertical_rate, ff) = f(&vd);
            table.push(ClimbDescentBand {
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

    /// Builds a performance table from a cumulative "time, fuel, and distance
    /// to climb" table as found in most POH / AFM documents.
    ///
    /// The `entries` slice must contain at least two rows sorted in ascending
    /// [`level`](CumulativeClimbDescentEntry::level) order. The first entry
    /// is the baseline (typically sea level with all cumulative values at
    /// zero). Each subsequent entry is differenced against the previous one
    /// to derive the per-band rate of climb, TAS, and fuel flow that
    /// [`between`](Self::between) needs.
    ///
    /// `fuel_type` is needed to convert the POH's volumetric fuel figures
    /// into mass. Returns `None` if `entries` has fewer than two rows, any
    /// per-band Δtime is zero, or any level cannot be expressed in feet.
    ///
    /// # Derivation per band
    ///
    /// For two consecutive entries at altitudes *h₁* and *h₂*:
    ///
    /// - *Δalt* = *h₂* − *h₁* (ft)
    /// - *Δtime* = time(*h₂*) − time(*h₁*)
    /// - *Δfuel* = fuel(*h₂*) − fuel(*h₁*) (volume)
    /// - *Δdist* = dist(*h₂*) − dist(*h₁*)
    ///
    /// From these:
    ///
    /// - `vertical_rate` = Δalt / Δtime (ft/min)
    /// - `tas` = Δdist / Δtime
    /// - `ff` = Δfuel / Δtime (volume/h, converted to mass via `fuel_type`)
    ///
    /// # Examples
    ///
    /// The following shows how a PA-28-181 Archer II climb table might look like:
    ///
    /// ```
    /// use efb::prelude::*;
    ///
    /// let entries = [
    ///     CumulativeClimbDescentEntry {
    ///         level: VerticalDistance::Gnd,
    ///         time: Duration::m(0),
    ///         fuel: Volume::gal(0.0),
    ///         distance: Length::nm(0.0),
    ///     },
    ///     CumulativeClimbDescentEntry {
    ///         level: VerticalDistance::PressureAltitude(2_000),
    ///         time: Duration::m(4),
    ///         fuel: Volume::gal(0.9),
    ///         distance: Length::nm(5.0),
    ///     },
    ///     CumulativeClimbDescentEntry {
    ///         level: VerticalDistance::PressureAltitude(4_000),
    ///         time: Duration::m(8),
    ///         fuel: Volume::gal(1.8),
    ///         distance: Length::nm(11.0),
    ///     },
    ///     CumulativeClimbDescentEntry {
    ///         level: VerticalDistance::PressureAltitude(6_000),
    ///         time: Duration::m(13),
    ///         fuel: Volume::gal(2.9),
    ///         distance: Length::nm(18.0),
    ///     },
    ///     CumulativeClimbDescentEntry {
    ///         level: VerticalDistance::PressureAltitude(8_000),
    ///         time: Duration::m(18),
    ///         fuel: Volume::gal(4.1),
    ///         distance: Length::nm(27.0),
    ///     },
    /// ];
    ///
    /// let climb = ClimbDescentPerformance::from_cumulative(
    ///     &entries,
    ///     FuelType::AvGas,
    /// ).expect("valid table");
    ///
    /// // Now use it to compute a climb from field elevation to cruise
    /// let result = climb.between(
    ///     &VerticalDistance::Gnd,
    ///     &VerticalDistance::PressureAltitude(6_000),
    /// );
    /// assert!(result.is_some());
    /// ```
    pub fn from_cumulative(
        entries: &[CumulativeClimbDescentEntry],
        fuel_type: FuelType,
    ) -> Option<Self> {
        if entries.len() < 2 {
            return None;
        }

        let mut table: Vec<ClimbDescentBand> = Vec::with_capacity(entries.len() - 1);

        for pair in entries.windows(2) {
            let prev = &pair[0];
            let cur = &pair[1];

            let delta_alt = to_altitude(&cur.level)? - to_altitude(&prev.level)?;
            let delta_time = cur.time - prev.time;

            if *delta_time.value() == 0 {
                return None;
            }

            // TAS = Δdist / Δtime (Length / Duration = Speed)
            let tas = (cur.distance - prev.distance) / delta_time;

            // vertical_rate = Δalt / Δtime (Altitude / Duration = VerticalRate)
            let vertical_rate = delta_alt / delta_time;

            // fuel_flow = Δfuel / Δtime, scaled to per-hour
            let delta_fuel = cur.fuel - prev.fuel;
            let ff = Fuel::from_volume(delta_fuel, fuel_type) / delta_time;

            table.push(ClimbDescentBand {
                level: cur.level,
                tas,
                vertical_rate,
                ff,
            });
        }

        Some(Self { table })
    }

    /// Returns the performance row applicable at `level`.
    ///
    /// Uses a reverse-find to return the row with the highest level that is
    /// less than or equal to the target. Does not interpolate.
    ///
    /// # Panics
    ///
    /// Panics if the table is empty.
    fn at_level(&self, level: &VerticalDistance) -> &ClimbDescentBand {
        self.table
            .iter()
            .rfind(|row| &row.level <= level)
            .expect("climb/descent performance table must not be empty")
    }

    /// Returns the time, fuel, and horizontal distance for a climb or descent
    /// between two altitude levels.
    ///
    /// The computation walks the performance table band by band, applying the
    /// appropriate row for each altitude segment within `[from_level,
    /// to_level]`. If `to_level` exceeds the highest row in the table, the
    /// topmost row's performance is extrapolated for the remaining altitude.
    ///
    /// For **descent** planning, pass the *lower* altitude (destination
    /// elevation) as `from_level` and the *higher* altitude (cruise level) as
    /// `to_level`. The resulting fuel and time represent the descent through
    /// those bands.
    ///
    /// Returns `None` if `from_level >= to_level`, the table is empty, or
    /// either level cannot be expressed in feet (e.g. [AGL], [Unlimited]).
    ///
    /// [AGL]: `VerticalDistance::Agl`
    /// [Unlimited]: `VerticalDistance::Unlimited`
    pub fn between(
        &self,
        from_level: &VerticalDistance,
        to_level: &VerticalDistance,
    ) -> Option<ClimbDescentResult> {
        if from_level >= to_level || self.table.is_empty() {
            return None;
        }

        let from_alt = to_altitude(from_level)?;
        let to_alt = to_altitude(to_level)?;

        let mut band_floor = from_alt;
        let mut total_time = Duration::s(0);
        let mut accumulated_fuel: Option<Fuel> = None;
        let mut total_dist = Length::m(0.0);

        let mut accumulate = |row: &ClimbDescentBand, delta_alt: Altitude| {
            let time = delta_alt / row.vertical_rate;
            let fuel = row.ff * time;

            total_time = total_time + time;
            accumulated_fuel = Some(match accumulated_fuel {
                Some(f) => f + fuel,
                None => fuel,
            });
            total_dist = total_dist + row.tas * time;
        };

        // Walk the table to cover all bands within (from_alt, to_alt]
        for row in &self.table {
            let row_alt = to_altitude(&row.level)?;

            if row_alt <= band_floor {
                continue; // this band is below our starting altitude
            }

            let band_ceiling = if row_alt < to_alt { row_alt } else { to_alt };
            let delta_alt = band_ceiling - band_floor;

            accumulate(row, delta_alt);
            band_floor = band_ceiling;

            if band_floor >= to_alt {
                break;
            }
        }

        // If to_alt exceeds all table rows, apply the highest row for the tail
        if band_floor < to_alt {
            let row = self.at_level(to_level);
            let delta_alt = to_alt - band_floor;
            accumulate(row, delta_alt);
        }

        let fuel = accumulated_fuel?;

        Some(ClimbDescentResult {
            time: total_time,
            fuel,
            horizontal_distance: total_dist,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurements::{LengthUnit, SpeedUnit, VerticalRateUnit};
    use crate::FuelType;

    fn avgas_ff(lph: f32) -> FuelFlow {
        FuelFlow::PerHour(avgas!(Volume::l(lph)))
    }

    fn simple_table() -> ClimbDescentPerformance {
        // Two-band table: ground→2000 ft and 2000→4000 ft
        ClimbDescentPerformance::new(vec![
            ClimbDescentBand {
                level: VerticalDistance::Altitude(2000),
                tas: Speed::kt(80.0),
                vertical_rate: VerticalRate::fpm(800.0),
                ff: avgas_ff(20.0),
            },
            ClimbDescentBand {
                level: VerticalDistance::Altitude(4000),
                tas: Speed::kt(85.0),
                vertical_rate: VerticalRate::fpm(600.0),
                ff: avgas_ff(18.0),
            },
        ])
    }

    #[test]
    fn between_single_band() {
        let perf = simple_table();
        // Climb from 0 to 2000 ft: 2000 ft / 800 fpm = 2.5 min = 150 s
        let result = perf
            .between(&VerticalDistance::Gnd, &VerticalDistance::Altitude(2000))
            .expect("should produce a result");

        assert_eq!(*result.time.value(), 150, "time should be 150 s");
        // Horizontal distance: 80 kt * (2.5/60) h ≈ 3.33 NM
        let dist_nm = *result
            .horizontal_distance
            .convert_to(LengthUnit::NauticalMiles)
            .value();
        assert!(
            (dist_nm - 3.333).abs() < 0.05,
            "distance ~3.33 NM, got {dist_nm}"
        );
    }

    #[test]
    fn between_two_bands() {
        let perf = simple_table();
        // Climb from 0 to 4000 ft:
        //   Band 0→2000: 2000 / 800 = 2.5 min = 150 s
        //   Band 2000→4000: 2000 / 600 = 3.33 min = 200 s
        //   Total: 350 s
        let result = perf
            .between(&VerticalDistance::Gnd, &VerticalDistance::Altitude(4000))
            .expect("should produce a result");

        assert_eq!(*result.time.value(), 350, "time should be 350 s");
    }

    #[test]
    fn from_to_equal_returns_none() {
        let perf = simple_table();
        let result = perf.between(
            &VerticalDistance::Altitude(2000),
            &VerticalDistance::Altitude(2000),
        );
        assert!(result.is_none());
    }

    #[test]
    fn from_above_to_returns_none() {
        let perf = simple_table();
        let result = perf.between(
            &VerticalDistance::Altitude(3000),
            &VerticalDistance::Altitude(1000),
        );
        assert!(result.is_none());
    }

    #[test]
    fn with_wind_reduces_distance_for_headwind() {
        let perf = simple_table();
        let result = perf
            .between(&VerticalDistance::Gnd, &VerticalDistance::Altitude(2000))
            .unwrap();
        let no_wind_dist = *result
            .horizontal_distance
            .convert_to(LengthUnit::NauticalMiles)
            .value();
        let with_headwind = result.with_wind(Speed::kt(20.0));
        let headwind_dist = *with_headwind
            .horizontal_distance
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

    // --- from_cumulative tests ---

    fn pa28_cumulative_entries() -> Vec<CumulativeClimbDescentEntry> {
        // Approximate PA-28-181 Archer II climb data (ISA, gross weight)
        vec![
            CumulativeClimbDescentEntry {
                level: VerticalDistance::Gnd,
                time: Duration::m(0),
                fuel: Volume::gal(0.0),
                distance: Length::nm(0.0),
            },
            CumulativeClimbDescentEntry {
                level: VerticalDistance::PressureAltitude(2_000),
                time: Duration::m(4),
                fuel: Volume::gal(0.9),
                distance: Length::nm(5.0),
            },
            CumulativeClimbDescentEntry {
                level: VerticalDistance::PressureAltitude(4_000),
                time: Duration::m(8),
                fuel: Volume::gal(1.8),
                distance: Length::nm(11.0),
            },
            CumulativeClimbDescentEntry {
                level: VerticalDistance::PressureAltitude(6_000),
                time: Duration::m(13),
                fuel: Volume::gal(2.9),
                distance: Length::nm(18.0),
            },
            CumulativeClimbDescentEntry {
                level: VerticalDistance::PressureAltitude(8_000),
                time: Duration::m(18),
                fuel: Volume::gal(4.1),
                distance: Length::nm(27.0),
            },
        ]
    }

    #[test]
    fn from_cumulative_builds_correct_table_size() {
        let entries = pa28_cumulative_entries();
        let perf = ClimbDescentPerformance::from_cumulative(&entries, FuelType::AvGas)
            .expect("valid table");
        // 5 entries → 4 bands (windows of 2)
        assert_eq!(perf.table.len(), 4);
    }

    #[test]
    fn from_cumulative_derives_correct_vertical_rate() {
        let entries = pa28_cumulative_entries();
        let perf = ClimbDescentPerformance::from_cumulative(&entries, FuelType::AvGas).unwrap();

        // Band 0→2000: 2000 ft / 4 min = 500 fpm
        let row0 = &perf.table[0];
        let roc_fpm = *row0
            .vertical_rate
            .convert_to(VerticalRateUnit::FeetPerMinute)
            .value();
        assert!(
            (roc_fpm - 500.0).abs() < 1.0,
            "first band RoC should be ~500 fpm, got {roc_fpm}"
        );

        // Band 2000→4000: 2000 ft / 4 min = 500 fpm
        let row1 = &perf.table[1];
        let roc_fpm = *row1
            .vertical_rate
            .convert_to(VerticalRateUnit::FeetPerMinute)
            .value();
        assert!(
            (roc_fpm - 500.0).abs() < 1.0,
            "second band RoC should be ~500 fpm, got {roc_fpm}"
        );

        // Band 4000→6000: 2000 ft / 5 min = 400 fpm
        let row2 = &perf.table[2];
        let roc_fpm = *row2
            .vertical_rate
            .convert_to(VerticalRateUnit::FeetPerMinute)
            .value();
        assert!(
            (roc_fpm - 400.0).abs() < 1.0,
            "third band RoC should be ~400 fpm, got {roc_fpm}"
        );
    }

    #[test]
    fn from_cumulative_derives_correct_tas() {
        let entries = pa28_cumulative_entries();
        let perf = ClimbDescentPerformance::from_cumulative(&entries, FuelType::AvGas).unwrap();

        // Band 0→2000: 5 NM / 4 min * 60 = 75 kt
        let row0 = &perf.table[0];
        let tas_kt = *row0.tas.convert_to(SpeedUnit::Knots).value();
        assert!(
            (tas_kt - 75.0).abs() < 0.5,
            "first band TAS should be ~75 kt, got {tas_kt}"
        );

        // Band 4000→6000: 7 NM / 5 min * 60 = 84 kt
        let row2 = &perf.table[2];
        let tas_kt = *row2.tas.convert_to(SpeedUnit::Knots).value();
        assert!(
            (tas_kt - 84.0).abs() < 0.5,
            "third band TAS should be ~84 kt, got {tas_kt}"
        );
    }

    #[test]
    fn from_cumulative_matches_poh_totals() {
        let entries = pa28_cumulative_entries();
        let perf = ClimbDescentPerformance::from_cumulative(&entries, FuelType::AvGas).unwrap();

        // Compute climb from ground to 8000 ft PA
        let result = perf
            .between(
                &VerticalDistance::Gnd,
                &VerticalDistance::PressureAltitude(8_000),
            )
            .expect("should produce a result");

        // POH says 18 min total → 1080 s
        let time_s = *result.time.value();
        assert!(
            (time_s as f32 - 1080.0).abs() < 30.0,
            "total time should be ~1080 s, got {time_s}"
        );

        // POH says 27 NM total distance
        let dist_nm = *result
            .horizontal_distance
            .convert_to(LengthUnit::NauticalMiles)
            .value();
        assert!(
            (dist_nm - 27.0).abs() < 1.0,
            "total distance should be ~27 NM, got {dist_nm}"
        );
    }

    #[test]
    fn from_cumulative_partial_climb() {
        let entries = pa28_cumulative_entries();
        let perf = ClimbDescentPerformance::from_cumulative(&entries, FuelType::AvGas).unwrap();

        // Climb from ground to 4000 ft PA only (first two bands)
        let result = perf
            .between(
                &VerticalDistance::Gnd,
                &VerticalDistance::PressureAltitude(4_000),
            )
            .expect("should produce a result");

        // POH says 8 min → 480 s
        let time_s = *result.time.value();
        assert!(
            (time_s as f32 - 480.0).abs() < 10.0,
            "time to 4000 ft should be ~480 s, got {time_s}"
        );
    }

    #[test]
    fn from_cumulative_too_few_entries_returns_none() {
        let entries = [CumulativeClimbDescentEntry {
            level: VerticalDistance::Gnd,
            time: Duration::m(0),
            fuel: Volume::gal(0.0),
            distance: Length::nm(0.0),
        }];
        assert!(ClimbDescentPerformance::from_cumulative(&entries, FuelType::AvGas).is_none());
    }

    #[test]
    fn from_cumulative_empty_returns_none() {
        assert!(ClimbDescentPerformance::from_cumulative(&[], FuelType::AvGas).is_none());
    }
}
