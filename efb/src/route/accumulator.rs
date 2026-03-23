// SPDX-License-Identifier: Apache-2.0
// Copyright 2025, 2026 Joe Pearson
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

use crate::fp::LegPerformance;
use crate::measurements::{Duration, Length};
use super::{Leg, LegFuel};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Cumulative totals up to a leg.
pub struct TotalsToLeg {
    dist: Length,
    ete: Option<Duration>,
    fuel: Option<LegFuel>,
}

impl TotalsToLeg {
    /// Creates totals for the first leg.
    ///
    /// Accumulates the fuel if the performance is [`Some`].
    pub fn new(leg: &Leg, perf: Option<&LegPerformance>) -> Self {
        Self {
            dist: *leg.dist(),
            ete: leg.ete().cloned(),
            fuel: perf.and_then(|p| leg.fuel(p)),
        }
    }

    /// Creates totals that add the current leg to previous totals.
    ///
    /// Accumulates the fuel if the performance is [`Some`].
    pub fn accumulate(&self, leg: &Leg, perf: Option<&LegPerformance>) -> Self {
        let ete = match (self.ete, leg.ete()) {
            (Some(a), Some(b)) => Some(a + *b),
            _ => None,
        };

        let fuel = match (self.fuel, perf) {
            (Some(a), Some(perf)) => leg.fuel(perf).map(|b| a + b),
            _ => None,
        };

        Self {
            dist: self.dist + *leg.dist(),
            ete,
            fuel,
        }
    }

    /// The cumulative distance.
    pub fn dist(&self) -> &Length {
        &self.dist
    }

    /// The cumulative ETE or [`None`] if ETE is missing for any leg.
    pub fn ete(&self) -> Option<&Duration> {
        self.ete.as_ref()
    }

    /// The cumulative fuel or [`None`] if fuel is missing for any leg.
    pub fn fuel(&self) -> Option<&LegFuel> {
        self.fuel.as_ref()
    }
}
