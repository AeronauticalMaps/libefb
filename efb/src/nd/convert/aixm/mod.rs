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

//! Converts AIXM 5.1 features into [`NavigationData`] entries.
//!
//! Airports, waypoints, navaids, and airspaces are converted directly as they
//! stream in. Runways require a deferred resolution step because each AIXM
//! `RunwayDirection` references its parent `Runway` and `AirportHeliport`
//! through UUIDs that may appear in any order.

use std::collections::HashMap;

use crate::error::Error;
use crate::nd::*;

mod fields;
mod records;

/// Runway properties needed when resolving deferred runway directions.
struct RunwayInfo {
    airport_uuid: Option<String>,
    length: crate::measurements::Length,
    surface: RunwaySurface,
}

impl NavigationData {
    /// Builds navigation data from an AIXM 5.1 XML byte slice.
    ///
    /// Streams through the document, converts each supported feature into the
    /// corresponding [`NavigationData`] entry, and resolves cross-references
    /// between runways and airports at the end.
    ///
    /// Parse errors for individual features are collected as non-fatal errors
    /// accessible via [`NavigationData::errors`]. The returned data contains
    /// all features that parsed successfully.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use efb::nd::{Fix, NavigationData};
    ///
    /// let data = std::fs::read("aixm_data.xml").unwrap();
    /// let nd = NavigationData::try_from_aixm(&data).unwrap();
    ///
    /// if let Some(airport) = nd.find("EADD") {
    ///     println!("{}", airport.ident());
    /// }
    /// ```
    pub fn try_from_aixm(data: &[u8]) -> Result<Self, Error> {
        let mut builder = NavigationData::builder();

        // Cross-reference lookup maps
        let mut airport_uuids: HashMap<String, String> = HashMap::new();
        let mut runway_infos: HashMap<String, RunwayInfo> = HashMap::new();
        let mut deferred_rwys: Vec<aixm::Runway> = Vec::new();
        let mut deferred_rdns: Vec<(aixm::RunwayDirection, String)> = Vec::new();

        for feature in aixm::Features::new(data) {
            if let Err(e) = || -> Result<(), aixm::Error> {
                match feature? {
                    aixm::Feature::AirportHeliport(ahp) => {
                        let uuid = ahp.uuid().to_string();
                        let arpt = Airport::try_from(&ahp)?;
                        let ident = arpt.ident();
                        airport_uuids.insert(uuid, ident.clone());
                        builder.add_airport(arpt);
                    }

                    aixm::Feature::Runway(rwy) => {
                        deferred_rwys.push(rwy);
                    }

                    aixm::Feature::RunwayDirection(rdn) => {
                        let runway_uuid = match rdn.used_runway_uuid() {
                            Some(uuid) => uuid.to_string(),
                            None => return Ok(()),
                        };
                        deferred_rdns.push((rdn, runway_uuid));
                    }

                    aixm::Feature::DesignatedPoint(dp) => {
                        let wp = Waypoint::try_from(&dp)?;
                        builder.add_waypoint(wp);
                    }

                    aixm::Feature::Navaid(nav) => {
                        let wp = Waypoint::try_from(&nav)?;
                        builder.add_waypoint(wp);
                    }

                    aixm::Feature::Airspace(arsp) => {
                        let airspace = Airspace::try_from(&arsp)?;
                        builder.add_airspace(airspace);
                    }
                }

                Ok(())
            }() {
                builder.add_error(Error::InvalidAixm {
                    error: e.to_string(),
                });
            }
        }

        // Build the runway UUID -> info lookup from deferred runways.
        for rwy in &deferred_rwys {
            let (length_val, length_uom) = rwy.nominal_length();
            let length = fields::runway_length(length_val, length_uom);
            let surface = fields::runway_surface(rwy.surface_composition());

            runway_infos.insert(
                rwy.uuid().to_string(),
                RunwayInfo {
                    airport_uuid: rwy.associated_airport_uuid().map(str::to_string),
                    length,
                    surface,
                },
            );
        }

        // Resolve each RunwayDirection -> Runway -> Airport chain and add the
        // final Runway entries to their airports.
        for (rdn, runway_uuid) in deferred_rdns {
            if let Some(rwy_info) = runway_infos.get(&runway_uuid) {
                let airport_ident = rwy_info
                    .airport_uuid
                    .as_ref()
                    .and_then(|uuid| airport_uuids.get(uuid))
                    .cloned();

                if let Some(ident) = airport_ident {
                    let rwy = Runway {
                        designator: rdn.designator().to_string(),
                        bearing: fields::bearing(rdn.true_bearing(), rdn.magnetic_bearing()),
                        length: rwy_info.length,
                        tora: rwy_info.length,
                        toda: rwy_info.length,
                        lda: rwy_info.length,
                        surface: rwy_info.surface,
                        slope: 0.0,
                        elev: crate::VerticalDistance::Gnd,
                    };
                    builder.add_runway(ident, rwy);
                }
            }
        }

        Ok(builder.with_source(data).build())
    }
}

#[cfg(test)]
mod tests {
    use crate::nd::Fix;
    use crate::nd::NavigationData;

    const AIXM_DATA: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
    <message:AIXMBasicMessage
      xmlns:aixm="http://www.aixm.aero/schema/5.1"
      xmlns:gml="http://www.opengis.net/gml/3.2"
      xmlns:message="http://www.aixm.aero/schema/5.1/message"
      xmlns:xlink="http://www.w3.org/1999/xlink">
      <message:hasMember>
        <aixm:AirportHeliport gml:id="uuid.1b54b2d6">
          <aixm:timeSlice>
            <aixm:AirportHeliportTimeSlice gml:id="AHP1">
              <aixm:interpretation>BASELINE</aixm:interpretation>
              <aixm:designator>EADD</aixm:designator>
              <aixm:name>DONLON</aixm:name>
              <aixm:locationIndicatorICAO>EADD</aixm:locationIndicatorICAO>
              <aixm:fieldElevation uom="M">30</aixm:fieldElevation>
              <aixm:ARP>
                <aixm:ElevatedPoint>
                  <gml:pos>52.36 -31.94</gml:pos>
                </aixm:ElevatedPoint>
              </aixm:ARP>
            </aixm:AirportHeliportTimeSlice>
          </aixm:timeSlice>
        </aixm:AirportHeliport>
      </message:hasMember>
      <message:hasMember>
        <aixm:Runway gml:id="uuid.9e51668f">
          <aixm:timeSlice>
            <aixm:RunwayTimeSlice gml:id="RWY1">
              <aixm:interpretation>BASELINE</aixm:interpretation>
              <aixm:designator>09L/27R</aixm:designator>
              <aixm:nominalLength uom="M">2800.0</aixm:nominalLength>
              <aixm:surfaceProperties>
                <aixm:SurfaceCharacteristics gml:id="SC1">
                  <aixm:composition>CONC</aixm:composition>
                </aixm:SurfaceCharacteristics>
              </aixm:surfaceProperties>
              <aixm:associatedAirportHeliport xlink:href="urn:uuid:1b54b2d6"/>
            </aixm:RunwayTimeSlice>
          </aixm:timeSlice>
        </aixm:Runway>
      </message:hasMember>
      <message:hasMember>
        <aixm:RunwayDirection gml:id="uuid.c8455a6b">
          <aixm:timeSlice>
            <aixm:RunwayDirectionTimeSlice gml:id="RDN1">
              <aixm:interpretation>BASELINE</aixm:interpretation>
              <aixm:designator>09L</aixm:designator>
              <aixm:trueBearing>85.23</aixm:trueBearing>
              <aixm:usedRunway xlink:href="urn:uuid:9e51668f"/>
            </aixm:RunwayDirectionTimeSlice>
          </aixm:timeSlice>
        </aixm:RunwayDirection>
      </message:hasMember>
    </message:AIXMBasicMessage>"#;

    #[test]
    fn runway_cross_reference_resolution() {
        let nd = NavigationData::try_from_aixm(AIXM_DATA).unwrap();
        assert!(nd.errors().is_empty(), "{:?}", nd.errors());

        let arpt = nd.airports().find(|a| a.ident() == "EADD").unwrap();
        assert!(!arpt.runways.is_empty(), "EADD should have runways");

        let rwy = &arpt.runways[0];
        assert_eq!(rwy.designator, "09L");
        assert_eq!(rwy.surface, crate::nd::RunwaySurface::Concrete);
        assert!((rwy.length.to_si() - 2800.0).abs() < 0.1);
    }
}
