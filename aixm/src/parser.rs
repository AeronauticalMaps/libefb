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

//! Streaming AIXM 5.1 XML parser.
//!
//! Scans the XML byte stream for supported AIXM feature elements and
//! deserializes each subtree with serde into the public feature types.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::Error;
use crate::features::*;

/// Streaming iterator over AIXM 5.1 features in an XML document.
///
/// Yields one [`Feature`] at a time while scanning through the XML byte slice.
/// Only AIXM feature types relevant for navigation are returned — all other
/// elements are silently skipped.
///
/// # Supported feature types
///
/// | AIXM element             | Yielded as                          |
/// |--------------------------|-------------------------------------|
/// | `AirportHeliport`        | [`Feature::AirportHeliport`]        |
/// | `Runway`                 | [`Feature::Runway`]                 |
/// | `RunwayDirection`        | [`Feature::RunwayDirection`]        |
/// | `DesignatedPoint`        | [`Feature::DesignatedPoint`]        |
/// | `Navaid`                 | [`Feature::Navaid`]                 |
/// | `Airspace`               | [`Feature::Airspace`]               |
///
/// # Examples
///
/// Parse an AIXM file and collect all airport designators:
///
/// ```no_run
/// let data = std::fs::read("aixm_data.xml").unwrap();
/// let airports: Vec<String> = aixm::Features::new(&data)
///     .filter_map(Result::ok)
///     .filter_map(|f| match f {
///         aixm::Feature::AirportHeliport(ahp) => Some(ahp.designator().to_string()),
///         _ => None,
///     })
///     .collect();
/// ```
pub struct Features<'a> {
    reader: Reader<&'a [u8]>,
    data: &'a [u8],
    buf: Vec<u8>,
}

impl<'a> Features<'a> {
    /// Creates a new streaming parser from an AIXM XML byte slice.
    ///
    /// The returned iterator lazily parses features as they are consumed.
    ///
    /// # Examples
    ///
    /// ```
    /// let xml = br#"<message:AIXMBasicMessage
    ///   xmlns:aixm="http://www.aixm.aero/schema/5.1"
    ///   xmlns:gml="http://www.opengis.net/gml/3.2"
    ///   xmlns:message="http://www.aixm.aero/schema/5.1/message">
    /// </message:AIXMBasicMessage>"#;
    ///
    /// let features: Vec<_> = aixm::Features::new(&xml[..])
    ///     .collect::<Result<_, _>>()
    ///     .unwrap();
    ///
    /// assert!(features.is_empty());
    /// ```
    pub fn new(data: &'a [u8]) -> Self {
        let mut reader = Reader::from_reader(data);
        reader.config_mut().trim_text(true);
        Self {
            reader,
            data,
            buf: Vec::new(),
        }
    }
}

impl<'a> Iterator for Features<'a> {
    type Item = Result<Feature, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());

                    let kind = match local {
                        b"AirportHeliport" => FeatureKind::AirportHeliport,
                        b"Runway" => FeatureKind::Runway,
                        b"RunwayDirection" => FeatureKind::RunwayDirection,
                        b"DesignatedPoint" => FeatureKind::DesignatedPoint,
                        b"Navaid" => FeatureKind::Navaid,
                        b"Airspace" => FeatureKind::Airspace,
                        _ => continue,
                    };

                    let tag = String::from_utf8_lossy(e.as_ref()).to_string();
                    let end = e.to_end().into_owned();
                    let result = self
                        .reader
                        .read_to_end(end.name())
                        .map_err(Error::from)
                        .and_then(|span| {
                            let content = std::str::from_utf8(
                                &self.data[span.start as usize..span.end as usize],
                            )?;
                            let end_name = end.name();
                            let end_tag = std::str::from_utf8(end_name.as_ref())?;
                            let xml = format!("<{tag}>{content}</{end_tag}>");
                            deserialize_feature(kind, &xml)
                        });

                    return Some(result);
                }
                Ok(Event::Eof) => return None,
                Err(e) => return Some(Err(e.into())),
                _ => continue,
            }
        }
    }
}

enum FeatureKind {
    AirportHeliport,
    Runway,
    RunwayDirection,
    DesignatedPoint,
    Navaid,
    Airspace,
}

fn deserialize_feature(kind: FeatureKind, xml: &str) -> Result<Feature, Error> {
    Ok(match kind {
        FeatureKind::AirportHeliport => {
            Feature::AirportHeliport(quick_xml::de::from_str(xml)?)
        }
        FeatureKind::Runway => Feature::Runway(quick_xml::de::from_str(xml)?),
        FeatureKind::RunwayDirection => {
            Feature::RunwayDirection(quick_xml::de::from_str(xml)?)
        }
        FeatureKind::DesignatedPoint => {
            Feature::DesignatedPoint(quick_xml::de::from_str(xml)?)
        }
        FeatureKind::Navaid => Feature::Navaid(quick_xml::de::from_str(xml)?),
        FeatureKind::Airspace => Feature::Airspace(quick_xml::de::from_str(xml)?),
    })
}

/// Returns the local name portion of a possibly namespace-prefixed XML name.
fn local_name(name: &[u8]) -> &[u8] {
    name.iter()
        .position(|&b| b == b':')
        .map_or(name, |pos| &name[pos + 1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_airport_heliport_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:AirportHeliport gml:id="uuid.dd062d88-3e64-4a5d-bebd-89476db9ebea">
              <gml:identifier codeSpace="urn:uuid:">dd062d88-3e64-4a5d-bebd-89476db9ebea</gml:identifier>
              <aixm:timeSlice>
                <aixm:AirportHeliportTimeSlice gml:id="AHP_EADH">
                  <gml:validTime>
                    <gml:TimePeriod gml:id="vt1">
                      <gml:beginPosition>2009-01-01T00:00:00Z</gml:beginPosition>
                      <gml:endPosition indeterminatePosition="unknown"/>
                    </gml:TimePeriod>
                  </gml:validTime>
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:sequenceNumber>1</aixm:sequenceNumber>
                  <aixm:designator>EADH</aixm:designator>
                  <aixm:name>DONLON/DOWNTOWN HELIPORT</aixm:name>
                  <aixm:locationIndicatorICAO>EADH</aixm:locationIndicatorICAO>
                  <aixm:fieldElevation uom="M">18</aixm:fieldElevation>
                  <aixm:ARP>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep1">
                      <gml:pos>52.288888888888884 -32.035</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:ARP>
                </aixm:AirportHeliportTimeSlice>
              </aixm:timeSlice>
            </aixm:AirportHeliport>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::AirportHeliport(ahp) => {
                assert_eq!(ahp.uuid(), "dd062d88-3e64-4a5d-bebd-89476db9ebea");
                assert_eq!(ahp.designator(), "EADH");
                assert_eq!(ahp.name(), "DONLON/DOWNTOWN HELIPORT");
                assert_eq!(ahp.location_indicator_icao(), Some("EADH"));
                let (elev, uom) = ahp.field_elevation();
                assert_eq!(elev, Some(18.0));
                assert_eq!(uom, Some("M"));
                let (lat, lon) = ahp.coordinate().unwrap();
                assert!((lat - 52.2889).abs() < 0.001);
                assert!((lon - (-32.035)).abs() < 0.001);
            }
            _ => panic!("expected AirportHeliport"),
        }
    }

    #[test]
    fn parse_runway_and_direction() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:Runway gml:id="uuid.9e51668f-bf8a-4f5b-ba6e-27087972b9b8">
              <gml:identifier codeSpace="urn:uuid:">9e51668f-bf8a-4f5b-ba6e-27087972b9b8</gml:identifier>
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
                  <aixm:associatedAirportHeliport xlink:href="urn:uuid:1b54b2d6-a5ff-4e57-94c2-f4047a381c64"/>
                </aixm:RunwayTimeSlice>
              </aixm:timeSlice>
            </aixm:Runway>
          </message:hasMember>
          <message:hasMember>
            <aixm:RunwayDirection gml:id="uuid.c8455a6b-9319-4bb7-b797-08e644342d64">
              <gml:identifier codeSpace="urn:uuid:">c8455a6b-9319-4bb7-b797-08e644342d64</gml:identifier>
              <aixm:timeSlice>
                <aixm:RunwayDirectionTimeSlice gml:id="RDN1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>09L</aixm:designator>
                  <aixm:trueBearing>85.23</aixm:trueBearing>
                  <aixm:usedRunway xlink:href="urn:uuid:9e51668f-bf8a-4f5b-ba6e-27087972b9b8"/>
                </aixm:RunwayDirectionTimeSlice>
              </aixm:timeSlice>
            </aixm:RunwayDirection>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 2);

        match &features[0] {
            Feature::Runway(rwy) => {
                assert_eq!(rwy.designator(), "09L/27R");
                let (len, uom) = rwy.nominal_length();
                assert_eq!(len, Some(2800.0));
                assert_eq!(uom, Some("M"));
                assert_eq!(rwy.surface_composition(), Some("CONC"));
                assert_eq!(
                    rwy.associated_airport_uuid(),
                    Some("1b54b2d6-a5ff-4e57-94c2-f4047a381c64")
                );
            }
            _ => panic!("expected Runway"),
        }

        match &features[1] {
            Feature::RunwayDirection(rdn) => {
                assert_eq!(rdn.designator(), "09L");
                assert_eq!(rdn.true_bearing(), Some(85.23));
                assert_eq!(
                    rdn.used_runway_uuid(),
                    Some("9e51668f-bf8a-4f5b-ba6e-27087972b9b8")
                );
            }
            _ => panic!("expected RunwayDirection"),
        }
    }

    #[test]
    fn parse_designated_point_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:DesignatedPoint gml:id="uuid.abc123">
              <gml:identifier codeSpace="urn:uuid:">abc123</gml:identifier>
              <aixm:timeSlice>
                <aixm:DesignatedPointTimeSlice gml:id="DP1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>ABLAN</aixm:designator>
                  <aixm:name>ABLAN</aixm:name>
                  <aixm:type>ICAO</aixm:type>
                  <aixm:location>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326">
                      <gml:pos>52.123 10.456</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:DesignatedPointTimeSlice>
              </aixm:timeSlice>
            </aixm:DesignatedPoint>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::DesignatedPoint(dp) => {
                assert_eq!(dp.designator(), "ABLAN");
                assert_eq!(dp.name(), Some("ABLAN"));
                assert_eq!(dp.point_type(), Some("ICAO"));
                let (lat, lon) = dp.coordinate().unwrap();
                assert!((lat - 52.123).abs() < 0.001);
                assert!((lon - 10.456).abs() < 0.001);
            }
            _ => panic!("expected DesignatedPoint"),
        }
    }

    #[test]
    fn parse_navaid_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:Navaid gml:id="uuid.08a1bbd5-ea70-4fe3-836a-ea9686349495">
              <gml:identifier codeSpace="urn:uuid:">08a1bbd5-ea70-4fe3-836a-ea9686349495</gml:identifier>
              <aixm:timeSlice>
                <aixm:NavaidTimeSlice gml:id="NAV_BOR">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:type>VOR_DME</aixm:type>
                  <aixm:designator>BOR</aixm:designator>
                  <aixm:name>BOORSPIJK</aixm:name>
                  <aixm:location>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep1">
                      <gml:pos>52.368389 -32.375222</gml:pos>
                      <aixm:elevation uom="M">30.0</aixm:elevation>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:NavaidTimeSlice>
              </aixm:timeSlice>
            </aixm:Navaid>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::Navaid(nav) => {
                assert_eq!(nav.designator(), "BOR");
                assert_eq!(nav.name(), Some("BOORSPIJK"));
                assert_eq!(nav.navaid_type(), Some("VOR_DME"));
                let (lat, lon) = nav.coordinate().unwrap();
                assert!((lat - 52.368389).abs() < 0.0001);
                assert!((lon - (-32.375222)).abs() < 0.0001);
                assert_eq!(nav.elevation(), Some(30.0));
            }
            _ => panic!("expected Navaid"),
        }
    }

    #[test]
    fn parse_airspace_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:Airspace gml:id="uuid.4fd9f4be-8c65-43f6-b083-3ced9a4b2a7f">
              <gml:identifier codeSpace="urn:uuid:">4fd9f4be-8c65-43f6-b083-3ced9a4b2a7f</gml:identifier>
              <aixm:timeSlice>
                <aixm:AirspaceTimeSlice gml:id="ASE1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:type>CTR</aixm:type>
                  <aixm:designator>EADD CTR</aixm:designator>
                  <aixm:name>DONLON CTR</aixm:name>
                  <aixm:geometryComponent>
                    <aixm:AirspaceGeometryComponent gml:id="AGC1">
                      <aixm:theAirspaceVolume>
                        <aixm:AirspaceVolume gml:id="AV1">
                          <aixm:upperLimit uom="FL">195</aixm:upperLimit>
                          <aixm:upperLimitReference>MSL</aixm:upperLimitReference>
                          <aixm:lowerLimit>GND</aixm:lowerLimit>
                          <aixm:lowerLimitReference>SFC</aixm:lowerLimitReference>
                          <aixm:horizontalProjection>
                            <aixm:Surface srsName="urn:ogc:def:crs:EPSG::4326" gml:id="S1">
                              <gml:patches>
                                <gml:PolygonPatch>
                                  <gml:exterior>
                                    <gml:Ring>
                                      <gml:curveMember>
                                        <gml:Curve gml:id="C1">
                                          <gml:segments>
                                            <gml:GeodesicString>
                                              <gml:posList>52.0 -32.0 52.5 -32.0 52.5 -31.5 52.0 -31.5 52.0 -32.0</gml:posList>
                                            </gml:GeodesicString>
                                          </gml:segments>
                                        </gml:Curve>
                                      </gml:curveMember>
                                    </gml:Ring>
                                  </gml:exterior>
                                </gml:PolygonPatch>
                              </gml:patches>
                            </aixm:Surface>
                          </aixm:horizontalProjection>
                        </aixm:AirspaceVolume>
                      </aixm:theAirspaceVolume>
                    </aixm:AirspaceGeometryComponent>
                  </aixm:geometryComponent>
                </aixm:AirspaceTimeSlice>
              </aixm:timeSlice>
            </aixm:Airspace>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::Airspace(arsp) => {
                assert_eq!(arsp.airspace_type(), Some("CTR"));
                assert_eq!(arsp.designator(), Some("EADD CTR"));
                assert_eq!(arsp.name(), Some("DONLON CTR"));
                let volumes = arsp.volumes();
                assert_eq!(volumes.len(), 1);

                let vol = &volumes[0];
                assert_eq!(vol.upper_limit.as_deref(), Some("195"));
                assert_eq!(vol.upper_limit_uom.as_deref(), Some("FL"));
                assert_eq!(vol.upper_limit_ref.as_deref(), Some("MSL"));
                assert_eq!(vol.lower_limit.as_deref(), Some("GND"));
                assert_eq!(vol.lower_limit_ref.as_deref(), Some("SFC"));
                assert_eq!(vol.polygon.len(), 5);
                assert!((vol.polygon[0].0 - 52.0).abs() < 0.001);
                assert!((vol.polygon[0].1 - (-32.0)).abs() < 0.001);
            }
            _ => panic!("expected Airspace"),
        }
    }

    #[test]
    fn skips_unsupported_features() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:OrganisationAuthority gml:id="uuid.org1">
              <aixm:timeSlice>
                <aixm:OrganisationAuthorityTimeSlice gml:id="OA1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:name>SOME ORG</aixm:name>
                </aixm:OrganisationAuthorityTimeSlice>
              </aixm:timeSlice>
            </aixm:OrganisationAuthority>
          </message:hasMember>
          <message:hasMember>
            <aixm:DesignatedPoint gml:id="uuid.dp1">
              <aixm:timeSlice>
                <aixm:DesignatedPointTimeSlice gml:id="DPT1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>ALPHA</aixm:designator>
                  <aixm:location>
                    <aixm:ElevatedPoint>
                      <gml:pos>50.0 8.0</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:DesignatedPointTimeSlice>
              </aixm:timeSlice>
            </aixm:DesignatedPoint>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);
        assert!(matches!(&features[0], Feature::DesignatedPoint(_)));
    }
}
