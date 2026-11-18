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

use super::{degrees, keep_record_error, Measure, Point, Reference, TimeSlice, EPSG_4326};
use crate::codes::{
    code, Code, CodeAirspaceActivityType, CodeAirspaceAggregationType,
    CodeAirspaceClassificationType, CodeAirspaceDependencyType, CodeAirspaceType,
    CodeVerticalReferenceType, UomDistanceType, UomDistanceVerticalType,
};
use crate::stream::{attribute, owned, Stream};
use crate::timeslice::number;
use crate::Error;

/// An airspace (UML class `Airspace`).
///
/// The airspace is made of the volumes of its geometry components. A
/// volume has vertical limits and either a horizontal projection of its
/// own or takes the geometry of a contributing airspace.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Airspace {
    /// The `type`, e.g. CTR, R or CLASS.
    pub kind: Option<CodeAirspaceType>,
    /// The coded identifier of the airspace, e.g. EDDK.
    pub designator: Option<String>,
    /// The type as the publishing state names it, e.g. TMZ or RMZ.
    pub local_type: Option<String>,
    /// The name of the airspace.
    pub name: Option<String>,
    /// The ICAO classification of the airspace's layers.
    pub class: Vec<AirspaceLayerClass>,
    /// The components the airspace's geometry is made of.
    pub geometry_component: Vec<AirspaceGeometryComponent>,
    /// The activations of the airspace.
    pub activation: Vec<AirspaceActivation>,
}

/// The ICAO classification of an airspace layer (UML class
/// `AirspaceLayerClass`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AirspaceLayerClass {
    /// The `classification`, e.g. D.
    pub classification: Option<CodeAirspaceClassificationType>,
}

/// An activation of an airspace (UML class `AirspaceActivation`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AirspaceActivation {
    /// The `activity`, e.g. PARACHUTE.
    pub activity: Option<CodeAirspaceActivityType>,
}

/// A volume and how it takes part in the airspace's geometry (UML class
/// `AirspaceGeometryComponent`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AirspaceGeometryComponent {
    /// The `operation`, e.g. BASE or UNION.
    pub operation: Option<CodeAirspaceAggregationType>,
    /// The position of the operation in the sequence of all components.
    pub operation_sequence: Option<u32>,
    /// The volume of the component.
    pub the_airspace_volume: Option<AirspaceVolume>,
}

/// A volume of an airspace (UML class `AirspaceVolume`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AirspaceVolume {
    /// The upper limit of the volume.
    pub upper_limit: Option<DistanceVertical>,
    /// The reference of the upper limit, e.g. MSL or STD.
    pub upper_limit_reference: Option<CodeVerticalReferenceType>,
    /// The lower limit of the volume.
    pub lower_limit: Option<DistanceVertical>,
    /// The reference of the lower limit, e.g. SFC or MSL.
    pub lower_limit_reference: Option<CodeVerticalReferenceType>,
    /// The horizontal projection of the volume.
    pub horizontal_projection: Option<Surface>,
    /// The airspace the volume takes its geometry from.
    pub contributor_airspace: Option<AirspaceVolumeDependency>,
}

/// A volume's dependency on the geometry of another airspace (UML class
/// `AirspaceVolumeDependency`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AirspaceVolumeDependency {
    /// The `dependency`, e.g. FULL_GEOMETRY.
    pub dependency: Option<CodeAirspaceDependencyType>,
    /// The airspace the geometry is taken from.
    pub the_airspace: Option<Reference>,
}

/// A vertical distance (`ValDistanceVerticalType`), which is either a value
/// with its unit or a special value.
#[derive(Clone, PartialEq, Debug)]
pub enum DistanceVertical {
    /// A value with its unit, e.g. 2500 FT or 95 FL.
    Value(Measure<UomDistanceVerticalType>),
    /// The ground (GND).
    Gnd,
    /// Unlimited (UNL).
    Unl,
    /// The floor of the airspace above (FLOOR).
    Floor,
    /// The ceiling of the airspace below (CEILING).
    Ceiling,
}

/// The horizontal projection of an airspace volume (`aixm:Surface`).
///
/// The surface is the exterior ring of one polygon patch, given as the
/// segments of its curves in the order they are drawn.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Surface {
    /// The segments of the exterior ring.
    pub exterior: Vec<CurveSegment>,
}

/// A segment of a curve (`gml:segments`).
#[derive(Clone, PartialEq, Debug)]
pub enum CurveSegment {
    /// Positions joined by geodesics (`gml:GeodesicString`).
    GeodesicString(Vec<Point>),
    /// Positions joined by straight lines (`gml:LineStringSegment`).
    LineStringSegment(Vec<Point>),
    /// An arc around a centre (`gml:ArcByCenterPoint`).
    ArcByCenterPoint {
        /// The centre of the arc.
        center: Point,
        /// The radius of the arc.
        radius: Measure<UomDistanceType>,
        /// The angle the arc starts at, in degrees.
        start_angle: f64,
        /// The angle the arc ends at, in degrees.
        end_angle: f64,
    },
    /// A circle around a centre (`gml:CircleByCenterPoint`).
    CircleByCenterPoint {
        /// The centre of the circle.
        center: Point,
        /// The radius of the circle.
        radius: Measure<UomDistanceType>,
    },
}

impl Surface {
    /// Reads an `aixm:horizontalProjection` up to its end tag.
    fn read_horizontal_projection<R: BufRead>(stream: &mut Stream<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut surface = Self::default();
        let mut patches = 0;
        let mut record_error = None;

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "Surface" => {
                        let srs_name = attribute(&element, "srsName")?;
                        if srs_name.as_deref() != Some(EPSG_4326) {
                            record_error.get_or_insert(format!(
                                "surface should be in {EPSG_4326} but is in {srs_name:?}"
                            ));
                        }
                    }
                    "PolygonPatch" => {
                        patches += 1;
                        if patches > 1 {
                            record_error
                                .get_or_insert("surface should have one polygon patch".to_string());
                        }
                    }
                    "segments" => {
                        let result = read_segments(stream);
                        if let Some(segments) = keep_record_error(result, &mut record_error)? {
                            surface.exterior.extend(segments);
                        }
                    }
                    _ => {}
                },
                // a curve member without content refers to a curve of
                // another feature, e.g. a border
                Event::Empty(element) if element.local_name().as_ref() == "curveMember" => {
                    record_error.get_or_insert("referenced curves are not supported".to_string());
                }
                Event::End(element) if element.local_name().as_ref() == "horizontalProjection" => {
                    break
                }
                Event::Eof => {
                    return Err(Error::InvalidXml(
                        "horizontalProjection is not closed".to_string(),
                    ))
                }
                _ => {}
            }
        }

        match record_error {
            Some(message) => Err(Error::InvalidRecord(message)),
            None => Ok(surface),
        }
    }
}

/// Reads a `gml:segments` up to its end tag.
fn read_segments<R: BufRead>(stream: &mut Stream<R>) -> Result<Vec<CurveSegment>, Error> {
    let mut buf = Vec::new();
    let mut segments = Vec::new();
    let mut record_error = None;

    loop {
        buf.clear();
        match stream.read_event_into(&mut buf)? {
            Event::Start(element) => {
                let segment = match element.local_name().as_ref() {
                    "GeodesicString" => {
                        read_positions(stream, "GeodesicString").map(CurveSegment::GeodesicString)
                    }
                    "LineStringSegment" => read_positions(stream, "LineStringSegment")
                        .map(CurveSegment::LineStringSegment),
                    "ArcByCenterPoint" => read_by_center_point(stream, "ArcByCenterPoint"),
                    "CircleByCenterPoint" => read_by_center_point(stream, "CircleByCenterPoint"),
                    name => {
                        stream.skip(&element)?;
                        Err(Error::InvalidRecord(format!(
                            "segment {name} is not supported"
                        )))
                    }
                };

                if let Some(segment) = keep_record_error(segment, &mut record_error)? {
                    segments.push(segment);
                }
            }
            Event::End(element) if element.local_name().as_ref() == "segments" => break,
            Event::Eof => return Err(Error::InvalidXml("segments is not closed".to_string())),
            _ => {}
        }
    }

    match record_error {
        Some(message) => Err(Error::InvalidRecord(message)),
        None => Ok(segments),
    }
}

/// Reads the `gml:pos` and `gml:posList` of the segment `element` up to
/// its end tag.
fn read_positions<R: BufRead>(stream: &mut Stream<R>, element: &str) -> Result<Vec<Point>, Error> {
    let mut buf = Vec::new();
    let mut positions = Vec::new();
    let mut record_error = None;

    loop {
        buf.clear();
        match stream.read_event_into(&mut buf)? {
            Event::Start(start) => match start.local_name().as_ref() {
                "pos" => {
                    let result = stream.read_text(&start, Point::from_pos);
                    positions.extend(keep_record_error(result, &mut record_error)?);
                }
                "posList" => {
                    let result = stream.read_text(&start, Point::from_pos_list);
                    positions.extend(
                        keep_record_error(result, &mut record_error)?
                            .into_iter()
                            .flatten(),
                    );
                }
                _ => {}
            },
            Event::End(end) if end.local_name().as_ref() == element => break,
            Event::Eof => return Err(Error::InvalidXml(format!("{element} is not closed"))),
            _ => {}
        }
    }

    match record_error {
        Some(message) => Err(Error::InvalidRecord(message)),
        None => Ok(positions),
    }
}

/// Reads the arc or circle `element` around a centre up to its end tag.
fn read_by_center_point<R: BufRead>(
    stream: &mut Stream<R>,
    element: &str,
) -> Result<CurveSegment, Error> {
    let mut buf = Vec::new();
    let mut center = None;
    let mut radius = None;
    let mut start_angle = None;
    let mut end_angle = None;
    let mut record_error = None;

    loop {
        buf.clear();
        match stream.read_event_into(&mut buf)? {
            Event::Start(start) => match start.local_name().as_ref() {
                "pos" => {
                    let result = stream.read_text(&start, Point::from_pos);
                    center = keep_record_error(result, &mut record_error)?;
                }
                "radius" => {
                    let result = Measure::read(stream, &start);
                    radius = keep_record_error(result, &mut record_error)?.flatten();
                }
                "startAngle" => {
                    let result = stream.read_text(&start, degrees);
                    start_angle = keep_record_error(result, &mut record_error)?;
                }
                "endAngle" => {
                    let result = stream.read_text(&start, degrees);
                    end_angle = keep_record_error(result, &mut record_error)?;
                }
                _ => {}
            },
            Event::End(end) if end.local_name().as_ref() == element => break,
            Event::Eof => return Err(Error::InvalidXml(format!("{element} is not closed"))),
            _ => {}
        }
    }

    if let Some(message) = record_error {
        return Err(Error::InvalidRecord(message));
    }

    let (Some(center), Some(radius)) = (center, radius) else {
        return Err(Error::InvalidRecord(format!(
            "{element} has no centre or no radius"
        )));
    };

    if element == "CircleByCenterPoint" {
        return Ok(CurveSegment::CircleByCenterPoint { center, radius });
    }

    match (start_angle, end_angle) {
        (Some(start_angle), Some(end_angle)) => Ok(CurveSegment::ArcByCenterPoint {
            center,
            radius,
            start_angle,
            end_angle,
        }),
        _ => Err(Error::InvalidRecord(format!(
            "{element} has no start or no end angle"
        ))),
    }
}

impl DistanceVertical {
    /// Reads the vertical distance `element` up to its end tag. Returns
    /// `None` if the element has no value.
    fn read<R: BufRead>(
        stream: &mut Stream<R>,
        element: &BytesStart,
    ) -> Result<Option<Self>, Error> {
        let uom = attribute(element, "uom")?.map(|uom| UomDistanceVerticalType::from_code(&uom));

        stream.read_optional_text(element, |text| match text {
            "GND" => Ok(Self::Gnd),
            "UNL" => Ok(Self::Unl),
            "FLOOR" => Ok(Self::Floor),
            "CEILING" => Ok(Self::Ceiling),
            value => Measure::parse(value, uom).map(Self::Value),
        })
    }
}

impl AirspaceLayerClass {
    /// Reads an `aixm:class` up to its end tag.
    fn read_class<R: BufRead>(stream: &mut Stream<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut class = Self::default();

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(element) if element.local_name().as_ref() == "classification" => {
                    class.classification = stream.read_optional_text(&element, code)?
                }
                Event::End(element) if element.local_name().as_ref() == "class" => break,
                Event::Eof => return Err(Error::InvalidXml("class is not closed".to_string())),
                _ => {}
            }
        }

        Ok(class)
    }
}

impl AirspaceActivation {
    /// Reads an `aixm:activation` up to its end tag.
    fn read_activation<R: BufRead>(stream: &mut Stream<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut activation = Self::default();

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(element) if element.local_name().as_ref() == "activity" => {
                    activation.activity = stream.read_optional_text(&element, code)?
                }
                Event::End(element) if element.local_name().as_ref() == "activation" => break,
                Event::Eof => {
                    return Err(Error::InvalidXml("activation is not closed".to_string()))
                }
                _ => {}
            }
        }

        Ok(activation)
    }
}

impl AirspaceGeometryComponent {
    /// Reads an `aixm:geometryComponent` up to its end tag.
    fn read_geometry_component<R: BufRead>(stream: &mut Stream<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut component = Self::default();
        let mut record_error = None;

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "operation" => {
                        component.operation = stream.read_optional_text(&element, code)?
                    }
                    "operationSequence" => {
                        let result = stream.read_optional_text(&element, number);
                        component.operation_sequence =
                            keep_record_error(result, &mut record_error)?.flatten();
                    }
                    "theAirspaceVolume" => {
                        let result = AirspaceVolume::read_the_airspace_volume(stream);
                        component.the_airspace_volume =
                            keep_record_error(result, &mut record_error)?;
                    }
                    _ => {}
                },
                Event::End(element) if element.local_name().as_ref() == "geometryComponent" => {
                    break
                }
                Event::Eof => {
                    return Err(Error::InvalidXml(
                        "geometryComponent is not closed".to_string(),
                    ))
                }
                _ => {}
            }
        }

        match record_error {
            Some(message) => Err(Error::InvalidRecord(message)),
            None => Ok(component),
        }
    }
}

impl AirspaceVolume {
    /// Reads an `aixm:theAirspaceVolume` up to its end tag.
    fn read_the_airspace_volume<R: BufRead>(stream: &mut Stream<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut volume = Self::default();
        let mut record_error = None;

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "upperLimit" => {
                        let result = DistanceVertical::read(stream, &element);
                        volume.upper_limit =
                            keep_record_error(result, &mut record_error)?.flatten();
                    }
                    "upperLimitReference" => {
                        volume.upper_limit_reference = stream.read_optional_text(&element, code)?
                    }
                    "lowerLimit" => {
                        let result = DistanceVertical::read(stream, &element);
                        volume.lower_limit =
                            keep_record_error(result, &mut record_error)?.flatten();
                    }
                    "lowerLimitReference" => {
                        volume.lower_limit_reference = stream.read_optional_text(&element, code)?
                    }
                    "contributorAirspace" => {
                        let result = AirspaceVolumeDependency::read_contributor_airspace(stream);
                        volume.contributor_airspace = keep_record_error(result, &mut record_error)?;
                    }
                    "horizontalProjection" => {
                        let result = Surface::read_horizontal_projection(stream);
                        volume.horizontal_projection =
                            keep_record_error(result, &mut record_error)?;
                    }
                    // the centreline of a corridor is not read
                    "centreline" => stream.skip(&element)?,
                    _ => {}
                },
                Event::End(element) if element.local_name().as_ref() == "theAirspaceVolume" => {
                    break
                }
                Event::Eof => {
                    return Err(Error::InvalidXml(
                        "theAirspaceVolume is not closed".to_string(),
                    ))
                }
                _ => {}
            }
        }

        match record_error {
            Some(message) => Err(Error::InvalidRecord(message)),
            None => Ok(volume),
        }
    }
}

impl AirspaceVolumeDependency {
    /// Reads an `aixm:contributorAirspace` up to its end tag.
    fn read_contributor_airspace<R: BufRead>(stream: &mut Stream<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut dependency = Self::default();
        let mut record_error = None;

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "dependency" => {
                        dependency.dependency = stream.read_optional_text(&element, code)?
                    }
                    "theAirspace" => {
                        let reference = Reference::from_element(&element);
                        stream.skip(&element)?;
                        dependency.the_airspace = keep_record_error(reference, &mut record_error)?;
                    }
                    _ => {}
                },
                Event::Empty(element) if element.local_name().as_ref() == "theAirspace" => {
                    let reference = Reference::from_element(&element);
                    dependency.the_airspace = keep_record_error(reference, &mut record_error)?;
                }
                Event::End(element) if element.local_name().as_ref() == "contributorAirspace" => {
                    break
                }
                Event::Eof => {
                    return Err(Error::InvalidXml(
                        "contributorAirspace is not closed".to_string(),
                    ))
                }
                _ => {}
            }
        }

        match record_error {
            Some(message) => Err(Error::InvalidRecord(message)),
            None => Ok(dependency),
        }
    }
}

impl TimeSlice for Airspace {
    const FEATURE: &'static str = "Airspace";

    fn read_property<R: BufRead>(
        &mut self,
        stream: &mut Stream<R>,
        element: &BytesStart,
    ) -> Result<(), Error> {
        match element.local_name().as_ref() {
            "type" => self.kind = stream.read_optional_text(element, code)?,
            "designator" => self.designator = stream.read_optional_text(element, owned)?,
            "localType" => self.local_type = stream.read_optional_text(element, owned)?,
            "name" => self.name = stream.read_optional_text(element, owned)?,
            "class" => self.class.push(AirspaceLayerClass::read_class(stream)?),
            "geometryComponent" => self
                .geometry_component
                .push(AirspaceGeometryComponent::read_geometry_component(stream)?),
            "activation" => self
                .activation
                .push(AirspaceActivation::read_activation(stream)?),
            _ => stream.skip(element)?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::read_feature;
    use chrono::{DateTime, Utc};

    const CTR_EDDK: &str = r#"
        <aixm:Airspace gml:id="id.c700554c-d360-4476-925c-86e411f1a350"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
            <gml:identifier codeSpace="urn:uuid:">c700554c-d360-4476-925c-86e411f1a350</gml:identifier>
            <aixm:timeSlice>
                <aixm:AirspaceTimeSlice gml:id="id.ts.c700554c-d360-4476-925c-86e411f1a350">
                    <gml:validTime>
                        <gml:TimePeriod gml:id="id.ts.vt.c700554c-d360-4476-925c-86e411f1a350">
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:featureLifetime>
                        <gml:TimePeriod gml:id="id.ts.flt.c700554c-d360-4476-925c-86e411f1a350">
                            <gml:beginPosition>2015-05-01T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="unknown"/>
                        </gml:TimePeriod>
                    </aixm:featureLifetime>
                    <aixm:type>CTR</aixm:type>
                    <aixm:designator>EDDK</aixm:designator>
                    <aixm:localType xsi:nil="true" nilReason="unknown"/>
                    <aixm:name>KOELN/BONN</aixm:name>
                    <aixm:designatorICAO xsi:nil="true" nilReason="unknown"/>
                    <aixm:controlType xsi:nil="true" nilReason="unknown"/>
                    <aixm:upperLowerSeparation xsi:nil="true" nilReason="unknown"/>
                    <aixm:class>
                        <aixm:AirspaceLayerClass gml:id="id.ts.alc.c700554c-d360-4476-925c-86e411f1a350">
                            <aixm:classification>D</aixm:classification>
                        </aixm:AirspaceLayerClass>
                    </aixm:class>
                    <aixm:geometryComponent>
                        <aixm:AirspaceGeometryComponent gml:id="id.asegeomcomp.c700554c-d360-4476-925c-86e411f1a350">
                            <aixm:theAirspaceVolume>
                                <aixm:AirspaceVolume gml:id="id.asevol.c700554c-d360-4476-925c-86e411f1a350">
                                    <aixm:upperLimit uom="FT">2500</aixm:upperLimit>
                                    <aixm:upperLimitReference>MSL</aixm:upperLimitReference>
                                    <aixm:lowerLimit uom="FT">0</aixm:lowerLimit>
                                    <aixm:lowerLimitReference>SFC</aixm:lowerLimitReference>
                                    <aixm:width xsi:nil="true" nilReason="inapplicable"/>
                                    <aixm:horizontalProjection>
                                        <aixm:Surface gml:id="ID_SURF_962" srsName="urn:ogc:def:crs:EPSG::4326">
                                            <gml:patches>
                                                <gml:PolygonPatch>
                                                    <gml:exterior>
                                                        <gml:Ring>
                                                            <gml:curveMember>
                                                                <gml:Curve gml:id="ID_CURV_962-6233">
                                                                    <gml:segments>
                                                                        <gml:GeodesicString>
                                                                            <gml:pos>51.017222000000 7.030278000000</gml:pos>
                                                                            <gml:pos>50.950278000000 7.129722000000</gml:pos>
                                                                            <gml:pos>50.940833000000 7.205833000000</gml:pos>
                                                                            <gml:pos>50.975278000000 7.323333000000</gml:pos>
                                                                            <gml:pos>50.884722000000 7.390000000000</gml:pos>
                                                                            <gml:pos>50.851389000000 7.275833000000</gml:pos>
                                                                            <gml:pos>50.785833333000 7.372500000000</gml:pos>
                                                                            <gml:pos>50.723055556000 7.265555556000</gml:pos>
                                                                            <gml:pos>50.786944000000 7.172500000000</gml:pos>
                                                                            <gml:pos>50.794722000000 7.084444000000</gml:pos>
                                                                            <gml:pos>50.779444000000 7.033889000000</gml:pos>
                                                                            <gml:pos>50.861666667000 6.973055556000</gml:pos>
                                                                            <gml:pos>50.948889000000 6.914167000000</gml:pos>
                                                                            <gml:pos>51.017222000000 7.030278000000</gml:pos>
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
                    <aixm:activation>
                        <aixm:AirspaceActivation gml:id="id.ts.c700554c-d360-4476-925c-86e411f1a350-ac-1">
                            <aixm:activity xsi:nil="true" nilReason="unknown"/>
                            <aixm:status>ACTIVE</aixm:status>
                        </aixm:AirspaceActivation>
                    </aixm:activation>
                    <aixm:annotation>
                        <aixm:Note gml:id="id.ts.note.c700554c-d360-4476-925c-86e411f1a350-2-1">
                            <aixm:propertyName>activation</aixm:propertyName>
                            <aixm:purpose>DESCRIPTION</aixm:purpose>
                            <aixm:translatedNote>
                                <aixm:LinguisticNote gml:id="id.ts.note.c700554c-d360-4476-925c-86e411f1a350-2-2">
                                    <aixm:note>H24</aixm:note>
                                </aixm:LinguisticNote>
                            </aixm:translatedNote>
                        </aixm:Note>
                    </aixm:annotation>
                </aixm:AirspaceTimeSlice>
            </aixm:timeSlice>
        </aixm:Airspace>"#;

    const CLASS_EDLW: &str = r#"
        <aixm:Airspace gml:id="id.2a5dfa32-1b9f-49ad-94e0-f24288a1265b"
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
            <gml:identifier codeSpace="urn:uuid:">2a5dfa32-1b9f-49ad-94e0-f24288a1265b</gml:identifier>
            <aixm:timeSlice>
                <aixm:AirspaceTimeSlice gml:id="id.ts.2a5dfa32-1b9f-49ad-94e0-f24288a1265b">
                    <gml:validTime>
                        <gml:TimePeriod gml:id="id.ts.vt.2a5dfa32-1b9f-49ad-94e0-f24288a1265b">
                            <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                            <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00Z</gml:endPosition>
                        </gml:TimePeriod>
                    </gml:validTime>
                    <aixm:interpretation>BASELINE</aixm:interpretation>
                    <aixm:type>CLASS</aixm:type>
                    <aixm:designator>EDLWCLD</aixm:designator>
                    <aixm:localType xsi:nil="true" nilReason="unknown"/>
                    <aixm:name>DORTMUND</aixm:name>
                    <aixm:class>
                        <aixm:AirspaceLayerClass gml:id="id.ts.alc.2a5dfa32-1b9f-49ad-94e0-f24288a1265b">
                            <aixm:classification>D</aixm:classification>
                        </aixm:AirspaceLayerClass>
                    </aixm:class>
                    <aixm:geometryComponent>
                        <aixm:AirspaceGeometryComponent gml:id="id.asegeomcomp.2a5dfa32-1b9f-49ad-94e0-f24288a1265b-10">
                            <aixm:operation>BASE</aixm:operation>
                            <aixm:operationSequence>1</aixm:operationSequence>
                            <aixm:theAirspaceVolume>
                                <aixm:AirspaceVolume gml:id="id.asevol.2a5dfa32-1b9f-49ad-94e0-f24288a1265b-10">
                                    <aixm:contributorAirspace>
                                        <aixm:AirspaceVolumeDependency gml:id="id.avdep.2a5dfa32-1b9f-49ad-94e0-f24288a1265b-10">
                                            <aixm:dependency>FULL_GEOMETRY</aixm:dependency>
                                            <aixm:theAirspace xlink:href="urn:uuid:737ab4d5-2442-4063-8854-0dec4ba430a9" xlink:title="EDLWCLDA"/>
                                        </aixm:AirspaceVolumeDependency>
                                    </aixm:contributorAirspace>
                                </aixm:AirspaceVolume>
                            </aixm:theAirspaceVolume>
                        </aixm:AirspaceGeometryComponent>
                    </aixm:geometryComponent>
                    <aixm:geometryComponent>
                        <aixm:AirspaceGeometryComponent gml:id="id.asegeomcomp.2a5dfa32-1b9f-49ad-94e0-f24288a1265b-20">
                            <aixm:operation>UNION</aixm:operation>
                            <aixm:operationSequence>2</aixm:operationSequence>
                            <aixm:theAirspaceVolume>
                                <aixm:AirspaceVolume gml:id="id.asevol.2a5dfa32-1b9f-49ad-94e0-f24288a1265b-20">
                                    <aixm:contributorAirspace>
                                        <aixm:AirspaceVolumeDependency gml:id="id.avdep.2a5dfa32-1b9f-49ad-94e0-f24288a1265b-20">
                                            <aixm:dependency>FULL_GEOMETRY</aixm:dependency>
                                            <aixm:theAirspace xlink:href="urn:uuid:376b66ab-edb9-4f2d-bf74-899f8818259b" xlink:title="EDLWCLDB"/>
                                        </aixm:AirspaceVolumeDependency>
                                    </aixm:contributorAirspace>
                                </aixm:AirspaceVolume>
                            </aixm:theAirspaceVolume>
                        </aixm:AirspaceGeometryComponent>
                    </aixm:geometryComponent>
                </aixm:AirspaceTimeSlice>
            </aixm:timeSlice>
        </aixm:Airspace>"#;

    fn read(xml: &str) -> Result<Airspace, Error> {
        let mut stream = Stream::new(xml.as_bytes());
        let mut buf = Vec::new();

        // enter the feature element
        loop {
            buf.clear();
            if let Event::Start(_) = stream.read_event_into(&mut buf)? {
                break;
            }
        }

        let date = DateTime::parse_from_rfc3339("2026-10-29T00:00:00Z")
            .expect("test date should be RFC 3339")
            .with_timezone(&Utc);

        read_feature::<Airspace, _>(&mut stream, date).map(|feature| {
            feature
                .map(|(_, time_slice)| time_slice)
                .expect("feature should have a valid time slice")
        })
    }

    /// The horizontal projection of BIRKEN-65 (EDDMGLD01).
    const SURFACE_EDDMGLD01: &str = r#"
        <aixm:horizontalProjection
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2">
            <aixm:Surface gml:id="ID_SURF_5554" srsName="urn:ogc:def:crs:EPSG::4326">
                <gml:patches>
                    <gml:PolygonPatch>
                        <gml:exterior>
                            <gml:Ring>
                                <gml:curveMember>
                                    <gml:Curve gml:id="ID_CURV_5554-27417">
                                        <gml:segments>
                                            <gml:GeodesicString>
                                                <gml:pos>48.449444444000 12.149166667000</gml:pos>
                                                <gml:pos>48.501111111000 12.114722222000</gml:pos>
                                                <gml:pos>48.520846091000 12.386567680000</gml:pos>
                                            </gml:GeodesicString>
                                            <gml:GeodesicString>
                                                <gml:posList>48.520846091 12.38656768 48.517347513 12.388717125</gml:posList>
                                            </gml:GeodesicString>
                                            <gml:ArcByCenterPoint numArc="1">
                                                <gml:pos>48.353888889 11.786111111</gml:pos>
                                                <gml:radius uom="[nmi_i]">26</gml:radius>
                                                <gml:startAngle uom="deg">67.597181792000001</gml:startAngle>
                                                <gml:endAngle uom="deg">73.903431901999994</gml:endAngle>
                                            </gml:ArcByCenterPoint>
                                            <gml:GeodesicString>
                                                <gml:posList>48.472250349 12.411802638 48.468606844 12.413309599</gml:posList>
                                            </gml:GeodesicString>
                                            <gml:GeodesicString>
                                                <gml:pos>48.468606844000 12.413309599000</gml:pos>
                                                <gml:pos>48.449444444000 12.149166667000</gml:pos>
                                            </gml:GeodesicString>
                                        </gml:segments>
                                    </gml:Curve>
                                </gml:curveMember>
                            </gml:Ring>
                        </gml:exterior>
                    </gml:PolygonPatch>
                </gml:patches>
            </aixm:Surface>
        </aixm:horizontalProjection>"#;

    /// The horizontal projection of HETZLESER BERG/BAYERN (EDQXACR).
    const SURFACE_EDQXACR: &str = r#"
        <aixm:horizontalProjection
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:xlink="http://www.w3.org/1999/xlink">
            <aixm:Surface gml:id="ID_SURF_12461" srsName="urn:ogc:def:crs:EPSG::4326">
                <gml:patches>
                    <gml:PolygonPatch>
                        <gml:exterior>
                            <gml:Ring>
                                <gml:curveMember>
                                    <gml:Curve gml:id="ID_CURV_12461-100394">
                                        <gml:segments>
                                            <gml:CircleByCenterPoint numArc="1">
                                                <gml:pos>49.641666667 11.162777778</gml:pos>
                                                <gml:radius uom="[nmi_i]">1.4</gml:radius>
                                            </gml:CircleByCenterPoint>
                                        </gml:segments>
                                    </gml:Curve>
                                </gml:curveMember>
                            </gml:Ring>
                        </gml:exterior>
                    </gml:PolygonPatch>
                </gml:patches>
            </aixm:Surface>
        </aixm:horizontalProjection>"#;

    fn read_surface(xml: &str) -> Result<Surface, Error> {
        let mut stream = Stream::new(xml.as_bytes());
        let mut buf = Vec::new();

        // enter the horizontal projection
        loop {
            buf.clear();
            if let Event::Start(_) = stream.read_event_into(&mut buf)? {
                break;
            }
        }

        Surface::read_horizontal_projection(&mut stream)
    }

    fn points(positions: &[(f64, f64)]) -> Vec<Point> {
        positions
            .iter()
            .map(|&(latitude, longitude)| point(latitude, longitude))
            .collect()
    }

    fn point(latitude: f64, longitude: f64) -> Point {
        Point {
            latitude,
            longitude,
        }
    }

    fn nautical_miles(value: f64) -> Measure<UomDistanceType> {
        Measure {
            value,
            uom: UomDistanceType::NMI_I,
        }
    }

    fn feet(value: f64) -> Option<DistanceVertical> {
        Some(DistanceVertical::Value(Measure {
            value,
            uom: UomDistanceVerticalType::FT,
        }))
    }

    fn contributor(
        operation: CodeAirspaceAggregationType,
        operation_sequence: u32,
        uuid: &str,
        title: &str,
    ) -> AirspaceGeometryComponent {
        AirspaceGeometryComponent {
            operation: Some(operation),
            operation_sequence: Some(operation_sequence),
            the_airspace_volume: Some(AirspaceVolume {
                contributor_airspace: Some(AirspaceVolumeDependency {
                    dependency: Some(CodeAirspaceDependencyType::FULL_GEOMETRY),
                    the_airspace: Some(Reference {
                        uuid: uuid.to_string(),
                        title: Some(title.to_string()),
                    }),
                }),
                ..Default::default()
            }),
        }
    }

    #[test]
    fn reads_airspace_with_class_and_vertical_limits() {
        assert_eq!(
            read(CTR_EDDK),
            Ok(Airspace {
                kind: Some(CodeAirspaceType::CTR),
                designator: Some("EDDK".to_string()),
                local_type: None,
                name: Some("KOELN/BONN".to_string()),
                class: vec![AirspaceLayerClass {
                    classification: Some(CodeAirspaceClassificationType::D),
                }],
                geometry_component: vec![AirspaceGeometryComponent {
                    operation: None,
                    operation_sequence: None,
                    the_airspace_volume: Some(AirspaceVolume {
                        upper_limit: feet(2500.0),
                        upper_limit_reference: Some(CodeVerticalReferenceType::MSL),
                        lower_limit: feet(0.0),
                        lower_limit_reference: Some(CodeVerticalReferenceType::SFC),
                        horizontal_projection: Some(Surface {
                            exterior: vec![CurveSegment::GeodesicString(points(&[
                                (51.017222, 7.030278),
                                (50.950278, 7.129722),
                                (50.940833, 7.205833),
                                (50.975278, 7.323333),
                                (50.884722, 7.39),
                                (50.851389, 7.275833),
                                (50.785833333, 7.3725),
                                (50.723055556, 7.265555556),
                                (50.786944, 7.1725),
                                (50.794722, 7.084444),
                                (50.779444, 7.033889),
                                (50.861666667, 6.973055556),
                                (50.948889, 6.914167),
                                (51.017222, 7.030278),
                            ]))],
                        }),
                        contributor_airspace: None,
                    }),
                }],
                activation: vec![AirspaceActivation { activity: None }],
            })
        );
    }

    #[test]
    fn reads_volumes_of_contributing_airspaces() {
        let airspace = read(CLASS_EDLW).expect("EDLWCLD should be readable");
        assert_eq!(
            airspace.geometry_component,
            [
                contributor(
                    CodeAirspaceAggregationType::BASE,
                    1,
                    "737ab4d5-2442-4063-8854-0dec4ba430a9",
                    "EDLWCLDA"
                ),
                contributor(
                    CodeAirspaceAggregationType::UNION,
                    2,
                    "376b66ab-edb9-4f2d-bf74-899f8818259b",
                    "EDLWCLDB"
                ),
            ]
        );
    }

    #[test]
    fn reads_special_value_of_vertical_limit() {
        let xml = CTR_EDDK.replace(
            r#"<aixm:upperLimit uom="FT">2500</aixm:upperLimit>"#,
            r#"<aixm:upperLimit uom="OTHER">UNL</aixm:upperLimit>"#,
        );
        let airspace = read(&xml).expect("EDDK should be readable");
        let volume = airspace.geometry_component[0]
            .the_airspace_volume
            .as_ref()
            .expect("EDDK should have a volume");
        assert_eq!(volume.upper_limit, Some(DistanceVertical::Unl));
    }

    #[test]
    fn reads_segments_of_horizontal_projection() {
        assert_eq!(
            read_surface(SURFACE_EDDMGLD01),
            Ok(Surface {
                exterior: vec![
                    CurveSegment::GeodesicString(points(&[
                        (48.449444444, 12.149166667),
                        (48.501111111, 12.114722222),
                        (48.520846091, 12.38656768),
                    ])),
                    CurveSegment::GeodesicString(points(&[
                        (48.520846091, 12.38656768),
                        (48.517347513, 12.388717125),
                    ])),
                    CurveSegment::ArcByCenterPoint {
                        center: point(48.353888889, 11.786111111),
                        radius: nautical_miles(26.0),
                        start_angle: 67.597181792000001,
                        end_angle: 73.903431901999994,
                    },
                    CurveSegment::GeodesicString(points(&[
                        (48.472250349, 12.411802638),
                        (48.468606844, 12.413309599),
                    ])),
                    CurveSegment::GeodesicString(points(&[
                        (48.468606844, 12.413309599),
                        (48.449444444, 12.149166667),
                    ])),
                ],
            })
        );
    }

    #[test]
    fn reads_circle_of_horizontal_projection() {
        assert_eq!(
            read_surface(SURFACE_EDQXACR),
            Ok(Surface {
                exterior: vec![CurveSegment::CircleByCenterPoint {
                    center: point(49.641666667, 11.162777778),
                    radius: nautical_miles(1.4),
                }],
            })
        );
    }

    #[test]
    fn rejects_referenced_curve() {
        let xml = SURFACE_EDQXACR.replace(
            r#"<gml:curveMember>"#,
            r#"<gml:curveMember xlink:href="urn:uuid:0bf35787-9bc9-473b-b308-c04343fec2f5"/><gml:curveMember>"#,
        );
        assert!(matches!(read_surface(&xml), Err(Error::InvalidRecord(_))));
    }

    #[test]
    fn rejects_arc_without_angles() {
        let xml = SURFACE_EDQXACR.replace("CircleByCenterPoint", "ArcByCenterPoint");
        assert!(matches!(read_surface(&xml), Err(Error::InvalidRecord(_))));
    }

    #[test]
    fn rejects_vertical_limit_without_uom() {
        let xml = CTR_EDDK.replace(r#"<aixm:upperLimit uom="FT">"#, "<aixm:upperLimit>");
        assert!(matches!(read(&xml), Err(Error::InvalidRecord(_))));
    }
}
