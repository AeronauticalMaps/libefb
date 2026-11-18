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

-- Alert areas are a new airspace type with an activity. SQLite can't change a CHECK
-- constraint, and rebuilding the airspaces would cascade to their
-- vertices, so the airspaces are cleared: they are back once the
-- navigation data is loaded again.
--
-- The vertices are dropped first, so nothing cascades. Dropping the
-- airspaces drops their triggers but fires none, so their R-tree is
-- cleared explicitly.
DROP TABLE airspace_vertices;
DROP TABLE airspaces;
DELETE FROM airspaces_rtree;

CREATE TABLE airspaces (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    partition_id    TEXT NOT NULL
                    REFERENCES partitions(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    airspace_type   TEXT NOT NULL CHECK (airspace_type IN
                    ('cta','ctr','tma','restricted','danger',
                     'prohibited','tmz','rmz','radar_zone','alert')),
    classification  TEXT CHECK (classification IN ('A','B','C','D','E','F','G')),
    activity        TEXT CHECK (activity IN
                    ('aerodrome_traffic','helicopter_traffic',
                     'training','aerobatics','air_show','air_sports',
                     'ultralight','gliding','paragliding',
                     'hang_gliding','parachuting','air_drop',
                     'ballooning','radiosonde','space_flight','uav',
                     'aerial_work','crop_dusting','fire_fighting',
                     'military','air_refuelling','jet_climbing',
                     'interception_exercise','target_towing',
                     'naval_exercise','missiles','air_gunnery',
                     'artillery','shooting','blasting',
                     'underwater_blasting','anti_hail','birds',
                     'bird_migration','fireworks',
                     'high_intensity_radio','high_intensity_light',
                     'laser','nature_reserve','fauna','noise_abatement',
                     'accident_investigation','population','vip',
                     'head_of_state','vice_head_of_state','oil','gas',
                     'refinery','chemical_plant','nuclear_plant',
                     'technical','ats','procedure')),
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
