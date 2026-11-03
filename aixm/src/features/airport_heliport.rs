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

use std::io::BufRead;

use chrono::{DateTime, Utc};
use quick_xml::events::{BytesStart, Event};
use quick_xml::NsReader;

use super::{keep_record_error, Point};
use crate::stream::{attribute, read_optional_text, skip};
use crate::timeslice::{select_valid, TimeSliceMetadata};
use crate::Error;

/// An aerodrome or heliport (UML class `AirportHeliport`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AirportHeliport {
    /// The coded identifier of the aerodrome.
    pub designator: Option<String>,
    /// The four letter ICAO location indicator, e.g. EDDH.
    pub location_indicator_icao: Option<String>,
    /// The three letter IATA designator, e.g. HAM.
    pub designator_iata: Option<String>,
    /// The name of the aerodrome.
    pub name: Option<String>,
    /// The aerodrome reference point (`ARP`).
    pub arp: Option<Point>,
    /// The highest point of the landing area (`fieldElevation`).
    pub field_elevation: Option<Elevation>,
}

/// A vertical distance (`ValDistanceVerticalType`).
#[derive(Clone, PartialEq, Debug)]
pub struct Elevation {
    /// The value in the unit of measurement.
    pub value: f64,
    /// The unit of measurement (`uom`), e.g. FT or M.
    pub uom: String,
}

impl Elevation {
    /// Reads the vertical distance `element` up to its end tag. Returns `None`
    /// if the element has no value.
    fn read<R: BufRead>(
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<Option<Self>, Error> {
        let uom = attribute(element, "uom")?;

        let Some(text) = read_optional_text(reader, element)? else {
            return Ok(None);
        };

        let value = text
            .parse()
            .map_err(|_| Error::InvalidRecord(format!("{text:?} is not a vertical distance")))?;

        let uom = uom.ok_or_else(|| {
            Error::InvalidRecord(format!("vertical distance {text} has no uom"))
        })?;

        Ok(Some(Self { value, uom }))
    }
}

/// Reads an `aixm:AirportHeliport` up to its end tag and returns its
/// identifier with the time slice that is valid on `date`, or `None` if no
/// time slice is valid. The start tag must be the last event read.
///
/// An [`Error::InvalidRecord`] is returned only after the whole element is
/// read, so the reader can continue with the next feature.
pub(crate) fn read_airport_heliport<R: BufRead>(
    reader: &mut NsReader<R>,
    date: DateTime<Utc>,
) -> Result<Option<(String, AirportHeliport)>, Error> {
    let mut buf = Vec::new();
    let mut identifier = None;
    let mut time_slices = Vec::new();
    let mut record_error = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) => match element.local_name().as_ref() {
                "identifier" => identifier = read_optional_text(reader, &element)?,
                "timeSlice" => {
                    if let Some(time_slice) =
                        keep_record_error(read_time_slice(reader), &mut record_error)?
                    {
                        time_slices.push(time_slice);
                    }
                }
                _ => skip(reader, &element)?,
            },
            Event::End(_) => break,
            Event::Eof => {
                return Err(Error::InvalidXml(
                    "AirportHeliport is not closed".to_string(),
                ))
            }
            _ => {}
        }
    }

    let identifier = identifier.ok_or_else(|| {
        Error::InvalidRecord("AirportHeliport has no gml:identifier".to_string())
    })?;

    if let Some(message) = record_error {
        return Err(Error::InvalidRecord(format!(
            "AirportHeliport {identifier}: {message}"
        )));
    }

    Ok(select_valid(time_slices, date).map(|airport| (identifier, airport)))
}

/// Reads an `aixm:timeSlice` up to its end tag.
fn read_time_slice<R: BufRead>(
    reader: &mut NsReader<R>,
) -> Result<(TimeSliceMetadata, AirportHeliport), Error> {
    let mut buf = Vec::new();
    let mut time_slice = None;
    let mut record_error = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) => match element.local_name().as_ref() {
                "AirportHeliportTimeSlice" => {
                    time_slice = keep_record_error(read_properties(reader), &mut record_error)?
                }
                _ => skip(reader, &element)?,
            },
            Event::End(_) => break,
            Event::Eof => return Err(Error::InvalidXml("timeSlice is not closed".to_string())),
            _ => {}
        }
    }

    if let Some(message) = record_error {
        return Err(Error::InvalidRecord(message));
    }

    time_slice.ok_or_else(|| {
        Error::InvalidRecord("timeSlice has no AirportHeliportTimeSlice".to_string())
    })
}

/// Reads the properties of an `aixm:AirportHeliportTimeSlice` up to its end
/// tag.
fn read_properties<R: BufRead>(
    reader: &mut NsReader<R>,
) -> Result<(TimeSliceMetadata, AirportHeliport), Error> {
    let mut buf = Vec::new();
    let mut metadata = TimeSliceMetadata::default();
    let mut airport = AirportHeliport::default();
    let mut record_error = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) => {
                let result = read_property(reader, &element, &mut metadata, &mut airport);
                keep_record_error(result, &mut record_error)?;
            }
            Event::End(_) => break,
            Event::Eof => {
                return Err(Error::InvalidXml(
                    "AirportHeliportTimeSlice is not closed".to_string(),
                ))
            }
            _ => {}
        }
    }

    match record_error {
        Some(message) => Err(Error::InvalidRecord(message)),
        None => Ok((metadata, airport)),
    }
}

/// Reads the property `element` of a time slice up to its end tag.
fn read_property<R: BufRead>(
    reader: &mut NsReader<R>,
    element: &BytesStart,
    metadata: &mut TimeSliceMetadata,
    airport: &mut AirportHeliport,
) -> Result<(), Error> {
    if metadata.read_property(reader, element)? {
        return Ok(());
    }

    match element.local_name().as_ref() {
        "designator" => airport.designator = read_optional_text(reader, element)?,
        "locationIndicatorICAO" => {
            airport.location_indicator_icao = read_optional_text(reader, element)?
        }
        "designatorIATA" => airport.designator_iata = read_optional_text(reader, element)?,
        "name" => airport.name = read_optional_text(reader, element)?,
        "ARP" => airport.arp = Point::read_location(reader, "ARP")?,
        "fieldElevation" => airport.field_elevation = Elevation::read(reader, element)?,
        _ => skip(reader, element)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDDH: &str = r#"
        <aixm:AirportHeliport gml:id="ID_839296db-4c81-4cb0-a22f-bb9c035483eb-0"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
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
                    <aixm:featureLifetime>
                        <gml:TimePeriod gml:id="ID_TS_FL_TP_839296db-4c81-4cb0-a22f-bb9c035483eb">
                            <gml:beginPosition>2015-05-01T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="unknown"/>
                        </gml:TimePeriod>
                    </aixm:featureLifetime>
                    <aixm:designator>EDDH</aixm:designator>
                    <aixm:name>HAMBURG</aixm:name>
                    <aixm:locationIndicatorICAO>EDDH</aixm:locationIndicatorICAO>
                    <aixm:designatorIATA>HAM</aixm:designatorIATA>
                    <aixm:type>AD</aixm:type>
                    <aixm:controlType>CIVIL</aixm:controlType>
                    <aixm:fieldElevation uom="FT">53</aixm:fieldElevation>
                    <aixm:verticalDatum>EGM_96</aixm:verticalDatum>
                    <aixm:magneticVariation>4.22</aixm:magneticVariation>
                    <aixm:magneticVariationAccuracy xsi:nil="true" nilReason="unknown"/>
                    <aixm:dateMagneticVariation>2025</aixm:dateMagneticVariation>
                    <aixm:servedCity>
                        <aixm:City gml:id="ID_TS_C_839296db-4c81-4cb0-a22f-bb9c035483eb-0">
                            <aixm:name>HAMBURG</aixm:name>
                        </aixm:City>
                    </aixm:servedCity>
                    <aixm:responsibleOrganisation>
                        <aixm:AirportHeliportResponsibilityOrganisation gml:id="ID_TS_AHPRORG_839296db-4c81-4cb0-a22f-bb9c035483eb">
                            <aixm:role nilReason="unknown" xsi:nil="true"/>
                            <aixm:theOrganisationAuthority xlink:href="urn:uuid:09424934-8352-0ae9-e053-0b751dc2b779"/>
                        </aixm:AirportHeliportResponsibilityOrganisation>
                    </aixm:responsibleOrganisation>
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
        </aixm:AirportHeliport>"#;

    fn read(xml: &str) -> Result<Option<(String, AirportHeliport)>, Error> {
        let mut reader = NsReader::from_reader(xml.as_bytes());
        let mut buf = Vec::new();

        // enter the AirportHeliport element
        loop {
            buf.clear();
            if let Event::Start(_) = reader.read_event_into(&mut buf)? {
                break;
            }
        }

        let date = DateTime::parse_from_rfc3339("2026-10-29T00:00:00Z")
            .expect("test date should be RFC 3339")
            .with_timezone(&Utc);

        read_airport_heliport(&mut reader, date)
    }

    #[test]
    fn reads_airport_heliport() {
        assert_eq!(
            read(EDDH),
            Ok(Some((
                "839296db-4c81-4cb0-a22f-bb9c035483eb".to_string(),
                AirportHeliport {
                    designator: Some("EDDH".to_string()),
                    location_indicator_icao: Some("EDDH".to_string()),
                    designator_iata: Some("HAM".to_string()),
                    name: Some("HAMBURG".to_string()),
                    arp: Some(Point {
                        latitude: 53.630387834,
                        longitude: 9.988227138,
                    }),
                    field_elevation: Some(Elevation {
                        value: 53.0,
                        uom: "FT".to_string(),
                    }),
                }
            )))
        );
    }

    #[test]
    fn reads_nil_field_elevation_as_none() {
        let xml = EDDH.replace(
            r#"<aixm:fieldElevation uom="FT">53</aixm:fieldElevation>"#,
            r#"<aixm:fieldElevation xsi:nil="true" nilReason="unknown"/>"#,
        );
        let elevation = read(&xml).map(|airport| airport.and_then(|(_, a)| a.field_elevation));
        assert_eq!(elevation, Ok(None));
    }

    #[test]
    fn rejects_invalid_field_elevation() {
        let xml = EDDH.replace(">53</aixm:fieldElevation>", ">high</aixm:fieldElevation>");
        assert!(matches!(read(&xml), Err(Error::InvalidRecord(_))));
    }
}
