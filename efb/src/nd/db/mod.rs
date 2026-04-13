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

//! SQLite navigation data storage.
//!
//! The [`NavigationData`] can be stored into a SQLite database and read from
//! it. This module doesn't own nor manages the database [connection], and only
//! [reads], migrates and [writes] on the provided database connection.
//!
//! Navigation data partitions can be removed from the database by
//! [`remove_partition`].
//!
//! # Examples
//!
//! ```
//! # use efb::nd::NavigationData;
//! // the database connection we are going to write to
//! let mut conn = rusqlite::Connection::open_in_memory().expect("memory database should open");
//!
//! // ARINC 424 records of Hamburg airport with its runways
//! let records = br#"
//! SEURP EDDHEDA        0        N N53374900E009591762E002000053                   P    MWGE    HAMBURG                       356462409
//! SEURP EDDHEDGRW05    0106630500 N53371100E009580180                          151                                           124362502
//! SEURP EDDHEDGRW23    0106632300 N53380900E009595876                          151                                           124362502
//! SEURP EDDHEDGRW15    0120271530 N53391500E009583076                          151                                           124362502
//! SEURP EDDHEDGRW33    0120273330 N53374300E009595081                          151                                           124362502
//! "#;
//!
//! // read the ARINC 424 data
//! let nd = NavigationData::try_from_arinc424(records).expect("ARINC 424 should be valid");
//!
//! // now we can store the navigation data in the database
//! nd.try_into_sqlite(&mut conn).expect("memory database should be writable");
//! ```
//!
//! The FMS navigation data can be load from a SQLite database connection:
//!
//! ```
//! # use efb::prelude::{FMS, NavigationData};
//! # fn from_sqlite(mut conn: rusqlite::Connection) {
//! let mut fms = FMS::new();
//!
//! // read all data partitions from the database
//! let partitions = NavigationData::try_from_sqlite(&mut conn).expect("stored data should be valid");
//!
//! // bulk load all partitions into the FMS
//! fms.modify_nd(|nd| nd.concat(partitions)).expect("FMS should reevaluate");
//! # }
//! ```
//!
//! [connection]: rusqlite::Connection
//! [reads]: NavigationData::try_from_sqlite
//! [writes]: NavigationData::try_into_sqlite

use rusqlite::{params, Connection};

use crate::error::Result;
use crate::nd::NavigationData;

mod encoding;
mod migrations;
mod read;
mod types;
mod write;

/// Removes a single navigation data partition from the database.
pub fn remove_partition(conn: &mut Connection, partition_id: u64) -> Result<()> {
    migrations::migrate(conn)?;
    conn.execute(
        "DELETE FROM partitions WHERE id = ?1",
        params![partition_id.to_string()],
    )?;
    Ok(())
}

impl NavigationData {
    /// Loads navigation data from a SQLite connection.
    ///
    /// Runs any pending schema migrations, then reads every partition stored in
    /// the database.
    pub fn try_from_sqlite(conn: &mut Connection) -> Result<Vec<NavigationData>> {
        migrations::migrate(conn)?;
        read::all_partitions(conn)
    }

    /// Writes this navigation data and every nested partition into the
    /// supplied connection.
    pub fn try_into_sqlite(&self, conn: &mut Connection) -> Result<()> {
        migrations::migrate(conn)?;
        write::all_partitions(conn, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nd::Fix;

    const HAMBURG_A424: &[u8] = br#"
SEURP EDDHEDA        0        N N53374900E009591762E002000053                   P    MWGE    HAMBURG                       356462409
SEURP EDDHEDGRW05    0106630500 N53371100E009580180                          151                                           124362502
SEURP EDDHEDGRW23    0106632300 N53380900E009595876                          151                                           124362502
"#;

    #[test]
    fn round_trip_preserves_airport_and_runways() {
        let mut conn = Connection::open_in_memory().unwrap();

        let original =
            NavigationData::try_from_arinc424(HAMBURG_A424).expect("ARINC 424 should parse");
        original
            .try_into_sqlite(&mut conn)
            .expect("write should succeed");

        let loaded = NavigationData::try_from_sqlite(&mut conn).expect("read should succeed");

        assert_eq!(loaded.len(), 1, "one partition expected");
        let partition = &loaded[0];
        assert_eq!(partition.partition_id(), original.partition_id());
        assert_eq!(partition.source_format(), original.source_format());

        let eddh = partition.find("EDDH").expect("EDDH should be present");
        assert_eq!(eddh.ident(), "EDDH");

        let original_eddh = original.find("EDDH").unwrap();
        match (original_eddh, eddh) {
            (crate::nd::NavAid::Airport(a), crate::nd::NavAid::Airport(b)) => {
                assert_eq!(a.runways.len(), b.runways.len());
            }
            _ => panic!("expected airport"),
        }
    }

    #[test]
    fn empty_database_returns_no_partitions() {
        let mut conn = Connection::open_in_memory().unwrap();
        let loaded = NavigationData::try_from_sqlite(&mut conn).expect("read should succeed");
        assert!(loaded.is_empty());
    }
}
