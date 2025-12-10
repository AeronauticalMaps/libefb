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

use super::{Field, FieldError};
use std::str::FromStr;

/// Section Code field (ARINC 424 Spec §5.4).
///
/// Position: 4 (1 character)
/// Identifies the type of record.
#[derive(Debug, PartialEq)]
pub enum SecCode {
    MORA,
    Navaid,
    Enroute,
    Heliport,
    Airport,
    CompanyRoute,
    Table,
    Airspace,
}

impl SecCode {
    /// The position of the section code field.
    pub const POSITION: usize = 4;
    /// The length of the section code field.
    pub const LENGTH: usize = 1;
}

impl Field for SecCode {}

impl FromStr for SecCode {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < Self::POSITION + Self::LENGTH {
            return Err(FieldError::invalid_length(
                "SecCode",
                Self::POSITION,
                Self::LENGTH,
            ));
        }

        let code = &s[4..5];
        match code {
            "A" => Ok(Self::MORA),
            "D" => Ok(Self::Navaid),
            "E" => Ok(Self::Enroute),
            "H" => Ok(Self::Heliport),
            "P" => Ok(Self::Airport),
            "R" => Ok(Self::CompanyRoute),
            "T" => Ok(Self::Table),
            "U" => Ok(Self::Airspace),
            c => Err(FieldError::invalid_value(
                "SecCode",
                Self::POSITION,
                Self::LENGTH,
                "unknown section code",
            )
            .with_actual(c)),
        }
    }
}

/// Subsection Code field (ARINC 424 Spec §5.5).
///
/// Position I (1 character)
/// Further classifies the record within a section.
#[derive(Debug, PartialEq)]
pub enum SubCode<const I: usize> {
    // MORA
    GridMORA,
    // Navaid
    VHFNavaid,
    NDBNavaid,
    // Enroute
    Waypoint,
    // Heliport,
    Pad,
    // Airport
    ReferencePoint,
    Gate,
    Runway,
    // Heliport, Airport
    TerminalWaypoint,
    MSA,
    // CompanyRoute
    CompanyRoute,
    AlternateRecord,
    // Tables
    CruisingTable,
    // Airspace
    ControlledAirspace,
}

impl<const I: usize> SubCode<I> {
    /// The length of the subsection code field.
    pub const LENGTH: usize = 1;
}

impl<const I: usize> Field for SubCode<I> {}

impl<const I: usize> FromStr for SubCode<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("SubCode", I, Self::LENGTH));
        }

        let sec_code: SecCode = s.parse()?;
        let sub = &s[I..I + 1];

        match sub {
            " " => match sec_code {
                SecCode::Navaid => Ok(Self::VHFNavaid),
                SecCode::CompanyRoute => Ok(Self::CompanyRoute),
                _ => Err(FieldError::invalid_value(
                    "SubCode",
                    I,
                    Self::LENGTH,
                    "invalid section code for blank subsection",
                )
                .with_actual(sub)),
            },
            "A" => match sec_code {
                SecCode::Enroute => Ok(Self::Waypoint),
                SecCode::Heliport => Ok(Self::Pad),
                SecCode::Airport => Ok(Self::ReferencePoint),
                SecCode::CompanyRoute => Ok(Self::AlternateRecord),
                _ => Err(FieldError::invalid_value(
                    "SubCode",
                    I,
                    Self::LENGTH,
                    "invalid section code for subsection A",
                )
                .with_actual(sub)),
            },
            "B" => match sec_code {
                SecCode::Navaid => Ok(Self::NDBNavaid),
                SecCode::Airport => Ok(Self::Gate),
                _ => Err(FieldError::invalid_value(
                    "SubCode",
                    I,
                    Self::LENGTH,
                    "invalid section code for subsection B",
                )
                .with_actual(sub)),
            },
            "C" => match sec_code {
                SecCode::Heliport | SecCode::Airport => Ok(Self::TerminalWaypoint),
                SecCode::Table => Ok(Self::CruisingTable),
                SecCode::Airspace => Ok(Self::ControlledAirspace),
                _ => Err(FieldError::invalid_value(
                    "SubCode",
                    I,
                    Self::LENGTH,
                    "invalid section code for subsection C",
                )
                .with_actual(sub)),
            },
            "G" => match sec_code {
                SecCode::Airport => Ok(Self::Runway),
                _ => Err(FieldError::invalid_value(
                    "SubCode",
                    I,
                    Self::LENGTH,
                    "invalid section code for subsection G",
                )
                .with_actual(sub)),
            },
            "S" => match sec_code {
                SecCode::MORA => Ok(Self::GridMORA),
                SecCode::Heliport | SecCode::Airport => Ok(Self::MSA),
                _ => Err(FieldError::invalid_value(
                    "SubCode",
                    I,
                    Self::LENGTH,
                    "invalid section code for subsection S",
                )
                .with_actual(sub)),
            },
            _ => todo!("implement missing SUB CODE D..Z"),
        }
    }
}
