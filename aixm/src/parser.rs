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

//! Streaming AIXM 5.1 XML parser.
//!
//! The parser scans the XML byte stream for supported AIXM feature elements
//! (e.g. `<aixm:AirportHeliport>`, `<aixm:Navaid>`, …), captures each feature
//! subtree, and deserializes it with serde into an internal XML-mapping struct.
//! That struct is then converted into a flat public [`Feature`] type.
//!
//! AIXM XML uses deeply nested elements (time slices, GML geometry, xlink
//! references) that require matching serde structs. These internal structs are
//! private to this module — callers only see the flat types from
//! [`crate::features`].

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Deserialize;

use crate::error::Error;
use crate::features::*;

// ===========================================================================
// Public iterator
// ===========================================================================

/// Streaming iterator over AIXM 5.1 features in an XML document.
///
/// Yields one [`Feature`] at a time while scanning through the XML byte slice.
/// Only AIXM feature types relevant for navigation are returned — all other
/// elements are silently skipped.
///
/// # Supported feature types
///
/// | AIXM element             | Yielded as                          |
/// |--------------------------|-------------------------------------|
/// | `AirportHeliport`        | [`Feature::AirportHeliport`]        |
/// | `Runway`                 | [`Feature::Runway`]                 |
/// | `RunwayDirection`        | [`Feature::RunwayDirection`]        |
/// | `DesignatedPoint`        | [`Feature::DesignatedPoint`]        |
/// | `Navaid`                 | [`Feature::Navaid`]                 |
/// | `Airspace`               | [`Feature::Airspace`]               |
///
/// # Examples
///
/// Parse an AIXM file and collect all airport designators:
///
/// ```no_run
/// let data = std::fs::read("aixm_data.xml").unwrap();
/// let airports: Vec<String> = aixm::Features::new(&data)
///     .filter_map(Result::ok)
///     .filter_map(|f| match f {
///         aixm::Feature::AirportHeliport(ahp) => Some(ahp.designator),
///         _ => None,
///     })
///     .collect();
/// ```
pub struct Features<'a> {
    reader: Reader<&'a [u8]>,
    data: &'a [u8],
    buf: Vec<u8>,
}

impl<'a> Features<'a> {
    /// Creates a new streaming parser from an AIXM XML byte slice.
    ///
    /// The returned iterator lazily parses features as they are consumed.
    ///
    /// # Examples
    ///
    /// ```
    /// let xml = br#"<message:AIXMBasicMessage
    ///   xmlns:aixm="http://www.aixm.aero/schema/5.1"
    ///   xmlns:gml="http://www.opengis.net/gml/3.2"
    ///   xmlns:message="http://www.aixm.aero/schema/5.1/message">
    /// </message:AIXMBasicMessage>"#;
    ///
    /// let features: Vec<_> = aixm::Features::new(&xml[..])
    ///     .collect::<Result<_, _>>()
    ///     .unwrap();
    ///
    /// assert!(features.is_empty());
    /// ```
    pub fn new(data: &'a [u8]) -> Self {
        let mut reader = Reader::from_reader(data);
        reader.config_mut().trim_text(true);
        Self {
            reader,
            data,
            buf: Vec::new(),
        }
    }
}

impl<'a> Iterator for Features<'a> {
    type Item = Result<Feature, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());

                    let kind = match local {
                        b"AirportHeliport" => FeatureKind::AirportHeliport,
                        b"Runway" => FeatureKind::Runway,
                        b"RunwayDirection" => FeatureKind::RunwayDirection,
                        b"DesignatedPoint" => FeatureKind::DesignatedPoint,
                        b"Navaid" => FeatureKind::Navaid,
                        b"Airspace" => FeatureKind::Airspace,
                        _ => continue,
                    };

                    let tag = String::from_utf8_lossy(e.as_ref()).to_string();
                    let end = e.to_end().into_owned();
                    let result = self
                        .reader
                        .read_to_end(end.name())
                        .map_err(Error::from)
                        .and_then(|span| {
                            let content = std::str::from_utf8(
                                &self.data[span.start as usize..span.end as usize],
                            )?;
                            let end_name = end.name();
                            let end_tag = std::str::from_utf8(end_name.as_ref())?;
                            let xml = format!("<{tag}>{content}</{end_tag}>");
                            deserialize_feature(kind, &xml)
                        });

                    return Some(result);
                }
                Ok(Event::Eof) => return None,
                Err(e) => return Some(Err(e.into())),
                _ => continue,
            }
        }
    }
}

// ===========================================================================
// Internal deserialization dispatch
// ===========================================================================

enum FeatureKind {
    AirportHeliport,
    Runway,
    RunwayDirection,
    DesignatedPoint,
    Navaid,
    Airspace,
}

fn deserialize_feature(kind: FeatureKind, xml: &str) -> Result<Feature, Error> {
    match kind {
        FeatureKind::AirportHeliport => {
            let x: XmlAirportHeliport = quick_xml::de::from_str(xml)?;
            Ok(Feature::AirportHeliport(x.into()))
        }
        FeatureKind::Runway => {
            let x: XmlRunway = quick_xml::de::from_str(xml)?;
            Ok(Feature::Runway(x.into()))
        }
        FeatureKind::RunwayDirection => {
            let x: XmlRunwayDirection = quick_xml::de::from_str(xml)?;
            Ok(Feature::RunwayDirection(x.into()))
        }
        FeatureKind::DesignatedPoint => {
            let x: XmlDesignatedPoint = quick_xml::de::from_str(xml)?;
            Ok(Feature::DesignatedPoint(x.into()))
        }
        FeatureKind::Navaid => {
            let x: XmlNavaid = quick_xml::de::from_str(xml)?;
            Ok(Feature::Navaid(x.into()))
        }
        FeatureKind::Airspace => {
            let x: XmlAirspace = quick_xml::de::from_str(xml)?;
            Ok(Feature::Airspace(x.into()))
        }
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Returns the local name portion of a possibly namespace-prefixed XML name.
fn local_name(name: &[u8]) -> &[u8] {
    name.iter()
        .position(|&b| b == b':')
        .map_or(name, |pos| &name[pos + 1..])
}

/// Strips the `uuid.` prefix that `gml:id` attributes carry in AIXM.
fn strip_uuid_prefix(id: &str) -> &str {
    id.strip_prefix("uuid.").unwrap_or(id)
}

/// Strips the `urn:uuid:` prefix from an `xlink:href` value.
fn strip_xlink_prefix(href: &str) -> &str {
    href.strip_prefix("urn:uuid:").unwrap_or(href)
}

/// Parses a GML `pos` value (`"lat lon"`) into a coordinate pair.
fn parse_pos(text: &str) -> Option<(f64, f64)> {
    let mut parts = text.split_whitespace();
    let lat = parts.next()?.parse().ok()?;
    let lon = parts.next()?.parse().ok()?;
    Some((lat, lon))
}

/// Parses a GML `posList` value into a list of coordinate pairs.
fn parse_pos_list(text: &str) -> Vec<(f64, f64)> {
    let values: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    values.chunks_exact(2).map(|c| (c[0], c[1])).collect()
}

// ===========================================================================
// Conversions: XML serde structs → public feature types
// ===========================================================================

impl From<XmlAirportHeliport> for AirportHeliport {
    fn from(x: XmlAirportHeliport) -> Self {
        let ts = x.time_slice.inner;
        let (latitude, longitude) = ts
            .arp
            .and_then(|arp| arp.elevated_point)
            .and_then(|ep| ep.pos.as_deref().and_then(parse_pos))
            .map_or((None, None), |(lat, lon)| (Some(lat), Some(lon)));

        AirportHeliport {
            uuid: x
                .id
                .as_deref()
                .map(strip_uuid_prefix)
                .unwrap_or_default()
                .to_string(),
            designator: ts.designator.unwrap_or_default(),
            name: ts.name.unwrap_or_default(),
            location_indicator_icao: ts.location_indicator_icao,
            iata_designator: ts.iata_designator,
            field_elevation: ts
                .field_elevation
                .as_ref()
                .and_then(|v| v.value.as_deref()?.parse().ok()),
            field_elevation_uom: ts.field_elevation.and_then(|v| v.uom),
            latitude,
            longitude,
        }
    }
}

impl From<XmlRunway> for Runway {
    fn from(x: XmlRunway) -> Self {
        let ts = x.time_slice.inner;
        Runway {
            uuid: x
                .id
                .as_deref()
                .map(strip_uuid_prefix)
                .unwrap_or_default()
                .to_string(),
            designator: ts.designator.unwrap_or_default(),
            nominal_length: ts
                .nominal_length
                .as_ref()
                .and_then(|v| v.value.as_deref()?.parse().ok()),
            length_uom: ts.nominal_length.and_then(|v| v.uom),
            surface_composition: ts
                .surface_properties
                .and_then(|sp| sp.characteristics)
                .and_then(|sc| sc.composition),
            associated_airport_uuid: ts
                .associated_airport_heliport
                .and_then(|r| r.href)
                .map(|h| strip_xlink_prefix(&h).to_string()),
        }
    }
}

impl From<XmlRunwayDirection> for RunwayDirection {
    fn from(x: XmlRunwayDirection) -> Self {
        let ts = x.time_slice.inner;
        RunwayDirection {
            uuid: x
                .id
                .as_deref()
                .map(strip_uuid_prefix)
                .unwrap_or_default()
                .to_string(),
            designator: ts.designator.unwrap_or_default(),
            true_bearing: ts.true_bearing.as_deref().and_then(|s| s.parse().ok()),
            magnetic_bearing: ts.magnetic_bearing.as_deref().and_then(|s| s.parse().ok()),
            used_runway_uuid: ts
                .used_runway
                .and_then(|r| r.href)
                .map(|h| strip_xlink_prefix(&h).to_string()),
        }
    }
}

impl From<XmlDesignatedPoint> for DesignatedPoint {
    fn from(x: XmlDesignatedPoint) -> Self {
        let ts = x.time_slice.inner;
        let (latitude, longitude) = ts
            .location
            .and_then(|loc| loc.elevated_point)
            .and_then(|ep| ep.pos.as_deref().and_then(parse_pos))
            .map_or((None, None), |(lat, lon)| (Some(lat), Some(lon)));

        DesignatedPoint {
            uuid: x
                .id
                .as_deref()
                .map(strip_uuid_prefix)
                .unwrap_or_default()
                .to_string(),
            designator: ts.designator.unwrap_or_default(),
            name: ts.name,
            point_type: ts.point_type,
            latitude,
            longitude,
        }
    }
}

impl From<XmlNavaid> for Navaid {
    fn from(x: XmlNavaid) -> Self {
        let ts = x.time_slice.inner;
        let loc = ts.location.and_then(|l| l.elevated_point);
        let (latitude, longitude) = loc
            .as_ref()
            .and_then(|ep| ep.pos.as_deref().and_then(parse_pos))
            .map_or((None, None), |(lat, lon)| (Some(lat), Some(lon)));
        let elevation = loc
            .and_then(|ep| ep.elevation)
            .and_then(|v| v.value.as_deref()?.parse().ok());

        Navaid {
            uuid: x
                .id
                .as_deref()
                .map(strip_uuid_prefix)
                .unwrap_or_default()
                .to_string(),
            designator: ts.designator.unwrap_or_default(),
            name: ts.name,
            navaid_type: ts.navaid_type,
            latitude,
            longitude,
            elevation,
        }
    }
}

impl From<XmlAirspace> for Airspace {
    fn from(x: XmlAirspace) -> Self {
        let ts = x.time_slice.inner;

        let volume = ts
            .geometry_component
            .and_then(|gc| gc.inner)
            .and_then(|gc| gc.the_airspace_volume)
            .and_then(|tav| tav.volume);

        let volumes = match volume {
            Some(vol) => {
                let polygon = vol
                    .horizontal_projection
                    .and_then(|hp| hp.surface)
                    .and_then(|s| s.patches)
                    .and_then(|p| p.polygon_patch)
                    .and_then(|pp| pp.exterior)
                    .and_then(|ext| ext.ring)
                    .and_then(|r| r.curve_member)
                    .and_then(|cm| cm.curve)
                    .and_then(|c| c.segments)
                    .and_then(|s| s.geodesic_string)
                    .and_then(|gs| gs.pos_list)
                    .map(|pl| parse_pos_list(&pl))
                    .unwrap_or_default();

                vec![AirspaceVolume {
                    upper_limit: vol.upper_limit.as_ref().and_then(|v| v.value.clone()),
                    upper_limit_uom: vol.upper_limit.and_then(|v| v.uom),
                    upper_limit_ref: vol.upper_limit_reference,
                    lower_limit: vol.lower_limit.as_ref().and_then(|v| v.value.clone()),
                    lower_limit_uom: vol.lower_limit.and_then(|v| v.uom),
                    lower_limit_ref: vol.lower_limit_reference,
                    polygon,
                }]
            }
            None => Vec::new(),
        };

        Airspace {
            uuid: x
                .id
                .as_deref()
                .map(strip_uuid_prefix)
                .unwrap_or_default()
                .to_string(),
            airspace_type: ts.airspace_type,
            designator: ts.designator,
            name: ts.name,
            volumes,
        }
    }
}

// ===========================================================================
// Internal XML serde structs
//
// AIXM 5.1 XML uses deeply nested elements: every feature wraps its data in a
// TimeSlice, coordinates live inside GML geometry elements, and cross-references
// use xlink:href attributes.  Serde needs a struct for each nesting level to
// match the XML structure.  These types are private — the From conversions
// above flatten them into the public types in `crate::features`.
// ===========================================================================

// -- Shared GML helpers ----------------------------------------------------

#[derive(Debug, Deserialize)]
struct XmlElevatedPoint {
    #[serde(rename = "pos", default)]
    pos: Option<String>,
    #[serde(rename = "elevation", default)]
    elevation: Option<XmlValWithUom>,
}

#[derive(Debug, Deserialize)]
struct XmlValWithUom {
    #[serde(rename = "@uom", default)]
    uom: Option<String>,
    #[serde(rename = "$text", default)]
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XmlXlinkRef {
    #[serde(rename = "@href", default)]
    href: Option<String>,
}

// -- AirportHeliport -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct XmlAirportHeliport {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: XmlAhpTimeSlice,
}

#[derive(Debug, Deserialize)]
struct XmlAhpTimeSlice {
    #[serde(rename = "AirportHeliportTimeSlice")]
    inner: XmlAhpFields,
}

#[derive(Debug, Deserialize)]
struct XmlAhpFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "locationIndicatorICAO", default)]
    location_indicator_icao: Option<String>,
    #[serde(rename = "designatorIATA", default)]
    iata_designator: Option<String>,
    #[serde(rename = "fieldElevation", default)]
    field_elevation: Option<XmlValWithUom>,
    #[serde(rename = "ARP", default)]
    arp: Option<XmlArp>,
}

#[derive(Debug, Deserialize)]
struct XmlArp {
    #[serde(rename = "ElevatedPoint")]
    elevated_point: Option<XmlElevatedPoint>,
}

// -- Runway ----------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct XmlRunway {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: XmlRwyTimeSlice,
}

#[derive(Debug, Deserialize)]
struct XmlRwyTimeSlice {
    #[serde(rename = "RunwayTimeSlice")]
    inner: XmlRwyFields,
}

#[derive(Debug, Deserialize)]
struct XmlRwyFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(rename = "nominalLength", default)]
    nominal_length: Option<XmlValWithUom>,
    #[serde(rename = "surfaceProperties", default)]
    surface_properties: Option<XmlSurfaceProperties>,
    #[serde(rename = "associatedAirportHeliport", default)]
    associated_airport_heliport: Option<XmlXlinkRef>,
}

#[derive(Debug, Deserialize)]
struct XmlSurfaceProperties {
    #[serde(rename = "SurfaceCharacteristics")]
    characteristics: Option<XmlSurfaceCharacteristics>,
}

#[derive(Debug, Deserialize)]
struct XmlSurfaceCharacteristics {
    #[serde(default)]
    composition: Option<String>,
}

// -- RunwayDirection -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct XmlRunwayDirection {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: XmlRdnTimeSlice,
}

#[derive(Debug, Deserialize)]
struct XmlRdnTimeSlice {
    #[serde(rename = "RunwayDirectionTimeSlice")]
    inner: XmlRdnFields,
}

#[derive(Debug, Deserialize)]
struct XmlRdnFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(rename = "trueBearing", default)]
    true_bearing: Option<String>,
    #[serde(rename = "magneticBearing", default)]
    magnetic_bearing: Option<String>,
    #[serde(rename = "usedRunway", default)]
    used_runway: Option<XmlXlinkRef>,
}

// -- DesignatedPoint -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct XmlDesignatedPoint {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: XmlDpTimeSlice,
}

#[derive(Debug, Deserialize)]
struct XmlDpTimeSlice {
    #[serde(rename = "DesignatedPointTimeSlice")]
    inner: XmlDpFields,
}

#[derive(Debug, Deserialize)]
struct XmlDpFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "type", default)]
    point_type: Option<String>,
    #[serde(default)]
    location: Option<XmlPointLocation>,
}

#[derive(Debug, Deserialize)]
struct XmlPointLocation {
    #[serde(rename = "ElevatedPoint", default)]
    elevated_point: Option<XmlElevatedPoint>,
}

// -- Navaid ----------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct XmlNavaid {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: XmlNavTimeSlice,
}

#[derive(Debug, Deserialize)]
struct XmlNavTimeSlice {
    #[serde(rename = "NavaidTimeSlice")]
    inner: XmlNavFields,
}

#[derive(Debug, Deserialize)]
struct XmlNavFields {
    #[serde(rename = "type", default)]
    navaid_type: Option<String>,
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    location: Option<XmlNavLocation>,
}

#[derive(Debug, Deserialize)]
struct XmlNavLocation {
    #[serde(rename = "ElevatedPoint")]
    elevated_point: Option<XmlElevatedPoint>,
}

// -- Airspace --------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct XmlAirspace {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: XmlArspTimeSlice,
}

#[derive(Debug, Deserialize)]
struct XmlArspTimeSlice {
    #[serde(rename = "AirspaceTimeSlice")]
    inner: XmlArspFields,
}

#[derive(Debug, Deserialize)]
struct XmlArspFields {
    #[serde(rename = "type", default)]
    airspace_type: Option<String>,
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "geometryComponent", default)]
    geometry_component: Option<XmlGeometryComponent>,
}

// Airspace geometry nesting mirrors the AIXM/GML XML structure:
//   geometryComponent > AirspaceGeometryComponent > theAirspaceVolume >
//   AirspaceVolume > horizontalProjection > Surface > patches >
//   PolygonPatch > exterior > Ring > curveMember > Curve > segments >
//   GeodesicString > posList

#[derive(Debug, Deserialize)]
struct XmlGeometryComponent {
    #[serde(rename = "AirspaceGeometryComponent")]
    inner: Option<XmlGeometryComponentInner>,
}

#[derive(Debug, Deserialize)]
struct XmlGeometryComponentInner {
    #[serde(rename = "theAirspaceVolume")]
    the_airspace_volume: Option<XmlTheAirspaceVolume>,
}

#[derive(Debug, Deserialize)]
struct XmlTheAirspaceVolume {
    #[serde(rename = "AirspaceVolume")]
    volume: Option<XmlAirspaceVolume>,
}

#[derive(Debug, Deserialize)]
struct XmlAirspaceVolume {
    #[serde(rename = "upperLimit", default)]
    upper_limit: Option<XmlValWithUom>,
    #[serde(rename = "upperLimitReference", default)]
    upper_limit_reference: Option<String>,
    #[serde(rename = "lowerLimit", default)]
    lower_limit: Option<XmlValWithUom>,
    #[serde(rename = "lowerLimitReference", default)]
    lower_limit_reference: Option<String>,
    #[serde(rename = "horizontalProjection", default)]
    horizontal_projection: Option<XmlHorizontalProjection>,
}

#[derive(Debug, Deserialize)]
struct XmlHorizontalProjection {
    #[serde(rename = "Surface")]
    surface: Option<XmlSurface>,
}

#[derive(Debug, Deserialize)]
struct XmlSurface {
    #[serde(default)]
    patches: Option<XmlPatches>,
}

#[derive(Debug, Deserialize)]
struct XmlPatches {
    #[serde(rename = "PolygonPatch")]
    polygon_patch: Option<XmlPolygonPatch>,
}

#[derive(Debug, Deserialize)]
struct XmlPolygonPatch {
    exterior: Option<XmlExterior>,
}

#[derive(Debug, Deserialize)]
struct XmlExterior {
    #[serde(rename = "Ring")]
    ring: Option<XmlRing>,
}

#[derive(Debug, Deserialize)]
struct XmlRing {
    #[serde(rename = "curveMember")]
    curve_member: Option<XmlCurveMember>,
}

#[derive(Debug, Deserialize)]
struct XmlCurveMember {
    #[serde(rename = "Curve")]
    curve: Option<XmlCurve>,
}

#[derive(Debug, Deserialize)]
struct XmlCurve {
    segments: Option<XmlSegments>,
}

#[derive(Debug, Deserialize)]
struct XmlSegments {
    #[serde(rename = "GeodesicString")]
    geodesic_string: Option<XmlGeodesicString>,
}

#[derive(Debug, Deserialize)]
struct XmlGeodesicString {
    #[serde(rename = "posList")]
    pos_list: Option<String>,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_airport_heliport_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:AirportHeliport gml:id="uuid.dd062d88-3e64-4a5d-bebd-89476db9ebea">
              <gml:identifier codeSpace="urn:uuid:">dd062d88-3e64-4a5d-bebd-89476db9ebea</gml:identifier>
              <aixm:timeSlice>
                <aixm:AirportHeliportTimeSlice gml:id="AHP_EADH">
                  <gml:validTime>
                    <gml:TimePeriod gml:id="vt1">
                      <gml:beginPosition>2009-01-01T00:00:00Z</gml:beginPosition>
                      <gml:endPosition indeterminatePosition="unknown"/>
                    </gml:TimePeriod>
                  </gml:validTime>
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:sequenceNumber>1</aixm:sequenceNumber>
                  <aixm:designator>EADH</aixm:designator>
                  <aixm:name>DONLON/DOWNTOWN HELIPORT</aixm:name>
                  <aixm:locationIndicatorICAO>EADH</aixm:locationIndicatorICAO>
                  <aixm:fieldElevation uom="M">18</aixm:fieldElevation>
                  <aixm:ARP>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep1">
                      <gml:pos>52.288888888888884 -32.035</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:ARP>
                </aixm:AirportHeliportTimeSlice>
              </aixm:timeSlice>
            </aixm:AirportHeliport>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::AirportHeliport(ahp) => {
                assert_eq!(ahp.uuid, "dd062d88-3e64-4a5d-bebd-89476db9ebea");
                assert_eq!(ahp.designator, "EADH");
                assert_eq!(ahp.name, "DONLON/DOWNTOWN HELIPORT");
                assert_eq!(ahp.location_indicator_icao.as_deref(), Some("EADH"));
                assert_eq!(ahp.field_elevation, Some(18.0));
                assert_eq!(ahp.field_elevation_uom.as_deref(), Some("M"));
                assert!((ahp.latitude.unwrap() - 52.2889).abs() < 0.001);
                assert!((ahp.longitude.unwrap() - (-32.035)).abs() < 0.001);
            }
            _ => panic!("expected AirportHeliport"),
        }
    }

    #[test]
    fn parse_runway_and_direction() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
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
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 2);

        match &features[0] {
            Feature::Runway(rwy) => {
                assert_eq!(rwy.designator, "09L/27R");
                assert_eq!(rwy.nominal_length, Some(2800.0));
                assert_eq!(rwy.length_uom.as_deref(), Some("M"));
                assert_eq!(rwy.surface_composition.as_deref(), Some("CONC"));
                assert_eq!(
                    rwy.associated_airport_uuid.as_deref(),
                    Some("1b54b2d6-a5ff-4e57-94c2-f4047a381c64")
                );
            }
            _ => panic!("expected Runway"),
        }

        match &features[1] {
            Feature::RunwayDirection(rdn) => {
                assert_eq!(rdn.designator, "09L");
                assert_eq!(rdn.true_bearing, Some(85.23));
                assert_eq!(
                    rdn.used_runway_uuid.as_deref(),
                    Some("9e51668f-bf8a-4f5b-ba6e-27087972b9b8")
                );
            }
            _ => panic!("expected RunwayDirection"),
        }
    }

    #[test]
    fn parse_designated_point_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:DesignatedPoint gml:id="uuid.abc123">
              <gml:identifier codeSpace="urn:uuid:">abc123</gml:identifier>
              <aixm:timeSlice>
                <aixm:DesignatedPointTimeSlice gml:id="DP1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>ABLAN</aixm:designator>
                  <aixm:name>ABLAN</aixm:name>
                  <aixm:type>ICAO</aixm:type>
                  <aixm:location>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326">
                      <gml:pos>52.123 10.456</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:DesignatedPointTimeSlice>
              </aixm:timeSlice>
            </aixm:DesignatedPoint>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::DesignatedPoint(dp) => {
                assert_eq!(dp.designator, "ABLAN");
                assert_eq!(dp.name.as_deref(), Some("ABLAN"));
                assert_eq!(dp.point_type.as_deref(), Some("ICAO"));
                assert!((dp.latitude.unwrap() - 52.123).abs() < 0.001);
                assert!((dp.longitude.unwrap() - 10.456).abs() < 0.001);
            }
            _ => panic!("expected DesignatedPoint"),
        }
    }

    #[test]
    fn parse_navaid_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:Navaid gml:id="uuid.08a1bbd5-ea70-4fe3-836a-ea9686349495">
              <gml:identifier codeSpace="urn:uuid:">08a1bbd5-ea70-4fe3-836a-ea9686349495</gml:identifier>
              <aixm:timeSlice>
                <aixm:NavaidTimeSlice gml:id="NAV_BOR">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:type>VOR_DME</aixm:type>
                  <aixm:designator>BOR</aixm:designator>
                  <aixm:name>BOORSPIJK</aixm:name>
                  <aixm:location>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep1">
                      <gml:pos>52.368389 -32.375222</gml:pos>
                      <aixm:elevation uom="M">30.0</aixm:elevation>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:NavaidTimeSlice>
              </aixm:timeSlice>
            </aixm:Navaid>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::Navaid(nav) => {
                assert_eq!(nav.designator, "BOR");
                assert_eq!(nav.name.as_deref(), Some("BOORSPIJK"));
                assert_eq!(nav.navaid_type.as_deref(), Some("VOR_DME"));
                assert!((nav.latitude.unwrap() - 52.368389).abs() < 0.0001);
                assert!((nav.longitude.unwrap() - (-32.375222)).abs() < 0.0001);
                assert_eq!(nav.elevation, Some(30.0));
            }
            _ => panic!("expected Navaid"),
        }
    }

    #[test]
    fn parse_airspace_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:Airspace gml:id="uuid.4fd9f4be-8c65-43f6-b083-3ced9a4b2a7f">
              <gml:identifier codeSpace="urn:uuid:">4fd9f4be-8c65-43f6-b083-3ced9a4b2a7f</gml:identifier>
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
                          <aixm:upperLimit uom="FL">195</aixm:upperLimit>
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
                                              <gml:posList>52.0 -32.0 52.5 -32.0 52.5 -31.5 52.0 -31.5 52.0 -32.0</gml:posList>
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

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::Airspace(arsp) => {
                assert_eq!(arsp.airspace_type.as_deref(), Some("CTR"));
                assert_eq!(arsp.designator.as_deref(), Some("EADD CTR"));
                assert_eq!(arsp.name.as_deref(), Some("DONLON CTR"));
                assert_eq!(arsp.volumes.len(), 1);

                let vol = &arsp.volumes[0];
                assert_eq!(vol.upper_limit.as_deref(), Some("195"));
                assert_eq!(vol.upper_limit_uom.as_deref(), Some("FL"));
                assert_eq!(vol.upper_limit_ref.as_deref(), Some("MSL"));
                assert_eq!(vol.lower_limit.as_deref(), Some("GND"));
                assert_eq!(vol.lower_limit_ref.as_deref(), Some("SFC"));
                assert_eq!(vol.polygon.len(), 5);
                assert!((vol.polygon[0].0 - 52.0).abs() < 0.001);
                assert!((vol.polygon[0].1 - (-32.0)).abs() < 0.001);
            }
            _ => panic!("expected Airspace"),
        }
    }

    #[test]
    fn skips_unsupported_features() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:OrganisationAuthority gml:id="uuid.org1">
              <aixm:timeSlice>
                <aixm:OrganisationAuthorityTimeSlice gml:id="OA1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:name>SOME ORG</aixm:name>
                </aixm:OrganisationAuthorityTimeSlice>
              </aixm:timeSlice>
            </aixm:OrganisationAuthority>
          </message:hasMember>
          <message:hasMember>
            <aixm:DesignatedPoint gml:id="uuid.dp1">
              <aixm:timeSlice>
                <aixm:DesignatedPointTimeSlice gml:id="DPT1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>ALPHA</aixm:designator>
                  <aixm:location>
                    <aixm:ElevatedPoint>
                      <gml:pos>50.0 8.0</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:DesignatedPointTimeSlice>
              </aixm:timeSlice>
            </aixm:DesignatedPoint>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        // OrganisationAuthority should be skipped
        assert_eq!(features.len(), 1);
        assert!(matches!(&features[0], Feature::DesignatedPoint(_)));
    }
}
