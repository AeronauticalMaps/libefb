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

use super::{keep_record_error, Measure, Point, Reference, TimeSlice};
use crate::stream::{read_optional_text, read_text, skip};
use crate::Error;

/// A navigation aid as a service, provided by one or more pieces of
/// equipment (UML class `Navaid`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Navaid {
    /// The `type` (CodeNavaidServiceType), e.g. VOR_DME or NDB.
    pub kind: Option<String>,
    /// The coded identifier of the navaid.
    pub designator: Option<String>,
    /// The name of the navaid.
    pub name: Option<String>,
    /// The position of the navaid.
    pub location: Option<Point>,
    /// The equipment that provides the navaid.
    pub navaid_equipment: Vec<NavaidComponent>,
    /// The aerodromes or heliports the navaid serves.
    pub served_airport: Vec<Reference>,
}

/// A piece of equipment that is part of a navaid (UML class
/// `NavaidComponent`).
#[derive(Clone, PartialEq, Debug)]
pub struct NavaidComponent {
    /// The equipment, e.g. a VOR or DME.
    pub the_navaid_equipment: Reference,
    /// Whether the navaid's position is the position of this equipment.
    pub provides_navigable_location: Option<bool>,
}

impl NavaidComponent {
    /// Reads an `aixm:navaidEquipment` up to its end tag.
    fn read_navaid_equipment<R: BufRead>(reader: &mut NsReader<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut the_navaid_equipment = None;
        let mut provides_navigable_location = None;
        let mut record_error = None;

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "providesNavigableLocation" => {
                        let result = read_text(reader, &element).and_then(|text| yes_no(&text));
                        provides_navigable_location =
                            keep_record_error(result, &mut record_error)?;
                    }
                    "theNavaidEquipment" => {
                        let result = Reference::from_element(&element);
                        skip(reader, &element)?;
                        the_navaid_equipment = keep_record_error(result, &mut record_error)?;
                    }
                    _ => {}
                },
                Event::Empty(element) if element.local_name().as_ref() == "theNavaidEquipment" => {
                    let result = Reference::from_element(&element);
                    the_navaid_equipment = keep_record_error(result, &mut record_error)?;
                }
                Event::End(element) if element.local_name().as_ref() == "navaidEquipment" => break,
                Event::Eof => {
                    return Err(Error::InvalidXml(
                        "navaidEquipment is not closed".to_string(),
                    ))
                }
                _ => {}
            }
        }

        if let Some(message) = record_error {
            return Err(Error::InvalidRecord(message));
        }

        let the_navaid_equipment = the_navaid_equipment.ok_or_else(|| {
            Error::InvalidRecord("navaidEquipment has no theNavaidEquipment".to_string())
        })?;

        Ok(Self {
            the_navaid_equipment,
            provides_navigable_location,
        })
    }
}

/// The equipment of a navaid (UML class `NavaidEquipment`).
#[derive(Clone, PartialEq, Debug)]
pub enum NavaidEquipment {
    /// A VHF omnidirectional radio range.
    VOR(VOR),
    /// A distance measuring equipment.
    DME(DME),
    /// A non-directional radio beacon.
    NDB(NDB),
    /// A tactical air navigation beacon.
    TACAN(TACAN),
}

/// A VHF omnidirectional radio range (UML class `VOR`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct VOR {
    /// The coded identifier of the VOR.
    pub designator: Option<String>,
    /// The name of the VOR.
    pub name: Option<String>,
    /// The position of the VOR.
    pub location: Option<Point>,
    /// The `type` (CodeVOR), e.g. VOR or DVOR.
    pub kind: Option<String>,
    /// The frequency of the VOR.
    pub frequency: Option<Measure>,
}

/// A distance measuring equipment (UML class `DME`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct DME {
    /// The coded identifier of the DME.
    pub designator: Option<String>,
    /// The name of the DME.
    pub name: Option<String>,
    /// The position of the DME.
    pub location: Option<Point>,
    /// The channel of the DME, e.g. 78X.
    pub channel: Option<String>,
    /// The VHF frequency paired with the channel.
    pub ghost_frequency: Option<Measure>,
}

/// A non-directional radio beacon (UML class `NDB`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct NDB {
    /// The coded identifier of the NDB.
    pub designator: Option<String>,
    /// The name of the NDB.
    pub name: Option<String>,
    /// The position of the NDB.
    pub location: Option<Point>,
    /// The frequency of the NDB.
    pub frequency: Option<Measure>,
    /// The `class` (CodeNDBUsage), e.g. ENR or LOC.
    pub class: Option<String>,
}

/// A tactical air navigation beacon (UML class `TACAN`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TACAN {
    /// The coded identifier of the TACAN.
    pub designator: Option<String>,
    /// The name of the TACAN.
    pub name: Option<String>,
    /// The position of the TACAN.
    pub location: Option<Point>,
    /// The channel of the TACAN, e.g. 109X.
    pub channel: Option<String>,
}

impl TimeSlice for Navaid {
    const FEATURE: &'static str = "Navaid";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "type" => self.kind = read_optional_text(reader, element)?,
            "designator" => self.designator = read_optional_text(reader, element)?,
            "name" => self.name = read_optional_text(reader, element)?,
            "location" => self.location = Point::read_location(reader, "location")?,
            "navaidEquipment" => self
                .navaid_equipment
                .push(NavaidComponent::read_navaid_equipment(reader)?),
            "servedAirport" => {
                let reference = Reference::from_element(element);
                skip(reader, element)?;
                self.served_airport.push(reference?);
            }
            _ => skip(reader, element)?,
        }

        Ok(())
    }

    fn read_empty_property(&mut self, element: &BytesStart) -> Result<(), Error> {
        if element.local_name().as_ref() == "servedAirport" {
            self.served_airport.push(Reference::from_element(element)?);
        }

        Ok(())
    }
}

impl TimeSlice for VOR {
    const FEATURE: &'static str = "VOR";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "designator" => self.designator = read_optional_text(reader, element)?,
            "name" => self.name = read_optional_text(reader, element)?,
            "location" => self.location = Point::read_location(reader, "location")?,
            "type" => self.kind = read_optional_text(reader, element)?,
            "frequency" => self.frequency = Measure::read(reader, element)?,
            _ => skip(reader, element)?,
        }

        Ok(())
    }
}

impl TimeSlice for DME {
    const FEATURE: &'static str = "DME";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "designator" => self.designator = read_optional_text(reader, element)?,
            "name" => self.name = read_optional_text(reader, element)?,
            "location" => self.location = Point::read_location(reader, "location")?,
            "channel" => self.channel = read_optional_text(reader, element)?,
            "ghostFrequency" => self.ghost_frequency = Measure::read(reader, element)?,
            _ => skip(reader, element)?,
        }

        Ok(())
    }
}

impl TimeSlice for NDB {
    const FEATURE: &'static str = "NDB";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "designator" => self.designator = read_optional_text(reader, element)?,
            "name" => self.name = read_optional_text(reader, element)?,
            "location" => self.location = Point::read_location(reader, "location")?,
            "frequency" => self.frequency = Measure::read(reader, element)?,
            "class" => self.class = read_optional_text(reader, element)?,
            _ => skip(reader, element)?,
        }

        Ok(())
    }
}

impl TimeSlice for TACAN {
    const FEATURE: &'static str = "TACAN";

    fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "designator" => self.designator = read_optional_text(reader, element)?,
            "name" => self.name = read_optional_text(reader, element)?,
            "location" => self.location = Point::read_location(reader, "location")?,
            "channel" => self.channel = read_optional_text(reader, element)?,
            _ => skip(reader, element)?,
        }

        Ok(())
    }
}

/// Parses a CodeYesNo.
fn yes_no(text: &str) -> Result<bool, Error> {
    match text {
        "YES" => Ok(true),
        "NO" => Ok(false),
        _ => Err(Error::InvalidRecord(format!("{text:?} is neither YES nor NO"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::read_feature;
    use chrono::{DateTime, Utc};

    const VOR_DME_HAM: &str = r#"
        <aixm:Navaid gml:id="ID_NAVAID_449"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
            <gml:identifier codeSpace="urn:uuid:">1d25c8cd-217e-4da8-8651-bd85211bbe7e</gml:identifier>
            <aixm:timeSlice>
                <aixm:NavaidTimeSlice gml:id="ID_NAVAID_TS_449">
                    <gml:validTime>
                        <gml:TimePeriod gml:id="ID_NAVAID_TS_VT_449">
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:sequenceNumber>1</aixm:sequenceNumber>
                    <aixm:correctionNumber>0</aixm:correctionNumber>
                    <aixm:type>VOR_DME</aixm:type>
                    <aixm:designator>HAM</aixm:designator>
                    <aixm:name>HAMBURG</aixm:name>
                    <aixm:flightChecked xsi:nil="true" nilReason="unknown"/>
                    <aixm:purpose xsi:nil="true" nilReason="unknown"/>
                    <aixm:navaidEquipment>
                        <aixm:NavaidComponent gml:id="ID_NAVAID_TS_COMP_1164">
                            <aixm:collocationGroup xsi:nil="true" nilReason="unknown"/>
                            <aixm:providesNavigableLocation>YES</aixm:providesNavigableLocation>
                            <aixm:theNavaidEquipment xlink:href="urn:uuid:ea3f76a6-724d-4802-b189-3777234faa1f"/>
                        </aixm:NavaidComponent>
                    </aixm:navaidEquipment>
                    <aixm:navaidEquipment>
                        <aixm:NavaidComponent gml:id="ID_NAVAID_TS_COMP_1165">
                            <aixm:collocationGroup xsi:nil="true" nilReason="unknown"/>
                            <aixm:providesNavigableLocation>NO</aixm:providesNavigableLocation>
                            <aixm:theNavaidEquipment xlink:href="urn:uuid:d0209481-44e3-43ad-aec8-be086d8e0f58"/>
                        </aixm:NavaidComponent>
                    </aixm:navaidEquipment>
                    <aixm:location>
                        <aixm:ElevatedPoint gml:id="ID_NAVAID_TS_EPT_449" srsName="urn:ogc:def:crs:EPSG::4326">
                            <gml:pos>53.685575338 10.204997112</gml:pos>
                            <aixm:horizontalAccuracy xsi:nil="true" nilReason="missing"/>
                            <aixm:elevation xsi:nil="true" nilReason="unknown"/>
                        </aixm:ElevatedPoint>
                    </aixm:location>
                </aixm:NavaidTimeSlice>
            </aixm:timeSlice>
        </aixm:Navaid>"#;

    const VOR_HAM: &str = r#"
        <aixm:Navaid gml:id="ID_NAVAID_VOR_408"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink">
            <gml:identifier codeSpace="urn:uuid:">620b53b8-899c-4291-8854-5c426fd9c0e0</gml:identifier>
            <aixm:timeSlice>
                <aixm:NavaidTimeSlice gml:id="ID_NAVAID_VOR_TS_408">
                    <gml:validTime>
                        <gml:TimePeriod gml:id="ID_NAVAID_VOR_TS_VT_408">
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:type>VOR</aixm:type>
                    <aixm:designator>HAM</aixm:designator>
                    <aixm:name>HAMBURG</aixm:name>
                    <aixm:navaidEquipment>
                        <aixm:NavaidComponent gml:id="ID_NAVAID_VOR_TS_COMP_408">
                            <aixm:theNavaidEquipment xlink:href="urn:uuid:ea3f76a6-724d-4802-b189-3777234faa1f"/>
                        </aixm:NavaidComponent>
                    </aixm:navaidEquipment>
                    <aixm:location>
                        <aixm:ElevatedPoint gml:id="ID_NAVAID_VOR_TS_EPT_408" srsName="urn:ogc:def:crs:EPSG::4326">
                            <gml:pos>53.685575338 10.204997112</gml:pos>
                            <aixm:elevation uom="FT">185.3675</aixm:elevation>
                        </aixm:ElevatedPoint>
                    </aixm:location>
                    <aixm:servedAirport xlink:href="urn:uuid:33616417-8527-4653-8605-9240351e11f2" xlink:title="EDHI"/>
                    <aixm:servedAirport xlink:href="urn:uuid:839296db-4c81-4cb0-a22f-bb9c035483eb" xlink:title="EDDH"/>
                    <aixm:servedAirport xlink:href="urn:uuid:e999fed7-b219-4298-a920-e21fe5c49186" xlink:title="EDHL"/>
                </aixm:NavaidTimeSlice>
            </aixm:timeSlice>
        </aixm:Navaid>"#;

    const DME_HAM: &str = r#"
        <aixm:DME gml:id="id.d0209481-44e3-43ad-aec8-be086d8e0f58"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink">
            <gml:identifier codeSpace="urn:uuid:">d0209481-44e3-43ad-aec8-be086d8e0f58</gml:identifier>
            <aixm:timeSlice>
                <aixm:DMETimeSlice gml:id="id.ts.d0209481-44e3-43ad-aec8-be086d8e0f58">
                    <gml:validTime>
                        <gml:TimePeriod gml:id="id.ts.vt.d0209481-44e3-43ad-aec8-be086d8e0f58">
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:designator>HAM</aixm:designator>
                    <aixm:name>HAMBURG</aixm:name>
                    <aixm:emissionClass>PON</aixm:emissionClass>
                    <aixm:location>
                        <aixm:ElevatedPoint gml:id="id.ts.ept.d0209481-44e3-43ad-aec8-be086d8e0f58" srsName="urn:ogc:def:crs:EPSG::4326">
                            <gml:pos>53.685636556 10.204832889</gml:pos>
                            <aixm:elevation uom="FT">166.31</aixm:elevation>
                            <aixm:verticalDatum>EGM_96</aixm:verticalDatum>
                        </aixm:ElevatedPoint>
                    </aixm:location>
                    <aixm:authority>
                        <aixm:AuthorityForNavaidEquipment gml:id="id.ts.d0209481-44e3-43ad-aec8-be086d8e0f58-auth">
                            <aixm:theOrganisationAuthority xlink:href="urn:uuid:09424934-8352-0ae9-e053-0b751dc2b779"/>
                        </aixm:AuthorityForNavaidEquipment>
                    </aixm:authority>
                    <aixm:channel>78X</aixm:channel>
                    <aixm:ghostFrequency uom="MHZ">113.1</aixm:ghostFrequency>
                </aixm:DMETimeSlice>
            </aixm:timeSlice>
        </aixm:DME>"#;

    const NDB_CEL: &str = r#"
        <aixm:NDB gml:id="ID_NDB_65"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
            <gml:identifier codeSpace="urn:uuid:">5be61e0e-a136-43c3-a29f-378848e3d44b</gml:identifier>
            <aixm:timeSlice>
                <aixm:NDBTimeSlice gml:id="ID_NDB_TS_65">
                    <gml:validTime>
                        <gml:TimePeriod gml:id="ID_NDB_TS_VT_65">
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:designator>CEL</aixm:designator>
                    <aixm:name>CELLE</aixm:name>
                    <aixm:magneticVariation>3</aixm:magneticVariation>
                    <aixm:location>
                        <aixm:ElevatedPoint gml:id="ID_NDB_TS_EPT_65" srsName="urn:ogc:def:crs:EPSG::4326">
                            <gml:pos>52.589594944 10.029460833</gml:pos>
                            <aixm:elevation uom="FT">172.4487</aixm:elevation>
                        </aixm:ElevatedPoint>
                    </aixm:location>
                    <aixm:frequency uom="KHZ">311</aixm:frequency>
                    <aixm:class>ENR</aixm:class>
                    <aixm:emissionBand xsi:nil="true"/>
                </aixm:NDBTimeSlice>
            </aixm:timeSlice>
        </aixm:NDB>"#;

    const TACAN_NVO: &str = r#"
        <aixm:TACAN gml:id="id.edaa7c19-4101-4459-9fff-319959a615a7"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2">
            <gml:identifier codeSpace="urn:uuid:">edaa7c19-4101-4459-9fff-319959a615a7</gml:identifier>
            <aixm:timeSlice>
                <aixm:TACANTimeSlice gml:id="id.ts.edaa7c19-4101-4459-9fff-319959a615a7">
                    <gml:validTime>
                        <gml:TimePeriod gml:id="id.ts.vt.edaa7c19-4101-4459-9fff-319959a615a7">
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:designator>NVO</aixm:designator>
                    <aixm:name>NOERVENICH</aixm:name>
                    <aixm:location>
                        <aixm:ElevatedPoint gml:id="id.ts.ept.edaa7c19-4101-4459-9fff-319959a615a7" srsName="urn:ogc:def:crs:EPSG::4326">
                            <gml:pos>50.822674298 6.636534248</gml:pos>
                            <aixm:horizontalAccuracy uom="M">0.3</aixm:horizontalAccuracy>
                            <aixm:elevation uom="FT">429.5161</aixm:elevation>
                        </aixm:ElevatedPoint>
                    </aixm:location>
                    <aixm:channel>109X</aixm:channel>
                </aixm:TACANTimeSlice>
            </aixm:timeSlice>
        </aixm:TACAN>"#;

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

    fn equipment(uuid: &str) -> Reference {
        Reference {
            uuid: uuid.to_string(),
            title: None,
        }
    }

    #[test]
    fn reads_navaid_with_its_equipment() {
        assert_eq!(
            read::<Navaid>(VOR_DME_HAM),
            Ok(Navaid {
                kind: Some("VOR_DME".to_string()),
                designator: Some("HAM".to_string()),
                name: Some("HAMBURG".to_string()),
                location: Some(Point {
                    latitude: 53.685575338,
                    longitude: 10.204997112,
                }),
                navaid_equipment: vec![
                    NavaidComponent {
                        the_navaid_equipment: equipment("ea3f76a6-724d-4802-b189-3777234faa1f"),
                        provides_navigable_location: Some(true),
                    },
                    NavaidComponent {
                        the_navaid_equipment: equipment("d0209481-44e3-43ad-aec8-be086d8e0f58"),
                        provides_navigable_location: Some(false),
                    },
                ],
                served_airport: Vec::new(),
            })
        );
    }

    #[test]
    fn reads_served_airports_of_navaid() {
        let served_airport = read::<Navaid>(VOR_HAM)
            .expect("navaid should be readable")
            .served_airport;
        let titles: Vec<_> = served_airport
            .iter()
            .map(|airport| airport.title.as_deref())
            .collect();
        assert_eq!(titles, [Some("EDHI"), Some("EDDH"), Some("EDHL")]);
    }

    #[test]
    fn reads_dme() {
        let dme = read::<DME>(DME_HAM).expect("DME should be readable");
        assert_eq!(dme.designator.as_deref(), Some("HAM"));
        assert_eq!(dme.channel.as_deref(), Some("78X"));
        assert_eq!(
            dme.ghost_frequency,
            Some(Measure {
                value: 113.1,
                uom: "MHZ".to_string(),
            })
        );
    }

    #[test]
    fn reads_ndb() {
        let ndb = read::<NDB>(NDB_CEL).expect("NDB should be readable");
        assert_eq!(ndb.designator.as_deref(), Some("CEL"));
        assert_eq!(
            ndb.frequency,
            Some(Measure {
                value: 311.0,
                uom: "KHZ".to_string(),
            })
        );
        assert_eq!(ndb.class.as_deref(), Some("ENR"));
    }

    #[test]
    fn reads_tacan() {
        let tacan = read::<TACAN>(TACAN_NVO).expect("TACAN should be readable");
        assert_eq!(tacan.designator.as_deref(), Some("NVO"));
        assert_eq!(tacan.channel.as_deref(), Some("109X"));
        assert_eq!(
            tacan.location,
            Some(Point {
                latitude: 50.822674298,
                longitude: 6.636534248,
            })
        );
    }

    #[test]
    fn rejects_invalid_navigable_location() {
        let xml = VOR_DME_HAM.replace(">YES<", ">MAYBE<");
        assert!(matches!(read::<Navaid>(&xml), Err(Error::InvalidRecord(_))));
    }
}
