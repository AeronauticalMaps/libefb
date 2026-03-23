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

use std::ops::Add;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Fuel;

/// Per-phase fuel breakdown for a leg.
///
/// Splits the fuel consumed on a leg into climb, cruise, and descent
/// components. A phase is [`None`] when the leg does not include that phase
/// (e.g. a leg that is entirely a climb has no cruise component).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LegFuel {
    climb: Option<Fuel>,
    cruise: Option<Fuel>,
    descent: Option<Fuel>,
    total: Fuel,
}

impl LegFuel {
    /// Creates a new `LegFuel` from optional per-phase components.
    ///
    /// The total is computed as the sum of all present components.
    ///
    /// # Panics
    ///
    /// Panics if all three components are [`None`] — at least one phase must
    /// have fuel.
    pub fn new(climb: Option<Fuel>, cruise: Option<Fuel>, descent: Option<Fuel>) -> Self {
        let total = [climb, cruise, descent]
            .into_iter()
            .flatten()
            .reduce(|a, b| a + b)
            .expect("at least one fuel component must be Some");

        Self {
            climb,
            cruise,
            descent,
            total,
        }
    }

    /// The climb fuel, or [`None`] if the leg has no climb phase.
    pub fn climb(&self) -> Option<&Fuel> {
        self.climb.as_ref()
    }

    /// The cruise fuel, or [`None`] if the leg has no cruise phase.
    pub fn cruise(&self) -> Option<&Fuel> {
        self.cruise.as_ref()
    }

    /// The descent fuel, or [`None`] if the leg has no descent phase.
    pub fn descent(&self) -> Option<&Fuel> {
        self.descent.as_ref()
    }

    /// The total fuel consumed on the leg.
    pub fn total(&self) -> &Fuel {
        &self.total
    }
}

/// Merges two `LegFuel` values by adding matching phases.
///
/// For each phase: `Some + Some = Some(sum)`, `Some + None = Some`,
/// `None + None = None`. Totals are added directly.
impl Add for LegFuel {
    type Output = LegFuel;

    fn add(self, rhs: Self) -> Self::Output {
        fn merge(a: Option<Fuel>, b: Option<Fuel>) -> Option<Fuel> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            }
        }

        Self {
            climb: merge(self.climb, rhs.climb),
            cruise: merge(self.cruise, rhs.cruise),
            descent: merge(self.descent, rhs.descent),
            total: self.total + rhs.total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurements::Mass;
    use crate::FuelType;

    fn fuel(kg: f32) -> Fuel {
        Fuel::new(Mass::kg(kg), FuelType::AvGas)
    }

    #[test]
    fn total_all_phases() {
        let lf = LegFuel::new(Some(fuel(2.0)), Some(fuel(5.0)), Some(fuel(1.0)));
        assert_eq!(*lf.total(), fuel(8.0));
    }

    #[test]
    fn total_climb_only() {
        let lf = LegFuel::new(Some(fuel(3.0)), None, None);
        assert_eq!(*lf.total(), fuel(3.0));
    }

    #[test]
    fn total_cruise_only() {
        let lf = LegFuel::new(None, Some(fuel(4.0)), None);
        assert_eq!(*lf.total(), fuel(4.0));
    }

    #[test]
    fn total_climb_and_descent() {
        let lf = LegFuel::new(Some(fuel(2.0)), None, Some(fuel(1.5)));
        assert_eq!(*lf.total(), fuel(3.5));
    }

    #[test]
    fn add_some_some() {
        let a = LegFuel::new(Some(fuel(2.0)), Some(fuel(5.0)), Some(fuel(1.0)));
        let b = LegFuel::new(Some(fuel(1.0)), Some(fuel(3.0)), Some(fuel(0.5)));
        let sum = a + b;
        assert_eq!(sum.climb(), Some(&fuel(3.0)));
        assert_eq!(sum.cruise(), Some(&fuel(8.0)));
        assert_eq!(sum.descent(), Some(&fuel(1.5)));
        assert_eq!(*sum.total(), fuel(12.5));
    }

    #[test]
    fn add_some_none() {
        let a = LegFuel::new(Some(fuel(2.0)), Some(fuel(5.0)), None);
        let b = LegFuel::new(None, Some(fuel(3.0)), Some(fuel(1.0)));
        let sum = a + b;
        assert_eq!(sum.climb(), Some(&fuel(2.0)));
        assert_eq!(sum.cruise(), Some(&fuel(8.0)));
        assert_eq!(sum.descent(), Some(&fuel(1.0)));
        assert_eq!(*sum.total(), fuel(11.0));
    }

    #[test]
    fn add_none_none() {
        let a = LegFuel::new(None, Some(fuel(5.0)), None);
        let b = LegFuel::new(None, Some(fuel(3.0)), None);
        let sum = a + b;
        assert_eq!(sum.climb(), None);
        assert_eq!(sum.cruise(), Some(&fuel(8.0)));
        assert_eq!(sum.descent(), None);
        assert_eq!(*sum.total(), fuel(8.0));
    }
}
