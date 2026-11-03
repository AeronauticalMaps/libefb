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

use quick_xml::events::BytesStart;
use quick_xml::NsReader;

use super::{Note, Point, Reference, TimeSlice};
use crate::stream::{read_optional_text, skip};
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

impl TimeSlice for DesignatedPoint {
    const FEATURE: &'static str = "DesignatedPoint";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "designator" => self.designator = read_optional_text(reader, element)?,
            "type" => self.kind = read_optional_text(reader, element)?,
            "name" => self.name = read_optional_text(reader, element)?,
            "location" => self.location = Point::read_location(reader, "location")?,
            "airportHeliport" => {
                let reference = Reference::from_element(element);
                skip(reader, element)?;
                self.airport_heliport = Some(reference?);
            }
            "annotation" => self.annotation.push(Note::read_annotation(reader)?),
            _ => skip(reader, element)?,
        }

        Ok(())
    }

    fn read_empty_property(&mut self, element: &BytesStart) -> Result<(), Error> {
        if element.local_name().as_ref() == "airportHeliport" {
            self.airport_heliport = Some(Reference::from_element(element)?);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::read_feature;
    use chrono::{DateTime, Utc};
    use quick_xml::events::Event;

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

        read_feature::<DesignatedPoint, _>(&mut reader, date)
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
