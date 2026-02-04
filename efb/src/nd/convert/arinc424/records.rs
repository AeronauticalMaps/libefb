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

use arinc424::records;

use super::fields::lat_lon_to_point;
use crate::measurements::Length;
use crate::nd::*;
use crate::VerticalDistance;

impl<'a> TryFrom<records::Airport<'a>> for Airport {
    type Error = arinc424::Error;

    fn try_from(arpt: records::Airport) -> Result<Self, Self::Error> {
        Ok(Airport {
            icao_ident: arpt.arpt_ident.to_string(),
            iata_designator: arpt.iata.to_string(),
            name: arpt.airport_name.to_string(),
            coordinate: lat_lon_to_point(arpt.latitude, arpt.longitude)?,
            mag_var: arpt.mag_var.map(Into::into),
            // TODO: Parse elevation and runways.
            elevation: VerticalDistance::Gnd,
            runways: Vec::new(),
            location: Some(arpt.icao_code.try_into()?),
            cycle: Some(arpt.cycle.try_into()?),
        })
    }
}

impl<'a> TryFrom<records::Runway<'a>> for Runway {
    type Error = arinc424::Error;

    fn try_from(rwy: records::Runway) -> Result<Self, Self::Error> {
        let length = Length::ft(rwy.runway_length.as_u32()? as f32);

        Ok(Runway {
            designator: rwy.runway_id.designator()?.to_string(),
            bearing: rwy.rwy_brg.into(),
            length,
            tora: length,
            toda: length,
            lda: length,
            // FIXME: Use proper surface!
            surface: RunwaySurface::Asphalt,
            slope: rwy
                .rwy_grad
                .map(|grad| grad.as_decimal())
                .transpose()?
                .unwrap_or_default(),
            // FIXME: Use proper elevation!
            elev: VerticalDistance::Gnd,
        })
    }
}

impl<'a> TryFrom<records::Waypoint<'a>> for Waypoint {
    type Error = arinc424::Error;

    fn try_from(wp: records::Waypoint) -> Result<Self, Self::Error> {
        Ok(Waypoint {
            fix_ident: wp.fix_ident.to_string(),
            desc: wp.name_desc.to_string(),
            // TODO change type to enum and add matching
            usage: if wp.waypoint_type.as_bytes() == b"V  " {
                WaypointUsage::VFROnly
            } else {
                WaypointUsage::Unknown
            },
            coordinate: lat_lon_to_point(wp.latitude, wp.longitude)?,
            region: wp.regn_code.into(),
            mag_var: wp.mag_var.map(Into::into),
            location: wp.icao_code().try_into().ok(),
            cycle: Some(wp.cycle.try_into()?),
        })
    }
}
