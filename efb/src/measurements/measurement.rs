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

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::UnitOfMeasure;

/// A measurement of a physical quantity.
///
/// The measurement has a value of type `T` and a unit `U` that implements a
/// [`UnitOfMeasure`]. For measurements of the same unit the operator `+`, `-`,
/// `*` and `/` are implemented. Differing units that have a value in a third
/// unit as result if divided or multiplied (e.g. length divided by duration is
/// speed) can implement those operations.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub struct Measurement<T, U>
where
    U: UnitOfMeasure<T>,
{
    pub(super) value: T,
    pub(super) unit: U,
}

impl<U> Hash for Measurement<f32, U>
where
    U: UnitOfMeasure<f32> + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
        self.unit.hash(state);
    }
}

impl<T, U> Measurement<T, U>
where
    U: UnitOfMeasure<T>,
{
    /// Create new measurement from a value and unit.
    pub fn new(value: T, unit: U) -> Self {
        Self { value, unit }
    }

    /// The measure's value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// The measure's unit.
    pub fn unit(&self) -> &U {
        &self.unit
    }

    /// The measure's unit symbol e.g. `m` for meters..
    pub fn symbol(&self) -> &str {
        self.unit.symbol()
    }

    /// Converts a measurement's value which is in its SI unit to a measurement
    /// with the specified unit.
    pub fn from_si(value: T, unit: U) -> Self {
        Self {
            value: U::from_si(value, &unit),
            unit,
        }
    }

    /// Converts to a measurement in SI unit.
    pub fn to_si(&self) -> T {
        self.unit.to_si(&self.value)
    }

    /// Converts to a measurement in another unit.
    pub fn convert_to(&self, other: U) -> Self {
        Self {
            value: self.unit.convert_to(&self.value, &other),
            unit: other,
        }
    }
}

macro_rules! abs_impl {
    ($($t:ty)*) => ($(
        impl<U> Measurement<$t, U>
        where
            U: UnitOfMeasure<$t> + Copy,
        {
            /// Returns the absolute value of the measurement.
            pub fn abs(&self) -> Self {
                Self {
                    value: self.value.abs(),
                    unit: self.unit,
                }
            }
        }
    )*)
}

abs_impl! { f32 f64 i8 i16 i32 i64 }

impl<T, U> Default for Measurement<T, U>
where
    T: Default,
    U: UnitOfMeasure<T>,
{
    fn default() -> Self {
        Self::new(T::default(), U::si())
    }
}

impl<T, U> fmt::Display for Measurement<T, U>
where
    T: std::fmt::Display,
    U: UnitOfMeasure<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tmp = if let Some(precision) = f.precision() {
            format!("{:.precision$} {}", self.value, self.symbol())
        } else {
            format!("{} {}", self.value, self.symbol())
        };

        f.pad_integral(true, "", &tmp)
    }
}

impl<T, U> PartialOrd for Measurement<T, U>
where
    T: PartialOrd,
    U: UnitOfMeasure<T>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_si().partial_cmp(&other.to_si())
    }
}

impl<T, U> Ord for Measurement<T, U>
where
    T: Ord,
    U: UnitOfMeasure<T>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_si().cmp(&other.to_si())
    }
}

impl<T, U> PartialEq for Measurement<T, U>
where
    T: PartialEq,
    U: UnitOfMeasure<T>,
{
    /// Compares the measurement's SI value.
    fn eq(&self, other: &Self) -> bool {
        self.to_si() == other.to_si()
    }
}

impl<T, U> Eq for Measurement<T, U>
where
    T: PartialEq,
    U: UnitOfMeasure<T>,
{
}

impl<T, U> Add for Measurement<T, U>
where
    T: Add<Output = T>,
    U: UnitOfMeasure<T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_si(self.to_si() + rhs.to_si(), self.unit)
    }
}

impl<T, U> Sub for Measurement<T, U>
where
    T: Sub<Output = T>,
    U: UnitOfMeasure<T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_si(self.to_si() - rhs.to_si(), self.unit)
    }
}

impl<T, U> Mul for Measurement<T, U>
where
    T: Mul<Output = T>,
    U: UnitOfMeasure<T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_si(self.to_si() * rhs.to_si(), self.unit)
    }
}

impl<T, U> Mul<T> for Measurement<T, U>
where
    T: Mul<Output = T>,
    U: UnitOfMeasure<T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::from_si(self.to_si() * rhs, self.unit)
    }
}

impl<T, U> Div for Measurement<T, U>
where
    T: Div<Output = T>,
    U: UnitOfMeasure<T>,
{
    type Output = T;

    fn div(self, rhs: Self) -> Self::Output {
        self.to_si() / rhs.to_si()
    }
}

impl<T, U> Div<T> for Measurement<T, U>
where
    T: Div<Output = T>,
    U: UnitOfMeasure<T>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::from_si(self.to_si() / rhs, self.unit)
    }
}

impl<T, U> Sum for Measurement<T, U>
where
    T: Default + Add<Output = T>,
    U: UnitOfMeasure<T>,
{
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        match iter.next() {
            Some(first) => iter.fold(first, |acc, x| acc + x),
            None => Self::default(),
        }
    }
}

impl<T, U> From<Measurement<T, U>> for f32
where
    T: Into<f32>,
    U: UnitOfMeasure<T>,
{
    fn from(value: Measurement<T, U>) -> Self {
        value.to_si().into()
    }
}
