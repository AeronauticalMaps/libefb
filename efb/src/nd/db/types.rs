// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Joe Pearson
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

use std::str::FromStr;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::core::MagneticVariation;
use crate::nd::{
    AiracCycle, AirspaceActivity, AirspaceClassification, AirspaceType, LocationIndicator, Region,
    RunwaySurface, SourceFormat, TerritoryLetters, WaypointUsage,
};

impl ToSql for AirspaceType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            Self::CTA => b"cta",
            Self::CTR => b"ctr",
            Self::TMA => b"tma",
            Self::Restricted => b"restricted",
            Self::Danger => b"danger",
            Self::Prohibited => b"prohibited",
            Self::TMZ => b"tmz",
            Self::RMZ => b"rmz",
            Self::RadarZone => b"radar_zone",
            Self::Alert { .. } => b"alert",
        })))
    }
}

impl FromSql for AirspaceType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "cta" => Ok(Self::CTA),
            "ctr" => Ok(Self::CTR),
            "tma" => Ok(Self::TMA),
            "restricted" => Ok(Self::Restricted),
            "danger" => Ok(Self::Danger),
            "prohibited" => Ok(Self::Prohibited),
            "tmz" => Ok(Self::TMZ),
            "rmz" => Ok(Self::RMZ),
            "radar_zone" => Ok(Self::RadarZone),
            // the activity is stored in a column of its own
            "alert" => Ok(Self::Alert { activity: None }),
            other => Err(FromSqlError::Other(
                format!("unknown airspace_type: {other}").into(),
            )),
        }
    }
}

impl ToSql for AirspaceActivity {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            Self::AerodromeTraffic => b"aerodrome_traffic",
            Self::HelicopterTraffic => b"helicopter_traffic",
            Self::Training => b"training",
            Self::Aerobatics => b"aerobatics",
            Self::AirShow => b"air_show",
            Self::AirSports => b"air_sports",
            Self::Ultralight => b"ultralight",
            Self::Gliding => b"gliding",
            Self::Paragliding => b"paragliding",
            Self::HangGliding => b"hang_gliding",
            Self::Parachuting => b"parachuting",
            Self::AirDrop => b"air_drop",
            Self::Ballooning => b"ballooning",
            Self::Radiosonde => b"radiosonde",
            Self::SpaceFlight => b"space_flight",
            Self::UAV => b"uav",
            Self::AerialWork => b"aerial_work",
            Self::CropDusting => b"crop_dusting",
            Self::FireFighting => b"fire_fighting",
            Self::Military => b"military",
            Self::AirRefuelling => b"air_refuelling",
            Self::JetClimbing => b"jet_climbing",
            Self::InterceptionExercise => b"interception_exercise",
            Self::TargetTowing => b"target_towing",
            Self::NavalExercise => b"naval_exercise",
            Self::Missiles => b"missiles",
            Self::AirGunnery => b"air_gunnery",
            Self::Artillery => b"artillery",
            Self::Shooting => b"shooting",
            Self::Blasting => b"blasting",
            Self::UnderwaterBlasting => b"underwater_blasting",
            Self::AntiHail => b"anti_hail",
            Self::Birds => b"birds",
            Self::BirdMigration => b"bird_migration",
            Self::Fireworks => b"fireworks",
            Self::HighIntensityRadio => b"high_intensity_radio",
            Self::HighIntensityLight => b"high_intensity_light",
            Self::Laser => b"laser",
            Self::NatureReserve => b"nature_reserve",
            Self::Fauna => b"fauna",
            Self::NoiseAbatement => b"noise_abatement",
            Self::AccidentInvestigation => b"accident_investigation",
            Self::Population => b"population",
            Self::VIP => b"vip",
            Self::HeadOfState => b"head_of_state",
            Self::ViceHeadOfState => b"vice_head_of_state",
            Self::Oil => b"oil",
            Self::Gas => b"gas",
            Self::Refinery => b"refinery",
            Self::ChemicalPlant => b"chemical_plant",
            Self::NuclearPlant => b"nuclear_plant",
            Self::Technical => b"technical",
            Self::ATS => b"ats",
            Self::Procedure => b"procedure",
        })))
    }
}

impl FromSql for AirspaceActivity {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "aerodrome_traffic" => Ok(Self::AerodromeTraffic),
            "helicopter_traffic" => Ok(Self::HelicopterTraffic),
            "training" => Ok(Self::Training),
            "aerobatics" => Ok(Self::Aerobatics),
            "air_show" => Ok(Self::AirShow),
            "air_sports" => Ok(Self::AirSports),
            "ultralight" => Ok(Self::Ultralight),
            "gliding" => Ok(Self::Gliding),
            "paragliding" => Ok(Self::Paragliding),
            "hang_gliding" => Ok(Self::HangGliding),
            "parachuting" => Ok(Self::Parachuting),
            "air_drop" => Ok(Self::AirDrop),
            "ballooning" => Ok(Self::Ballooning),
            "radiosonde" => Ok(Self::Radiosonde),
            "space_flight" => Ok(Self::SpaceFlight),
            "uav" => Ok(Self::UAV),
            "aerial_work" => Ok(Self::AerialWork),
            "crop_dusting" => Ok(Self::CropDusting),
            "fire_fighting" => Ok(Self::FireFighting),
            "military" => Ok(Self::Military),
            "air_refuelling" => Ok(Self::AirRefuelling),
            "jet_climbing" => Ok(Self::JetClimbing),
            "interception_exercise" => Ok(Self::InterceptionExercise),
            "target_towing" => Ok(Self::TargetTowing),
            "naval_exercise" => Ok(Self::NavalExercise),
            "missiles" => Ok(Self::Missiles),
            "air_gunnery" => Ok(Self::AirGunnery),
            "artillery" => Ok(Self::Artillery),
            "shooting" => Ok(Self::Shooting),
            "blasting" => Ok(Self::Blasting),
            "underwater_blasting" => Ok(Self::UnderwaterBlasting),
            "anti_hail" => Ok(Self::AntiHail),
            "birds" => Ok(Self::Birds),
            "bird_migration" => Ok(Self::BirdMigration),
            "fireworks" => Ok(Self::Fireworks),
            "high_intensity_radio" => Ok(Self::HighIntensityRadio),
            "high_intensity_light" => Ok(Self::HighIntensityLight),
            "laser" => Ok(Self::Laser),
            "nature_reserve" => Ok(Self::NatureReserve),
            "fauna" => Ok(Self::Fauna),
            "noise_abatement" => Ok(Self::NoiseAbatement),
            "accident_investigation" => Ok(Self::AccidentInvestigation),
            "population" => Ok(Self::Population),
            "vip" => Ok(Self::VIP),
            "head_of_state" => Ok(Self::HeadOfState),
            "vice_head_of_state" => Ok(Self::ViceHeadOfState),
            "oil" => Ok(Self::Oil),
            "gas" => Ok(Self::Gas),
            "refinery" => Ok(Self::Refinery),
            "chemical_plant" => Ok(Self::ChemicalPlant),
            "nuclear_plant" => Ok(Self::NuclearPlant),
            "technical" => Ok(Self::Technical),
            "ats" => Ok(Self::ATS),
            "procedure" => Ok(Self::Procedure),
            other => Err(FromSqlError::Other(
                format!("unknown activity: {other}").into(),
            )),
        }
    }
}

impl ToSql for AirspaceClassification {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            Self::A => b"A",
            Self::B => b"B",
            Self::C => b"C",
            Self::D => b"D",
            Self::E => b"E",
            Self::F => b"F",
            Self::G => b"G",
        })))
    }
}

impl FromSql for AirspaceClassification {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            "E" => Ok(Self::E),
            "F" => Ok(Self::F),
            "G" => Ok(Self::G),
            other => Err(FromSqlError::Other(
                format!("unknown classification: {other}").into(),
            )),
        }
    }
}

impl ToSql for WaypointUsage {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            Self::VFROnly => b"vfr_only",
            Self::Unknown => b"unknown",
        })))
    }
}

impl FromSql for WaypointUsage {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "vfr_only" => Ok(Self::VFROnly),
            "unknown" => Ok(Self::Unknown),
            other => Err(FromSqlError::Other(
                format!("unknown usage: {other}").into(),
            )),
        }
    }
}

impl ToSql for RunwaySurface {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            Self::Asphalt => b"asphalt",
            Self::Concrete => b"concrete",
            Self::Grass => b"grass",
        })))
    }
}

impl FromSql for RunwaySurface {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "asphalt" => Ok(Self::Asphalt),
            "concrete" => Ok(Self::Concrete),
            "grass" => Ok(Self::Grass),
            other => Err(FromSqlError::Other(
                format!("unknown surface: {other}").into(),
            )),
        }
    }
}

impl ToSql for SourceFormat {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            Self::A424 => b"a424",
            Self::OpenAir => b"openair",
            Self::Aixm => b"aixm",
        })))
    }
}

impl FromSql for SourceFormat {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "a424" => Ok(Self::A424),
            "openair" => Ok(Self::OpenAir),
            "aixm" => Ok(Self::Aixm),
            other => Err(FromSqlError::Other(
                format!("unknown source_format: {other}").into(),
            )),
        }
    }
}

impl ToSql for LocationIndicator {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(
            self.as_str().as_bytes(),
        )))
    }
}

impl FromSql for LocationIndicator {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        LocationIndicator::new(s).map_err(|e| FromSqlError::Other(e.to_string().into()))
    }
}

impl ToSql for TerritoryLetters {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(
            self.as_str().as_bytes(),
        )))
    }
}

impl FromSql for TerritoryLetters {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        TerritoryLetters::from_str(s).map_err(|e| FromSqlError::Other(e.to_string().into()))
    }
}

// MagneticVariation → signed REAL (east = positive, west = negative)
//
// `OrientedToTrueNorth` round-trips as 0.0. `East(0.0)` and
// `OrientedToTrueNorth` are functionally equivalent so the ambiguity is
// benign.

impl ToSql for MagneticVariation {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let degrees: f64 = match self {
            Self::East(d) => *d as f64,
            Self::West(d) => -(*d as f64),
            Self::OrientedToTrueNorth => 0.0,
        };
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Real(degrees)))
    }
}

impl FromSql for MagneticVariation {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Ok((value.as_f64()? as f32).into())
    }
}

// AiracCycle → YYNN INTEGER (e.g. 2510 for cycle 25/10)

impl ToSql for AiracCycle {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let encoded = self.year() as i64 * 100 + self.cycle() as i64;
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Integer(encoded)))
    }
}

impl FromSql for AiracCycle {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let n = value.as_i64()?;
        Ok(AiracCycle::new((n / 100) as u8, (n % 100) as u8))
    }
}

// Region → terminal_airport_ident TEXT (NULL = enroute)
//
// Stored as a single optional TEXT column: NULL for `Enroute`, the ICAO
// airport ident for `TerminalArea`. The `region` discriminant can always be
// derived from IS NULL, so no separate column is needed.

impl ToSql for Region {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            Self::Enroute => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null)),
            Self::TerminalArea(bytes) => {
                let s = String::from_utf8_lossy(bytes).into_owned();
                Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(s)))
            }
        }
    }
}

impl FromSql for Region {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(Self::Enroute),
            ValueRef::Text(bytes) => {
                let s = std::str::from_utf8(bytes)
                    .map_err(|e| FromSqlError::Other(e.to_string().into()))?;
                if s.len() > 4 {
                    return Err(FromSqlError::Other(
                        format!("terminal ident too long: {s}").into(),
                    ));
                }
                let mut buf = [b' '; 4];
                buf[..s.len()].copy_from_slice(s.as_bytes());
                Ok(Self::TerminalArea(buf))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}
