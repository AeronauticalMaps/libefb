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

use super::{keep_record_error, Note, Point, Reference};
use crate::stream::{read_optional_text, skip};
use crate::timeslice::{select_valid, TimeSliceMetadata};
use crate::Error;

/// A point used in route and procedure design (UML class `DesignatedPoint`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct DesignatedPoint {
    /// The coded identifier of the point.
    pub designator: Option<String>,
    /// The `type` (CodeDesignatedPointType), e.g. ICAO, TERMINAL or
    /// OTHER:ADHP.
    pub kind: Option<String>,
    /// The name of the point.
    pub name: Option<String>,
    /// The position of the point.
    pub location: Option<Point>,
    /// The aerodrome or heliport the point belongs to.
    pub airport_heliport: Option<Reference>,
    /// The notes of the point.
    pub annotation: Vec<Note>,
}

/// Reads an `aixm:DesignatedPoint` up to its end tag and returns its
/// identifier with the time slice that is valid on `date`, or `None` if no
/// time slice is valid. The start tag must be the last event read.
///
/// An [`Error::InvalidRecord`] is returned only after the whole element is
/// read, so the reader can continue with the next feature.
pub(crate) fn read_designated_point<R: BufRead>(
    reader: &mut NsReader<R>,
    date: DateTime<Utc>,
) -> Result<Option<(String, DesignatedPoint)>, Error> {
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
                    "DesignatedPoint is not closed".to_string(),
                ))
            }
            _ => {}
        }
    }

    let identifier = identifier.ok_or_else(|| {
        Error::InvalidRecord("DesignatedPoint has no gml:identifier".to_string())
    })?;

    if let Some(message) = record_error {
        return Err(Error::InvalidRecord(format!(
            "DesignatedPoint {identifier}: {message}"
        )));
    }

    Ok(select_valid(time_slices, date).map(|point| (identifier, point)))
}

/// Reads an `aixm:timeSlice` up to its end tag.
fn read_time_slice<R: BufRead>(
    reader: &mut NsReader<R>,
) -> Result<(TimeSliceMetadata, DesignatedPoint), Error> {
    let mut buf = Vec::new();
    let mut time_slice = None;
    let mut record_error = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) => match element.local_name().as_ref() {
                "DesignatedPointTimeSlice" => {
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
        Error::InvalidRecord("timeSlice has no DesignatedPointTimeSlice".to_string())
    })
}

/// Reads the properties of an `aixm:DesignatedPointTimeSlice` up to its end
/// tag.
fn read_properties<R: BufRead>(
    reader: &mut NsReader<R>,
) -> Result<(TimeSliceMetadata, DesignatedPoint), Error> {
    let mut buf = Vec::new();
    let mut metadata = TimeSliceMetadata::default();
    let mut point = DesignatedPoint::default();
    let mut record_error = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) => {
                let result = read_property(reader, &element, &mut metadata, &mut point);
                keep_record_error(result, &mut record_error)?;
            }
            Event::Empty(element) if element.local_name().as_ref() == "airportHeliport" => {
                let result = Reference::from_element(&element);
                point.airport_heliport = keep_record_error(result, &mut record_error)?;
            }
            Event::End(_) => break,
            Event::Eof => {
                return Err(Error::InvalidXml(
                    "DesignatedPointTimeSlice is not closed".to_string(),
                ))
            }
            _ => {}
        }
    }

    match record_error {
        Some(message) => Err(Error::InvalidRecord(message)),
        None => Ok((metadata, point)),
    }
}

/// Reads the property `element` of a time slice up to its end tag.
fn read_property<R: BufRead>(
    reader: &mut NsReader<R>,
    element: &BytesStart,
    metadata: &mut TimeSliceMetadata,
    point: &mut DesignatedPoint,
) -> Result<(), Error> {
    if metadata.read_property(reader, element)? {
        return Ok(());
    }

    match element.local_name().as_ref() {
        "designator" => point.designator = read_optional_text(reader, element)?,
        "type" => point.kind = read_optional_text(reader, element)?,
        "name" => point.name = read_optional_text(reader, element)?,
        "location" => point.location = Point::read_location(reader)?,
        "airportHeliport" => {
            let reference = Reference::from_element(element);
            skip(reader, element)?;
            point.airport_heliport = Some(reference?);
        }
        "annotation" => point.annotation.push(Note::read_annotation(reader)?),
        _ => skip(reader, element)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VRP: &str = r#"
        <aixm:DesignatedPoint gml:id="id.b1cea6c1"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
            <gml:identifier codeSpace="urn:uuid:">b1cea6c1-2c20-460a-9c42-7ca11e83d900</gml:identifier>
            <aixm:timeSlice>
                <aixm:DesignatedPointTimeSlice gml:id="it.ts.b1cea6c1">
                    <gml:validTime>
                        <gml:TimePeriod>
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:sequenceNumber>1</aixm:sequenceNumber>
                    <aixm:correctionNumber>0</aixm:correctionNumber>
                    <aixm:featureLifetime>
                        <gml:TimePeriod>
                            <gml:beginPosition>2018-10-11T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="unknown"/>
                        </gml:TimePeriod>
                    </aixm:featureLifetime>
                    <aixm:designator>N</aixm:designator>
                    <aixm:type>OTHER:ADHP</aixm:type>
                    <aixm:name>EDDV NOVEMBER 1</aixm:name>
                    <aixm:location>
                        <aixm:Point srsName="urn:ogc:def:crs:EPSG::4326">
                            <gml:pos>52.23625277 13.3682043</gml:pos>
                            <aixm:horizontalAccuracy xsi:nil="true" nilReason="inapplicable"/>
                        </aixm:Point>
                    </aixm:location>
                    <aixm:airportHeliport xlink:href="urn:uuid:595049d5-7f73-44e6-965c-71869819f5ba" xlink:title="EDDV"/>
                    <aixm:annotation>
                        <aixm:Note>
                            <aixm:purpose>REMARK</aixm:purpose>
                            <aixm:translatedNote>
                                <aixm:LinguisticNote>
                                    <aixm:note lang="eng">
                                        VRP
                                    </aixm:note>
                                </aixm:LinguisticNote>
                            </aixm:translatedNote>
                        </aixm:Note>
                    </aixm:annotation>
                </aixm:DesignatedPointTimeSlice>
            </aixm:timeSlice>
        </aixm:DesignatedPoint>"#;

    fn read(xml: &str, date: &str) -> Result<Option<(String, DesignatedPoint)>, Error> {
        let mut reader = NsReader::from_reader(xml.as_bytes());
        let mut buf = Vec::new();

        // enter the DesignatedPoint element
        loop {
            buf.clear();
            if let Event::Start(_) = reader.read_event_into(&mut buf)? {
                break;
            }
        }

        let date = DateTime::parse_from_rfc3339(date)
            .expect("test date should be RFC 3339")
            .with_timezone(&Utc);

        read_designated_point(&mut reader, date)
    }

    #[test]
    fn reads_designated_point() {
        assert_eq!(
            read(VRP, "2026-10-29T00:00:00Z"),
            Ok(Some((
                "b1cea6c1-2c20-460a-9c42-7ca11e83d900".to_string(),
                DesignatedPoint {
                    designator: Some("N".to_string()),
                    kind: Some("OTHER:ADHP".to_string()),
                    name: Some("EDDV NOVEMBER 1".to_string()),
                    location: Some(Point {
                        latitude: 52.23625277,
                        longitude: 13.3682043,
                    }),
                    airport_heliport: Some(Reference {
                        uuid: "595049d5-7f73-44e6-965c-71869819f5ba".to_string(),
                        title: Some("EDDV".to_string()),
                    }),
                    annotation: vec![Note {
                        purpose: Some("REMARK".to_string()),
                        translated_note: vec!["VRP".to_string()],
                    }],
                }
            )))
        );
    }

    #[test]
    fn reads_nil_name_as_none() {
        let xml = VRP.replace(
            "<aixm:name>EDDV NOVEMBER 1</aixm:name>",
            r#"<aixm:name xsi:nil="true" nilReason="unknown"/>"#,
        );
        let name = read(&xml, "2026-10-29T00:00:00Z")
            .map(|point| point.and_then(|(_, point)| point.name));
        assert_eq!(name, Ok(None));
    }

    #[test]
    fn rejects_other_reference_system() {
        let xml = VRP.replace("urn:ogc:def:crs:EPSG::4326", "urn:ogc:def:crs:EPSG::3857");
        assert!(matches!(
            read(&xml, "2026-10-29T00:00:00Z"),
            Err(Error::InvalidRecord(_))
        ));
    }

    #[test]
    fn returns_none_without_valid_time_slice() {
        assert_eq!(read(VRP, "2026-10-01T00:00:00Z"), Ok(None));
    }
}
