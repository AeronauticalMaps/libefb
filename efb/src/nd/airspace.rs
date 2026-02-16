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

use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::VerticalDistance;

/// ICAO Airspace Classification (ICAO Annex 11, Chapter 2).
///
/// Defines the rules governing IFR/VFR operations, separation services,
/// and radio requirements within an airspace.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AirspaceClassification {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

/// Airspace type — structural or special-use designation.
///
/// Describes the kind of airspace structure (e.g. Control Area, Control Zone)
/// or special-use designation (e.g. Restricted, Danger, Prohibited).
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AirspaceType {
    /// Control Area (CTA)
    CTA,
    /// Control Zone (CTR)
    CTR,
    /// Terminal Manoeuvring Area (TMA)
    TMA,
    /// Restricted airspace
    Restricted,
    /// Danger area
    Danger,
    /// Prohibited area
    Prohibited,
    /// Transponder Mandatory Zone
    TMZ,
    /// Radio Mandatory Zone
    RMZ,
    /// Radar Zone
    RadarZone,
}

/// Airspace.
///
/// The airspace has a structural or special-use [`airspace_type`](Self::airspace_type)
/// and an optional ICAO [`classification`](Self::classification). It is enclosed
/// by the `polygon` and ranges from the `floor` to `ceiling` vertically.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Airspace {
    pub name: String,
    pub airspace_type: AirspaceType,
    pub classification: Option<AirspaceClassification>,
    pub ceiling: VerticalDistance,
    pub floor: VerticalDistance,
    pub polygon: geo::Polygon<f64>,
}

impl Display for AirspaceClassification {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AirspaceClassification::A => write!(f, "A"),
            AirspaceClassification::B => write!(f, "B"),
            AirspaceClassification::C => write!(f, "C"),
            AirspaceClassification::D => write!(f, "D"),
            AirspaceClassification::E => write!(f, "E"),
            AirspaceClassification::F => write!(f, "F"),
            AirspaceClassification::G => write!(f, "G"),
        }
    }
}

impl Display for AirspaceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AirspaceType::CTA => write!(f, "CTA"),
            AirspaceType::CTR => write!(f, "CTR"),
            AirspaceType::TMA => write!(f, "TMA"),
            AirspaceType::Restricted => write!(f, "Restricted"),
            AirspaceType::Danger => write!(f, "Danger"),
            AirspaceType::Prohibited => write!(f, "Prohibited"),
            AirspaceType::TMZ => write!(f, "TMZ"),
            AirspaceType::RMZ => write!(f, "RMZ"),
            AirspaceType::RadarZone => write!(f, "Radar Zone"),
        }
    }
}

impl Display for Airspace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.classification {
            Some(class) => write!(
                f,
                "{}: {} (Class {}) | {}/{}",
                self.name, self.airspace_type, class, self.ceiling, self.floor
            ),
            None => write!(
                f,
                "{}: {} | {}/{}",
                self.name, self.airspace_type, self.ceiling, self.floor
            ),
        }
    }
}
