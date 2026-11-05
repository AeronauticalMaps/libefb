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

use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;

use chrono::{DateTime, NaiveDate, Utc};
use quick_xml::events::Event;
use quick_xml::name::{Namespace, ResolveResult};
use quick_xml::NsReader;

use crate::features::{
    read_feature, AirportHeliport, DesignatedPoint, Navaid, NavaidEquipment, Reference, Runway,
    RunwayCentrelinePoint, RunwayDirection, TimeSlice,
};
use crate::stream::skip;
use crate::timeslice::ValidTime;
use crate::Error;

/// The AIXM version a message is written in.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Version {
    /// AIXM 5.1
    V5_1,
    /// AIXM 5.1.1
    V5_1_1,
    /// AIXM 5.2
    V5_2,
}

impl Version {
    fn from_message_namespace(namespace: &str) -> Option<Self> {
        match namespace {
            "http://www.aixm.aero/schema/5.1/message" => Some(Self::V5_1),
            "http://www.aixm.aero/schema/5.1.1/message" => Some(Self::V5_1_1),
            "http://www.aixm.aero/schema/5.2/message" => Some(Self::V5_2),
            _ => None,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V5_1 => write!(f, "AIXM 5.1"),
            Self::V5_1_1 => write!(f, "AIXM 5.1.1"),
            Self::V5_2 => write!(f, "AIXM 5.2"),
        }
    }
}

#[derive(Clone, Debug)]
enum Feature {
    AirportHeliport(AirportHeliport),
    DesignatedPoint(DesignatedPoint),
    Navaid(Navaid),
    NavaidEquipment(NavaidEquipment),
    Runway(Runway),
    RunwayDirection(RunwayDirection),
    RunwayCentrelinePoint(RunwayCentrelinePoint),
}

/// An AIXM 5.x message (`AIXMBasicMessage`).
#[derive(Clone, Debug)]
pub struct Message {
    version: Version,
    effective_date: NaiveDate,
    features: HashMap<String, Feature>,
    errors: Vec<Error>,
}

impl Message {
    /// Returns the AIXM version the message is written in.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the date on which the message becomes effective, which is the
    /// begin of the temporal extent in the message metadata.
    pub fn effective_date(&self) -> NaiveDate {
        self.effective_date
    }

    /// Returns the aerodromes and heliports of the message with their
    /// identifier, which other features refer to.
    pub fn airport_heliports(&self) -> impl Iterator<Item = (&str, &AirportHeliport)> {
        self.features
            .iter()
            .filter_map(|(identifier, feature)| match feature {
                Feature::AirportHeliport(airport) => Some((identifier.as_str(), airport)),
                _ => None,
            })
    }

    /// Returns the designated points of the message.
    pub fn designated_points(&self) -> impl Iterator<Item = &DesignatedPoint> {
        self.features.values().filter_map(|feature| match feature {
            Feature::DesignatedPoint(point) => Some(point),
            _ => None,
        })
    }

    /// Returns the navaids of the message.
    pub fn navaids(&self) -> impl Iterator<Item = &Navaid> {
        self.features.values().filter_map(|feature| match feature {
            Feature::Navaid(navaid) => Some(navaid),
            _ => None,
        })
    }

    /// Returns the navaid equipment that `reference` points to, e.g. the
    /// equipment of a navaid's component.
    pub fn navaid_equipment(&self, reference: &Reference) -> Option<&NavaidEquipment> {
        match self.features.get(&reference.uuid) {
            Some(Feature::NavaidEquipment(equipment)) => Some(equipment),
            _ => None,
        }
    }

    /// Returns the runway that `reference` points to, e.g. the runway a
    /// runway direction is used on.
    pub fn runway(&self, reference: &Reference) -> Option<&Runway> {
        match self.features.get(&reference.uuid) {
            Some(Feature::Runway(runway)) => Some(runway),
            _ => None,
        }
    }

    /// Returns the runway directions of the message with their identifier,
    /// which the centreline points refer to.
    pub fn runway_directions(&self) -> impl Iterator<Item = (&str, &RunwayDirection)> {
        self.features
            .iter()
            .filter_map(|(identifier, feature)| match feature {
                Feature::RunwayDirection(direction) => Some((identifier.as_str(), direction)),
                _ => None,
            })
    }

    /// Returns the runway centreline points of the message.
    pub fn runway_centreline_points(&self) -> impl Iterator<Item = &RunwayCentrelinePoint> {
        self.features.values().filter_map(|feature| match feature {
            Feature::RunwayCentrelinePoint(point) => Some(point),
            _ => None,
        })
    }

    /// Returns the errors of the records that could not be read.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }
}

/// Reads an AIXM 5.x message with the features that are valid when the
/// message becomes effective.
///
/// The message is read as a stream and is never loaded into memory as a
/// whole. The version is detected from the namespace of the root element.
/// The [effective date] is the begin of the temporal extent in the message
/// metadata, and features are valid if their BASELINE time slice is valid at
/// that time. Unsupported features are skipped.
///
/// [effective date]: Message::effective_date
///
/// # Errors
///
/// Returns an error if the document is not well-formed XML, if the root
/// element's namespace is not a supported AIXM 5.x message namespace or if the
/// message metadata has no temporal extent. Records with invalid values don't
/// fail the message but are listed in [`Message::errors`].
///
/// # Examples
///
/// ```
/// use aixm::{read_message, Version};
/// use chrono::NaiveDate;
///
/// # fn main() -> Result<(), aixm::Error> {
/// let xml = r#"
///     <message:AIXMBasicMessage
///         xmlns:message="http://www.aixm.aero/schema/5.1.1/message"
///         xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
///         xmlns:gmd="http://www.isotc211.org/2005/gmd"
///         xmlns:gml="http://www.opengis.net/gml/3.2">
///         <aixm:messageMetadata>
///             <gmd:EX_TemporalExtent>
///                 <gmd:extent>
///                     <gml:TimePeriod>
///                         <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
///                     </gml:TimePeriod>
///                 </gmd:extent>
///             </gmd:EX_TemporalExtent>
///         </aixm:messageMetadata>
///     </message:AIXMBasicMessage>"#;
///
/// let message = read_message(xml.as_bytes())?;
/// assert_eq!(message.version(), Version::V5_1_1);
/// let effective_date = NaiveDate::from_ymd_opt(2026, 10, 29).expect("date should be valid");
/// assert_eq!(message.effective_date(), effective_date);
/// # Ok(())
/// # }
/// ```
pub fn read_message<R: BufRead>(reader: R) -> Result<Message, Error> {
    let mut reader = NsReader::from_reader(reader);
    let mut buf = Vec::new();

    let (version, has_children) = read_root(&mut reader, &mut buf)?;

    let mut effective_time = None;
    let mut features = HashMap::new();
    let mut errors = Vec::new();

    if has_children {
        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "messageMetadata" => effective_time = read_temporal_extent(&mut reader)?,
                    "hasMember" => {
                        let date = effective_time.ok_or(Error::MissingTemporalExtent)?;
                        read_member(&mut reader, date, &mut features, &mut errors)?;
                    }
                    _ => skip(&mut reader, &element)?,
                },
                Event::End(_) => break,
                Event::Eof => {
                    return Err(Error::InvalidXml("root element is not closed".to_string()))
                }
                _ => {}
            }
        }
    }

    let effective_time = effective_time.ok_or(Error::MissingTemporalExtent)?;

    Ok(Message {
        version,
        effective_date: effective_time.date_naive(),
        features,
        errors,
    })
}

fn read_root<R: BufRead>(
    reader: &mut NsReader<R>,
    buf: &mut Vec<u8>,
) -> Result<(Version, bool), Error> {
    loop {
        buf.clear();
        let (namespace, event) = reader.read_resolved_event_into(buf)?;

        let has_children = match event {
            Event::Start(_) => true,
            Event::Empty(_) => false,
            Event::Eof => {
                return Err(Error::InvalidXml("document has no root element".to_string()))
            }
            _ => continue,
        };

        let namespace = match namespace {
            ResolveResult::Bound(Namespace(namespace)) => namespace,
            _ => "",
        };

        return Version::from_message_namespace(namespace)
            .map(|version| (version, has_children))
            .ok_or_else(|| Error::UnknownVersion(namespace.to_string()));
    }
}

/// Reads the `messageMetadata` up to its end tag and returns the begin of its
/// temporal extent (`gmd:EX_TemporalExtent`).
fn read_temporal_extent<R: BufRead>(
    reader: &mut NsReader<R>,
) -> Result<Option<DateTime<Utc>>, Error> {
    let mut buf = Vec::new();
    let mut begin = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) if element.local_name().as_ref() == "EX_TemporalExtent" => {
                begin = Some(ValidTime::read(reader, "EX_TemporalExtent")?.begin());
            }
            Event::End(element) if element.local_name().as_ref() == "messageMetadata" => {
                return Ok(begin)
            }
            Event::Eof => {
                return Err(Error::InvalidXml(
                    "messageMetadata is not closed".to_string(),
                ))
            }
            _ => {}
        }
    }
}

/// Reads a `message:hasMember` up to its end tag.
fn read_member<R: BufRead>(
    reader: &mut NsReader<R>,
    date: DateTime<Utc>,
    features: &mut HashMap<String, Feature>,
    errors: &mut Vec<Error>,
) -> Result<(), Error> {
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) => {
                let member = match element.local_name().as_ref() {
                    "AirportHeliport" => read_as(reader, date, Feature::AirportHeliport),
                    "DesignatedPoint" => read_as(reader, date, Feature::DesignatedPoint),
                    "Navaid" => read_as(reader, date, Feature::Navaid),
                    "VOR" => read_as(reader, date, |f| equipment(NavaidEquipment::VOR(f))),
                    "DME" => read_as(reader, date, |f| equipment(NavaidEquipment::DME(f))),
                    "NDB" => read_as(reader, date, |f| equipment(NavaidEquipment::NDB(f))),
                    "TACAN" => read_as(reader, date, |f| equipment(NavaidEquipment::TACAN(f))),
                    "Runway" => read_as(reader, date, Feature::Runway),
                    "RunwayDirection" => read_as(reader, date, Feature::RunwayDirection),
                    "RunwayCentrelinePoint" => {
                        read_as(reader, date, Feature::RunwayCentrelinePoint)
                    }
                    _ => {
                        skip(reader, &element)?;
                        continue;
                    }
                };

                match member {
                    Ok(Some((identifier, feature))) => {
                        features.insert(identifier, feature);
                    }
                    Ok(None) => {}
                    Err(e @ Error::InvalidRecord(_)) => errors.push(e),
                    Err(e) => return Err(e),
                }
            }
            Event::End(_) => return Ok(()),
            Event::Eof => return Err(Error::InvalidXml("hasMember is not closed".to_string())),
            _ => {}
        }
    }
}

/// Reads the feature `T` and turns it into a [`Feature`] with
/// `into_feature`.
fn read_as<T: TimeSlice, R: BufRead>(
    reader: &mut NsReader<R>,
    date: DateTime<Utc>,
    into_feature: impl FnOnce(T) -> Feature,
) -> Result<Option<(String, Feature)>, Error> {
    let member = read_feature::<T, _>(reader, date)?;
    Ok(member.map(|(identifier, time_slice)| (identifier, into_feature(time_slice))))
}

fn equipment(equipment: NavaidEquipment) -> Feature {
    Feature::NavaidEquipment(equipment)
}

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA: &str = r#"
        <aixm:messageMetadata>
            <gmd:MD_Metadata>
                <gmd:dateStamp>
                    <gco:DateTime>2026-09-30T01:46:10.846228843Z</gco:DateTime>
                </gmd:dateStamp>
                <gmd:identificationInfo>
                    <gmd:MD_DataIdentification>
                        <gmd:extent>
                            <gmd:EX_Extent>
                                <gmd:temporalElement>
                                    <gmd:EX_TemporalExtent>
                                        <gmd:extent>
                                            <gml:TimePeriod>
                                                <gml:beginPosition>2026-10-29T00:00:00.000Z</gml:beginPosition>
                                                <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00.000Z</gml:endPosition>
                                            </gml:TimePeriod>
                                        </gmd:extent>
                                    </gmd:EX_TemporalExtent>
                                </gmd:temporalElement>
                            </gmd:EX_Extent>
                        </gmd:extent>
                    </gmd:MD_DataIdentification>
                </gmd:identificationInfo>
            </gmd:MD_Metadata>
        </aixm:messageMetadata>"#;

    fn message(namespace: &str, content: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <message:AIXMBasicMessage xmlns:message="{namespace}"
                xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
                xmlns:gco="http://www.isotc211.org/2005/gco"
                xmlns:gmd="http://www.isotc211.org/2005/gmd"
                xmlns:gml="http://www.opengis.net/gml/3.2"
                xmlns:xlink="http://www.w3.org/1999/xlink">
                {content}
            </message:AIXMBasicMessage>"#
        )
    }

    fn designated_point(identifier: &str, pos: &str) -> String {
        format!(
            r#"<message:hasMember>
                <aixm:DesignatedPoint>
                    <gml:identifier>{identifier}</gml:identifier>
                    <aixm:timeSlice>
                        <aixm:DesignatedPointTimeSlice>
                            <gml:validTime>
                                <gml:TimePeriod>
                                    <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                    <gml:endPosition>2026-11-26T00:00:00Z</gml:endPosition>
                                </gml:TimePeriod>
                            </gml:validTime>
                            <aixm:interpretation>BASELINE</aixm:interpretation>
                            <aixm:designator>{identifier}</aixm:designator>
                            <aixm:location>
                                <aixm:Point srsName="urn:ogc:def:crs:EPSG::4326">
                                    <gml:pos>{pos}</gml:pos>
                                </aixm:Point>
                            </aixm:location>
                        </aixm:DesignatedPointTimeSlice>
                    </aixm:timeSlice>
                </aixm:DesignatedPoint>
            </message:hasMember>"#
        )
    }

    #[test]
    fn detects_version_from_message_namespace() {
        for (namespace, version) in [
            ("http://www.aixm.aero/schema/5.1/message", Version::V5_1),
            ("http://www.aixm.aero/schema/5.1.1/message", Version::V5_1_1),
            ("http://www.aixm.aero/schema/5.2/message", Version::V5_2),
        ] {
            let xml = message(namespace, METADATA);
            assert_eq!(
                read_message(xml.as_bytes()).map(|message| message.version()),
                Ok(version)
            );
        }
    }

    #[test]
    fn rejects_unknown_namespace() {
        let namespace = "http://www.aixm.aero/schema/4.5/message";
        let xml = message(namespace, METADATA);
        assert_eq!(
            read_message(xml.as_bytes()).map(|message| message.version()),
            Err(Error::UnknownVersion(namespace.to_string()))
        );
    }

    #[test]
    fn reads_effective_date_from_temporal_extent() {
        let xml = message("http://www.aixm.aero/schema/5.1.1/message", METADATA);
        assert_eq!(
            read_message(xml.as_bytes()).map(|message| message.effective_date()),
            Ok(NaiveDate::from_ymd_opt(2026, 10, 29).expect("test date should be valid"))
        );
    }

    #[test]
    fn rejects_message_without_temporal_extent() {
        let members = designated_point("VALID", "52.2 13.3");
        let xml = message("http://www.aixm.aero/schema/5.1.1/message", &members);
        assert_eq!(
            read_message(xml.as_bytes()).map(|message| message.version()),
            Err(Error::MissingTemporalExtent)
        );
    }

    #[test]
    fn rejects_unclosed_element() {
        let content = [METADATA, "<message:hasMember>"].concat();
        let xml = message("http://www.aixm.aero/schema/5.1.1/message", &content);
        assert!(matches!(
            read_message(xml.as_bytes()),
            Err(Error::InvalidXml(_))
        ));
    }

    #[test]
    fn collects_valid_records_and_errors_of_invalid_records() {
        let content = [
            METADATA,
            "<message:hasMember><aixm:DistanceIndication/></message:hasMember>",
            &designated_point("BROKEN", "52.2"),
            &designated_point("VALID", "52.2 13.3"),
        ]
        .concat();
        let xml = message("http://www.aixm.aero/schema/5.1.1/message", &content);

        let message = read_message(xml.as_bytes()).expect("message should be readable");
        let designators: Vec<_> = message
            .designated_points()
            .map(|point| point.designator.as_deref())
            .collect();

        assert_eq!(designators, [Some("VALID")]);
        assert!(matches!(message.errors(), [Error::InvalidRecord(_)]));
    }

    #[test]
    fn resolves_equipment_of_navaid() {
        let content = [
            METADATA,
            r#"<message:hasMember>
                <aixm:Navaid gml:id="ID_NAVAID_449">
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
                            <aixm:type>VOR_DME</aixm:type>
                            <aixm:designator>HAM</aixm:designator>
                            <aixm:navaidEquipment>
                                <aixm:NavaidComponent gml:id="ID_NAVAID_TS_COMP_1164">
                                    <aixm:providesNavigableLocation>YES</aixm:providesNavigableLocation>
                                    <aixm:theNavaidEquipment xlink:href="urn:uuid:ea3f76a6-724d-4802-b189-3777234faa1f"/>
                                </aixm:NavaidComponent>
                            </aixm:navaidEquipment>
                            <aixm:navaidEquipment>
                                <aixm:NavaidComponent gml:id="ID_NAVAID_TS_COMP_1165">
                                    <aixm:providesNavigableLocation>NO</aixm:providesNavigableLocation>
                                    <aixm:theNavaidEquipment xlink:href="urn:uuid:d0209481-44e3-43ad-aec8-be086d8e0f58"/>
                                </aixm:NavaidComponent>
                            </aixm:navaidEquipment>
                        </aixm:NavaidTimeSlice>
                    </aixm:timeSlice>
                </aixm:Navaid>
            </message:hasMember>
            <message:hasMember>
                <aixm:VOR gml:id="ID_VOR_408">
                    <gml:identifier codeSpace="urn:uuid:">ea3f76a6-724d-4802-b189-3777234faa1f</gml:identifier>
                    <aixm:timeSlice>
                        <aixm:VORTimeSlice gml:id="ID_VOR_TS_408">
                            <gml:validTime>
                                <gml:TimePeriod gml:id="ID_VOR_TS_VT_408">
                                    <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                                    <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                                </gml:TimePeriod>
                            </gml:validTime>
                            <aixm:interpretation>BASELINE</aixm:interpretation>
                            <aixm:designator>HAM</aixm:designator>
                            <aixm:name>HAMBURG</aixm:name>
                            <aixm:location>
                                <aixm:ElevatedPoint gml:id="ID_VOR_TS_EPT_408" srsName="urn:ogc:def:crs:EPSG::4326">
                                    <gml:pos>53.685575338 10.204997112</gml:pos>
                                    <aixm:elevation uom="FT">185.3675</aixm:elevation>
                                </aixm:ElevatedPoint>
                            </aixm:location>
                            <aixm:type>DVOR</aixm:type>
                            <aixm:frequency uom="MHZ">113.1</aixm:frequency>
                            <aixm:zeroBearingDirection>MAG</aixm:zeroBearingDirection>
                        </aixm:VORTimeSlice>
                    </aixm:timeSlice>
                </aixm:VOR>
            </message:hasMember>
            <message:hasMember>
                <aixm:DME gml:id="id.d0209481-44e3-43ad-aec8-be086d8e0f58">
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
                            <aixm:channel>78X</aixm:channel>
                            <aixm:ghostFrequency uom="MHZ">113.1</aixm:ghostFrequency>
                        </aixm:DMETimeSlice>
                    </aixm:timeSlice>
                </aixm:DME>
            </message:hasMember>"#,
        ]
        .concat();
        let xml = message("http://www.aixm.aero/schema/5.1.1/message", &content);
        let message = read_message(xml.as_bytes()).expect("message should be readable");

        let navaid = message.navaids().next().expect("HAM should be a navaid");
        let equipment: Vec<_> = navaid
            .navaid_equipment
            .iter()
            .map(|component| message.navaid_equipment(&component.the_navaid_equipment))
            .collect();

        let [Some(NavaidEquipment::VOR(vor)), Some(NavaidEquipment::DME(dme))] =
            equipment.as_slice()
        else {
            panic!("HAM should consist of a VOR and a DME");
        };
        assert_eq!(vor.kind.as_deref(), Some("DVOR"));
        assert_eq!(dme.channel.as_deref(), Some("78X"));
    }
}
