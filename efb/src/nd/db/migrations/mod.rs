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

pub(super) const SCHEMA_VERSION: u32 = 1;

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
    Migrations::new(vec![M::up(INITIAL)])
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
    fn running_twice_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
    }
}
