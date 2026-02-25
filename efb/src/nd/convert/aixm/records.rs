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

//! Conversions from AIXM feature types to [`NavigationData`](crate::nd::NavigationData)
//! record types.
//!
//! Each [`TryFrom`] implementation maps a single AIXM feature into the
//! corresponding efb navigation type. Runways are handled separately in the
//! parent module because they require cross-reference resolution.

use crate::geom::{Coordinate, Polygon};
use crate::nd::*;

use super::fields;

impl TryFrom<&aixm::AirportHeliport> for Airport {
    type Error = aixm::Error;

    fn try_from(ahp: &aixm::AirportHeliport) -> Result<Self, Self::Error> {
        let (lat, lon) = ahp
            .coordinate()
            .ok_or(aixm::Error::MissingField("ARP coordinates"))?;

        Ok(Airport {
            icao_ident: ahp
                .location_indicator_icao()
                .unwrap_or(ahp.designator())
                .to_string(),
            iata_designator: ahp.iata_designator().unwrap_or_default().to_string(),
            name: ahp.name().to_string(),
            coordinate: Coordinate::new(lat, lon),
            mag_var: None,
            elevation: {
                let (value, uom) = ahp.field_elevation();
                fields::field_elevation(value, uom)
            },
            runways: Vec::new(),
            location: ahp
                .location_indicator_icao()
                .and_then(|icao| LocationIndicator::try_from(icao).ok()),
            cycle: None,
        })
    }
}

impl TryFrom<&aixm::DesignatedPoint> for Waypoint {
    type Error = aixm::Error;

    fn try_from(dp: &aixm::DesignatedPoint) -> Result<Self, Self::Error> {
        let (lat, lon) = dp
            .coordinate()
            .ok_or(aixm::Error::MissingField("location coordinates"))?;

        Ok(Waypoint {
            fix_ident: dp.designator().to_string(),
            desc: dp.name().unwrap_or_default().to_string(),
            usage: WaypointUsage::Unknown,
            coordinate: Coordinate::new(lat, lon),
            mag_var: None,
            region: Region::Enroute,
            location: None,
            cycle: None,
        })
    }
}

impl TryFrom<&aixm::Navaid> for Waypoint {
    type Error = aixm::Error;

    fn try_from(nav: &aixm::Navaid) -> Result<Self, Self::Error> {
        let (lat, lon) = nav
            .coordinate()
            .ok_or(aixm::Error::MissingField("navaid location coordinates"))?;

        Ok(Waypoint {
            fix_ident: nav.designator().to_string(),
            desc: nav.name().unwrap_or_default().to_string(),
            usage: WaypointUsage::Unknown,
            coordinate: Coordinate::new(lat, lon),
            mag_var: None,
            region: Region::Enroute,
            location: None,
            cycle: None,
        })
    }
}

impl TryFrom<&aixm::Airspace> for Airspace {
    type Error = aixm::Error;

    fn try_from(arsp: &aixm::Airspace) -> Result<Self, Self::Error> {
        let class = fields::airspace_class(arsp.airspace_type());
        let volumes = arsp.volumes();

        let (ceiling, floor) = volumes
            .first()
            .map(fields::volume_limits)
            .unwrap_or((
                crate::VerticalDistance::Unlimited,
                crate::VerticalDistance::Gnd,
            ));

        let polygon = volumes
            .first()
            .map(|vol| {
                Polygon::from(
                    vol.polygon
                        .iter()
                        .map(|&(lat, lon)| Coordinate::new(lat, lon))
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap_or_default();

        Ok(Airspace {
            name: arsp
                .name()
                .or(arsp.designator())
                .unwrap_or_default()
                .to_string(),
            class,
            ceiling,
            floor,
            polygon,
        })
    }
}
