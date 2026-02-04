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

//! Spatial indexing for efficient airspace and navaid queries.

use std::rc::Rc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use geo::{Distance, Geodesic, Point};
use rstar::primitives::{GeomWithData, Rectangle};
use rstar::{RTree, RTreeObject, AABB};

use super::{Airport, Airspace, NavAid, Waypoint};
use crate::measurements::{Length, LengthUnit};

/// Approximate conversion factor: 1 nautical mile ≈ 1/60 degree.
const NM_TO_DEG: f64 = 1.0 / 60.0;

/// Spatial index for efficient airspace queries using an R-tree.
///
/// The index stores bounding boxes of airspaces, allowing quick filtering
/// of candidates before performing precise polygon containment checks.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AirspaceIndex {
    tree: RTree<GeomWithData<Rectangle<Point<f64>>, Rc<Airspace>>>,
}

impl AirspaceIndex {
    /// Creates an index from an iterator of airspaces.
    pub fn new<'a>(airspaces: impl Iterator<Item = &'a Rc<Airspace>>) -> Self {
        let entries = airspaces
            .map(|a| {
                let rect = Rectangle::from_aabb(a.polygon.envelope());
                GeomWithData::new(rect, Rc::clone(a))
            })
            .collect();

        Self {
            tree: RTree::bulk_load(entries),
        }
    }

    /// Returns airspaces whose bounding boxes contain the point.
    pub fn candidates_at(&self, lon: f64, lat: f64) -> impl Iterator<Item = &Rc<Airspace>> + '_ {
        let point_envelope = AABB::from_point(Point::new(lon, lat));
        self.tree
            .locate_in_envelope_intersecting(&point_envelope)
            .map(|entry| &entry.data)
    }

    /// Returns airspaces whose bounding boxes intersect the given envelope.
    pub fn candidates_intersecting(
        &self,
        envelope: &AABB<Point<f64>>,
    ) -> impl Iterator<Item = &Rc<Airspace>> + '_ {
        self.tree
            .locate_in_envelope_intersecting(envelope)
            .map(|entry| &entry.data)
    }
}

/// Spatial index for efficient navaid proximity queries using an R-tree.
///
/// Indexes airports and waypoints by their coordinates for fast
/// radius-based searches.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavAidIndex {
    tree: RTree<GeomWithData<Point<f64>, NavAid>>,
}

impl NavAidIndex {
    /// Creates an index from iterators of airports and waypoints.
    pub fn new<'a>(
        airports: impl Iterator<Item = &'a Rc<Airport>>,
        waypoints: impl Iterator<Item = &'a Rc<Waypoint>>,
    ) -> Self {
        let mut entries = Vec::new();

        for airport in airports {
            entries.push(GeomWithData::new(
                airport.coordinate,
                NavAid::Airport(Rc::clone(airport)),
            ));
        }

        for waypoint in waypoints {
            entries.push(GeomWithData::new(
                waypoint.coordinate,
                NavAid::Waypoint(Rc::clone(waypoint)),
            ));
        }

        Self {
            tree: RTree::bulk_load(entries),
        }
    }

    /// Returns navaids within the given radius of a coordinate.
    ///
    /// The radius is converted to an approximate degree-based bounding box
    /// for the R-tree query. Results are then filtered by actual geodesic
    /// distance.
    pub fn within_radius(
        &self,
        coord: &Point<f64>,
        radius: Length,
    ) -> impl Iterator<Item = &NavAid> {
        let radius_nm = *radius.convert_to(LengthUnit::NauticalMiles).value() as f64;
        let radius_deg = radius_nm * NM_TO_DEG;

        // Create bounding box around the point
        // Adjust longitude expansion for latitude (degrees are smaller near poles)
        let lat_rad = coord.y().to_radians();
        let lon_expansion = if lat_rad.cos().abs() > 0.01 {
            radius_deg / lat_rad.cos()
        } else {
            radius_deg * 100.0 // Near poles, use large expansion
        };

        let envelope = AABB::from_corners(
            Point::new(coord.x() - lon_expansion, coord.y() - radius_deg),
            Point::new(coord.x() + lon_expansion, coord.y() + radius_deg),
        );

        let center = *coord;
        let radius_m = radius.to_si() as f64;

        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .filter(move |entry| Geodesic.distance(center, *entry.geom()) <= radius_m)
            .map(|entry| &entry.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nd::AirspaceClass;
    use crate::VerticalDistance;

    fn test_airspace(name: &str, coords: &[(f64, f64)]) -> Rc<Airspace> {
        let exterior: Vec<geo::Coord<f64>> = coords
            .iter()
            .map(|&(lat, lon)| geo::Coord { x: lon, y: lat })
            .collect();

        Rc::new(Airspace {
            name: name.to_string(),
            class: AirspaceClass::D,
            ceiling: VerticalDistance::Fl(65),
            floor: VerticalDistance::Msl(1500),
            polygon: geo::Polygon::new(geo::LineString::from(exterior), vec![]),
        })
    }

    fn test_airport(ident: &str, lat: f64, lon: f64) -> Rc<Airport> {
        Rc::new(Airport {
            icao_ident: ident.to_string(),
            iata_designator: String::new(),
            name: ident.to_string(),
            coordinate: Point::new(lon, lat),
            mag_var: None,
            elevation: VerticalDistance::Gnd,
            runways: vec![],
            location: None,
            cycle: None,
        })
    }

    fn test_waypoint(ident: &str, lat: f64, lon: f64) -> Rc<Waypoint> {
        use crate::nd::waypoint::{Region, WaypointUsage};
        Rc::new(Waypoint {
            fix_ident: ident.to_string(),
            desc: String::new(),
            usage: WaypointUsage::Unknown,
            coordinate: Point::new(lon, lat),
            mag_var: None,
            region: Region::Enroute,
            location: None,
            cycle: None,
        })
    }

    #[test]
    fn index_finds_point_inside_airspace() {
        //         9.0      9.5     10.0
        //  54.0    +--------+--------+
        //          |                 |
        //  53.5    |        x        |  x = query point (9.5, 53.5)
        //          |                 |
        //  53.0    +--------+--------+
        let airspaces = vec![test_airspace(
            "Test",
            &[
                (53.0, 9.0),
                (53.0, 10.0),
                (54.0, 10.0),
                (54.0, 9.0),
                (53.0, 9.0),
            ],
        )];

        let index = AirspaceIndex::new(airspaces.iter());

        let candidates: Vec<_> = index.candidates_at(9.5, 53.5).collect();
        assert_eq!(candidates.len(), 1);
        assert!(Rc::ptr_eq(candidates[0], &airspaces[0]));
    }

    #[test]
    fn index_does_not_find_point_outside_airspace() {
        //       8.0   9.0             10.0
        //  54.0        +---------------+
        //              |               |
        //  53.0        +---------------+
        //  52.0  x                        x = query point (8.0, 52.0)
        let airspaces = vec![test_airspace(
            "Test",
            &[
                (53.0, 9.0),
                (53.0, 10.0),
                (54.0, 10.0),
                (54.0, 9.0),
                (53.0, 9.0),
            ],
        )];

        let index = AirspaceIndex::new(airspaces.iter());

        let candidates: Vec<_> = index.candidates_at(8.0, 52.0).collect();
        assert!(candidates.is_empty());
    }

    #[test]
    fn point_index_finds_airports_within_radius() {
        //           9.99          10.70
        //  53.81              EDHL
        //            .---.
        //  53.63    (EDDH )   10 NM radius finds only EDDH
        //            '---'    50 NM radius finds both
        let airports = vec![
            test_airport("EDDH", 53.63, 9.99),  // Hamburg
            test_airport("EDHL", 53.81, 10.70), // Luebeck (~35 NM from Hamburg)
        ];
        let waypoints: Vec<Rc<Waypoint>> = vec![];

        let index = NavAidIndex::new(airports.iter(), waypoints.iter());

        let center = Point::new(9.99, 53.63);
        let results: Vec<_> = index.within_radius(&center, Length::nm(10.0)).collect();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], NavAid::Airport(a) if a.icao_ident == "EDDH"));

        let results: Vec<_> = index.within_radius(&center, Length::nm(50.0)).collect();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn point_index_finds_waypoints_within_radius() {
        //          9.5   9.55  9.6              10.5
        //  54.5                                  WP3   (far away)
        //
        //  53.6          .---WP2-.
        //  53.55        ( center  )  10 NM radius
        //  53.5          '-WP1---'
        let airports: Vec<Rc<Airport>> = vec![];
        let waypoints = vec![
            test_waypoint("WP1", 53.5, 9.5),
            test_waypoint("WP2", 53.6, 9.6),
            test_waypoint("WP3", 54.5, 10.5),
        ];

        let index = NavAidIndex::new(airports.iter(), waypoints.iter());

        let center = Point::new(9.55, 53.55);
        let results: Vec<_> = index.within_radius(&center, Length::nm(10.0)).collect();

        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(matches!(r, NavAid::Waypoint(_)));
        }
    }

    #[test]
    fn point_index_finds_mixed_navaids() {
        //          9.95  9.97  9.99
        //  53.63          .--EDDH--.
        //  53.62         ( center   )  5 NM radius
        //  53.60          '-DHN1---'
        let airports = vec![test_airport("EDDH", 53.63, 9.99)];
        let waypoints = vec![test_waypoint("DHN1", 53.60, 9.95)];

        let index = NavAidIndex::new(airports.iter(), waypoints.iter());

        let center = Point::new(9.97, 53.62);
        let results: Vec<_> = index.within_radius(&center, Length::nm(5.0)).collect();

        assert_eq!(results.len(), 2);

        let has_airport = results.iter().any(|r| matches!(r, NavAid::Airport(_)));
        let has_waypoint = results.iter().any(|r| matches!(r, NavAid::Waypoint(_)));

        assert!(has_airport);
        assert!(has_waypoint);
    }
}
