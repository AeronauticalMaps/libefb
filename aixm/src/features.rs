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

/// A parsed AIXM feature, yielded one at a time by the [`Features`] iterator.
///
/// [`Features`]: crate::Features
pub enum Feature {
    AirportHeliport(AirportHeliport),
    Runway(Runway),
    RunwayDirection(RunwayDirection),
    DesignatedPoint(DesignatedPoint),
    Navaid(Navaid),
    Airspace(Airspace),
}

/// An airport or heliport.
#[derive(Clone, Debug)]
pub struct AirportHeliport {
    /// The UUID identifier (`gml:id` on the root element).
    pub uuid: String,
    /// ICAO designator (e.g. "EDDH").
    pub designator: String,
    /// Human-readable name.
    pub name: String,
    /// ICAO location indicator (e.g. "EDDH").
    pub location_indicator_icao: Option<String>,
    /// IATA designator (e.g. "HAM").
    pub iata_designator: Option<String>,
    /// Field elevation in meters.
    pub field_elevation: Option<f64>,
    /// Unit of measurement for field elevation.
    pub field_elevation_uom: Option<String>,
    /// Aerodrome reference point latitude (WGS-84, decimal degrees).
    pub latitude: Option<f64>,
    /// Aerodrome reference point longitude (WGS-84, decimal degrees).
    pub longitude: Option<f64>,
}

/// A physical runway strip.
#[derive(Clone, Debug)]
pub struct Runway {
    /// The UUID identifier.
    pub uuid: String,
    /// Designator (e.g. "09L/27R").
    pub designator: String,
    /// Nominal length in the unit given by `length_uom`.
    pub nominal_length: Option<f64>,
    /// Unit of measurement for the length (e.g. "M").
    pub length_uom: Option<String>,
    /// Surface composition (e.g. "ASPH", "CONC", "GRASS").
    pub surface_composition: Option<String>,
    /// UUID of the associated `AirportHeliport` (from `xlink:href`).
    pub associated_airport_uuid: Option<String>,
}

/// A runway direction (one end of a physical runway).
#[derive(Clone, Debug)]
pub struct RunwayDirection {
    /// The UUID identifier.
    pub uuid: String,
    /// Designator (e.g. "09L").
    pub designator: String,
    /// True bearing in degrees.
    pub true_bearing: Option<f64>,
    /// Magnetic bearing in degrees.
    pub magnetic_bearing: Option<f64>,
    /// UUID of the parent `Runway` (from `xlink:href`).
    pub used_runway_uuid: Option<String>,
}

/// A designated point (fix / waypoint).
#[derive(Clone, Debug)]
pub struct DesignatedPoint {
    /// The UUID identifier.
    pub uuid: String,
    /// Designator (e.g. "ABLAN").
    pub designator: String,
    /// Human-readable name.
    pub name: Option<String>,
    /// Point type (e.g. "ICAO", "COORD").
    pub point_type: Option<String>,
    /// Latitude (WGS-84, decimal degrees).
    pub latitude: Option<f64>,
    /// Longitude (WGS-84, decimal degrees).
    pub longitude: Option<f64>,
}

/// A navaid (VOR, DME, NDB, etc.).
#[derive(Clone, Debug)]
pub struct Navaid {
    /// The UUID identifier.
    pub uuid: String,
    /// Designator (e.g. "BOR").
    pub designator: String,
    /// Human-readable name.
    pub name: Option<String>,
    /// Type (e.g. "VOR", "VOR_DME", "NDB").
    pub navaid_type: Option<String>,
    /// Latitude (WGS-84, decimal degrees).
    pub latitude: Option<f64>,
    /// Longitude (WGS-84, decimal degrees).
    pub longitude: Option<f64>,
    /// Elevation in meters.
    pub elevation: Option<f64>,
}

/// An airspace boundary.
#[derive(Clone, Debug)]
pub struct Airspace {
    /// The UUID identifier.
    pub uuid: String,
    /// Airspace type (e.g. "CTR", "TMA", "CTA", "D", "R", "P").
    pub airspace_type: Option<String>,
    /// Designator (e.g. "EDD01").
    pub designator: Option<String>,
    /// Human-readable name.
    pub name: Option<String>,
    /// Geometry components (one per volume).
    pub volumes: Vec<AirspaceVolume>,
}

/// A single airspace volume with vertical limits and horizontal geometry.
#[derive(Clone, Debug)]
pub struct AirspaceVolume {
    /// Upper limit value (e.g. "195", "FL195", "UNL").
    pub upper_limit: Option<String>,
    /// Upper limit unit of measurement (e.g. "FL", "FT", "M").
    pub upper_limit_uom: Option<String>,
    /// Upper limit reference (e.g. "MSL", "SFC").
    pub upper_limit_ref: Option<String>,
    /// Lower limit value.
    pub lower_limit: Option<String>,
    /// Lower limit unit of measurement.
    pub lower_limit_uom: Option<String>,
    /// Lower limit reference.
    pub lower_limit_ref: Option<String>,
    /// Polygon coordinates as (latitude, longitude) pairs in WGS-84.
    pub polygon: Vec<(f64, f64)>,
}
