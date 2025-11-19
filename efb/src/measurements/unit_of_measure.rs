// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Joe Pearson
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

/// Physical quantities that can be measured with units.
pub enum PhysicalQuantity {
    Angle,
    Length,
    Density,
    Duration,
    Mass,
    Pressure,
    Speed,
    Temperature,
    Volume,
}

/// A trait that implements a unit of measure.
///
/// The unit of measure of some value converts to and from the International
/// System of Units (SI) and to other units of the same measured entity. The
/// unit of measure can then be used to define a [`Measurement`].
///
/// # Examples
///
/// Basic usage implementing a length measurement:
///
/// ```
/// # use efb::measurements::{UnitOfMeasure, PhysicalQuantity};
/// enum LengthUnit {
///     Meters,
/// }
///
/// // the length's value is a float
/// impl UnitOfMeasure<f32> for LengthUnit {
///     fn quantity() -> PhysicalQuantity {
///         PhysicalQuantity::Length
///     }
///
///     fn si() -> Self {
///         Self::Meters
///     }
///
///     fn symbol(&self) -> &'static str {
///         match self {
///             Self::Meters => "m",
///         }
///     }
///
///     fn from_si(value: f32, to: &Self) -> f32 {
///         match to {
///             Self::Meters => value,
///         }
///     }
///
///     fn to_si(&self, value: &f32) -> f32 {
///         match self {
///             Self::Meters => *value,
///         }
///     }
/// }
/// ```
/// [`Measurement`]: super::Measurement
pub trait UnitOfMeasure<T> {
    /// Returns the physical quantity measured in this unit.
    fn quantity() -> PhysicalQuantity;

    /// Returns the SI unit.
    fn si() -> Self;

    /// The unit's symbol e.g. `m` for meters.
    fn symbol(&self) -> &str;

    /// Converts the value which is in the SI unit to another unit.
    fn from_si(value: T, to: &Self) -> T;

    /// Converts to the value in the SI unit.
    fn to_si(&self, value: &T) -> T;

    /// Converts the value from any unit to another.
    fn convert_to(&self, value: &T, other: &Self) -> T {
        Self::from_si(self.to_si(value), other)
    }
}
