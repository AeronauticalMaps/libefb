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

use std::fmt::{Display, Formatter, Result};
use std::ops::{Add, Div, Mul, Sub};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::measurements::{Density, Duration, Mass, Volume};

mod constants {
    use super::Density;

    pub const AVGAS_AT_ISA: Density = Density::kg_per_l(0.75);
    pub const DIESEL_AT_ISA: Density = Density::kg_per_l(0.838);
    pub const JET_A_AT_ISA: Density = Density::kg_per_l(0.8);
}

/// Type of fuel used by an aircraft.
///
/// Represents different fuel types that can be used in aircraft. Each fuel type
/// has an associated density at ISA conditions used for mass/volume
/// conversions.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum FuelType {
    /// Aviation gasoline (100LL) with density of 0.75 kg/L at ISA.
    AvGas,
    /// Diesel fuel with density of 0.838 kg/L at ISA.
    Diesel,
    /// Jet-A with density of 0.8 kg/L at ISA.
    JetA,
}

impl FuelType {
    /// Returns the density of the fuel type at ISA conditions.
    pub fn density(&self) -> Density {
        match self {
            Self::AvGas => constants::AVGAS_AT_ISA,
            Self::Diesel => constants::DIESEL_AT_ISA,
            Self::JetA => constants::JET_A_AT_ISA,
        }
    }
}

/// Fuel quantity with a specific type and mass.
///
/// Represents a quantity of fuel, tracking both the fuel type and mass.
/// Fuel quantities can be created from either mass or volume, and converted
/// between the two at ISA conditions.
///
/// # Examples
///
/// ```
/// # use efb::prelude::*;
/// # use efb::measurements::{Mass, Volume};
/// // Create fuel from volume
/// let fuel = Fuel::from_volume(Volume::l(100.0), FuelType::Diesel);
///
/// // Create fuel from mass
/// let fuel = Fuel::new(Mass::kg(83.8), FuelType::Diesel);
///
/// // Add fuel quantities
/// let total = Fuel::from_volume(Volume::l(50.0), FuelType::Diesel)
///     + Fuel::from_volume(Volume::l(50.0), FuelType::Diesel);
/// assert_eq!(total.volume(), Volume::l(100.0));
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub struct Fuel {
    pub fuel_type: FuelType,
    pub mass: Mass,
}

impl Fuel {
    /// Creates new fuel from mass.
    pub fn new(mass: Mass, fuel_type: FuelType) -> Self {
        Self { fuel_type, mass }
    }

    /// Creates new fuel from volume.
    ///
    /// The mass is calculated using the fuel type's density at ISA conditions.
    pub fn from_volume(v: Volume, fuel_type: FuelType) -> Self {
        Self {
            fuel_type,
            mass: v * fuel_type.density(),
        }
    }

    /// Returns the volume of fuel.
    ///
    /// The volume is calculated from the mass using the fuel type's density.
    pub fn volume(self) -> Volume {
        self.mass / self.fuel_type.density()
    }
}

impl Display for Fuel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let tmp = if let Some(precision) = f.precision() {
            format!("{:.precision$}", self.volume())
        } else {
            format!("{}", self.volume())
        };

        f.pad_integral(true, "", &tmp)
    }
}

impl Add for Fuel {
    type Output = Fuel;

    fn add(self, rhs: Self) -> Self::Output {
        if self.fuel_type == rhs.fuel_type {
            Fuel {
                fuel_type: self.fuel_type,
                mass: self.mass + rhs.mass,
            }
        } else {
            self
        }
    }
}

impl Sub for Fuel {
    type Output = Fuel;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.fuel_type == rhs.fuel_type {
            Self {
                fuel_type: self.fuel_type,
                mass: self.mass - rhs.mass,
            }
        } else {
            self
        }
    }
}

macro_rules! mul_impl {
    ($($t:ty)*) => ($(
        impl Mul<$t> for Fuel {
            type Output = Fuel;

            fn mul(self, rhs: $t) -> Self {
                Self {
                    fuel_type: self.fuel_type,
                    mass: self.mass * rhs as f32,
                }

            }
        }
    )*)
}

mul_impl! { usize f32 }

macro_rules! div_impl {
    ($($t:ty)*) => ($(
        impl Div<$t> for Fuel {
            type Output = Fuel;

            fn div(self, rhs: $t) -> Self {
                Self {
                    fuel_type: self.fuel_type,
                    mass: self.mass / rhs as f32,
                }

            }
        }
    )*)
}

div_impl! { usize f32 }

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FuelFlow {
    PerHour(Fuel),
}

impl Mul<Duration> for FuelFlow {
    type Output = Fuel;

    fn mul(self, rhs: Duration) -> Self::Output {
        let hours: f32 = *rhs.value() as f32 / 3600.0;

        match self {
            Self::PerHour(fuel) => fuel * hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_from_volume() {
        let lhs = Fuel::from_volume(Volume::l(10.0), FuelType::Diesel);
        let rhs = diesel!(Volume::l(10.0));
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn add_fuel() {
        let lhs = diesel!(Volume::l(10.0));
        let rhs = diesel!(Volume::l(10.0));
        assert_eq!(lhs + rhs, diesel!(Volume::l(20.0)));
    }

    #[test]
    fn sub_fuel() {
        let lhs = diesel!(Volume::l(10.0));
        let rhs = diesel!(Volume::l(10.0));
        assert_eq!(lhs - rhs, diesel!(Volume::l(0.0)));
    }

    #[test]
    fn mul_fuel() {
        let lhs = diesel!(Volume::l(10.0));
        assert_eq!(lhs * 10.0, diesel!(Volume::l(100.0)));
    }

    #[test]
    fn mul_fuel_flow() {
        let lhs = FuelFlow::PerHour(diesel!(Volume::l(10.0)));
        let rhs = Duration::s(7200); // 2h
        assert_eq!(lhs * rhs, diesel!(Volume::l(20.0)));
    }
}
