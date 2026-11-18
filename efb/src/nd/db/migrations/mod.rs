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

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::error::{Error, Result};

const INITIAL: &str = include_str!("V001__Initial_navigation_data.sql");
const OPTIONAL_RUNWAY_DATA: &str = include_str!("V002__Optional_runway_data.sql");
const TERRITORY_LETTERS: &str = include_str!("V003__Territory_letters.sql");
const ALERT_AREAS: &str = include_str!("V004__Alert_areas.sql");

#[allow(unused)]
pub(super) const SCHEMA_VERSION: u32 = 4;

/// Brings the database up to [`SCHEMA_VERSION`].
///
/// Also turns on `PRAGMA foreign_keys` for this connection so the
/// `ON DELETE CASCADE` behaviour we rely on is actually honoured.
/// Foreign keys are a per-connection pragma in SQLite and libefb
/// doesn't own the connection, so we flip it here where we know
/// we're about to touch the schema.
pub(super) fn migrate(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;

    let foreign_keys_enabled: i64 =
        conn.pragma_query_value(None, "foreign_keys", |row| row.get(0))?;

    if foreign_keys_enabled == 1 {
        migrations().to_latest(conn)?;
        Ok(())
    } else {
        Err(Error::Database(
            "failed to enable SQLite foreign keys; migrate() should not called inside an active transaction".into(),
        ))
    }
}

fn migrations() -> Migrations<'static> {
    // Adding a future migration: append a new `M::up(...)`, bump
    // `SCHEMA_VERSION`, and drop the matching `XXX.sql` file in this module.
    Migrations::new(vec![
        M::up(INITIAL),
        M::up(OPTIONAL_RUNWAY_DATA),
        M::up(TERRITORY_LETTERS),
        M::up(ALERT_AREAS),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        migrations().validate().expect("migrations should be valid");
    }

    #[test]
    fn fresh_database_runs_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION as i64);
    }

    #[test]
    fn alert_areas_clear_stored_airspaces() {
        let mut conn = Connection::open_in_memory().expect("database should open");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("foreign keys should be enabled");
        migrations()
            .to_version(&mut conn, 3)
            .expect("schema 3 should be created");

        conn.execute_batch(
            "INSERT INTO partitions (id, source_format) VALUES ('ed', 'aixm');
             INSERT INTO airspaces (partition_id, name, airspace_type, ceiling_kind,
                                    floor_kind, min_lat, min_lon, max_lat, max_lon)
             VALUES ('ed', 'KOELN/BONN', 'ctr', 'msl', 'gnd', 50.7, 6.9, 51.0, 7.4);
             INSERT INTO airspace_vertices (airspace_id, ring, ordinal, lat, lon)
             VALUES (1, 0, 0, 51.0, 7.0);",
        )
        .expect("airspace should be stored in schema 3");

        migrate(&mut conn).expect("schema 4 should be migrated to");

        let count = |table: &str| -> i64 {
            conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("table should be counted")
        };
        assert_eq!(count("airspaces"), 0);
        assert_eq!(count("airspace_vertices"), 0);
        assert_eq!(count("airspaces_rtree"), 0);

        conn.execute(
            "INSERT INTO airspaces (partition_id, name, airspace_type, ceiling_kind,
                                    floor_kind, min_lat, min_lon, max_lat, max_lon)
             VALUES ('ed', 'ITZEHOE', 'alert', 'fl', 'gnd', 53.9, 9.5, 54.1, 9.7)",
            [],
        )
        .expect("alert area should be stored");
        assert_eq!(count("airspaces_rtree"), 1);
    }

    #[test]
    fn running_twice_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
    }
}
