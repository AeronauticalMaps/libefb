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

use std::collections::{HashMap, HashSet};

use log::{debug, info, trace, warn};

use crate::error::Error;
use crate::measurements::{Angle, Length, LengthUnit};
use crate::nd::{
    AiracCycle, Airport, NavigationData, Region, Runway, RunwaySurface, SourceFormat, Waypoint,
    WaypointUsage,
};
use crate::VerticalDistance;

const PHONETIC_ALPHABET: [&str; 26] = [
    "ALFA", "BRAVO", "CHARLIE", "DELTA", "ECHO", "FOXTROT", "GOLF", "HOTEL", "INDIA", "JULIETT",
    "KILO", "LIMA", "MIKE", "NOVEMBER", "OSCAR", "PAPA", "QUEBEC", "ROMEO", "SIERRA", "TANGO",
    "UNIFORM", "VICTOR", "WHISKEY", "XRAY", "YANKEE", "ZULU",
];

impl NavigationData {
    /// Creates navigation data from AIXM 5.x messages.
    ///
    /// All messages go into one partition, so a feature may refer to a
    /// feature of another message, e.g. a runway to its aerodrome. The
    /// features of each message are taken over as they are valid on the
    /// message's effective date, and the AIRAC cycle is the one effective on
    /// that date. Aerodromes become airports with their ICAO location
    /// indicator as ident, and runway directions become the runways of their
    /// aerodrome. Designated points that belong to an aerodrome become
    /// terminal waypoints of that aerodrome. Visual reporting points get an
    /// ident that is derived from their name, e.g. "EDDV NOVEMBER 1" becomes
    /// "N1" in the terminal area of EDDV.
    ///
    /// # Errors
    ///
    /// Returns an error if a message can't be read. Invalid records don't
    /// fail the navigation data but are listed in its errors.
    pub fn try_from_aixm(data: &[&[u8]]) -> Result<Self, Error> {
        info!(
            "loading navigation data from {} AIXM message(s) ({} bytes)",
            data.len(),
            data.iter().map(|message| message.len()).sum::<usize>()
        );

        let messages = data
            .iter()
            .map(|message| aixm::read_message(*message))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::InvalidAixm(e.to_string()))?;

        let aerodromes: HashMap<&str, &str> = messages
            .iter()
            .flat_map(|message| message.airport_heliports())
            .filter_map(|(uuid, arpt)| Some((uuid, arpt.location_indicator_icao.as_deref()?)))
            .collect();

        let declared_distances: HashMap<&str, &[aixm::RunwayDeclaredDistance]> = messages
            .iter()
            .flat_map(|message| message.runway_centreline_points())
            .filter(|point| {
                matches!(point.role.as_deref(), Some("THR" | "DISTHR" | "OTHER:RDD"))
            })
            .filter(|point| !point.associated_declared_distance.is_empty())
            .filter_map(|point| {
                Some((
                    point.on_runway.as_ref()?.uuid.as_str(),
                    point.associated_declared_distance.as_slice(),
                ))
            })
            .collect();

        let mut builder = NavigationData::builder();
        let mut terminal_waypoints = HashSet::new();
        let mut airports = 0u32;
        let mut runways = 0u32;
        let mut waypoints = 0u32;

        for message in &messages {
            let cycle = AiracCycle::from(message.effective_date());
            debug!("{} effective in AIRAC cycle {}", message.version(), cycle);

            for error in message.errors() {
                warn!("skip record: {}", error);
                builder.add_error(Error::InvalidAixm(error.to_string()));
            }

            for (_, arpt) in message.airport_heliports() {
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

            for (uuid, direction) in message.runway_directions() {
                let used_runway = direction
                    .used_runway
                    .as_ref()
                    .and_then(|reference| messages.iter().find_map(|other| other.runway(reference)));

                let result = match used_runway {
                    Some(rwy) => runway(
                        direction,
                        rwy,
                        declared_distances.get(uuid).copied().unwrap_or_default(),
                        &aerodromes,
                    ),
                    None => Err(Error::InvalidAixm(format!(
                        "runway direction {uuid} has no runway"
                    ))),
                };

                match result {
                    Ok(Some((airport, rwy))) => {
                        trace!("loaded runway {} at {}", rwy.designator, airport);
                        builder.add_runway(airport.to_string(), rwy);
                        runways += 1;
                    }
                    Ok(None) => trace!("skip FATO direction {}", uuid),
                    Err(e) => {
                        warn!("skip record: {}", e);
                        builder.add_error(e);
                    }
                }
            }

            for point in message.designated_points() {
                match waypoint(point, &aerodromes, cycle) {
                    Ok(wp) => {
                        if let Some(airport) = wp.terminal_area() {
                            if !terminal_waypoints
                                .insert((airport.to_string(), wp.fix_ident.clone()))
                            {
                                warn!(
                                    "terminal waypoint {} at {} is not unique",
                                    wp.fix_ident, airport
                                );
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
        }

        let nd = builder
            .with_source(data)
            .with_format(SourceFormat::Aixm)
            .build();

        info!(
            "AIXM loading complete: {} airports, {} runways, {} waypoints",
            airports, runways, waypoints
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

impl TryFrom<&aixm::Measure> for Length {
    type Error = Error;

    fn try_from(measure: &aixm::Measure) -> Result<Self, Self::Error> {
        let value = measure.value as f32;

        match measure.uom.as_str() {
            "M" => Ok(Length::m(value)),
            "FT" => Ok(Length::ft(value)),
            uom => Err(Error::InvalidAixm(format!("unsupported length unit {uom}"))),
        }
    }
}

/// Converts an elevation in feet or metres to a distance above mean sea level
/// in feet.
fn msl(elevation: &aixm::Measure) -> Result<VerticalDistance, Error> {
    let ft = *Length::try_from(elevation)?
        .convert_to(LengthUnit::Feet)
        .value();

    u16::try_from(ft.round() as i64)
        .map(VerticalDistance::Msl)
        .map_err(|_| Error::InvalidAixm(format!("elevation {ft} ft is out of range")))
}

/// Converts a runway direction with the distances declared from its
/// threshold to a runway of its aerodrome. Returns `None` for a direction of
/// a FATO.
fn runway<'a>(
    direction: &aixm::RunwayDirection,
    rwy: &'a aixm::Runway,
    distances: &[aixm::RunwayDeclaredDistance],
    aerodromes: &HashMap<&str, &'a str>,
) -> Result<Option<(&'a str, Runway)>, Error> {
    if rwy.kind.as_deref() == Some("FATO") {
        return Ok(None);
    }

    let designator = direction
        .designator
        .as_deref()
        .ok_or_else(|| Error::InvalidAixm("runway direction has no designator".to_string()))?;

    let airport = rwy
        .associated_airport_heliport
        .as_ref()
        .ok_or_else(|| Error::InvalidAixm(format!("runway {designator} has no aerodrome")))?;
    let airport = aerodrome(aerodromes, airport)?;

    let bearing = direction.magnetic_bearing.ok_or_else(|| {
        Error::InvalidAixm(format!("runway {designator} at {airport} has no magnetic bearing"))
    })?;

    let length = rwy.nominal_length.as_ref().ok_or_else(|| {
        Error::InvalidAixm(format!("runway {designator} at {airport} has no length"))
    })?;

    Ok(Some((
        airport,
        Runway {
            designator: designator.to_string(),
            bearing: Angle::m(bearing as f32),
            length: Length::try_from(length)?,
            tora: declared(distances, "TORA")?,
            toda: declared(distances, "TODA")?,
            lda: declared(distances, "LDA")?,
            displaced_threshold: declared(distances, "DTHR")?,
            surface: match rwy.composition.as_deref() {
                Some("ASPH" | "CONC_ASPH") => Some(RunwaySurface::Asphalt),
                Some("CONC") => Some(RunwaySurface::Concrete),
                Some("GRASS") => Some(RunwaySurface::Grass),
                _ => None,
            },
            slope: None,
            elev: VerticalDistance::Gnd,
        },
    )))
}

/// Returns the declared distance of `kind`, e.g. TORA, among `distances`.
fn declared(
    distances: &[aixm::RunwayDeclaredDistance],
    kind: &str,
) -> Result<Option<Length>, Error> {
    distances
        .iter()
        .find(|declared| declared.kind.as_deref() == Some(kind))
        .and_then(|declared| declared.distance.as_ref())
        .map(Length::try_from)
        .transpose()
}

fn waypoint(
    point: &aixm::DesignatedPoint,
    aerodromes: &HashMap<&str, &str>,
    cycle: AiracCycle,
) -> Result<Waypoint, Error> {
    let designator = point
        .designator
        .as_deref()
        .ok_or_else(|| Error::InvalidAixm("designated point has no designator".to_string()))?;

    let location = point.location.ok_or_else(|| {
        Error::InvalidAixm(format!("designated point {designator} has no location"))
    })?;

    let airport = point
        .airport_heliport
        .as_ref()
        .map(|reference| aerodrome(aerodromes, reference))
        .transpose()?;
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
            Some(airport) => Region::TerminalArea(airport.as_bytes().try_into().map_err(|_| {
                Error::InvalidAixm(format!("{airport:?} is not an ICAO code"))
            })?),
            None => Region::Enroute,
        },
        location: None,
        cycle: Some(cycle),
    })
}

/// Returns the ICAO code of the aerodrome that `reference` points to, or the
/// one in its title if the aerodrome is in none of the messages.
fn aerodrome<'a>(
    aerodromes: &HashMap<&str, &'a str>,
    reference: &'a aixm::Reference,
) -> Result<&'a str, Error> {
    match aerodromes.get(reference.uuid.as_str()) {
        Some(&icao_code) => Ok(icao_code),
        None => icao_code(reference),
    }
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

    const RWY_EDHF: &str = r#"
        <message:hasMember>
            <aixm:Runway gml:id="ID_e1ff5b8a-89c2-48df-9bfc-32e5616f239f">
                <gml:identifier codeSpace="urn:uuid:">e1ff5b8a-89c2-48df-9bfc-32e5616f239f</gml:identifier>
                <aixm:timeSlice>
                    <aixm:RunwayTimeSlice gml:id="ID_TS_e1ff5b8a-89c2-48df-9bfc-32e5616f239f">
                        <gml:validTime>
                            <gml:TimePeriod gml:id="ID_TS_VT_TP_e1ff5b8a-89c2-48df-9bfc-32e5616f239f">
                                <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                <gml:endPosition>2026-11-25T23:59:59Z</gml:endPosition>
                            </gml:TimePeriod>
                        </gml:validTime>
                        <aixm:interpretation>BASELINE</aixm:interpretation>
                        <aixm:designator>02/20</aixm:designator>
                        <aixm:type>RWY</aixm:type>
                        <aixm:nominalLength uom="M">1040</aixm:nominalLength>
                        <aixm:surfaceProperties>
                            <aixm:SurfaceCharacteristics gml:id="ID_TS_SC_e1ff5b8a-89c2-48df-9bfc-32e5616f239f">
                                <aixm:composition>GRASS</aixm:composition>
                            </aixm:SurfaceCharacteristics>
                        </aixm:surfaceProperties>
                        <aixm:associatedAirportHeliport xlink:href="urn:uuid:1463eef0-65d4-4b92-b357-1dd54b0b2e15" xlink:title="EDHF"/>
                    </aixm:RunwayTimeSlice>
                </aixm:timeSlice>
            </aixm:Runway>
        </message:hasMember>"#;

    const RWY_DIRECTION_EDHF_20: &str = r#"
        <message:hasMember>
            <aixm:RunwayDirection gml:id="ID_10a97dbb-16bf-4be6-8a33-630daa28915a">
                <gml:identifier codeSpace="urn:uuid:">10a97dbb-16bf-4be6-8a33-630daa28915a</gml:identifier>
                <aixm:timeSlice>
                    <aixm:RunwayDirectionTimeSlice gml:id="ID_TS_10a97dbb-16bf-4be6-8a33-630daa28915a">
                        <gml:validTime>
                            <gml:TimePeriod gml:id="ID_TS_VT_TP_10a97dbb-16bf-4be6-8a33-630daa28915a">
                                <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                <gml:endPosition>2026-11-25T23:59:59Z</gml:endPosition>
                            </gml:TimePeriod>
                        </gml:validTime>
                        <aixm:interpretation>BASELINE</aixm:interpretation>
                        <aixm:designator>20</aixm:designator>
                        <aixm:trueBearing>206</aixm:trueBearing>
                        <aixm:magneticBearing>202</aixm:magneticBearing>
                        <aixm:usedRunway xlink:href="urn:uuid:e1ff5b8a-89c2-48df-9bfc-32e5616f239f" xlink:title="02/20"/>
                    </aixm:RunwayDirectionTimeSlice>
                </aixm:timeSlice>
            </aixm:RunwayDirection>
        </message:hasMember>"#;

    const THR_EDHF_20: &str = r#"
        <message:hasMember>
            <aixm:RunwayCentrelinePoint gml:id="ID_014f6782-7c8f-41f8-a0db-d861e7d1e522">
                <gml:identifier codeSpace="urn:uuid:">014f6782-7c8f-41f8-a0db-d861e7d1e522</gml:identifier>
                <aixm:timeSlice>
                    <aixm:RunwayCentrelinePointTimeSlice gml:id="ID_TS_014f6782-7c8f-41f8-a0db-d861e7d1e522">
                        <gml:validTime>
                            <gml:TimePeriod gml:id="ID_TS_VT_TP_014f6782-7c8f-41f8-a0db-d861e7d1e522">
                                <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                <gml:endPosition>2026-11-25T23:59:59Z</gml:endPosition>
                            </gml:TimePeriod>
                        </gml:validTime>
                        <aixm:interpretation>BASELINE</aixm:interpretation>
                        <aixm:role>THR</aixm:role>
                        <aixm:onRunway xlink:href="urn:uuid:10a97dbb-16bf-4be6-8a33-630daa28915a" xlink:title="20"/>
                        <aixm:associatedDeclaredDistance>
                            <aixm:RunwayDeclaredDistance gml:id="ID_TS_DD_3bcbbf50-caae-4bff-bb76-8f2efe1dffcf">
                                <aixm:type>TORA</aixm:type>
                                <aixm:declaredValue>
                                    <aixm:RunwayDeclaredDistanceValue gml:id="ID_TS_DD_V_3bcbbf50-caae-4bff-bb76-8f2efe1dffcf">
                                        <aixm:distance uom="M">1040</aixm:distance>
                                    </aixm:RunwayDeclaredDistanceValue>
                                </aixm:declaredValue>
                            </aixm:RunwayDeclaredDistance>
                        </aixm:associatedDeclaredDistance>
                        <aixm:associatedDeclaredDistance>
                            <aixm:RunwayDeclaredDistance gml:id="ID_TS_DD_099b721d-f4cd-44d4-b225-d0bb670a552d">
                                <aixm:type>LDA</aixm:type>
                                <aixm:declaredValue>
                                    <aixm:RunwayDeclaredDistanceValue gml:id="ID_TS_DD_V_099b721d-f4cd-44d4-b225-d0bb670a552d">
                                        <aixm:distance uom="M">1040</aixm:distance>
                                    </aixm:RunwayDeclaredDistanceValue>
                                </aixm:declaredValue>
                            </aixm:RunwayDeclaredDistance>
                        </aixm:associatedDeclaredDistance>
                    </aixm:RunwayCentrelinePointTimeSlice>
                </aixm:timeSlice>
            </aixm:RunwayCentrelinePoint>
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

        NavigationData::try_from_aixm(&[xml.as_bytes()]).expect("AIXM should be readable")
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
        let nd = NavigationData::try_from_aixm(&[xml.as_bytes()]).expect("AIXM should be readable");

        let Some(NavAid::Airport(edhf)) = nd.find("EDHF") else {
            panic!("EDHF should be an airport");
        };
        assert_eq!(edhf.elevation, VerticalDistance::Gnd);
    }

    #[test]
    fn converts_elevation_in_metres_to_feet() {
        let elevation = aixm::Measure {
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
    fn resolves_aerodrome_of_vrp_in_other_message() {
        let aerodromes = message(&[EDHF.to_string()]);
        let points = message(&[designated_point(
            "N",
            "EDHF NOVEMBER 1",
            &location("54.0 9.6"),
            &vrp_at("ED0432").replace(
                "urn:uuid:595049d5",
                "urn:uuid:1463eef0-65d4-4b92-b357-1dd54b0b2e15",
            ),
        )]);

        let nd = NavigationData::try_from_aixm(&[aerodromes.as_bytes(), points.as_bytes()])
            .expect("AIXM should be readable");

        assert!(nd.find_terminal_waypoint("EDHF", "N1").is_some());
    }

    /// Returns the runways of EDHF with the aerodrome in one message and
    /// `runway_members` in another.
    fn edhf_runways(runway_members: &[String]) -> Vec<Runway> {
        let aerodromes = message(&[EDHF.to_string()]);
        let runways = message(runway_members);

        let nd = NavigationData::try_from_aixm(&[aerodromes.as_bytes(), runways.as_bytes()])
            .expect("AIXM should be readable");
        assert!(nd.errors().is_empty(), "{:?}", nd.errors());

        let Some(NavAid::Airport(edhf)) = nd.find("EDHF") else {
            panic!("EDHF should be an airport");
        };
        edhf.runways.clone()
    }

    #[test]
    fn loads_runways_of_aerodrome_in_other_message() {
        let runways = edhf_runways(&[
            RWY_EDHF.to_string(),
            RWY_DIRECTION_EDHF_20.to_string(),
            THR_EDHF_20.to_string(),
        ]);

        assert_eq!(
            runways,
            [Runway {
                designator: "20".to_string(),
                bearing: Angle::m(202.0),
                length: Length::m(1040.0),
                tora: Some(Length::m(1040.0)),
                toda: None,
                lda: Some(Length::m(1040.0)),
                displaced_threshold: None,
                surface: Some(RunwaySurface::Grass),
                slope: None,
                elev: VerticalDistance::Gnd,
            }]
        );
    }

    #[test]
    fn loads_runway_without_declared_distances() {
        let runways = edhf_runways(&[RWY_EDHF.to_string(), RWY_DIRECTION_EDHF_20.to_string()]);

        let [rwy] = runways.as_slice() else {
            panic!("EDHF should have one runway");
        };
        assert_eq!((rwy.tora, rwy.lda), (None, None));
    }

    #[test]
    fn skips_fato() {
        let fato = RWY_EDHF.replace("<aixm:type>RWY</aixm:type>", "<aixm:type>FATO</aixm:type>");
        let runways = edhf_runways(&[fato, RWY_DIRECTION_EDHF_20.to_string()]);
        assert!(runways.is_empty());
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
