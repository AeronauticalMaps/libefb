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

use geo::{Coord, LineString, Point, Polygon};
use rusqlite::{params, Connection};

use crate::core::MagneticVariation;
use crate::error::{Error, Result};
use crate::measurements::{Angle, Length};
use crate::nd::{
    AiracCycle, Airport, Airspace, AirspaceClassification, AirspaceType, LocationIndicator,
    NavigationData, NavigationDataBuilder, Region, Runway, RunwaySurface, SourceFormat, Waypoint,
    WaypointUsage,
};

use super::encoding::vd_from_row;

pub(super) fn all_partitions(conn: &mut Connection) -> Result<Vec<NavigationData>> {
    let partitions = select_partitions(conn)?;

    let mut out = Vec::with_capacity(partitions.len());
    for (partition_id_str, source_format) in partitions {
        let partition_id: u64 =
            partition_id_str
                .parse()
                .map_err(|e: std::num::ParseIntError| {
                    Error::Database(format!("invalid partition id {:?}: {e}", partition_id_str))
                })?;

        let mut builder = NavigationDataBuilder::new()
            .with_format(source_format)
            .with_partition_id(partition_id);

        load_airports(conn, &partition_id_str, &mut builder)?;
        load_waypoints(conn, &partition_id_str, &mut builder)?;
        load_airspaces(conn, &partition_id_str, &mut builder)?;

        out.push(builder.build());
    }

    Ok(out)
}

fn select_partitions(conn: &Connection) -> Result<Vec<(String, SourceFormat)>> {
    let mut stmt = conn.prepare("SELECT id, source_format FROM partitions")?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, SourceFormat>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn load_airports(
    conn: &Connection,
    partition_id: &str,
    builder: &mut NavigationDataBuilder,
) -> Result<()> {
    let runways_by_airport = load_runways_by_airport(conn, partition_id)?;

    let mut stmt = conn.prepare(
        "SELECT id, icao_ident, iata_designator, name, lat, lon, \
                mag_var_degrees, elevation_kind, elevation_value, \
                location_indicator, airac_cycle \
         FROM airports WHERE partition_id = ?1",
    )?;

    let mut rows = stmt.query(params![partition_id])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let icao_ident: String = row.get(1)?;
        let iata_designator: String = row.get(2)?;
        let name: String = row.get(3)?;
        let lat: f64 = row.get(4)?;
        let lon: f64 = row.get(5)?;
        let mag_var: Option<MagneticVariation> = row.get(6)?;
        let elevation_kind: String = row.get(7)?;
        let elevation_value: Option<i64> = row.get(8)?;
        let location: Option<LocationIndicator> = row.get(9)?;
        let cycle: Option<AiracCycle> = row.get(10)?;

        let elevation = vd_from_row(&elevation_kind, elevation_value).ok_or_else(|| {
            Error::Database(format!(
                "airport {icao_ident:?} has invalid elevation kind {elevation_kind:?}"
            ))
        })?;

        let runways = runways_by_airport.get(&id).cloned().unwrap_or_default();

        builder.add_airport(Airport {
            icao_ident,
            iata_designator,
            name,
            coordinate: Point::new(lon, lat),
            mag_var,
            elevation,
            runways,
            location,
            cycle,
        });
    }

    Ok(())
}

/// Loads every runway for the partition in one query and groups them by
/// `airport_id`. Avoids the N+1 query pattern of reading runways per airport.
fn load_runways_by_airport(
    conn: &Connection,
    partition_id: &str,
) -> Result<HashMap<i64, Vec<Runway>>> {
    let mut stmt = conn.prepare(
        "SELECT r.airport_id, r.designator, r.bearing_degrees, r.length_meters, \
                r.tora_meters, r.toda_meters, r.lda_meters, \
                r.surface, r.slope_percent, r.elev_kind, r.elev_value \
         FROM runways r \
         JOIN airports a ON a.id = r.airport_id \
         WHERE a.partition_id = ?1",
    )?;

    let mut rows = stmt.query(params![partition_id])?;
    let mut by_airport: HashMap<i64, Vec<Runway>> = HashMap::new();
    while let Some(row) = rows.next()? {
        let airport_id: i64 = row.get(0)?;
        let designator: String = row.get(1)?;
        let bearing_degrees: f64 = row.get(2)?;
        let length_meters: f64 = row.get(3)?;
        let tora_meters: f64 = row.get(4)?;
        let toda_meters: f64 = row.get(5)?;
        let lda_meters: f64 = row.get(6)?;
        let surface: RunwaySurface = row.get(7)?;
        let slope_percent: f64 = row.get(8)?;
        let elev_kind: String = row.get(9)?;
        let elev_value: Option<i64> = row.get(10)?;

        let elev = vd_from_row(&elev_kind, elev_value).ok_or_else(|| {
            Error::Database(format!(
                "runway {designator:?} has invalid elev kind {elev_kind:?}"
            ))
        })?;

        by_airport.entry(airport_id).or_default().push(Runway {
            designator,
            bearing: Angle::t(bearing_degrees as f32),
            length: Length::m(length_meters as f32),
            tora: Length::m(tora_meters as f32),
            toda: Length::m(toda_meters as f32),
            lda: Length::m(lda_meters as f32),
            surface,
            slope: slope_percent as f32,
            elev,
        });
    }
    Ok(by_airport)
}

fn load_waypoints(
    conn: &Connection,
    partition_id: &str,
    builder: &mut NavigationDataBuilder,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT fix_ident, description, usage, lat, lon, \
                mag_var_degrees, terminal_airport_ident, \
                location_indicator, airac_cycle \
         FROM waypoints WHERE partition_id = ?1",
    )?;

    let mut rows = stmt.query(params![partition_id])?;
    while let Some(row) = rows.next()? {
        let fix_ident: String = row.get(0)?;
        let desc: String = row.get(1)?;
        let usage: WaypointUsage = row.get(2)?;
        let lat: f64 = row.get(3)?;
        let lon: f64 = row.get(4)?;
        let mag_var: Option<MagneticVariation> = row.get(5)?;
        let region: Region = row.get(6)?;
        let location: Option<LocationIndicator> = row.get(7)?;
        let cycle: Option<AiracCycle> = row.get(8)?;
        builder.add_waypoint(Waypoint {
            fix_ident,
            desc,
            usage,
            coordinate: Point::new(lon, lat),
            mag_var,
            region,
            location,
            cycle,
        });
    }

    Ok(())
}

fn load_airspaces(
    conn: &Connection,
    partition_id: &str,
    builder: &mut NavigationDataBuilder,
) -> Result<()> {
    let mut rings_by_airspace = load_airspace_vertices(conn, partition_id)?;

    let mut stmt = conn.prepare(
        "SELECT id, name, airspace_type, classification, \
                ceiling_kind, ceiling_value, floor_kind, floor_value \
         FROM airspaces WHERE partition_id = ?1",
    )?;

    let mut rows = stmt.query(params![partition_id])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let airspace_type: AirspaceType = row.get(2)?;
        let classification: Option<AirspaceClassification> = row.get(3)?;
        let ceiling_kind: String = row.get(4)?;
        let ceiling_value: Option<i64> = row.get(5)?;
        let floor_kind: String = row.get(6)?;
        let floor_value: Option<i64> = row.get(7)?;

        let ceiling = vd_from_row(&ceiling_kind, ceiling_value).ok_or_else(|| {
            Error::Database(format!(
                "airspace {name:?} has invalid ceiling kind {ceiling_kind:?}"
            ))
        })?;
        let floor = vd_from_row(&floor_kind, floor_value).ok_or_else(|| {
            Error::Database(format!(
                "airspace {name:?} has invalid floor kind {floor_kind:?}"
            ))
        })?;
        let rings = rings_by_airspace.remove(&id).unwrap_or_default();
        let polygon = polygon_from_rings(id, rings)?;

        builder.add_airspace(Airspace {
            name,
            airspace_type,
            classification,
            ceiling,
            floor,
            polygon,
        });
    }

    Ok(())
}

/// Loads every airspace vertex for the partition in one query and groups them
/// by `(airspace_id, ring)` in a single pass. Avoids the per-airspace query.
fn load_airspace_vertices(
    conn: &Connection,
    partition_id: &str,
) -> Result<HashMap<i64, Vec<Vec<Coord<f64>>>>> {
    let mut stmt = conn.prepare(
        "SELECT v.airspace_id, v.ring, v.lat, v.lon \
         FROM airspace_vertices v \
         JOIN airspaces a ON a.id = v.airspace_id \
         WHERE a.partition_id = ?1 \
         ORDER BY v.airspace_id, v.ring, v.ordinal",
    )?;

    let mut rows = stmt.query(params![partition_id])?;
    let mut by_airspace: HashMap<i64, Vec<Vec<Coord<f64>>>> = HashMap::new();
    while let Some(row) = rows.next()? {
        let airspace_id: i64 = row.get(0)?;
        let ring: i64 = row.get(1)?;
        let lat: f64 = row.get(2)?;
        let lon: f64 = row.get(3)?;

        let rings = by_airspace.entry(airspace_id).or_default();
        let idx = ring as usize;
        if rings.len() <= idx {
            rings.resize_with(idx + 1, Vec::new);
        }
        rings[idx].push(Coord { x: lon, y: lat });
    }
    Ok(by_airspace)
}

fn polygon_from_rings(airspace_id: i64, rings: Vec<Vec<Coord<f64>>>) -> Result<Polygon<f64>> {
    let mut rings = rings.into_iter();
    let exterior = rings
        .next()
        .map(LineString::from)
        .ok_or_else(|| Error::Database(format!("airspace {airspace_id} has no exterior ring")))?;
    let interiors: Vec<LineString<f64>> = rings.map(LineString::from).collect();
    Ok(Polygon::new(exterior, interiors))
}
