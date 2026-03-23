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

use super::{ClimbDescentPerformance, Performance};

/// Bundles cruise, climb, and descent performance for per-leg fuel calculation.
///
/// The performance is used to calculate [fuel] per leg and [accumulated] with
/// climb/descent fuel for level transitions and cruise fuel.
///
/// [fuel]: crate::route::Leg::fuel
/// [accumulated]: crate::route::Route::accumulate_legs
pub struct LegPerformance<'a> {
    cruise: Option<&'a Performance>,
    climb: Option<&'a ClimbDescentPerformance>,
    descent: Option<&'a ClimbDescentPerformance>,
}

impl<'a> LegPerformance<'a> {
    /// Creates a new leg performance bundle from optional cruise, climb, and
    /// descent performance data.
    pub fn new(
        cruise: Option<&'a Performance>,
        climb: Option<&'a ClimbDescentPerformance>,
        descent: Option<&'a ClimbDescentPerformance>,
    ) -> Self {
        Self {
            cruise,
            climb,
            descent,
        }
    }

    pub fn cruise(&self) -> Option<&Performance> {
        self.cruise
    }

    pub fn climb(&self) -> Option<&ClimbDescentPerformance> {
        self.climb
    }

    pub fn descent(&self) -> Option<&ClimbDescentPerformance> {
        self.descent
    }
}
