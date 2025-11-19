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

//! Measurement of physical quantities.
//!
//! A [`Measurement`] represents a physical quantity with a value measured in
//! some unit. Each measurement is provided with a unit that is part of the
//! International System of Units (SI) unit system to allow operations between
//! measurements.
//!
//! # Examples
//!
//! Basic example to add two lengths and get the speed by dividing the distance
//! by time:
//!
//! ```
//! # use efb::measurements::*;
//! #
//! // we have a distance a of 10 nautical miles
//! let a = Length::nm(10.0);
//! // and want to add b which is 40 kilometer
//! let b = Length::m(40000.0);
//!
//! let dist = a + b;
//! // 40 km equals 21.6 NM
//! assert_eq!(dist.value().round(), Length::nm(31.6).value().round());
//!
//! // lets say we need for this distance 15 minutes
//! let speed = dist / Duration::s(60 * 15);
//! assert_eq!(speed.value().round(), Speed::kt(126.0).value().round());
//! ```

mod angle;
mod constants;
mod density;
mod duration;
mod length;
mod mass;
mod measurement;
mod pressure;
mod speed;
mod temperature;
mod unit_of_measure;
mod volume;

pub use angle::{Angle, AngleUnit};
pub use density::{Density, DensityUnit};
pub use duration::{Duration, DurationUnit};
pub use length::{Length, LengthUnit};
pub use mass::{Mass, MassUnit};
pub use measurement::*;
pub use pressure::{Pressure, PressureUnit};
pub use speed::{Speed, SpeedUnit};
pub use temperature::{Temperature, TemperatureUnit};
pub use unit_of_measure::{PhysicalQuantity, UnitOfMeasure};
pub use volume::{Volume, VolumeUnit};
