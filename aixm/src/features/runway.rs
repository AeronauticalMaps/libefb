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

use quick_xml::events::{BytesStart, Event};
use quick_xml::NsReader;

use super::{degrees, keep_record_error, Measure, Reference, TimeSlice};
use crate::stream::{read_optional_text, skip};
use crate::Error;

/// A runway or a final approach and take-off area (UML class `Runway`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Runway {
    /// The designator of both directions, e.g. 02/20.
    pub designator: Option<String>,
    /// The `type` (CodeRunwayType), e.g. RWY or FATO.
    pub kind: Option<String>,
    /// The declared length of the runway.
    pub nominal_length: Option<Measure>,
    /// The `composition` (CodeSurfaceCompositionType) of the runway's
    /// surface, e.g. ASPH, CONC or GRASS.
    pub composition: Option<String>,
    /// The aerodrome or heliport the runway belongs to.
    pub associated_airport_heliport: Option<Reference>,
}

/// A direction of a runway (UML class `RunwayDirection`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct RunwayDirection {
    /// The designator of the direction, e.g. 20.
    pub designator: Option<String>,
    /// The true bearing of the direction in degrees.
    pub true_bearing: Option<f64>,
    /// The magnetic bearing of the direction in degrees.
    pub magnetic_bearing: Option<f64>,
    /// The runway the direction belongs to.
    pub used_runway: Option<Reference>,
}

/// A point on the centreline of a runway direction (UML class
/// `RunwayCentrelinePoint`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct RunwayCentrelinePoint {
    /// The `role` (CodeRunwayPointRole), e.g. THR, DISTHR or END.
    pub role: Option<String>,
    /// The runway direction the point lies on.
    pub on_runway: Option<Reference>,
    /// The distances declared from the point on.
    pub associated_declared_distance: Vec<RunwayDeclaredDistance>,
}

/// A distance declared from a centreline point on (UML class
/// `RunwayDeclaredDistance`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct RunwayDeclaredDistance {
    /// The `type` (CodeDeclaredDistance), e.g. TORA, TODA, ASDA, LDA or
    /// DTHR.
    pub kind: Option<String>,
    /// The declared distance.
    pub distance: Option<Measure>,
}

impl RunwayDeclaredDistance {
    /// Reads an `aixm:associatedDeclaredDistance` up to its end tag.
    fn read_associated_declared_distance<R: BufRead>(
        reader: &mut NsReader<R>,
    ) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut declared_distance = Self::default();
        let mut record_error = None;

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "type" => declared_distance.kind = read_optional_text(reader, &element)?,
                    "distance" => {
                        let result = Measure::read(reader, &element);
                        declared_distance.distance =
                            keep_record_error(result, &mut record_error)?.flatten();
                    }
                    _ => {}
                },
                Event::End(element)
                    if element.local_name().as_ref() == "associatedDeclaredDistance" =>
                {
                    break
                }
                Event::Eof => {
                    return Err(Error::InvalidXml(
                        "associatedDeclaredDistance is not closed".to_string(),
                    ))
                }
                _ => {}
            }
        }

        match record_error {
            Some(message) => Err(Error::InvalidRecord(message)),
            None => Ok(declared_distance),
        }
    }
}

impl TimeSlice for Runway {
    const FEATURE: &'static str = "Runway";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "designator" => self.designator = read_optional_text(reader, element)?,
            "type" => self.kind = read_optional_text(reader, element)?,
            "nominalLength" => self.nominal_length = Measure::read(reader, element)?,
            "surfaceProperties" => self.composition = read_composition(reader)?,
            "associatedAirportHeliport" => {
                let reference = Reference::from_element(element);
                skip(reader, element)?;
                self.associated_airport_heliport = Some(reference?);
            }
            _ => skip(reader, element)?,
        }

        Ok(())
    }

    fn read_empty_property(&mut self, element: &BytesStart) -> Result<(), Error> {
        if element.local_name().as_ref() == "associatedAirportHeliport" {
            self.associated_airport_heliport = Some(Reference::from_element(element)?);
        }

        Ok(())
    }
}

impl TimeSlice for RunwayDirection {
    const FEATURE: &'static str = "RunwayDirection";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "designator" => self.designator = read_optional_text(reader, element)?,
            "trueBearing" => {
                let text = read_optional_text(reader, element)?;
                self.true_bearing = text.as_deref().map(degrees).transpose()?;
            }
            "magneticBearing" => {
                let text = read_optional_text(reader, element)?;
                self.magnetic_bearing = text.as_deref().map(degrees).transpose()?;
            }
            "usedRunway" => {
                let reference = Reference::from_element(element);
                skip(reader, element)?;
                self.used_runway = Some(reference?);
            }
            _ => skip(reader, element)?,
        }

        Ok(())
    }

    fn read_empty_property(&mut self, element: &BytesStart) -> Result<(), Error> {
        if element.local_name().as_ref() == "usedRunway" {
            self.used_runway = Some(Reference::from_element(element)?);
        }

        Ok(())
    }
}

impl TimeSlice for RunwayCentrelinePoint {
    const FEATURE: &'static str = "RunwayCentrelinePoint";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "role" => self.role = read_optional_text(reader, element)?,
            "onRunway" => {
                let reference = Reference::from_element(element);
                skip(reader, element)?;
                self.on_runway = Some(reference?);
            }
            "associatedDeclaredDistance" => self
                .associated_declared_distance
                .push(RunwayDeclaredDistance::read_associated_declared_distance(reader)?),
            _ => skip(reader, element)?,
        }

        Ok(())
    }

    fn read_empty_property(&mut self, element: &BytesStart) -> Result<(), Error> {
        if element.local_name().as_ref() == "onRunway" {
            self.on_runway = Some(Reference::from_element(element)?);
        }

        Ok(())
    }
}

/// Reads an `aixm:surfaceProperties` up to its end tag and returns the
/// `composition` of its `SurfaceCharacteristics`.
fn read_composition<R: BufRead>(reader: &mut NsReader<R>) -> Result<Option<String>, Error> {
    let mut buf = Vec::new();
    let mut composition = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) if element.local_name().as_ref() == "composition" => {
                composition = read_optional_text(reader, &element)?
            }
            Event::End(element) if element.local_name().as_ref() == "surfaceProperties" => break,
            Event::Eof => {
                return Err(Error::InvalidXml(
                    "surfaceProperties is not closed".to_string(),
                ))
            }
            _ => {}
        }
    }

    Ok(composition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::read_feature;
    use chrono::{DateTime, Utc};

    const RWY_EDHF: &str = r#"
        <aixm:Runway gml:id="ID_e1ff5b8a-89c2-48df-9bfc-32e5616f239f"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
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
                    <aixm:sequenceNumber>1</aixm:sequenceNumber>
                    <aixm:correctionNumber>1</aixm:correctionNumber>
                    <aixm:designator>02/20</aixm:designator>
                    <aixm:type>RWY</aixm:type>
                    <aixm:nominalLength uom="M">1040</aixm:nominalLength>
                    <aixm:lengthAccuracy xsi:nil="true" nilReason="unknown"/>
                    <aixm:nominalWidth uom="M">30</aixm:nominalWidth>
                    <aixm:surfaceProperties>
                        <aixm:SurfaceCharacteristics gml:id="ID_TS_SC_e1ff5b8a-89c2-48df-9bfc-32e5616f239f">
                            <aixm:composition>GRASS</aixm:composition>
                            <aixm:preparation xsi:nil="true" nilReason="unknown"/>
                            <aixm:annotation xsi:nil="true"/>
                        </aixm:SurfaceCharacteristics>
                    </aixm:surfaceProperties>
                    <aixm:associatedAirportHeliport xlink:href="urn:uuid:1463eef0-65d4-4b92-b357-1dd54b0b2e15" xlink:title="EDHF"/>
                    <aixm:annotation>
                        <aixm:Note gml:id="ID_TS_N_e1ff5b8a-89c2-48df-9bfc-32e5616f239f-0">
                            <aixm:purpose>REMARK</aixm:purpose>
                            <aixm:translatedNote>
                                <aixm:LinguisticNote gml:id="ID_TS_N_LN_e1ff5b8a-89c2-48df-9bfc-32e5616f239f-0">
                                    <aixm:note>1040 x 40 m Gras. 5700 kg MPW</aixm:note>
                                </aixm:LinguisticNote>
                            </aixm:translatedNote>
                        </aixm:Note>
                    </aixm:annotation>
                </aixm:RunwayTimeSlice>
            </aixm:timeSlice>
        </aixm:Runway>"#;

    const RWY_DIRECTION_EDHF_20: &str = r#"
        <aixm:RunwayDirection gml:id="ID_10a97dbb-16bf-4be6-8a33-630daa28915a"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
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
                    <aixm:sequenceNumber>1</aixm:sequenceNumber>
                    <aixm:correctionNumber>1</aixm:correctionNumber>
                    <aixm:designator>20</aixm:designator>
                    <aixm:trueBearing>206</aixm:trueBearing>
                    <aixm:trueBearingAccuracy xsi:nil="true" nilReason="unknown"/>
                    <aixm:magneticBearing>202</aixm:magneticBearing>
                    <aixm:elevationTDZ uom="FT">0</aixm:elevationTDZ>
                    <aixm:usedRunway xlink:href="urn:uuid:e1ff5b8a-89c2-48df-9bfc-32e5616f239f" xlink:title="02/20"/>
                </aixm:RunwayDirectionTimeSlice>
            </aixm:timeSlice>
        </aixm:RunwayDirection>"#;

    const THR_EDHF_20: &str = r#"
        <aixm:RunwayCentrelinePoint gml:id="ID_014f6782-7c8f-41f8-a0db-d861e7d1e522"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
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
                    <aixm:sequenceNumber>1</aixm:sequenceNumber>
                    <aixm:correctionNumber>1</aixm:correctionNumber>
                    <aixm:role>THR</aixm:role>
                    <aixm:designator xsi:nil="true" nilReason="unknown"/>
                    <aixm:onRunway xlink:href="urn:uuid:10a97dbb-16bf-4be6-8a33-630daa28915a" xlink:title="20"/>
                    <aixm:associatedDeclaredDistance>
                        <aixm:RunwayDeclaredDistance gml:id="ID_TS_DD_3bcbbf50-caae-4bff-bb76-8f2efe1dffcf">
                            <aixm:type>TORA</aixm:type>
                            <aixm:declaredValue>
                                <aixm:RunwayDeclaredDistanceValue gml:id="ID_TS_DD_V_3bcbbf50-caae-4bff-bb76-8f2efe1dffcf">
                                    <aixm:distance uom="M">1040</aixm:distance>
                                    <aixm:distanceAccuracy xsi:nil="true" nilReason="unknown"/>
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
                                    <aixm:distanceAccuracy xsi:nil="true" nilReason="unknown"/>
                                </aixm:RunwayDeclaredDistanceValue>
                            </aixm:declaredValue>
                        </aixm:RunwayDeclaredDistance>
                    </aixm:associatedDeclaredDistance>
                    <aixm:annotation>
                        <aixm:Note gml:id="ID_TS_N_014f6782-7c8f-41f8-a0db-d861e7d1e522-0">
                            <aixm:purpose>REMARK</aixm:purpose>
                            <aixm:translatedNote>
                                <aixm:LinguisticNote gml:id="ID_TS_N_LN_014f6782-7c8f-41f8-a0db-d861e7d1e522-0">
                                    <aixm:note>THR 20</aixm:note>
                                </aixm:LinguisticNote>
                            </aixm:translatedNote>
                        </aixm:Note>
                    </aixm:annotation>
                </aixm:RunwayCentrelinePointTimeSlice>
            </aixm:timeSlice>
        </aixm:RunwayCentrelinePoint>"#;

    fn read<T: TimeSlice>(xml: &str) -> Result<T, Error> {
        let mut reader = NsReader::from_reader(xml.as_bytes());
        let mut buf = Vec::new();

        // enter the feature element
        loop {
            buf.clear();
            if let Event::Start(_) = reader.read_event_into(&mut buf)? {
                break;
            }
        }

        let date = DateTime::parse_from_rfc3339("2026-10-29T00:00:00Z")
            .expect("test date should be RFC 3339")
            .with_timezone(&Utc);

        read_feature::<T, _>(&mut reader, date).map(|feature| {
            feature
                .map(|(_, time_slice)| time_slice)
                .expect("feature should have a valid time slice")
        })
    }

    fn meters(value: f64) -> Option<Measure> {
        Some(Measure {
            value,
            uom: "M".to_string(),
        })
    }

    #[test]
    fn reads_runway() {
        assert_eq!(
            read::<Runway>(RWY_EDHF),
            Ok(Runway {
                designator: Some("02/20".to_string()),
                kind: Some("RWY".to_string()),
                nominal_length: meters(1040.0),
                composition: Some("GRASS".to_string()),
                associated_airport_heliport: Some(Reference {
                    uuid: "1463eef0-65d4-4b92-b357-1dd54b0b2e15".to_string(),
                    title: Some("EDHF".to_string()),
                }),
            })
        );
    }

    #[test]
    fn reads_runway_direction() {
        assert_eq!(
            read::<RunwayDirection>(RWY_DIRECTION_EDHF_20),
            Ok(RunwayDirection {
                designator: Some("20".to_string()),
                true_bearing: Some(206.0),
                magnetic_bearing: Some(202.0),
                used_runway: Some(Reference {
                    uuid: "e1ff5b8a-89c2-48df-9bfc-32e5616f239f".to_string(),
                    title: Some("02/20".to_string()),
                }),
            })
        );
    }

    #[test]
    fn reads_declared_distances_of_centreline_point() {
        assert_eq!(
            read::<RunwayCentrelinePoint>(THR_EDHF_20),
            Ok(RunwayCentrelinePoint {
                role: Some("THR".to_string()),
                on_runway: Some(Reference {
                    uuid: "10a97dbb-16bf-4be6-8a33-630daa28915a".to_string(),
                    title: Some("20".to_string()),
                }),
                associated_declared_distance: vec![
                    RunwayDeclaredDistance {
                        kind: Some("TORA".to_string()),
                        distance: meters(1040.0),
                    },
                    RunwayDeclaredDistance {
                        kind: Some("LDA".to_string()),
                        distance: meters(1040.0),
                    },
                ],
            })
        );
    }

    #[test]
    fn rejects_invalid_bearing() {
        let xml = RWY_DIRECTION_EDHF_20.replace(">206<", ">NORTH<");
        assert!(matches!(
            read::<RunwayDirection>(&xml),
            Err(Error::InvalidRecord(_))
        ));
    }
}
