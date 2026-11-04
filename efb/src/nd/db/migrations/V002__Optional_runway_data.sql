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

-- The declared distances, the surface and the slope of a runway may be
-- unknown. SQLite can't drop a NOT NULL constraint, so the table is
-- rebuilt.
CREATE TABLE runways_new (
    id                         INTEGER PRIMARY KEY AUTOINCREMENT,
    airport_id                 INTEGER NOT NULL
                               REFERENCES airports(id) ON DELETE CASCADE,
    designator                 TEXT NOT NULL,
    bearing_degrees            REAL NOT NULL,
    length_meters              REAL NOT NULL,
    tora_meters                REAL,
    toda_meters                REAL,
    lda_meters                 REAL,
    displaced_threshold_meters REAL,
    surface                    TEXT CHECK (surface IN ('asphalt','concrete','grass')),
    slope_percent              REAL,
    elev_kind                  TEXT NOT NULL,
    elev_value                 INTEGER
) STRICT;

INSERT INTO runways_new
    (id, airport_id, designator, bearing_degrees, length_meters,
     tora_meters, toda_meters, lda_meters, surface, slope_percent,
     elev_kind, elev_value)
SELECT id, airport_id, designator, bearing_degrees, length_meters,
       tora_meters, toda_meters, lda_meters, surface, slope_percent,
       elev_kind, elev_value
FROM runways;

DROP TABLE runways;

ALTER TABLE runways_new RENAME TO runways;

CREATE INDEX idx_runways_airport ON runways(airport_id);
