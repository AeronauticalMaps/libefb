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

use std::collections::HashSet;

use log::{debug, info, trace, warn};

use crate::error::Error;
use crate::measurements::{Length, LengthUnit};
use crate::nd::{
    AiracCycle, Airport, NavigationData, Region, SourceFormat, Waypoint, WaypointUsage,
};
use crate::VerticalDistance;

const PHONETIC_ALPHABET: [&str; 26] = [
    "ALFA", "BRAVO", "CHARLIE", "DELTA", "ECHO", "FOXTROT", "GOLF", "HOTEL", "INDIA", "JULIETT",
    "KILO", "LIMA", "MIKE", "NOVEMBER", "OSCAR", "PAPA", "QUEBEC", "ROMEO", "SIERRA", "TANGO",
    "UNIFORM", "VICTOR", "WHISKEY", "XRAY", "YANKEE", "ZULU",
];

impl NavigationData {
    /// Creates navigation data from an AIXM 5.x message.
    ///
    /// The features are taken over as they are valid on the message's
    /// effective date, and the AIRAC cycle is the one effective on that date.
    /// Aerodromes become airports with their ICAO location indicator as
    /// ident. Designated points that belong to an aerodrome become terminal
    /// waypoints of that aerodrome. Visual reporting points get an ident that
    /// is derived from their name, e.g. "EDDV NOVEMBER 1" becomes "N1" in the
    /// terminal area of EDDV.
    ///
    /// # Errors
    ///
    /// Returns an error if the message can't be read. Invalid records don't
    /// fail the navigation data but are listed in its errors.
    pub fn try_from_aixm(data: &[u8]) -> Result<Self, Error> {
        info!("loading navigation data from AIXM ({} bytes)", data.len());

        let message = aixm::read_message(data).map_err(|e| Error::InvalidAixm(e.to_string()))?;
        let cycle = AiracCycle::from(message.effective_date());
        debug!("{} effective in AIRAC cycle {}", message.version(), cycle);

        let mut builder = NavigationData::builder();
        let mut terminal_waypoints = HashSet::new();
        let mut airports = 0u32;
        let mut waypoints = 0u32;

        for error in message.errors() {
            warn!("skip record: {}", error);
            builder.add_error(Error::InvalidAixm(error.to_string()));
        }

        for arpt in message.airport_heliports() {
            match airport(arpt, cycle) {
                Ok(arpt) => {
                    trace!("loaded airport {}", arpt.icao_ident);
                    builder.add_airport(arpt);
                    airports += 1;
                }
                Err(e) => {
                    warn!("skip record: {}", e);
                    builder.add_error(e);
                }
            }
        }

        for point in message.designated_points() {
            match waypoint(point, cycle) {
                Ok(wp) => {
                    if let Some(airport) = wp.terminal_area() {
                        if !terminal_waypoints.insert((airport.to_string(), wp.fix_ident.clone())) {
                            warn!("terminal waypoint {} at {} is not unique", wp.fix_ident, airport);
                        }
                    }

                    trace!("loaded waypoint {}", wp.fix_ident);
                    builder.add_waypoint(wp);
                    waypoints += 1;
                }
                Err(e) => {
                    warn!("skip record: {}", e);
                    builder.add_error(e);
                }
            }
        }

        let nd = builder
            .with_source(data)
            .with_format(SourceFormat::Aixm)
            .build();

        info!(
            "AIXM loading complete: {} airports, {} waypoints",
            airports, waypoints
        );

        if !nd.errors().is_empty() {
            warn!("AIXM loading produced {} error(s)", nd.errors().len());
        }

        Ok(nd)
    }
}

fn airport(arpt: &aixm::AirportHeliport, cycle: AiracCycle) -> Result<Airport, Error> {
    let designator = arpt.designator.as_deref().unwrap_or_default();

    let icao_ident = arpt.location_indicator_icao.as_deref().ok_or_else(|| {
        Error::InvalidAixm(format!("aerodrome {designator} has no ICAO location indicator"))
    })?;

    let arp = arpt.arp.ok_or_else(|| {
        Error::InvalidAixm(format!("aerodrome {icao_ident} has no reference point"))
    })?;

    let elevation = match &arpt.field_elevation {
        Some(elevation) => msl(elevation)?,
        None => VerticalDistance::Gnd,
    };

    Ok(Airport {
        icao_ident: icao_ident.to_string(),
        iata_designator: arpt.designator_iata.clone().unwrap_or_default(),
        name: arpt.name.clone().unwrap_or_default(),
        coordinate: geo::Point::new(arp.longitude, arp.latitude),
        mag_var: None,
        elevation,
        runways: Vec::new(),
        location: None,
        cycle: Some(cycle),
    })
}

/// Converts an elevation in feet or metres to a distance above mean sea level
/// in feet.
fn msl(elevation: &aixm::Elevation) -> Result<VerticalDistance, Error> {
    let value = elevation.value as f32;

    let ft = match elevation.uom.as_str() {
        "FT" => value,
        "M" => *Length::m(value).convert_to(LengthUnit::Feet).value(),
        uom => return Err(Error::InvalidAixm(format!("unsupported elevation unit {uom}"))),
    };

    u16::try_from(ft.round() as i64)
        .map(VerticalDistance::Msl)
        .map_err(|_| Error::InvalidAixm(format!("elevation {ft} ft is out of range")))
}

fn waypoint(point: &aixm::DesignatedPoint, cycle: AiracCycle) -> Result<Waypoint, Error> {
    let designator = point
        .designator
        .as_deref()
        .ok_or_else(|| Error::InvalidAixm("designated point has no designator".to_string()))?;

    let location = point.location.ok_or_else(|| {
        Error::InvalidAixm(format!("designated point {designator} has no location"))
    })?;

    let airport = point.airport_heliport.as_ref().map(icao_code).transpose()?;
    let is_vrp = is_vrp(point);

    let fix_ident = match (airport, point.name.as_deref()) {
        (Some(airport), Some(name)) if is_vrp => vrp_ident(name, airport),
        _ => designator.to_string(),
    };

    Ok(Waypoint {
        fix_ident,
        desc: point.name.as_deref().unwrap_or(designator).to_string(),
        usage: if is_vrp {
            WaypointUsage::VFROnly
        } else {
            WaypointUsage::Unknown
        },
        coordinate: geo::Point::new(location.longitude, location.latitude),
        mag_var: None,
        region: match airport {
            Some(airport) => Region::TerminalArea(
                airport
                    .as_bytes()
                    .try_into()
                    .expect("ICAO code should be 4 bytes long"),
            ),
            None => Region::Enroute,
        },
        location: None,
        cycle: Some(cycle),
    })
}

/// Returns the ICAO code from the title of the reference to an aerodrome.
fn icao_code(reference: &aixm::Reference) -> Result<&str, Error> {
    reference
        .title
        .as_deref()
        .filter(|title| title.len() == 4 && title.bytes().all(|b| b.is_ascii_alphanumeric()))
        .ok_or_else(|| {
            Error::InvalidAixm(format!(
                "reference to aerodrome {} has no ICAO code",
                reference.uuid
            ))
        })
}

/// Returns `true` if a remark of the point marks it as visual reporting point.
fn is_vrp(point: &aixm::DesignatedPoint) -> bool {
    point
        .annotation
        .iter()
        .filter(|note| note.purpose.as_deref() == Some("REMARK"))
        .flat_map(|note| &note.translated_note)
        .any(|text| text.starts_with("VRP"))
}

/// Derives the ident of a visual reporting point from its name.
///
/// The aerodrome's prefix is stripped from the name. A phonetic alphabet word
/// with an optional number becomes its initial letter with the number, e.g.
/// "EDDV NOVEMBER 1" becomes "N1". Any other name is the ident without the
/// prefix, e.g. "EDTQ AUTOKINO" becomes "AUTOKINO".
fn vrp_ident(name: &str, airport: &str) -> String {
    let name = name
        .strip_prefix(airport)
        .and_then(|rest| rest.strip_prefix(' '))
        .unwrap_or(name);

    let mut words = name.split_whitespace();

    match (words.next(), words.next(), words.next()) {
        (Some(word), number, None)
            if PHONETIC_ALPHABET.contains(&word)
                && number.is_none_or(|number| number.bytes().all(|b| b.is_ascii_digit())) =>
        {
            format!("{}{}", &word[..1], number.unwrap_or_default())
        }
        _ => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nd::{Fix, NavAid};
    use crate::route::Route;

    const EDDH: &str = r#"
        <message:hasMember>
            <aixm:AirportHeliport gml:id="ID_839296db-4c81-4cb0-a22f-bb9c035483eb-0">
                <gml:identifier codeSpace="urn:uuid:">839296db-4c81-4cb0-a22f-bb9c035483eb</gml:identifier>
                <aixm:timeSlice>
                    <aixm:AirportHeliportTimeSlice gml:id="ID_TS_839296db-4c81-4cb0-a22f-bb9c035483eb-0">
                        <gml:validTime>
                            <gml:TimePeriod gml:id="ID_TS_VT_TP_839296db-4c81-4cb0-a22f-bb9c035483eb-0">
                                <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                <gml:endPosition>2026-11-25T23:59:59Z</gml:endPosition>
                            </gml:TimePeriod>
                        </gml:validTime>
                        <aixm:interpretation>BASELINE</aixm:interpretation>
                        <aixm:sequenceNumber>1</aixm:sequenceNumber>
                        <aixm:correctionNumber>1</aixm:correctionNumber>
                        <aixm:designator>EDDH</aixm:designator>
                        <aixm:name>HAMBURG</aixm:name>
                        <aixm:locationIndicatorICAO>EDDH</aixm:locationIndicatorICAO>
                        <aixm:designatorIATA>HAM</aixm:designatorIATA>
                        <aixm:type>AD</aixm:type>
                        <aixm:fieldElevation uom="FT">53</aixm:fieldElevation>
                        <aixm:verticalDatum>EGM_96</aixm:verticalDatum>
                        <aixm:magneticVariation>4.22</aixm:magneticVariation>
                        <aixm:servedCity>
                            <aixm:City gml:id="ID_TS_C_839296db-4c81-4cb0-a22f-bb9c035483eb-0">
                                <aixm:name>HAMBURG</aixm:name>
                            </aixm:City>
                        </aixm:servedCity>
                        <aixm:ARP>
                            <aixm:ElevatedPoint gml:id="ID_TS_839296db-4c81-4cb0-a22f-bb9c035483eb-0-0" srsName="urn:ogc:def:crs:EPSG::4326">
                                <gml:pos>53.630387834 9.988227138</gml:pos>
                                <aixm:horizontalAccuracy xsi:nil="true" nilReason="unknown"/>
                                <aixm:elevation uom="FT">36.0</aixm:elevation>
                                <aixm:geoidUndulation xsi:nil="true" nilReason="unknown"/>
                                <aixm:verticalAccuracy xsi:nil="true" nilReason="unknown"/>
                            </aixm:ElevatedPoint>
                        </aixm:ARP>
                    </aixm:AirportHeliportTimeSlice>
                </aixm:timeSlice>
            </aixm:AirportHeliport>
        </message:hasMember>"#;

    const EDHF: &str = r#"
        <message:hasMember>
            <aixm:AirportHeliport gml:id="ID_1463eef0-65d4-4b92-b357-1dd54b0b2e15-0">
                <gml:identifier codeSpace="urn:uuid:">1463eef0-65d4-4b92-b357-1dd54b0b2e15</gml:identifier>
                <aixm:timeSlice>
                    <aixm:AirportHeliportTimeSlice gml:id="ID_TS_1463eef0-65d4-4b92-b357-1dd54b0b2e15-0">
                        <gml:validTime>
                            <gml:TimePeriod gml:id="ID_TS_VT_TP_1463eef0-65d4-4b92-b357-1dd54b0b2e15-0">
                                <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                <gml:endPosition>2026-11-25T23:59:59Z</gml:endPosition>
                            </gml:TimePeriod>
                        </gml:validTime>
                        <aixm:interpretation>BASELINE</aixm:interpretation>
                        <aixm:sequenceNumber>1</aixm:sequenceNumber>
                        <aixm:correctionNumber>1</aixm:correctionNumber>
                        <aixm:designator>EDHF</aixm:designator>
                        <aixm:name>ITZEHOE/HUNGRIGER WOLF</aixm:name>
                        <aixm:locationIndicatorICAO>EDHF</aixm:locationIndicatorICAO>
                        <aixm:designatorIATA>IZE</aixm:designatorIATA>
                        <aixm:type>AD</aixm:type>
                        <aixm:fieldElevation uom="FT">82</aixm:fieldElevation>
                        <aixm:verticalDatum>EGM_96</aixm:verticalDatum>
                        <aixm:magneticVariation>4</aixm:magneticVariation>
                        <aixm:servedCity>
                            <aixm:City gml:id="ID_TS_C_1463eef0-65d4-4b92-b357-1dd54b0b2e15-0">
                                <aixm:name>ITZEHOE</aixm:name>
                            </aixm:City>
                        </aixm:servedCity>
                        <aixm:ARP>
                            <aixm:ElevatedPoint gml:id="ID_TS_1463eef0-65d4-4b92-b357-1dd54b0b2e15-0-0" srsName="urn:ogc:def:crs:EPSG::4326">
                                <gml:pos>53.9945 9.5785</gml:pos>
                                <aixm:horizontalAccuracy xsi:nil="true" nilReason="unknown"/>
                                <aixm:elevation xsi:nil="true" nilReason="unknown"/>
                                <aixm:geoidUndulation xsi:nil="true" nilReason="unknown"/>
                                <aixm:verticalDatum xsi:nil="true" nilReason="unknown"/>
                                <aixm:verticalAccuracy xsi:nil="true" nilReason="unknown"/>
                            </aixm:ElevatedPoint>
                        </aixm:ARP>
                    </aixm:AirportHeliportTimeSlice>
                </aixm:timeSlice>
            </aixm:AirportHeliport>
        </message:hasMember>"#;

    fn designated_point(designator: &str, name: &str, location: &str, extra: &str) -> String {
        format!(
            r#"<message:hasMember>
                <aixm:DesignatedPoint>
                    <gml:identifier>{designator}-{name}</gml:identifier>
                    <aixm:timeSlice>
                        <aixm:DesignatedPointTimeSlice>
                            <gml:validTime>
                                <gml:TimePeriod>
                                    <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                    <gml:endPosition>2026-11-26T00:00:00Z</gml:endPosition>
                                </gml:TimePeriod>
                            </gml:validTime>
                            <aixm:interpretation>BASELINE</aixm:interpretation>
                            <aixm:designator>{designator}</aixm:designator>
                            <aixm:name>{name}</aixm:name>
                            {location}
                            {extra}
                        </aixm:DesignatedPointTimeSlice>
                    </aixm:timeSlice>
                </aixm:DesignatedPoint>
            </message:hasMember>"#
        )
    }

    fn location(pos: &str) -> String {
        format!(
            r#"<aixm:location>
                <aixm:Point srsName="urn:ogc:def:crs:EPSG::4326">
                    <gml:pos>{pos}</gml:pos>
                </aixm:Point>
            </aixm:location>"#
        )
    }

    fn vrp_at(airport: &str) -> String {
        format!(
            r#"<aixm:airportHeliport xlink:href="urn:uuid:595049d5" xlink:title="{airport}"/>
            <aixm:annotation>
                <aixm:Note>
                    <aixm:purpose>REMARK</aixm:purpose>
                    <aixm:translatedNote>
                        <aixm:LinguisticNote>
                            <aixm:note lang="eng">VRP</aixm:note>
                        </aixm:LinguisticNote>
                    </aixm:translatedNote>
                </aixm:Note>
            </aixm:annotation>"#
        )
    }

    fn message(members: &[String]) -> String {
        format!(
            r#"<message:AIXMBasicMessage
                xmlns:message="http://www.aixm.aero/schema/5.1.1/message"
                xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
                xmlns:gmd="http://www.isotc211.org/2005/gmd"
                xmlns:gml="http://www.opengis.net/gml/3.2"
                xmlns:xlink="http://www.w3.org/1999/xlink"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
                <aixm:messageMetadata>
                    <gmd:EX_TemporalExtent>
                        <gmd:extent>
                            <gml:TimePeriod>
                                <gml:beginPosition>2026-10-29T00:00:00.000Z</gml:beginPosition>
                                <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00.000Z</gml:endPosition>
                            </gml:TimePeriod>
                        </gmd:extent>
                    </gmd:EX_TemporalExtent>
                </aixm:messageMetadata>
                {}
            </message:AIXMBasicMessage>"#,
            members.concat()
        )
    }

    fn navigation_data() -> NavigationData {
        let xml = message(&[
            EDDH.to_string(),
            EDHF.to_string(),
            designated_point(
                "N",
                "EDDH NOVEMBER 1",
                &location("53.80583333 10.0295833"),
                &vrp_at("EDDH"),
            ),
            designated_point("N", "EDDV NOVEMBER 1", &location("52.5 9.7"), &vrp_at("EDDV")),
            designated_point("AUT", "EDTQ AUTOKINO", &location("48.9 9.3"), &vrp_at("EDTQ")),
            designated_point("GISDI", "GISDI", &location("51.27 8.81"), ""),
            designated_point("BROKEN", "BROKEN", "", ""),
        ]);

        NavigationData::try_from_aixm(xml.as_bytes()).expect("AIXM should be readable")
    }

    #[test]
    fn derives_vrp_ident_from_name() {
        assert_eq!(vrp_ident("EDDV NOVEMBER 1", "EDDV"), "N1");
        assert_eq!(vrp_ident("EDDB MIKE", "EDDB"), "M");
        assert_eq!(vrp_ident("EDTQ AUTOKINO", "EDTQ"), "AUTOKINO");
    }

    #[test]
    fn loads_vrps_as_terminal_waypoints() {
        let nd = navigation_data();

        let Some(NavAid::Waypoint(n1)) = nd.find_terminal_waypoint("EDDV", "N1") else {
            panic!("EDDV N1 should be a terminal waypoint");
        };
        assert_eq!(n1.desc, "EDDV NOVEMBER 1");
        assert_eq!(n1.usage, WaypointUsage::VFROnly);
        assert_eq!(n1.coordinate(), geo::Point::new(9.7, 52.5));

        assert!(nd.find_terminal_waypoint("EDTQ", "AUTOKINO").is_some());
    }

    #[test]
    fn loads_aerodromes_as_airports() {
        let nd = navigation_data();

        let Some(NavAid::Airport(eddh)) = nd.find("EDDH") else {
            panic!("EDDH should be an airport");
        };
        assert_eq!(eddh.name, "HAMBURG");
        assert_eq!(eddh.elevation, VerticalDistance::Msl(53));
        assert_eq!(eddh.iata_designator, "HAM");
        assert_eq!(eddh.coordinate(), geo::Point::new(9.988227138, 53.630387834));
    }

    #[test]
    fn uses_ground_as_elevation_without_field_elevation() {
        let edhf = EDHF.replace(
            r#"<aixm:fieldElevation uom="FT">82</aixm:fieldElevation>"#,
            r#"<aixm:fieldElevation xsi:nil="true" nilReason="unknown"/>"#,
        );
        let xml = message(&[edhf]);
        let nd = NavigationData::try_from_aixm(xml.as_bytes()).expect("AIXM should be readable");

        let Some(NavAid::Airport(edhf)) = nd.find("EDHF") else {
            panic!("EDHF should be an airport");
        };
        assert_eq!(edhf.elevation, VerticalDistance::Gnd);
    }

    #[test]
    fn converts_elevation_in_metres_to_feet() {
        let elevation = aixm::Elevation {
            value: 16.0,
            uom: "M".to_string(),
        };
        assert!(matches!(msl(&elevation), Ok(VerticalDistance::Msl(52))));
    }

    #[test]
    fn decodes_route_via_vrp_of_aixm_airport() {
        let nd = navigation_data();
        let mut route = Route::new();

        route
            .decode("EDDH N1 EDHF", &nd)
            .expect("route via EDDH N1 should be decodable");

        let idents: Vec<_> = route
            .legs()
            .iter()
            .map(|leg| (leg.from().ident(), leg.to().ident()))
            .collect();
        assert_eq!(idents, [("EDDH", "N1"), ("N1", "EDHF")]);
    }

    #[test]
    fn loads_points_without_aerodrome_as_enroute_waypoints() {
        let nd = navigation_data();
        assert!(nd.find("GISDI").is_some());
    }

    #[test]
    fn collects_errors_of_invalid_records() {
        let nd = navigation_data();
        assert!(matches!(nd.errors(), [Error::InvalidAixm(_)]));
    }

    #[test]
    fn takes_cycle_from_effective_date() {
        let nd = navigation_data();
        assert_eq!(nd.cycle(), Some(&AiracCycle::new(26, 11)));
    }
}
