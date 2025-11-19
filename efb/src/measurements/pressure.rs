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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Measurement, PhysicalQuantity, UnitOfMeasure};

mod constants {
    pub const IN_HG_IN_PA: f32 = 3386.39;
}

/// Pressure with _Pa_ as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum PressureUnit {
    InchesOfMercury,
    Hektopascal,
    Pascal,
}

impl UnitOfMeasure<f32> for PressureUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Pressure
    }

    fn si() -> Self {
        Self::Pascal
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::InchesOfMercury => "inHg",
            Self::Hektopascal => "hPa",
            Self::Pascal => "Pa",
        }
    }

    fn from_si(value: f32, to: &Self) -> f32 {
        match to {
            Self::InchesOfMercury => value / constants::IN_HG_IN_PA,
            Self::Hektopascal => value / 100.0,
            Self::Pascal => value,
        }
    }

    fn to_si(&self, value: &f32) -> f32 {
        match self {
            Self::InchesOfMercury => value * constants::IN_HG_IN_PA,
            Self::Hektopascal => value * 100.0,
            Self::Pascal => *value,
        }
    }
}

pub type Pressure = Measurement<f32, PressureUnit>;

impl Pressure {
    /// Returns the pressure in Inches of Mercury _inHg_.
    pub const fn in_hg(value: f32) -> Self {
        Measurement {
            value,
            unit: PressureUnit::InchesOfMercury,
        }
    }

    /// Returns the pressure in Hectopascal _hPa_.
    pub const fn h_pa(value: f32) -> Self {
        Measurement {
            value,
            unit: PressureUnit::Hektopascal,
        }
    }

    /// Returns the pressure in Pascal _Pa_.
    pub const fn pa(value: f32) -> Self {
        Measurement {
            value,
            unit: PressureUnit::Pascal,
        }
    }
}
