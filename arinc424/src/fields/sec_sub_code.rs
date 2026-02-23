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

use crate::{Alphanumeric, Error, FixedField};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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

impl FixedField<'_> for SecCode {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match bytes[0] {
            b'A' => Ok(Self::MORA),
            b'D' => Ok(Self::Navaid),
            b'E' => Ok(Self::Enroute),
            b'H' => Ok(Self::Heliport),
            b'P' => Ok(Self::Airport),
            b'R' => Ok(Self::CompanyRoute),
            b'T' => Ok(Self::Table),
            b'U' => Ok(Self::Airspace),
            byte => Err(Error::InvalidCharacter {
                field: "Section Code",
                byte,
                expected: "SEC CODE according to ARINC 424-23 5.4",
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SubCodeKind {
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
    RestrictiveAirspace,
}

macro_rules! sub_code_error {
    ($byte:expr) => {
        Err(Error::InvalidCharacter {
            field: "Subsection Code",
            byte: $byte,
            expected: "SUB CODE according to ARINC 424-23 5.5",
        })
    };
}

pub type SubCode<'a> = Alphanumeric<'a, 1>;

impl<'a> SubCode<'a> {
    /// Subsection code kind for the section.
    ///
    /// # Errors
    ///
    /// Will return an error if the subsection code is invalid for the section.
    pub fn kind(&self, sec_code: &SecCode) -> Result<SubCodeKind, Error> {
        match self.first() {
            b' ' => match sec_code {
                SecCode::Navaid => Ok(SubCodeKind::VHFNavaid),
                SecCode::CompanyRoute => Ok(SubCodeKind::CompanyRoute),
                _ => sub_code_error!(b' '),
            },
            b'A' => match sec_code {
                SecCode::Enroute => Ok(SubCodeKind::Waypoint),
                SecCode::Heliport => Ok(SubCodeKind::Pad),
                SecCode::Airport => Ok(SubCodeKind::ReferencePoint),
                SecCode::CompanyRoute => Ok(SubCodeKind::AlternateRecord),
                _ => sub_code_error!(b'A'),
            },
            b'B' => match sec_code {
                SecCode::Navaid => Ok(SubCodeKind::NDBNavaid),
                SecCode::Airport => Ok(SubCodeKind::Gate),
                _ => sub_code_error!(b'B'),
            },
            b'C' => match sec_code {
                SecCode::Heliport | SecCode::Airport => Ok(SubCodeKind::TerminalWaypoint),
                SecCode::Table => Ok(SubCodeKind::CruisingTable),
                SecCode::Airspace => Ok(SubCodeKind::ControlledAirspace),
                _ => sub_code_error!(b'C'),
            },
            b'G' => match sec_code {
                SecCode::Airport => Ok(SubCodeKind::Runway),
                _ => sub_code_error!(b'G'),
            },
            b'R' => match sec_code {
                SecCode::Airspace => Ok(SubCodeKind::RestrictiveAirspace),
                _ => sub_code_error!(b'R'),
            },
            b'S' => match sec_code {
                SecCode::MORA => Ok(SubCodeKind::GridMORA),
                SecCode::Heliport | SecCode::Airport => Ok(SubCodeKind::MSA),
                _ => sub_code_error!(b'S'),
            },
            _ => unimplemented!("SUB CODE D..Z"),
        }
    }
}
