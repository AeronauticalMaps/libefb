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

use geo::BoundingRect;
use rusqlite::{params, Connection, Statement, Transaction};

use crate::error::{Error, Result};
use crate::measurements::AngleUnit;
use crate::nd::{Airport, Airspace, NavigationData, Runway, Waypoint};

use super::encoding::{vd_kind, vd_value};

pub(super) fn all_partitions(conn: &mut Connection, nd: &NavigationData) -> Result<()> {
    let tx = conn.transaction()?;

    {
        let mut stmts = PreparedWrites::new(&tx)?;
        if nd.source_format.is_some() {
            write_one(&mut stmts, nd)?;
        }
        for partition in nd.partitions.values() {
            write_one(&mut stmts, partition)?;
        }
    }

    tx.commit()?;
    Ok(())
}

/// Statements reused across every row within a single transaction.
///
/// `Statement::execute` reuses the compiled plan, so hoisting `prepare` out of
/// the per-row loop collapses per-row SQL parsing to once-per-table.
struct PreparedWrites<'t> {
    partition: Statement<'t>,
    airport: Statement<'t>,
    runway: Statement<'t>,
    waypoint: Statement<'t>,
    airspace: Statement<'t>,
    airspace_vertex: Statement<'t>,
}

impl<'t> PreparedWrites<'t> {
    fn new(tx: &'t Transaction<'_>) -> Result<Self> {
        Ok(Self {
            partition: tx.prepare(
                "INSERT OR REPLACE INTO partitions (id, source_format, airac_cycle) \
                 VALUES (?1, ?2, ?3)",
            )?,
            airport: tx.prepare(
                "INSERT OR REPLACE INTO airports \
                 (partition_id, icao_ident, iata_designator, name, lat, lon, \
                  mag_var_degrees, elevation_kind, elevation_value, \
                  location_indicator, airac_cycle) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )?,
            runway: tx.prepare(
                "INSERT INTO runways \
                 (airport_id, designator, bearing_degrees, length_meters, \
                  tora_meters, toda_meters, lda_meters, surface, slope_percent, \
                  elev_kind, elev_value) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )?,
            waypoint: tx.prepare(
                "INSERT OR REPLACE INTO waypoints \
                 (partition_id, fix_ident, description, usage, lat, lon, \
                  mag_var_degrees, terminal_airport_ident, location_indicator, airac_cycle) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )?,
            airspace: tx.prepare(
                "INSERT INTO airspaces \
                 (partition_id, name, airspace_type, classification, \
                  ceiling_kind, ceiling_value, floor_kind, floor_value, \
                  min_lat, min_lon, max_lat, max_lon) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            )?,
            airspace_vertex: tx.prepare(
                "INSERT INTO airspace_vertices (airspace_id, ring, ordinal, lat, lon) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?,
        })
    }
}

fn write_one(stmts: &mut PreparedWrites<'_>, nd: &NavigationData) -> Result<()> {
    let source_format = nd.source_format.ok_or_else(|| {
        Error::Database("cannot write partition without a source_format".to_string())
    })?;

    let partition_id = nd.partition_id.to_string();

    stmts
        .partition
        .execute(params![&partition_id, source_format, nd.cycle.as_ref(),])?;

    for airport in &nd.airports {
        let airport_id = write_airport(&mut stmts.airport, &partition_id, airport)?;
        for runway in &airport.runways {
            write_runway(&mut stmts.runway, airport_id, runway)?;
        }
    }
    for waypoint in nd
        .waypoints
        .iter()
        .chain(nd.terminal_waypoints.values().flatten())
    {
        write_waypoint(&mut stmts.waypoint, &partition_id, waypoint)?;
    }
    for airspace in &nd.airspaces {
        write_airspace(
            &mut stmts.airspace,
            &mut stmts.airspace_vertex,
            &partition_id,
            airspace,
        )?;
    }

    Ok(())
}

fn write_airport(stmt: &mut Statement<'_>, partition_id: &str, a: &Airport) -> Result<i64> {
    let rowid = stmt.insert(params![
        partition_id,
        &a.icao_ident,
        &a.iata_designator,
        &a.name,
        a.coordinate.y(),
        a.coordinate.x(),
        a.mag_var.as_ref(),
        vd_kind(&a.elevation),
        vd_value(&a.elevation),
        a.location.as_ref(),
        a.cycle.as_ref(),
    ])?;
    Ok(rowid)
}

fn write_runway(stmt: &mut Statement<'_>, airport_id: i64, r: &Runway) -> Result<()> {
    stmt.execute(params![
        airport_id,
        &r.designator,
        *r.bearing.convert_to(AngleUnit::TrueNorth).value() as f64,
        r.length.to_si() as f64,
        r.tora.to_si() as f64,
        r.toda.to_si() as f64,
        r.lda.to_si() as f64,
        r.surface,
        r.slope as f64,
        vd_kind(&r.elev),
        vd_value(&r.elev),
    ])?;
    Ok(())
}

fn write_waypoint(stmt: &mut Statement<'_>, partition_id: &str, w: &Waypoint) -> Result<()> {
    stmt.execute(params![
        partition_id,
        &w.fix_ident,
        &w.desc,
        w.usage,
        w.coordinate.y(),
        w.coordinate.x(),
        w.mag_var.as_ref(),
        w.region,
        w.location.as_ref(),
        w.cycle.as_ref(),
    ])?;
    Ok(())
}

fn write_airspace(
    airspace_stmt: &mut Statement<'_>,
    vertex_stmt: &mut Statement<'_>,
    partition_id: &str,
    a: &Airspace,
) -> Result<()> {
    let bbox = a
        .polygon
        .bounding_rect()
        .ok_or_else(|| Error::Database(format!("airspace '{}' has an empty polygon", a.name)))?;

    let airspace_id = airspace_stmt.insert(params![
        partition_id,
        &a.name,
        a.airspace_type,
        a.classification.as_ref(),
        vd_kind(&a.ceiling),
        vd_value(&a.ceiling),
        vd_kind(&a.floor),
        vd_value(&a.floor),
        bbox.min().y,
        bbox.min().x,
        bbox.max().y,
        bbox.max().x,
    ])?;

    write_polygon(vertex_stmt, airspace_id, &a.polygon)?;

    Ok(())
}

fn write_polygon(
    stmt: &mut Statement<'_>,
    airspace_id: i64,
    polygon: &geo::Polygon<f64>,
) -> Result<()> {
    write_ring(stmt, airspace_id, 0, polygon.exterior())?;
    for (ring_idx, ring) in polygon.interiors().iter().enumerate() {
        write_ring(stmt, airspace_id, ring_idx + 1, ring)?;
    }
    Ok(())
}

fn write_ring(
    stmt: &mut Statement<'_>,
    airspace_id: i64,
    ring: usize,
    line_string: &geo::LineString<f64>,
) -> Result<()> {
    for (ordinal, coord) in line_string.coords().enumerate() {
        stmt.execute(params![
            airspace_id,
            ring as i64,
            ordinal as i64,
            coord.y,
            coord.x,
        ])?;
    }
    Ok(())
}
