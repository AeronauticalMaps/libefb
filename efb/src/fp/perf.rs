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

use crate::measurements::Speed;
use crate::{FuelFlow, VerticalDistance};

/// A row of the performance table presenting a performance up to a specific
/// level.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PerformanceTableRow {
    pub level: VerticalDistance,
    pub tas: Speed,
    pub ff: FuelFlow,
}

/// A table of performance values at a specified level.
pub type PerformanceTable = Vec<PerformanceTableRow>;

/// Used to provide [Speed] or [FuelFlow] for a defined performance setting at
/// different level.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Performance {
    table: PerformanceTable,
}

impl Performance {
    pub fn new(table: PerformanceTable) -> Self {
        Self { table }
    }

    /// Creates the performance profile from a function.
    ///
    /// The function `f` is called in 1000 ft intervals up to the ceiling.
    pub fn from_fn<F>(f: F, ceiling: VerticalDistance) -> Self
    where
        F: Fn(&VerticalDistance) -> (Speed, FuelFlow),
    {
        let mut table: PerformanceTable = Vec::new();
        let mut vd = VerticalDistance::Gnd;
        let mut alt = 0;

        while vd <= ceiling {
            let (tas, ff) = f(&vd);
            table.push(PerformanceTableRow { level: vd, tas, ff });

            alt += 1000;
            vd = VerticalDistance::Altitude(alt);
        }

        Self { table }
    }

    /// Returns the true airspeed at a level.
    pub fn tas(&self, level: &VerticalDistance) -> Speed {
        self.at_level(level).tas
    }

    /// Returns the fuel flow at a level.
    pub fn ff(&self, level: &VerticalDistance) -> FuelFlow {
        self.at_level(level).ff
    }

    /// Returns the performance at a level.
    ///
    /// Uses reverse find to return the performance at the highest level that is
    /// less than or equal to the target level. Does not interpolate between levels.
    ///
    /// # Panics
    ///
    /// Panics if the table is empty.
    fn at_level(&self, level: &VerticalDistance) -> &PerformanceTableRow {
        self.table
            .iter()
            .rfind(|row| &row.level <= level)
            .expect("There should be at least one row in the table.")
    }
}

// TODO: Add unit tests!
