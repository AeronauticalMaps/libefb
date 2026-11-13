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

use std::collections::HashMap;
use std::ops::{Div, RangeToInclusive};

use crate::core::VerticalDistance;
use crate::measurements::{Mass, Speed};
use crate::nd::{RunwayConditionCode, RunwaySurface};

use super::Influences;

/// The factor of an effect `T`.
///
/// Factors the alter the takeoff or landing performance depend on effects that
/// are varying like the fields elevation or temperature. Aircraft manufactures
/// provide these factors in different ways. For example, the factor for a
/// headwind influence might be provided as a rate in relation to the
/// wind. Another example is the influence of the field elevation where the
/// factor might be provided for ranges.
///
/// This type tries to provide the different kinds of factors that you might
/// find in a POH for different effects.
///
/// # Examples
///
/// Basic usage examples:
///
/// ```
/// # use efb::fp::FactorOfEffect;
/// # use efb::measurements::Speed;
/// // decrease by 10% for each 9kt of e.g. headwind
/// let factor = FactorOfEffect::Rate { numerator: 0.1, denominator: Speed::kt(9.0) };
/// // for 18kt we get a factor of 20%
/// assert_eq!(factor.factor(Speed::kt(18.0)), 0.2);
/// ```
#[derive(Clone, PartialEq, Debug)]
pub enum FactorOfEffect<T>
where
    T: Into<f32>,
    T: Div<T, Output = f32>,
{
    /// Factor for an effect in a range where `effect <= end`.
    Range(Vec<(RangeToInclusive<T>, f32)>),

    /// A factor that changes in the rate `numerator / denominator`.
    Rate { numerator: f32, denominator: T },
}

impl<T> FactorOfEffect<T>
where
    T: Into<f32>,
    T: Div<T, Output = f32>,
    T: PartialOrd,
{
    /// Returns the factor by which the ground roll should be multiplied for a
    /// given effect of type `T`.
    pub fn factor(self, effect: T) -> f32 {
        match self {
            Self::Range(ranges) => ranges
                .iter()
                .find(|range| range.0.contains(&effect))
                .map(|range| range.1)
                .expect("Range should be fully covered."),
            Self::Rate {
                numerator,
                denominator,
            } => effect / denominator * numerator,
        }
    }
}

/// A factor to in- or decrease the takeoff or landing distance.
///
/// The factor is applied to the ground roll and distance to clear a 50ft
/// obstacle depending on the influences affecting the takeoff or landing.
#[derive(Clone, PartialEq, Debug)]
pub enum AlteringFactor {
    DecreaseHeadwind(FactorOfEffect<Speed>),
    IncreaseTailwind(FactorOfEffect<Speed>),
    IncreaseAltitude(FactorOfEffect<VerticalDistance>),
    IncreaseRWYCC(HashMap<(Option<RunwayConditionCode>, Option<RunwaySurface>), f32>),
    RunwaySlope(FactorOfEffect<f32>),
    Mass(FactorOfEffect<Mass>),
}

impl AlteringFactor {
    /// Returns the factor altering the ground roll for some influences.
    pub fn ground_roll_factor(&self, influences: &Influences) -> f32 {
        match self {
            Self::DecreaseHeadwind(f) => {
                if influences.headwind() > &Speed::kt(0.0) {
                    1.0 - f.clone().factor(*influences.headwind())
                } else {
                    1.0
                }
            }
            Self::IncreaseTailwind(f) => {
                if influences.headwind() < &Speed::kt(0.0) {
                    1.0 + f.clone().factor(*influences.headwind() * -1.0)
                } else {
                    1.0
                }
            }
            Self::IncreaseAltitude(f) => 1.0 + f.clone().factor(*influences.level()),
            Self::IncreaseRWYCC(map) => {
                let surface = influences.surface().copied();
                let rwycc_and_surface = (Some(*influences.rwycc()), surface);
                let rwycc_any_surface = (Some(*influences.rwycc()), None::<RunwaySurface>);
                let any_rwycc_and_surface =
                    surface.map(|surface| (None::<RunwayConditionCode>, Some(surface)));

                let f = map
                    .get(&rwycc_and_surface)
                    .or_else(|| map.get(&rwycc_any_surface))
                    .or_else(|| any_rwycc_and_surface.and_then(|key| map.get(&key)))
                    .unwrap_or(&0.0);

                1.0 + f
            }
            Self::RunwaySlope(f) => influences
                .slope()
                .map_or(1.0, |slope| 1.0 + f.clone().factor(*slope)),
            Self::Mass(f) => 1.0 + f.clone().factor(*influences.mass()),
        }
    }

    /// Returns the factor altering the distance to clear a 50ft obstacle for
    /// some influences.
    pub fn clear_obstacle_factor(&self, influences: &Influences) -> f32 {
        match self {
            Self::DecreaseHeadwind(_)
            | Self::IncreaseTailwind(_)
            | Self::Mass(_)
            | Self::IncreaseAltitude(_) => self.ground_roll_factor(influences),
            _ => 1.0, // other factors only alter the ground roll
        }
    }
}

/// The product of all takeoff or landing altering factors.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AlteringFactors {
    factors: Vec<AlteringFactor>,
}

impl AlteringFactors {
    /// Creates a new product of altering factors.
    pub fn new<I>(factors: I) -> Self
    where
        I: IntoIterator<Item = AlteringFactor>,
    {
        Self {
            factors: factors.into_iter().collect(),
        }
    }

    /// Returns the product of all factor altering the ground roll.
    pub fn ground_roll_factor(&self, influences: &Influences) -> f32 {
        self.factors
            .iter()
            .map(|factor| factor.ground_roll_factor(influences))
            .reduce(|acc, factor| acc * factor)
            .unwrap_or(1.0)
    }

    /// Returns the product of all factor altering the distance to clear a 50ft
    /// obstacle.
    pub fn clear_obstacle_factor(&self, influences: &Influences) -> f32 {
        self.factors
            .iter()
            .map(|factor| factor.clear_obstacle_factor(influences))
            .reduce(|acc, factor| acc * factor)
            .unwrap_or(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factor_for_range() {
        let factor = FactorOfEffect::Range(vec![
            (..=VerticalDistance::PressureAltitude(1000), 0.1),
            (..=VerticalDistance::PressureAltitude(3000), 0.13),
            (..=VerticalDistance::Unlimited, 0.18),
        ]);

        assert!(0.1 - factor.clone().factor(VerticalDistance::Gnd) <= f32::EPSILON);
        assert!(
            0.13 - factor
                .clone()
                .factor(VerticalDistance::PressureAltitude(2000))
                <= f32::EPSILON
        );
        assert!(
            0.18 - factor
                .clone()
                .factor(VerticalDistance::PressureAltitude(4000))
                <= f32::EPSILON
        );
    }

    #[test]
    fn factor_changes_with_rate() {
        let factor = FactorOfEffect::Rate {
            numerator: 0.1,
            denominator: Speed::kt(9.0),
        };
        assert_eq!(0.2, factor.factor(Speed::kt(18.0)));
    }
}
