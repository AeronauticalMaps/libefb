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

//! Serde-deserializable structs that mirror the AIXM 5.1 XML structure.
//!
//! These are internal types used by the parser. They map directly to the XML
//! nesting with namespace-qualified element names, then get converted into the
//! flat public [`Feature`](crate::Feature) types.

#![allow(dead_code)]

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Shared GML types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct ElevatedPoint {
    #[serde(rename = "pos", default)]
    pub pos: Option<String>,
    #[serde(rename = "elevation", default)]
    pub elevation: Option<ValWithUom>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ValWithUom {
    #[serde(rename = "@uom", default)]
    pub uom: Option<String>,
    #[serde(rename = "$text", default)]
    pub value: Option<String>,
}

/// An xlink reference element (e.g. `<aixm:associatedAirportHeliport xlink:href="..."/>`).
#[derive(Debug, Deserialize)]
pub(crate) struct XlinkRef {
    #[serde(rename = "@href", default)]
    pub href: Option<String>,
}

// ---------------------------------------------------------------------------
// AirportHeliport
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct AirportHeliportXml {
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    #[serde(rename = "timeSlice")]
    pub time_slice: AhpTimeSliceWrapper,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AhpTimeSliceWrapper {
    #[serde(rename = "AirportHeliportTimeSlice")]
    pub inner: AhpTimeSlice,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AhpTimeSlice {
    #[serde(default)]
    pub interpretation: Option<String>,
    #[serde(default)]
    pub designator: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "locationIndicatorICAO", default)]
    pub location_indicator_icao: Option<String>,
    #[serde(rename = "designatorIATA", default)]
    pub iata_designator: Option<String>,
    #[serde(rename = "fieldElevation", default)]
    pub field_elevation: Option<ValWithUom>,
    #[serde(rename = "ARP", default)]
    pub arp: Option<Arp>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Arp {
    #[serde(rename = "ElevatedPoint")]
    pub elevated_point: Option<ElevatedPoint>,
}

// ---------------------------------------------------------------------------
// Runway
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct RunwayXml {
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    #[serde(rename = "timeSlice")]
    pub time_slice: RwyTimeSliceWrapper,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RwyTimeSliceWrapper {
    #[serde(rename = "RunwayTimeSlice")]
    pub inner: RwyTimeSlice,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RwyTimeSlice {
    #[serde(default)]
    pub interpretation: Option<String>,
    #[serde(default)]
    pub designator: Option<String>,
    #[serde(rename = "nominalLength", default)]
    pub nominal_length: Option<ValWithUom>,
    #[serde(rename = "surfaceProperties", default)]
    pub surface_properties: Option<SurfaceProperties>,
    #[serde(rename = "associatedAirportHeliport", default)]
    pub associated_airport_heliport: Option<XlinkRef>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SurfaceProperties {
    #[serde(rename = "SurfaceCharacteristics")]
    pub characteristics: Option<SurfaceCharacteristics>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SurfaceCharacteristics {
    #[serde(default)]
    pub composition: Option<String>,
}

// ---------------------------------------------------------------------------
// RunwayDirection
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct RunwayDirectionXml {
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    #[serde(rename = "timeSlice")]
    pub time_slice: RdnTimeSliceWrapper,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RdnTimeSliceWrapper {
    #[serde(rename = "RunwayDirectionTimeSlice")]
    pub inner: RdnTimeSlice,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RdnTimeSlice {
    #[serde(default)]
    pub interpretation: Option<String>,
    #[serde(default)]
    pub designator: Option<String>,
    #[serde(rename = "trueBearing", default)]
    pub true_bearing: Option<String>,
    #[serde(rename = "magneticBearing", default)]
    pub magnetic_bearing: Option<String>,
    #[serde(rename = "usedRunway", default)]
    pub used_runway: Option<XlinkRef>,
}

// ---------------------------------------------------------------------------
// DesignatedPoint
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct DesignatedPointXml {
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    #[serde(rename = "timeSlice")]
    pub time_slice: DpTimeSliceWrapper,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DpTimeSliceWrapper {
    #[serde(rename = "DesignatedPointTimeSlice")]
    pub inner: DpTimeSlice,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DpTimeSlice {
    #[serde(default)]
    pub interpretation: Option<String>,
    #[serde(default)]
    pub designator: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "type", default)]
    pub point_type: Option<String>,
    #[serde(default)]
    pub location: Option<PointLocation>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PointLocation {
    #[serde(rename = "ElevatedPoint", default)]
    pub elevated_point: Option<ElevatedPoint>,
}

// ---------------------------------------------------------------------------
// Navaid
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct NavaidXml {
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    #[serde(rename = "timeSlice")]
    pub time_slice: NavTimeSliceWrapper,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NavTimeSliceWrapper {
    #[serde(rename = "NavaidTimeSlice")]
    pub inner: NavTimeSlice,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NavTimeSlice {
    #[serde(default)]
    pub interpretation: Option<String>,
    #[serde(rename = "type", default)]
    pub navaid_type: Option<String>,
    #[serde(default)]
    pub designator: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub location: Option<NavLocation>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NavLocation {
    #[serde(rename = "ElevatedPoint")]
    pub elevated_point: Option<ElevatedPoint>,
}

// ---------------------------------------------------------------------------
// Airspace
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct AirspaceXml {
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    #[serde(rename = "timeSlice")]
    pub time_slice: ArspTimeSliceWrapper,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArspTimeSliceWrapper {
    #[serde(rename = "AirspaceTimeSlice")]
    pub inner: ArspTimeSlice,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArspTimeSlice {
    #[serde(default)]
    pub interpretation: Option<String>,
    #[serde(rename = "type", default)]
    pub airspace_type: Option<String>,
    #[serde(default)]
    pub designator: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "geometryComponent", default)]
    pub geometry_component: Option<AirspaceGeometryComponent>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AirspaceGeometryComponent {
    #[serde(rename = "AirspaceGeometryComponent")]
    pub inner: Option<AirspaceGeometryComponentInner>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AirspaceGeometryComponentInner {
    #[serde(rename = "theAirspaceVolume")]
    pub the_airspace_volume: Option<TheAirspaceVolume>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TheAirspaceVolume {
    #[serde(rename = "AirspaceVolume")]
    pub volume: Option<AirspaceVolumeXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AirspaceVolumeXml {
    #[serde(rename = "upperLimit", default)]
    pub upper_limit: Option<ValWithUom>,
    #[serde(rename = "upperLimitReference", default)]
    pub upper_limit_reference: Option<String>,
    #[serde(rename = "lowerLimit", default)]
    pub lower_limit: Option<ValWithUom>,
    #[serde(rename = "lowerLimitReference", default)]
    pub lower_limit_reference: Option<String>,
    #[serde(rename = "horizontalProjection", default)]
    pub horizontal_projection: Option<HorizontalProjection>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct HorizontalProjection {
    #[serde(rename = "Surface")]
    pub surface: Option<Surface>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Surface {
    #[serde(default)]
    pub patches: Option<Patches>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Patches {
    #[serde(rename = "PolygonPatch")]
    pub polygon_patch: Option<PolygonPatch>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PolygonPatch {
    pub exterior: Option<Exterior>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Exterior {
    #[serde(rename = "Ring")]
    pub ring: Option<Ring>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Ring {
    #[serde(rename = "curveMember")]
    pub curve_member: Option<CurveMember>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CurveMember {
    #[serde(rename = "Curve")]
    pub curve: Option<Curve>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Curve {
    pub segments: Option<Segments>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Segments {
    #[serde(rename = "GeodesicString")]
    pub geodesic_string: Option<GeodesicString>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GeodesicString {
    #[serde(rename = "posList")]
    pub pos_list: Option<String>,
}
