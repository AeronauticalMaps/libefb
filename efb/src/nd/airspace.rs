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
    /// Alert area.
    ///
    /// Airspace with an activity pilots should be aware of, e.g. parachute
    /// jumping or gliding, without restricting flight. These kind of airspace
    /// are usually defined in AIP ENR 5.5 and are not specified by ICAO.
    Alert { activity: Option<AirspaceActivity> },
}

/// Activity in an airspace, e.g. parachuting in an alert area.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AirspaceActivity {
    /// Traffic on and around an aerodrome.
    AerodromeTraffic,
    /// Helicopter traffic.
    HelicopterTraffic,
    /// Training flights.
    Training,
    /// Aerobatics.
    Aerobatics,
    /// An air show.
    AirShow,
    /// Air sports, e.g. air racing.
    AirSports,
    /// Ultralight flights.
    Ultralight,
    /// Gliding.
    Gliding,
    /// Paragliding.
    Paragliding,
    /// Hang gliding.
    HangGliding,
    /// Parachute jumping.
    Parachuting,
    /// Dropping of troops, supplies or other items.
    AirDrop,
    /// Balloon flights.
    Ballooning,
    /// Radiosondes carried aloft by balloons.
    Radiosonde,
    /// Launches of space flights.
    SpaceFlight,
    /// Unmanned aerial vehicles.
    UAV,
    /// Aerial work, e.g. photography or surveying.
    AerialWork,
    /// Crop dusting.
    CropDusting,
    /// Fire fighting from the air.
    FireFighting,
    /// Military operations.
    Military,
    /// Air-to-air refuelling.
    AirRefuelling,
    /// Jets climbing on common tracks.
    JetClimbing,
    /// Interception exercises.
    InterceptionExercise,
    /// Towing of aerial targets.
    TargetTowing,
    /// Naval firing exercises.
    NavalExercise,
    /// Guided missiles.
    Missiles,
    /// Air-to-air or air-to-ground weapons.
    AirGunnery,
    /// Artillery firing.
    Artillery,
    /// Shooting, e.g. on a range or a hunting area.
    Shooting,
    /// Blasting with explosives.
    Blasting,
    /// Underwater blasting.
    UnderwaterBlasting,
    /// Rockets fired against hail.
    AntiHail,
    /// A bird hazard.
    Birds,
    /// Bird migration.
    BirdMigration,
    /// Fireworks.
    Fireworks,
    /// High-intensity radio transmissions.
    HighIntensityRadio,
    /// High-intensity lights.
    HighIntensityLight,
    /// Lasers, e.g. a laser show.
    Laser,
    /// A nature reserve.
    NatureReserve,
    /// Animals sensitive to aircraft noise, e.g. a farm or a zoo.
    Fauna,
    /// Noise abatement.
    NoiseAbatement,
    /// The investigation of an accident.
    AccidentInvestigation,
    /// A densely populated area.
    Population,
    /// A very important person.
    VIP,
    /// A head of state.
    HeadOfState,
    /// A vice head of state.
    ViceHeadOfState,
    /// An oil field.
    Oil,
    /// A gas field or gas plant.
    Gas,
    /// A refinery.
    Refinery,
    /// A chemical plant.
    ChemicalPlant,
    /// A nuclear plant.
    NuclearPlant,
    /// A technical activity that affects air traffic.
    Technical,
    /// Air traffic services.
    ATS,
    /// A special procedure, e.g. an overhead approach.
    Procedure,
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
            AirspaceType::Alert { activity } => match activity {
                Some(activity) => write!(f, "Alert: {activity}"),
                None => write!(f, "Alert"),
            },
        }
    }
}

impl Display for AirspaceActivity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AirspaceActivity::AerodromeTraffic => write!(f, "Aerodrome traffic"),
            AirspaceActivity::HelicopterTraffic => write!(f, "Helicopter traffic"),
            AirspaceActivity::Training => write!(f, "Training"),
            AirspaceActivity::Aerobatics => write!(f, "Aerobatics"),
            AirspaceActivity::AirShow => write!(f, "Air show"),
            AirspaceActivity::AirSports => write!(f, "Air sports"),
            AirspaceActivity::Ultralight => write!(f, "Ultralight"),
            AirspaceActivity::Gliding => write!(f, "Gliding"),
            AirspaceActivity::Paragliding => write!(f, "Paragliding"),
            AirspaceActivity::HangGliding => write!(f, "Hang gliding"),
            AirspaceActivity::Parachuting => write!(f, "Parachuting"),
            AirspaceActivity::AirDrop => write!(f, "Air drop"),
            AirspaceActivity::Ballooning => write!(f, "Ballooning"),
            AirspaceActivity::Radiosonde => write!(f, "Radiosonde"),
            AirspaceActivity::SpaceFlight => write!(f, "Space flight"),
            AirspaceActivity::UAV => write!(f, "UAV"),
            AirspaceActivity::AerialWork => write!(f, "Aerial work"),
            AirspaceActivity::CropDusting => write!(f, "Crop dusting"),
            AirspaceActivity::FireFighting => write!(f, "Fire fighting"),
            AirspaceActivity::Military => write!(f, "Military operations"),
            AirspaceActivity::AirRefuelling => write!(f, "Air refuelling"),
            AirspaceActivity::JetClimbing => write!(f, "Jet climbing"),
            AirspaceActivity::InterceptionExercise => write!(f, "Interception exercise"),
            AirspaceActivity::TargetTowing => write!(f, "Target towing"),
            AirspaceActivity::NavalExercise => write!(f, "Naval exercise"),
            AirspaceActivity::Missiles => write!(f, "Missiles"),
            AirspaceActivity::AirGunnery => write!(f, "Air gunnery"),
            AirspaceActivity::Artillery => write!(f, "Artillery"),
            AirspaceActivity::Shooting => write!(f, "Shooting"),
            AirspaceActivity::Blasting => write!(f, "Blasting"),
            AirspaceActivity::UnderwaterBlasting => write!(f, "Underwater blasting"),
            AirspaceActivity::AntiHail => write!(f, "Anti-hail rockets"),
            AirspaceActivity::Birds => write!(f, "Birds"),
            AirspaceActivity::BirdMigration => write!(f, "Bird migration"),
            AirspaceActivity::Fireworks => write!(f, "Fireworks"),
            AirspaceActivity::HighIntensityRadio => write!(f, "High-intensity radio"),
            AirspaceActivity::HighIntensityLight => write!(f, "High-intensity light"),
            AirspaceActivity::Laser => write!(f, "Laser"),
            AirspaceActivity::NatureReserve => write!(f, "Nature reserve"),
            AirspaceActivity::Fauna => write!(f, "Fauna"),
            AirspaceActivity::NoiseAbatement => write!(f, "Noise abatement"),
            AirspaceActivity::AccidentInvestigation => write!(f, "Accident investigation"),
            AirspaceActivity::Population => write!(f, "Densely populated area"),
            AirspaceActivity::VIP => write!(f, "VIP"),
            AirspaceActivity::HeadOfState => write!(f, "Head of state"),
            AirspaceActivity::ViceHeadOfState => write!(f, "Vice head of state"),
            AirspaceActivity::Oil => write!(f, "Oil field"),
            AirspaceActivity::Gas => write!(f, "Gas"),
            AirspaceActivity::Refinery => write!(f, "Refinery"),
            AirspaceActivity::ChemicalPlant => write!(f, "Chemical plant"),
            AirspaceActivity::NuclearPlant => write!(f, "Nuclear plant"),
            AirspaceActivity::Technical => write!(f, "Technical activity"),
            AirspaceActivity::ATS => write!(f, "Air traffic services"),
            AirspaceActivity::Procedure => write!(f, "Special procedure"),
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
