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

use efb::nd::{Fix, NavigationData};

const AIXM_DATA: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<message:AIXMBasicMessage
  xmlns:aixm="http://www.aixm.aero/schema/5.1"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:message="http://www.aixm.aero/schema/5.1/message"
  xmlns:xlink="http://www.w3.org/1999/xlink">

  <!-- Airport -->
  <message:hasMember>
    <aixm:AirportHeliport gml:id="uuid.1b54b2d6-a5ff-4e57-94c2-f4047a381c64">
      <gml:identifier codeSpace="urn:uuid:">1b54b2d6-a5ff-4e57-94c2-f4047a381c64</gml:identifier>
      <aixm:timeSlice>
        <aixm:AirportHeliportTimeSlice gml:id="AHP_EADD">
          <gml:validTime>
            <gml:TimePeriod gml:id="vt1">
              <gml:beginPosition>2017-07-01T00:00:00Z</gml:beginPosition>
              <gml:endPosition indeterminatePosition="unknown"/>
            </gml:TimePeriod>
          </gml:validTime>
          <aixm:interpretation>BASELINE</aixm:interpretation>
          <aixm:sequenceNumber>1</aixm:sequenceNumber>
          <aixm:designator>EADD</aixm:designator>
          <aixm:name>DONLON/INTL</aixm:name>
          <aixm:locationIndicatorICAO>EADD</aixm:locationIndicatorICAO>
          <aixm:designatorIATA>DON</aixm:designatorIATA>
          <aixm:fieldElevation uom="M">30</aixm:fieldElevation>
          <aixm:ARP>
            <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep1">
              <gml:pos>52.3600 -31.9400</gml:pos>
            </aixm:ElevatedPoint>
          </aixm:ARP>
        </aixm:AirportHeliportTimeSlice>
      </aixm:timeSlice>
    </aixm:AirportHeliport>
  </message:hasMember>

  <!-- Runway -->
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

  <!-- Runway Direction -->
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

  <!-- Designated Point -->
  <message:hasMember>
    <aixm:DesignatedPoint gml:id="uuid.dp001">
      <gml:identifier codeSpace="urn:uuid:">dp001</gml:identifier>
      <aixm:timeSlice>
        <aixm:DesignatedPointTimeSlice gml:id="DP1">
          <aixm:interpretation>BASELINE</aixm:interpretation>
          <aixm:designator>ABLAN</aixm:designator>
          <aixm:name>ABLAN</aixm:name>
          <aixm:type>ICAO</aixm:type>
          <aixm:location>
            <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326">
              <gml:pos>52.123 -31.456</gml:pos>
            </aixm:ElevatedPoint>
          </aixm:location>
        </aixm:DesignatedPointTimeSlice>
      </aixm:timeSlice>
    </aixm:DesignatedPoint>
  </message:hasMember>

  <!-- Navaid -->
  <message:hasMember>
    <aixm:Navaid gml:id="uuid.nav001">
      <gml:identifier codeSpace="urn:uuid:">nav001</gml:identifier>
      <aixm:timeSlice>
        <aixm:NavaidTimeSlice gml:id="NAV1">
          <aixm:interpretation>BASELINE</aixm:interpretation>
          <aixm:type>VOR_DME</aixm:type>
          <aixm:designator>BOR</aixm:designator>
          <aixm:name>BOORSPIJK</aixm:name>
          <aixm:location>
            <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep2">
              <gml:pos>52.368389 -32.375222</gml:pos>
              <aixm:elevation uom="M">30.0</aixm:elevation>
            </aixm:ElevatedPoint>
          </aixm:location>
        </aixm:NavaidTimeSlice>
      </aixm:timeSlice>
    </aixm:Navaid>
  </message:hasMember>

  <!-- Airspace -->
  <message:hasMember>
    <aixm:Airspace gml:id="uuid.arsp001">
      <gml:identifier codeSpace="urn:uuid:">arsp001</gml:identifier>
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
                  <aixm:upperLimit uom="FL">65</aixm:upperLimit>
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
                                      <gml:posList>52.0 -32.5 52.7 -32.5 52.7 -31.5 52.0 -31.5 52.0 -32.5</gml:posList>
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

#[test]
fn parse_aixm_navigation_data() {
    let nd = NavigationData::try_from_aixm(AIXM_DATA).expect("should parse AIXM data");

    assert!(
        nd.errors().is_empty(),
        "should have no errors: {:?}",
        nd.errors()
    );

    // Airport: EADD
    let arpt = nd.find("EADD").expect("EADD should be found");
    assert_eq!(arpt.ident(), "EADD");
    assert!((arpt.coordinate().latitude - 52.36).abs() < 0.01);
    assert!((arpt.coordinate().longitude - (-31.94)).abs() < 0.01);

    // Designated Point: ABLAN
    let ablan = nd.find("ABLAN").expect("ABLAN should be found");
    assert_eq!(ablan.ident(), "ABLAN");
    assert!((ablan.coordinate().latitude - 52.123).abs() < 0.01);

    // Navaid: BOR
    let bor = nd.find("BOR").expect("BOR should be found");
    assert_eq!(bor.ident(), "BOR");
    assert!((bor.coordinate().latitude - 52.368389).abs() < 0.001);

    // Airspace: DONLON CTR should contain the airport
    let inside = efb::geom::Coordinate::new(52.36, -31.94);
    let airspaces = nd.at(&inside);
    assert_eq!(airspaces.len(), 1);
    assert_eq!(airspaces[0].name, "DONLON CTR");
}
