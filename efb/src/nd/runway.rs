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

use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::measurements::{Angle, Length};
use crate::VerticalDistance;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RunwaySurface {
    Asphalt,
    Concrete,
    Grass,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RunwayConditionCode {
    /// Dry.
    Six,

    /// Frost, wet, slush, dry snow or wet snow.
    Five,

    /// Compacted snow.
    Four,

    /// Wet (slippery), dry snow, wet snow, dry snow on top of compacted snow,
    /// wet snow on top of compacted snow or compacted snow.
    Three,

    /// Standing water or slush.
    Two,

    /// Ice.
    One,

    /// Wet ice, water on top of compacted snow or dry snow or wet snow on top
    /// of ice.
    Zero,
}

impl TryFrom<u8> for RunwayConditionCode {
    type Error = Error;

    fn try_from(value: u8) -> Result<RunwayConditionCode, Self::Error> {
        match value {
            6 => Ok(RunwayConditionCode::Six),
            5 => Ok(RunwayConditionCode::Five),
            4 => Ok(RunwayConditionCode::Four),
            3 => Ok(RunwayConditionCode::Three),
            2 => Ok(RunwayConditionCode::Two),
            1 => Ok(RunwayConditionCode::One),
            0 => Ok(RunwayConditionCode::Zero),
            _ => Err(Error::InvalidRWYCC),
        }
    }
}

/// Runway with physical and declared distances.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Runway {
    pub designator: String,
    pub bearing: Angle,
    pub length: Length,
    /// Takeoff Run Available - length available for ground run during takeoff.
    pub tora: Length,
    /// Takeoff Distance Available - length available for takeoff including clearway.
    pub toda: Length,
    /// Landing Distance Available - length available for landing ground roll.
    pub lda: Length,
    pub surface: RunwaySurface,
    /// Runway gradient as percentage (positive = upslope, negative = downslope).
    pub slope: f32,
    pub elev: VerticalDistance,
}

impl Hash for Runway {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.designator.hash(state);
        self.bearing.hash(state);
        self.length.hash(state);
        self.tora.hash(state);
        self.toda.hash(state);
        self.lda.hash(state);
        self.surface.hash(state);
        self.slope.to_bits().hash(state);
        self.elev.hash(state);
    }
}

impl fmt::Display for Runway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.designator)
    }
}
