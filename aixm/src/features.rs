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

//! Public AIXM feature types yielded by the [`Features`](crate::Features)
//! iterator.
//!
//! Each struct is a flat representation of one AIXM 5.1 feature with all
//! relevant fields already extracted and parsed. Coordinates are WGS-84
//! decimal degrees, numeric values are `f64`, and UUID references are plain
//! strings ready for cross-reference lookups.
//!
//! The AIXM XML format nests data deeply (TimeSlice wrappers, GML geometry
//! elements, xlink references). The parser handles that nesting internally
//! using serde-mapped structs and converts each feature into these flat types
//! so callers never deal with XML structure.

/// A parsed AIXM feature yielded by [`Features`](crate::Features).
///
/// Match on the variant to access the specific feature data.
///
/// # Examples
///
/// ```no_run
/// # let data = vec![];
/// for result in aixm::Features::new(&data) {
///     match result.unwrap() {
///         aixm::Feature::AirportHeliport(ahp) => {
///             println!("{}: {}", ahp.designator, ahp.name);
///         }
///         aixm::Feature::Navaid(nav) => {
///             println!("Navaid {} at ({}, {})",
///                 nav.designator,
///                 nav.latitude.unwrap_or(0.0),
///                 nav.longitude.unwrap_or(0.0),
///             );
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

/// An airport or heliport parsed from an AIXM `AirportHeliport` feature.
///
/// Contains the aerodrome reference point (ARP) coordinates, elevation, and
/// identification codes needed to place the airport on a map and look it up
/// by ICAO or IATA code.
///
/// # Examples
///
/// ```no_run
/// # let ahp: aixm::AirportHeliport = unimplemented!();
/// // Use the ICAO indicator for lookup, falling back to the designator
/// let ident = ahp.location_indicator_icao
///     .as_deref()
///     .unwrap_or(&ahp.designator);
/// ```
#[derive(Clone, Debug)]
pub struct AirportHeliport {
    /// UUID from the `gml:id` attribute (e.g.
    /// `"1b54b2d6-a5ff-4e57-94c2-f4047a381c64"`).
    pub uuid: String,
    /// ICAO designator (e.g. `"EADD"`).
    pub designator: String,
    /// Human-readable name (e.g. `"DONLON/INTL"`).
    pub name: String,
    /// ICAO location indicator, if assigned (e.g. `"EADD"`).
    pub location_indicator_icao: Option<String>,
    /// IATA designator, if assigned (e.g. `"DON"`).
    pub iata_designator: Option<String>,
    /// Field elevation value.
    pub field_elevation: Option<f64>,
    /// Unit of measurement for [`field_elevation`](Self::field_elevation)
    /// (e.g. `"M"`, `"FT"`).
    pub field_elevation_uom: Option<String>,
    /// ARP latitude in WGS-84 decimal degrees.
    pub latitude: Option<f64>,
    /// ARP longitude in WGS-84 decimal degrees.
    pub longitude: Option<f64>,
}

/// A physical runway strip parsed from an AIXM `Runway` feature.
///
/// Represents the full runway (e.g. `"09L/27R"`), not an individual direction.
/// The [`associated_airport_uuid`](Self::associated_airport_uuid) links this
/// runway to its parent airport via UUID cross-reference.
///
/// # Examples
///
/// ```no_run
/// # let rwy: aixm::Runway = unimplemented!();
/// if let Some(uuid) = &rwy.associated_airport_uuid {
///     // Look up which airport this runway belongs to
///     println!("Runway {} belongs to airport {uuid}", rwy.designator);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Runway {
    /// UUID from the `gml:id` attribute.
    pub uuid: String,
    /// Designator covering both ends (e.g. `"09L/27R"`).
    pub designator: String,
    /// Nominal length value.
    pub nominal_length: Option<f64>,
    /// Unit of measurement for [`nominal_length`](Self::nominal_length)
    /// (e.g. `"M"`, `"FT"`).
    pub length_uom: Option<String>,
    /// Surface composition code (e.g. `"ASPH"`, `"CONC"`, `"GRASS"`).
    pub surface_composition: Option<String>,
    /// UUID of the associated airport (from `xlink:href`).
    pub associated_airport_uuid: Option<String>,
}

/// One end of a physical runway, parsed from an AIXM `RunwayDirection`
/// feature.
///
/// Links back to the parent [`Runway`] via
/// [`used_runway_uuid`](Self::used_runway_uuid). The bearing values indicate
/// the runway heading from this direction.
///
/// # Examples
///
/// ```no_run
/// # let rdn: aixm::RunwayDirection = unimplemented!();
/// let heading = rdn.true_bearing
///     .or(rdn.magnetic_bearing)
///     .unwrap_or(0.0);
/// println!("Runway {} heading {heading:.0}°", rdn.designator);
/// ```
#[derive(Clone, Debug)]
pub struct RunwayDirection {
    /// UUID from the `gml:id` attribute.
    pub uuid: String,
    /// Designator for this end (e.g. `"09L"`).
    pub designator: String,
    /// True bearing in degrees.
    pub true_bearing: Option<f64>,
    /// Magnetic bearing in degrees.
    pub magnetic_bearing: Option<f64>,
    /// UUID of the parent [`Runway`] (from `xlink:href`).
    pub used_runway_uuid: Option<String>,
}

/// A named waypoint or fix, parsed from an AIXM `DesignatedPoint` feature.
///
/// Designated points are used as route waypoints in instrument procedures.
///
/// # Examples
///
/// ```no_run
/// # let dp: aixm::DesignatedPoint = unimplemented!();
/// println!("Fix {} at ({}, {})",
///     dp.designator,
///     dp.latitude.unwrap_or(0.0),
///     dp.longitude.unwrap_or(0.0),
/// );
/// ```
#[derive(Clone, Debug)]
pub struct DesignatedPoint {
    /// UUID from the `gml:id` attribute.
    pub uuid: String,
    /// Fix identifier (e.g. `"ABLAN"`).
    pub designator: String,
    /// Human-readable name.
    pub name: Option<String>,
    /// Type code (e.g. `"ICAO"`, `"COORD"`).
    pub point_type: Option<String>,
    /// Latitude in WGS-84 decimal degrees.
    pub latitude: Option<f64>,
    /// Longitude in WGS-84 decimal degrees.
    pub longitude: Option<f64>,
}

/// A radio navigation aid, parsed from an AIXM `Navaid` feature.
///
/// Covers VOR, DME, NDB, TACAN, and combined types. Used as route waypoints
/// and for instrument approaches.
///
/// # Examples
///
/// ```no_run
/// # let nav: aixm::Navaid = unimplemented!();
/// match nav.navaid_type.as_deref() {
///     Some("VOR_DME") => println!("{} is a VOR/DME", nav.designator),
///     Some("NDB") => println!("{} is an NDB", nav.designator),
///     _ => {}
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Navaid {
    /// UUID from the `gml:id` attribute.
    pub uuid: String,
    /// Identifier (e.g. `"BOR"`).
    pub designator: String,
    /// Human-readable name (e.g. `"BOORSPIJK"`).
    pub name: Option<String>,
    /// Type code (e.g. `"VOR"`, `"VOR_DME"`, `"NDB"`, `"TACAN"`).
    pub navaid_type: Option<String>,
    /// Latitude in WGS-84 decimal degrees.
    pub latitude: Option<f64>,
    /// Longitude in WGS-84 decimal degrees.
    pub longitude: Option<f64>,
    /// Station elevation.
    pub elevation: Option<f64>,
}

/// An airspace boundary, parsed from an AIXM `Airspace` feature.
///
/// Contains the airspace classification, vertical limits, and horizontal
/// geometry needed to determine whether a position is inside the airspace.
///
/// # Examples
///
/// ```no_run
/// # let arsp: aixm::Airspace = unimplemented!();
/// if arsp.airspace_type.as_deref() == Some("CTR") {
///     println!("Control zone: {}", arsp.name.as_deref().unwrap_or("unnamed"));
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Airspace {
    /// UUID from the `gml:id` attribute.
    pub uuid: String,
    /// Airspace type code (e.g. `"CTR"`, `"TMA"`, `"CTA"`, `"D"`, `"R"`,
    /// `"P"`).
    pub airspace_type: Option<String>,
    /// Designator (e.g. `"EADD CTR"`).
    pub designator: Option<String>,
    /// Human-readable name (e.g. `"DONLON CTR"`).
    pub name: Option<String>,
    /// Geometry volumes with vertical limits and horizontal boundaries.
    pub volumes: Vec<AirspaceVolume>,
}

/// A single airspace volume with vertical limits and a horizontal polygon.
///
/// # Examples
///
/// ```no_run
/// # let vol: aixm::AirspaceVolume = unimplemented!();
/// // Check if a coordinate falls inside the polygon
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
