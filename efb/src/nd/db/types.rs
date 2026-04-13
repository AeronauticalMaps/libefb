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

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::core::MagneticVariation;
use crate::nd::{
    AiracCycle, AirspaceClassification, AirspaceType, LocationIndicator, Region, RunwaySurface,
    SourceFormat, WaypointUsage,
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
            other => Err(FromSqlError::Other(
                format!("unknown airspace_type: {other}").into(),
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
        })))
    }
}

impl FromSql for SourceFormat {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "a424" => Ok(Self::A424),
            "openair" => Ok(Self::OpenAir),
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
        let d = value.as_f64()? as f32;
        Ok(if d > 0.0 {
            Self::East(d)
        } else if d < 0.0 {
            Self::West(-d)
        } else {
            Self::OrientedToTrueNorth
        })
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
