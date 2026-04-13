-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Joe Pearson
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- Navigation data partitions (one row per loaded data source).
-- The id is the u64 hash from NavigationData::partition_id, stored as a
-- decimal string (e.g. "12345566677") to avoid i64 sign confusion in SQLite.
CREATE TABLE partitions (
    id              TEXT PRIMARY KEY NOT NULL,
    source_format   TEXT NOT NULL CHECK (source_format IN ('a424','openair','aixm')),
    source_name     TEXT,
    airac_cycle     INTEGER,    -- YYNN, e.g. 2510 for AIRAC 25/10
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
) STRICT;

-- Key/value metadata. Known keys:
--   efb_version  — crate version that last opened the DB
CREATE TABLE schema_meta (
    key     TEXT PRIMARY KEY NOT NULL,
    value   TEXT NOT NULL
) STRICT;

-- Airports.
CREATE TABLE airports (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    partition_id        TEXT NOT NULL
                        REFERENCES partitions(id) ON DELETE CASCADE,
    icao_ident          TEXT NOT NULL,
    iata_designator     TEXT NOT NULL,
    name                TEXT NOT NULL,
    lat                 REAL NOT NULL,
    lon                 REAL NOT NULL,
    mag_var_degrees     REAL,
    elevation_kind      TEXT NOT NULL,
    elevation_value     INTEGER,
    location_indicator  TEXT,
    airac_cycle         INTEGER,    -- YYNN, e.g. 2510 for AIRAC 25/10
    UNIQUE (partition_id, icao_ident)
) STRICT;

CREATE INDEX idx_airports_ident ON airports(icao_ident);

-- Runways (belong to exactly one airport row).
CREATE TABLE runways (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    airport_id      INTEGER NOT NULL
                    REFERENCES airports(id) ON DELETE CASCADE,
    designator      TEXT NOT NULL,
    bearing_degrees REAL NOT NULL,
    length_meters   REAL NOT NULL,
    tora_meters     REAL NOT NULL,
    toda_meters     REAL NOT NULL,
    lda_meters      REAL NOT NULL,
    surface         TEXT NOT NULL CHECK (surface IN ('asphalt','concrete','grass')),
    slope_percent   REAL NOT NULL,
    elev_kind       TEXT NOT NULL,
    elev_value      INTEGER
) STRICT;

CREATE INDEX idx_runways_airport ON runways(airport_id);

-- Waypoints (enroute or terminal-area).
-- terminal_airport_ident IS NULL  →  enroute waypoint
-- terminal_airport_ident IS NOT NULL  →  terminal-area waypoint for that airport
CREATE TABLE waypoints (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    partition_id            TEXT NOT NULL
                            REFERENCES partitions(id) ON DELETE CASCADE,
    fix_ident               TEXT NOT NULL,
    description             TEXT NOT NULL,
    usage                   TEXT NOT NULL CHECK (usage IN ('vfr_only','unknown')),
    lat                     REAL NOT NULL,
    lon                     REAL NOT NULL,
    mag_var_degrees         REAL,
    terminal_airport_ident  TEXT,
    location_indicator      TEXT,
    airac_cycle             INTEGER     -- YYNN, e.g. 2510 for AIRAC 25/10
) STRICT;

CREATE INDEX idx_waypoints_ident ON waypoints(fix_ident);
CREATE INDEX idx_waypoints_terminal ON waypoints(terminal_airport_ident, fix_ident);

-- Airspaces (polygon features).
CREATE TABLE airspaces (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    partition_id    TEXT NOT NULL
                    REFERENCES partitions(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    airspace_type   TEXT NOT NULL CHECK (airspace_type IN
                    ('cta','ctr','tma','restricted','danger',
                     'prohibited','tmz','rmz','radar_zone')),
    classification  TEXT CHECK (classification IN ('A','B','C','D','E','F','G')),
    ceiling_kind    TEXT NOT NULL,
    ceiling_value   INTEGER,
    floor_kind      TEXT NOT NULL,
    floor_value     INTEGER,
    min_lat         REAL NOT NULL,
    min_lon         REAL NOT NULL,
    max_lat         REAL NOT NULL,
    max_lon         REAL NOT NULL
) STRICT;

CREATE INDEX idx_airspaces_partition ON airspaces(partition_id);

-- Ordered polygon vertices. ring=0 is the exterior, ring>=1 are
-- interior rings (holes), matching geo::Polygon.
CREATE TABLE airspace_vertices (
    airspace_id INTEGER NOT NULL
                REFERENCES airspaces(id) ON DELETE CASCADE,
    ring        INTEGER NOT NULL,
    ordinal     INTEGER NOT NULL,
    lat         REAL NOT NULL,
    lon         REAL NOT NULL,
    PRIMARY KEY (airspace_id, ring, ordinal)
) STRICT, WITHOUT ROWID;

-- R*Tree bounding-box indexes. These are fed by the triggers below
-- so host apps can do fast bbox / tile queries for a moving map.
CREATE VIRTUAL TABLE airports_rtree  USING rtree(id, min_lat, max_lat, min_lon, max_lon);
CREATE VIRTUAL TABLE waypoints_rtree USING rtree(id, min_lat, max_lat, min_lon, max_lon);
CREATE VIRTUAL TABLE airspaces_rtree USING rtree(id, min_lat, max_lat, min_lon, max_lon);

CREATE TRIGGER airports_rtree_ai AFTER INSERT ON airports BEGIN
    INSERT INTO airports_rtree(id, min_lat, max_lat, min_lon, max_lon)
    VALUES (NEW.id, NEW.lat, NEW.lat, NEW.lon, NEW.lon);
END;
CREATE TRIGGER airports_rtree_ad AFTER DELETE ON airports BEGIN
    DELETE FROM airports_rtree WHERE id = OLD.id;
END;

CREATE TRIGGER waypoints_rtree_ai AFTER INSERT ON waypoints BEGIN
    INSERT INTO waypoints_rtree(id, min_lat, max_lat, min_lon, max_lon)
    VALUES (NEW.id, NEW.lat, NEW.lat, NEW.lon, NEW.lon);
END;
CREATE TRIGGER waypoints_rtree_ad AFTER DELETE ON waypoints BEGIN
    DELETE FROM waypoints_rtree WHERE id = OLD.id;
END;

CREATE TRIGGER airspaces_rtree_ai AFTER INSERT ON airspaces BEGIN
    INSERT INTO airspaces_rtree(id, min_lat, max_lat, min_lon, max_lon)
    VALUES (NEW.id, NEW.min_lat, NEW.max_lat, NEW.min_lon, NEW.max_lon);
END;
CREATE TRIGGER airspaces_rtree_au AFTER UPDATE ON airspaces BEGIN
    UPDATE airspaces_rtree
       SET min_lat = NEW.min_lat, max_lat = NEW.max_lat,
           min_lon = NEW.min_lon, max_lon = NEW.max_lon
     WHERE id = NEW.id;
END;
CREATE TRIGGER airspaces_rtree_ad AFTER DELETE ON airspaces BEGIN
    DELETE FROM airspaces_rtree WHERE id = OLD.id;
END;

-- Declare the schema version so WASM reads can verify they match.
PRAGMA user_version = 1;
