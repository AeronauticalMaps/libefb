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

use arinc424::fields;

use crate::geom::Coordinate;
use crate::measurements::Angle;
use crate::nd::*;
use crate::MagneticVariation;

impl<'a> TryFrom<fields::Cycle<'a>> for AiracCycle {
    type Error = arinc424::Error;

    fn try_from(value: fields::Cycle) -> Result<Self, Self::Error> {
        Ok(AiracCycle::new(value.year()?, value.cycle()?))
    }
}

impl<'a> TryFrom<fields::IcaoCode<'a>> for LocationIndicator {
    type Error = arinc424::Error;

    fn try_from(value: fields::IcaoCode<'a>) -> Result<Self, Self::Error> {
        LocationIndicator::try_from(value.as_str()).map_err(|_| arinc424::Error::InvalidVariant {
            field: "IcaoCode",
            bytes: value.as_bytes().to_vec(),
            expected: "valid location identifier",
        })
    }
}

impl<'a> TryFrom<(fields::Latitude<'a>, fields::Longitude<'a>)> for Coordinate {
    type Error = arinc424::Error;

    fn try_from(value: (fields::Latitude<'a>, fields::Longitude<'a>)) -> Result<Self, Self::Error> {
        Ok(Coordinate {
            latitude: value.0.as_decimal()?,
            longitude: value.1.as_decimal()?,
        })
    }
}

impl From<fields::MagVar> for MagneticVariation {
    fn from(value: fields::MagVar) -> Self {
        match value {
            fields::MagVar::East(d) => Self::East(d),
            fields::MagVar::West(d) => Self::West(d),
            fields::MagVar::OrientedToTrueNorth => Self::OrientedToTrueNorth,
        }
    }
}

impl<'a> From<fields::RegnCode<'a>> for Region {
    fn from(value: fields::RegnCode) -> Self {
        match value.as_str() {
            "ENRT" => Self::Enroute,
            // TODO: Change terminal area code.
            icao => Self::TerminalArea(
                icao.as_bytes()
                    .try_into()
                    .expect("ICAO should be 4 character"),
            ),
        }
    }
}

impl From<fields::RwyBrg> for Angle {
    fn from(rwy_brg: fields::RwyBrg) -> Self {
        match rwy_brg {
            fields::RwyBrg::MagneticNorth(degree) => Self::m(degree),
            fields::RwyBrg::TrueNorth(degree) => Self::t(degree as f32),
        }
    }
}
