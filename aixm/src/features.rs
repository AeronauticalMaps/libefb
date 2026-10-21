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

//! AIXM 5.1 feature types yielded by the [`Features`](crate::Features)
//! iterator.
//!
//! Each type is deserialized directly from the AIXM XML using serde. The deeply
//! nested XML structure (TimeSlice wrappers, GML geometry, xlink references)
//! is hidden behind accessor methods that provide flat, easy-to-use values.

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
// Feature enum
// ===========================================================================

/// A parsed AIXM feature yielded by [`Features`](crate::Features).
///
/// Match on the variant to access the specific feature data. Each variant
/// wraps a type that provides accessor methods for the relevant fields.
///
/// # Examples
///
/// ```no_run
/// # let data = vec![];
/// for result in aixm::Features::new(&data) {
///     match result.unwrap() {
///         aixm::Feature::AirportHeliport(ahp) => {
///             println!("{}: {}", ahp.designator(), ahp.name());
///         }
///         aixm::Feature::Navaid(nav) => {
///             println!("Navaid {}", nav.designator());
///         }
///         _ => {}
///     }
/// }
/// ```
pub enum Feature {
    /// An airport or heliport (AIXM `AirportHeliport`).
    AirportHeliport(AirportHeliport),
    /// A physical runway strip (AIXM `Runway`).
    Runway(Runway),
    /// One end of a runway (AIXM `RunwayDirection`).
    RunwayDirection(RunwayDirection),
    /// A named waypoint or fix (AIXM `DesignatedPoint`).
    DesignatedPoint(DesignatedPoint),
    /// A radio navigation aid (AIXM `Navaid`).
    Navaid(Navaid),
    /// An airspace boundary (AIXM `Airspace`).
    Airspace(Airspace),
}

// ===========================================================================
// AirportHeliport
// ===========================================================================

/// An airport or heliport from an AIXM `AirportHeliport` feature.
///
/// Provides the aerodrome reference point (ARP), elevation, and identification
/// codes to place the airport on a map and look it up by ICAO or IATA code.
///
/// # Examples
///
/// ```no_run
/// # let ahp: &aixm::AirportHeliport = unimplemented!();
/// // Use the ICAO indicator for lookup, falling back to the designator.
/// let ident = ahp.location_indicator_icao()
///     .unwrap_or(ahp.designator());
/// ```
#[derive(Debug, Deserialize)]
pub struct AirportHeliport {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: AhpTimeSlice,
}

impl AirportHeliport {
    /// Returns the UUID identifier (e.g. `"1b54b2d6-…"`).
    pub fn uuid(&self) -> &str {
        self.id
            .as_deref()
            .map(strip_uuid_prefix)
            .unwrap_or_default()
    }

    /// Returns the ICAO designator (e.g. `"EADD"`).
    pub fn designator(&self) -> &str {
        self.time_slice.inner.designator.as_deref().unwrap_or_default()
    }

    /// Returns the human-readable name (e.g. `"DONLON/INTL"`).
    pub fn name(&self) -> &str {
        self.time_slice.inner.name.as_deref().unwrap_or_default()
    }

    /// Returns the ICAO location indicator, if assigned (e.g. `"EADD"`).
    pub fn location_indicator_icao(&self) -> Option<&str> {
        self.time_slice.inner.location_indicator_icao.as_deref()
    }

    /// Returns the IATA designator, if assigned (e.g. `"DON"`).
    pub fn iata_designator(&self) -> Option<&str> {
        self.time_slice.inner.iata_designator.as_deref()
    }

    /// Returns the field elevation value and unit of measurement.
    ///
    /// The unit is typically `"M"` (meters) or `"FT"` (feet).
    pub fn field_elevation(&self) -> (Option<f64>, Option<&str>) {
        let elev = self.time_slice.inner.field_elevation.as_ref();
        let value = elev.and_then(|v| v.value.as_deref()?.parse().ok());
        let uom = elev.and_then(|v| v.uom.as_deref());
        (value, uom)
    }

    /// Returns the aerodrome reference point as (latitude, longitude) in
    /// WGS-84 decimal degrees.
    pub fn coordinate(&self) -> Option<(f64, f64)> {
        self.time_slice
            .inner
            .arp
            .as_ref()
            .and_then(|arp| arp.elevated_point.as_ref())
            .and_then(|ep| ep.pos.as_deref().and_then(parse_pos))
    }
}

// ===========================================================================
// Runway
// ===========================================================================

/// A physical runway strip from an AIXM `Runway` feature.
///
/// Represents the full runway (e.g. `"09L/27R"`), not an individual direction.
/// Use [`associated_airport_uuid`](Self::associated_airport_uuid) to link this
/// runway to its parent airport.
///
/// # Examples
///
/// ```no_run
/// # let rwy: &aixm::Runway = unimplemented!();
/// if let Some(uuid) = rwy.associated_airport_uuid() {
///     println!("Runway {} belongs to airport {uuid}", rwy.designator());
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Runway {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: RwyTimeSlice,
}

impl Runway {
    /// Returns the UUID identifier.
    pub fn uuid(&self) -> &str {
        self.id
            .as_deref()
            .map(strip_uuid_prefix)
            .unwrap_or_default()
    }

    /// Returns the designator covering both ends (e.g. `"09L/27R"`).
    pub fn designator(&self) -> &str {
        self.time_slice.inner.designator.as_deref().unwrap_or_default()
    }

    /// Returns the nominal length value and unit of measurement.
    ///
    /// The unit is typically `"M"` (meters) or `"FT"` (feet).
    pub fn nominal_length(&self) -> (Option<f64>, Option<&str>) {
        let len = self.time_slice.inner.nominal_length.as_ref();
        let value = len.and_then(|v| v.value.as_deref()?.parse().ok());
        let uom = len.and_then(|v| v.uom.as_deref());
        (value, uom)
    }

    /// Returns the surface composition code (e.g. `"ASPH"`, `"CONC"`,
    /// `"GRASS"`).
    pub fn surface_composition(&self) -> Option<&str> {
        self.time_slice
            .inner
            .surface_properties
            .as_ref()
            .and_then(|sp| sp.characteristics.as_ref())
            .and_then(|sc| sc.composition.as_deref())
    }

    /// Returns the UUID of the associated airport (from `xlink:href`).
    pub fn associated_airport_uuid(&self) -> Option<&str> {
        self.time_slice
            .inner
            .associated_airport_heliport
            .as_ref()
            .and_then(|r| r.href.as_deref())
            .map(strip_xlink_prefix)
    }
}

// ===========================================================================
// RunwayDirection
// ===========================================================================

/// One end of a physical runway, from an AIXM `RunwayDirection` feature.
///
/// Links to the parent [`Runway`] via
/// [`used_runway_uuid`](Self::used_runway_uuid). The bearing values indicate
/// the heading from this direction.
///
/// # Examples
///
/// ```no_run
/// # let rdn: &aixm::RunwayDirection = unimplemented!();
/// let heading = rdn.true_bearing()
///     .or(rdn.magnetic_bearing())
///     .unwrap_or(0.0);
/// println!("Runway {} heading {heading:.0}°", rdn.designator());
/// ```
#[derive(Debug, Deserialize)]
pub struct RunwayDirection {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: RdnTimeSlice,
}

impl RunwayDirection {
    /// Returns the UUID identifier.
    pub fn uuid(&self) -> &str {
        self.id
            .as_deref()
            .map(strip_uuid_prefix)
            .unwrap_or_default()
    }

    /// Returns the designator for this end (e.g. `"09L"`).
    pub fn designator(&self) -> &str {
        self.time_slice.inner.designator.as_deref().unwrap_or_default()
    }

    /// Returns the true bearing in degrees.
    pub fn true_bearing(&self) -> Option<f64> {
        self.time_slice
            .inner
            .true_bearing
            .as_deref()
            .and_then(|s| s.parse().ok())
    }

    /// Returns the magnetic bearing in degrees.
    pub fn magnetic_bearing(&self) -> Option<f64> {
        self.time_slice
            .inner
            .magnetic_bearing
            .as_deref()
            .and_then(|s| s.parse().ok())
    }

    /// Returns the UUID of the parent [`Runway`] (from `xlink:href`).
    pub fn used_runway_uuid(&self) -> Option<&str> {
        self.time_slice
            .inner
            .used_runway
            .as_ref()
            .and_then(|r| r.href.as_deref())
            .map(strip_xlink_prefix)
    }
}

// ===========================================================================
// DesignatedPoint
// ===========================================================================

/// A named waypoint or fix from an AIXM `DesignatedPoint` feature.
///
/// Designated points serve as route waypoints in instrument procedures.
///
/// # Examples
///
/// ```no_run
/// # let dp: &aixm::DesignatedPoint = unimplemented!();
/// if let Some((lat, lon)) = dp.coordinate() {
///     println!("Fix {} at ({lat}, {lon})", dp.designator());
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct DesignatedPoint {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: DpTimeSlice,
}

impl DesignatedPoint {
    /// Returns the UUID identifier.
    pub fn uuid(&self) -> &str {
        self.id
            .as_deref()
            .map(strip_uuid_prefix)
            .unwrap_or_default()
    }

    /// Returns the fix identifier (e.g. `"ABLAN"`).
    pub fn designator(&self) -> &str {
        self.time_slice.inner.designator.as_deref().unwrap_or_default()
    }

    /// Returns the human-readable name.
    pub fn name(&self) -> Option<&str> {
        self.time_slice.inner.name.as_deref()
    }

    /// Returns the type code (e.g. `"ICAO"`, `"COORD"`).
    pub fn point_type(&self) -> Option<&str> {
        self.time_slice.inner.point_type.as_deref()
    }

    /// Returns the position as (latitude, longitude) in WGS-84 decimal
    /// degrees.
    pub fn coordinate(&self) -> Option<(f64, f64)> {
        self.time_slice
            .inner
            .location
            .as_ref()
            .and_then(|loc| loc.elevated_point.as_ref())
            .and_then(|ep| ep.pos.as_deref().and_then(parse_pos))
    }
}

// ===========================================================================
// Navaid
// ===========================================================================

/// A radio navigation aid from an AIXM `Navaid` feature.
///
/// Covers VOR, DME, NDB, TACAN, and combined types. Used as route waypoints
/// and for instrument approaches.
///
/// # Examples
///
/// ```no_run
/// # let nav: &aixm::Navaid = unimplemented!();
/// match nav.navaid_type() {
///     Some("VOR_DME") => println!("{} is a VOR/DME", nav.designator()),
///     Some("NDB") => println!("{} is an NDB", nav.designator()),
///     _ => {}
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Navaid {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: NavTimeSlice,
}

impl Navaid {
    /// Returns the UUID identifier.
    pub fn uuid(&self) -> &str {
        self.id
            .as_deref()
            .map(strip_uuid_prefix)
            .unwrap_or_default()
    }

    /// Returns the identifier (e.g. `"BOR"`).
    pub fn designator(&self) -> &str {
        self.time_slice.inner.designator.as_deref().unwrap_or_default()
    }

    /// Returns the human-readable name (e.g. `"BOORSPIJK"`).
    pub fn name(&self) -> Option<&str> {
        self.time_slice.inner.name.as_deref()
    }

    /// Returns the type code (e.g. `"VOR"`, `"VOR_DME"`, `"NDB"`, `"TACAN"`).
    pub fn navaid_type(&self) -> Option<&str> {
        self.time_slice.inner.navaid_type.as_deref()
    }

    /// Returns the position as (latitude, longitude) in WGS-84 decimal
    /// degrees.
    pub fn coordinate(&self) -> Option<(f64, f64)> {
        self.elevated_point()
            .and_then(|ep| ep.pos.as_deref().and_then(parse_pos))
    }

    /// Returns the station elevation.
    pub fn elevation(&self) -> Option<f64> {
        self.elevated_point()
            .and_then(|ep| ep.elevation.as_ref())
            .and_then(|v| v.value.as_deref()?.parse().ok())
    }

    fn elevated_point(&self) -> Option<&ElevatedPoint> {
        self.time_slice
            .inner
            .location
            .as_ref()
            .and_then(|l| l.elevated_point.as_ref())
    }
}

// ===========================================================================
// Airspace
// ===========================================================================

/// An airspace boundary from an AIXM `Airspace` feature.
///
/// Contains the airspace classification, vertical limits, and horizontal
/// geometry to determine whether a position falls inside the airspace.
///
/// # Examples
///
/// ```no_run
/// # let arsp: &aixm::Airspace = unimplemented!();
/// if arsp.airspace_type() == Some("CTR") {
///     println!("Control zone: {}", arsp.name().unwrap_or("unnamed"));
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Airspace {
    #[serde(rename = "@id", default)]
    id: Option<String>,
    #[serde(rename = "timeSlice")]
    time_slice: ArspTimeSlice,
}

impl Airspace {
    /// Returns the UUID identifier.
    pub fn uuid(&self) -> &str {
        self.id
            .as_deref()
            .map(strip_uuid_prefix)
            .unwrap_or_default()
    }

    /// Returns the airspace type code (e.g. `"CTR"`, `"TMA"`, `"CTA"`, `"D"`,
    /// `"R"`, `"P"`).
    pub fn airspace_type(&self) -> Option<&str> {
        self.time_slice.inner.airspace_type.as_deref()
    }

    /// Returns the designator (e.g. `"EADD CTR"`).
    pub fn designator(&self) -> Option<&str> {
        self.time_slice.inner.designator.as_deref()
    }

    /// Returns the human-readable name (e.g. `"DONLON CTR"`).
    pub fn name(&self) -> Option<&str> {
        self.time_slice.inner.name.as_deref()
    }

    /// Returns the airspace geometry volumes with vertical limits and
    /// horizontal boundaries.
    pub fn volumes(&self) -> Vec<AirspaceVolume> {
        let volume = self
            .time_slice
            .inner
            .geometry_component
            .as_ref()
            .and_then(|gc| gc.inner.as_ref())
            .and_then(|gc| gc.the_airspace_volume.as_ref())
            .and_then(|tav| tav.volume.as_ref());

        let Some(vol) = volume else {
            return Vec::new();
        };

        let polygon = vol
            .horizontal_projection
            .as_ref()
            .and_then(|hp| hp.surface.as_ref())
            .and_then(|s| s.patches.as_ref())
            .and_then(|p| p.polygon_patch.as_ref())
            .and_then(|pp| pp.exterior.as_ref())
            .and_then(|ext| ext.ring.as_ref())
            .and_then(|r| r.curve_member.as_ref())
            .and_then(|cm| cm.curve.as_ref())
            .and_then(|c| c.segments.as_ref())
            .and_then(|s| s.geodesic_string.as_ref())
            .and_then(|gs| gs.pos_list.as_deref())
            .map(parse_pos_list)
            .unwrap_or_default();

        vec![AirspaceVolume {
            upper_limit: vol.upper_limit.as_ref().and_then(|v| v.value.clone()),
            upper_limit_uom: vol.upper_limit.as_ref().and_then(|v| v.uom.clone()),
            upper_limit_ref: vol.upper_limit_reference.clone(),
            lower_limit: vol.lower_limit.as_ref().and_then(|v| v.value.clone()),
            lower_limit_uom: vol.lower_limit.as_ref().and_then(|v| v.uom.clone()),
            lower_limit_ref: vol.lower_limit_reference.clone(),
            polygon,
        }]
    }
}

/// A single airspace volume with vertical limits and a horizontal polygon.
///
/// Returned by [`Airspace::volumes`].
///
/// # Examples
///
/// ```no_run
/// # let vol: &aixm::AirspaceVolume = unimplemented!();
/// for &(lat, lon) in &vol.polygon {
///     println!("  vertex: {lat}, {lon}");
/// }
/// ```
#[derive(Clone, Debug)]
pub struct AirspaceVolume {
    /// Upper vertical limit value (e.g. `"195"`, `"UNL"`).
    pub upper_limit: Option<String>,
    /// Upper limit unit (e.g. `"FL"`, `"FT"`, `"M"`).
    pub upper_limit_uom: Option<String>,
    /// Upper limit datum reference (e.g. `"MSL"`, `"SFC"`).
    pub upper_limit_ref: Option<String>,
    /// Lower vertical limit value (e.g. `"GND"`, `"0"`).
    pub lower_limit: Option<String>,
    /// Lower limit unit.
    pub lower_limit_uom: Option<String>,
    /// Lower limit datum reference.
    pub lower_limit_ref: Option<String>,
    /// Horizontal boundary as (latitude, longitude) pairs in WGS-84 decimal
    /// degrees. The first and last point are typically identical to close the
    /// polygon.
    pub polygon: Vec<(f64, f64)>,
}

// ===========================================================================
// Internal serde structs
//
// AIXM 5.1 XML nests data inside TimeSlice wrappers, GML geometry elements,
// and xlink references.  Serde requires a struct for each nesting level to
// match the XML element hierarchy.  These types are private — the accessor
// methods above hide them from callers.
// ===========================================================================

#[derive(Debug, Deserialize)]
struct ElevatedPoint {
    #[serde(rename = "pos", default)]
    pos: Option<String>,
    #[serde(rename = "elevation", default)]
    elevation: Option<ValWithUom>,
}

#[derive(Debug, Deserialize)]
struct ValWithUom {
    #[serde(rename = "@uom", default)]
    uom: Option<String>,
    #[serde(rename = "$text", default)]
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XlinkRef {
    #[serde(rename = "@href", default)]
    href: Option<String>,
}

// -- AirportHeliport -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct AhpTimeSlice {
    #[serde(rename = "AirportHeliportTimeSlice")]
    inner: AhpFields,
}

#[derive(Debug, Deserialize)]
struct AhpFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "locationIndicatorICAO", default)]
    location_indicator_icao: Option<String>,
    #[serde(rename = "designatorIATA", default)]
    iata_designator: Option<String>,
    #[serde(rename = "fieldElevation", default)]
    field_elevation: Option<ValWithUom>,
    #[serde(rename = "ARP", default)]
    arp: Option<Arp>,
}

#[derive(Debug, Deserialize)]
struct Arp {
    #[serde(rename = "ElevatedPoint")]
    elevated_point: Option<ElevatedPoint>,
}

// -- Runway ----------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RwyTimeSlice {
    #[serde(rename = "RunwayTimeSlice")]
    inner: RwyFields,
}

#[derive(Debug, Deserialize)]
struct RwyFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(rename = "nominalLength", default)]
    nominal_length: Option<ValWithUom>,
    #[serde(rename = "surfaceProperties", default)]
    surface_properties: Option<SurfaceProperties>,
    #[serde(rename = "associatedAirportHeliport", default)]
    associated_airport_heliport: Option<XlinkRef>,
}

#[derive(Debug, Deserialize)]
struct SurfaceProperties {
    #[serde(rename = "SurfaceCharacteristics")]
    characteristics: Option<SurfaceCharacteristics>,
}

#[derive(Debug, Deserialize)]
struct SurfaceCharacteristics {
    #[serde(default)]
    composition: Option<String>,
}

// -- RunwayDirection -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RdnTimeSlice {
    #[serde(rename = "RunwayDirectionTimeSlice")]
    inner: RdnFields,
}

#[derive(Debug, Deserialize)]
struct RdnFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(rename = "trueBearing", default)]
    true_bearing: Option<String>,
    #[serde(rename = "magneticBearing", default)]
    magnetic_bearing: Option<String>,
    #[serde(rename = "usedRunway", default)]
    used_runway: Option<XlinkRef>,
}

// -- DesignatedPoint -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct DpTimeSlice {
    #[serde(rename = "DesignatedPointTimeSlice")]
    inner: DpFields,
}

#[derive(Debug, Deserialize)]
struct DpFields {
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "type", default)]
    point_type: Option<String>,
    #[serde(default)]
    location: Option<PointLocation>,
}

#[derive(Debug, Deserialize)]
struct PointLocation {
    #[serde(rename = "ElevatedPoint", default)]
    elevated_point: Option<ElevatedPoint>,
}

// -- Navaid ----------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct NavTimeSlice {
    #[serde(rename = "NavaidTimeSlice")]
    inner: NavFields,
}

#[derive(Debug, Deserialize)]
struct NavFields {
    #[serde(rename = "type", default)]
    navaid_type: Option<String>,
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    location: Option<NavLocation>,
}

#[derive(Debug, Deserialize)]
struct NavLocation {
    #[serde(rename = "ElevatedPoint")]
    elevated_point: Option<ElevatedPoint>,
}

// -- Airspace --------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ArspTimeSlice {
    #[serde(rename = "AirspaceTimeSlice")]
    inner: ArspFields,
}

#[derive(Debug, Deserialize)]
struct ArspFields {
    #[serde(rename = "type", default)]
    airspace_type: Option<String>,
    #[serde(default)]
    designator: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "geometryComponent", default)]
    geometry_component: Option<GeometryComponent>,
}

// Airspace geometry nesting mirrors the AIXM/GML XML structure:
//   geometryComponent > AirspaceGeometryComponent > theAirspaceVolume >
//   AirspaceVolume > horizontalProjection > Surface > patches >
//   PolygonPatch > exterior > Ring > curveMember > Curve > segments >
//   GeodesicString > posList

#[derive(Debug, Deserialize)]
struct GeometryComponent {
    #[serde(rename = "AirspaceGeometryComponent")]
    inner: Option<GeometryComponentInner>,
}

#[derive(Debug, Deserialize)]
struct GeometryComponentInner {
    #[serde(rename = "theAirspaceVolume")]
    the_airspace_volume: Option<TheAirspaceVolume>,
}

#[derive(Debug, Deserialize)]
struct TheAirspaceVolume {
    #[serde(rename = "AirspaceVolume")]
    volume: Option<XmlAirspaceVolume>,
}

#[derive(Debug, Deserialize)]
struct XmlAirspaceVolume {
    #[serde(rename = "upperLimit", default)]
    upper_limit: Option<ValWithUom>,
    #[serde(rename = "upperLimitReference", default)]
    upper_limit_reference: Option<String>,
    #[serde(rename = "lowerLimit", default)]
    lower_limit: Option<ValWithUom>,
    #[serde(rename = "lowerLimitReference", default)]
    lower_limit_reference: Option<String>,
    #[serde(rename = "horizontalProjection", default)]
    horizontal_projection: Option<HorizontalProjection>,
}

#[derive(Debug, Deserialize)]
struct HorizontalProjection {
    #[serde(rename = "Surface")]
    surface: Option<Surface>,
}

#[derive(Debug, Deserialize)]
struct Surface {
    #[serde(default)]
    patches: Option<Patches>,
}

#[derive(Debug, Deserialize)]
struct Patches {
    #[serde(rename = "PolygonPatch")]
    polygon_patch: Option<PolygonPatch>,
}

#[derive(Debug, Deserialize)]
struct PolygonPatch {
    exterior: Option<Exterior>,
}

#[derive(Debug, Deserialize)]
struct Exterior {
    #[serde(rename = "Ring")]
    ring: Option<Ring>,
}

#[derive(Debug, Deserialize)]
struct Ring {
    #[serde(rename = "curveMember")]
    curve_member: Option<CurveMember>,
}

#[derive(Debug, Deserialize)]
struct CurveMember {
    #[serde(rename = "Curve")]
    curve: Option<Curve>,
}

#[derive(Debug, Deserialize)]
struct Curve {
    segments: Option<Segments>,
}

#[derive(Debug, Deserialize)]
struct Segments {
    #[serde(rename = "GeodesicString")]
    geodesic_string: Option<GeodesicString>,
}

#[derive(Debug, Deserialize)]
struct GeodesicString {
    #[serde(rename = "posList")]
    pos_list: Option<String>,
}
