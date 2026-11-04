// SPDX-License-Identifier: Apache-2.0
// Copyright 2024, 2026 Joe Pearson
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
    pub tora: Option<Length>,
    /// Takeoff Distance Available - length available for takeoff including clearway.
    pub toda: Option<Length>,
    /// Landing Distance Available - length available for landing ground roll.
    pub lda: Option<Length>,
    /// The distance of the threshold from the beginning of the runway.
    pub displaced_threshold: Option<Length>,
    pub surface: Option<RunwaySurface>,
    /// Runway gradient as percentage (positive = upslope, negative = downslope).
    pub slope: Option<f32>,
    pub elev: VerticalDistance,
}

impl Runway {
    /// Returns the TORA, or the runway length if the TORA is unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use efb::measurements::{Angle, Length};
    /// use efb::nd::Runway;
    /// use efb::VerticalDistance;
    ///
    /// let rwy = Runway {
    ///     designator: String::from("20"),
    ///     bearing: Angle::m(202.0),
    ///     length: Length::m(1040.0),
    ///     tora: None,
    ///     toda: None,
    ///     lda: None,
    ///     displaced_threshold: None,
    ///     surface: None,
    ///     slope: None,
    ///     elev: VerticalDistance::Gnd,
    /// };
    ///
    /// assert_eq!(rwy.tora_or_length(), Length::m(1040.0));
    /// ```
    pub fn tora_or_length(&self) -> Length {
        self.tora.unwrap_or(self.length)
    }

    /// Returns the LDA, or the runway length less the displaced threshold if
    /// the LDA is unknown. An unknown displaced threshold counts as none.
    ///
    /// # Examples
    ///
    /// ```
    /// use efb::measurements::{Angle, Length};
    /// use efb::nd::Runway;
    /// use efb::VerticalDistance;
    ///
    /// let rwy = Runway {
    ///     designator: String::from("20"),
    ///     bearing: Angle::m(202.0),
    ///     length: Length::m(1040.0),
    ///     tora: None,
    ///     toda: None,
    ///     lda: None,
    ///     displaced_threshold: Some(Length::m(140.0)),
    ///     surface: None,
    ///     slope: None,
    ///     elev: VerticalDistance::Gnd,
    /// };
    ///
    /// assert_eq!(rwy.lda_or_length(), Length::m(900.0));
    /// ```
    pub fn lda_or_length(&self) -> Length {
        self.lda.unwrap_or(match self.displaced_threshold {
            Some(displaced_threshold) => self.length - displaced_threshold,
            None => self.length,
        })
    }
}

impl Hash for Runway {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.designator.hash(state);
        self.bearing.hash(state);
        self.length.hash(state);
        self.tora.hash(state);
        self.toda.hash(state);
        self.lda.hash(state);
        self.displaced_threshold.hash(state);
        self.surface.hash(state);
        self.slope.map(f32::to_bits).hash(state);
        self.elev.hash(state);
    }
}

impl fmt::Display for Runway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.designator)
    }
}
