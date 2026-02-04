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

use std::rc::Rc;

use geo::{
    Contains, Distance, Geodesic, Intersects, LineIntersection, LineLocatePoint, LineString, Point,
};
use rstar::RTreeObject;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::measurements::{Length, LengthUnit};
use crate::nd::{Airspace, Fix, NavAid, NavigationData};
use crate::VerticalDistance;

use super::Route;

/// An intersection of a route with an airspace.
///
/// Represents the segment where the route passes through an airspace,
/// including entry/exit points and distances from the route start.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AirspaceIntersection {
    airspace: Rc<Airspace>,
    entry_distance: Length,
    exit_distance: Length,
    entry_point: Point<f64>,
    exit_point: Point<f64>,
}

impl AirspaceIntersection {
    /// Returns the intersected airspace.
    pub fn airspace(&self) -> &Airspace {
        &self.airspace
    }

    /// Returns the distance from route start to the entry point.
    pub fn entry_distance(&self) -> &Length {
        &self.entry_distance
    }

    /// Returns the distance from route start to the exit point.
    pub fn exit_distance(&self) -> &Length {
        &self.exit_distance
    }

    /// Returns the geographic coordinate where the route enters the airspace.
    pub fn entry_point(&self) -> &Point<f64> {
        &self.entry_point
    }

    /// Returns the geographic coordinate where the route exits the airspace.
    pub fn exit_point(&self) -> &Point<f64> {
        &self.exit_point
    }

    /// Returns the floor (lower vertical bound) of the airspace.
    pub fn floor(&self) -> &VerticalDistance {
        &self.airspace.floor
    }

    /// Returns the ceiling (upper vertical bound) of the airspace.
    pub fn ceiling(&self) -> &VerticalDistance {
        &self.airspace.ceiling
    }

    /// Returns the length of the route segment within this airspace.
    pub fn length(&self) -> Length {
        self.exit_distance - self.entry_distance
    }
}

/// A point of interest on the vertical profile of a route.
///
/// Represents a significant altitude event along the route, such as the
/// top of climb, a navaid at cruise level, or the start of descent.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VerticalPoint {
    /// The point where the aircraft reaches its cruise level.
    TopOfClimb {
        level: VerticalDistance,
        distance: Length,
    },
    /// A navigation aid (airport or waypoint) at a given level and distance.
    NavAid {
        level: VerticalDistance,
        distance: Length,
        navaid: NavAid,
    },
    /// The point where the aircraft begins its descent from cruise level.
    TopOfDescent {
        level: VerticalDistance,
        distance: Length,
    },
    /// The transition point between climb/descent to level flight.
    ///
    /// The TOC and TOD are special cases of this point that occur at the
    /// transition to and from cruise level.
    LevelOf {
        level: VerticalDistance,
        distance: Length,
    },
}

impl VerticalPoint {
    /// Returns the vertical distance (altitude or flight level) at this point.
    pub fn level(&self) -> &VerticalDistance {
        match self {
            Self::TopOfClimb { level, .. } => level,
            Self::NavAid { level, .. } => level,
            Self::TopOfDescent { level, .. } => level,
            Self::LevelOf { level, .. } => level,
        }
    }
}

/// Vertical profile of a route with airspaces intersected by the route.
///
/// The profile slices through all airspaces that are along the route. It
/// includes not only the airspaces intersected at the route's level, but also
/// those above and below the route. Each [`AirspaceIntersection`] provides the
/// entry and exit point relative to the total route length. The profile
/// features also the levels of waypoints and significant points like TOC and
/// TOD as [`VerticalPoint`].
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VerticalProfile {
    intersections: Vec<AirspaceIntersection>,
    profile: Vec<VerticalPoint>,
}

impl VerticalProfile {
    /// Creates a vertical profile of the route.
    ///
    /// The profile includes the intersections of the route with the navigation
    /// data's airspaces.
    pub fn new(route: &Route, nd: &NavigationData) -> Self {
        let legs = route.legs();
        if legs.is_empty() {
            return Self::default();
        }

        // Build a LineString from the route for intersection tests
        let route_coords: Vec<geo::Coord<f64>> = std::iter::once(legs[0].from().coordinate())
            .chain(legs.iter().map(|leg| leg.to().coordinate()))
            .map(Into::into)
            .collect();

        let route_line = LineString::new(route_coords);

        // Compute per-segment geodesic lengths from the route
        let segment_lengths: Vec<Length> = route_line
            .lines()
            .map(|line| {
                Length::m(Geodesic.distance(Point::from(line.start), Point::from(line.end)) as f32)
            })
            .collect();
        let total_length: Length = segment_lengths.iter().copied().sum();

        // Use the spatial index: query candidates whose bounding boxes
        // intersect the route's envelope (LineString implements RTreeObject)
        let route_envelope = route_line.envelope();
        let candidates = nd.candidate_airspaces_for_envelope(&route_envelope);

        let mut intersections = Vec::new();

        for airspace in &candidates {
            // Check actual intersection
            if !route_line.intersects(&airspace.polygon) {
                continue;
            }

            // Compute entry/exit intersections (may produce multiple for re-entrant routes)
            intersections.extend(Self::compute_intersections(
                Rc::clone(airspace),
                &route_line,
                &segment_lengths,
                total_length,
            ));
        }

        // Sort by entry distance
        intersections.sort_by(|a, b| {
            a.entry_distance()
                .partial_cmp(b.entry_distance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let profile = Self::compute_profile(route);

        Self {
            intersections,
            profile,
        }
    }

    fn compute_intersections(
        airspace: Rc<Airspace>,
        route_line: &LineString<f64>,
        segment_lengths: &[Length],
        total_length: Length,
    ) -> Vec<AirspaceIntersection> {
        let geo_polygon = &airspace.polygon;
        let coords: Vec<_> = route_line.coords().collect();

        if coords.is_empty() {
            return Vec::new();
        }

        let first_inside = geo_polygon.contains(&Point::new(coords[0].x, coords[0].y));
        let last_inside = geo_polygon.contains(&Point::new(
            coords[coords.len() - 1].x,
            coords[coords.len() - 1].y,
        ));

        // Compute all boundary crossing points with their segment index
        let intersection_points = Self::compute_segment_intersections(route_line, geo_polygon);

        // Convert to geodesic distances
        let mut crossings: Vec<(Length, geo::Coord<f64>)> = intersection_points
            .into_iter()
            .map(|(seg_idx, coord)| {
                let dist =
                    geodesic_distance_to_intersection(seg_idx, &coord, route_line, segment_lengths);
                (dist, coord)
            })
            .collect();

        crossings.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate crossings within 1m (vertex-shared boundary segments)
        crossings.dedup_by(|a, b| (a.0 - b.0).abs() < Length::m(1.0));

        // Build the sequence of transitions:
        // If route starts inside, prepend entry at distance 0
        // Append sorted boundary crossings
        // If route ends inside, append exit at route end
        let mut transitions: Vec<(Length, geo::Coord<f64>)> = Vec::new();

        if first_inside {
            transitions.push((Length::m(0.0), *coords[0]));
        }

        transitions.extend(crossings);

        if last_inside {
            transitions.push((total_length, *coords[coords.len() - 1]));
        }

        // Pair transitions sequentially: (0,1), (2,3), ... each pair is one intersection
        let mut intersections = Vec::new();
        let mut i = 0;
        while i + 1 < transitions.len() {
            let (entry_dist, entry_coord) = transitions[i];
            let (exit_dist, exit_coord) = transitions[i + 1];

            intersections.push(AirspaceIntersection {
                airspace: Rc::clone(&airspace),
                // the leg distance is in NM too
                entry_distance: entry_dist.convert_to(LengthUnit::NauticalMiles),
                exit_distance: exit_dist.convert_to(LengthUnit::NauticalMiles),
                entry_point: Point::new(entry_coord.x, entry_coord.y),
                exit_point: Point::new(exit_coord.x, exit_coord.y),
            });

            i += 2;
        }
        // Odd leftover (tangent touch) is ignored

        intersections
    }

    /// Computes intersection points between route segments and polygon boundary.
    ///
    /// Returns `(segment_index, coord)` pairs for each intersection.
    fn compute_segment_intersections(
        route_line: &LineString<f64>,
        polygon: &geo::Polygon<f64>,
    ) -> Vec<(usize, geo::Coord<f64>)> {
        let mut intersections = Vec::new();
        // TODO: Are there any airspaces with holes inside?
        let boundary = polygon.exterior();

        for (seg_idx, route_segment) in route_line.lines().enumerate() {
            for boundary_segment in boundary.lines() {
                if let Some(intersection) =
                    geo::line_intersection::line_intersection(route_segment, boundary_segment)
                {
                    match intersection {
                        LineIntersection::SinglePoint { intersection, .. } => {
                            intersections.push((seg_idx, intersection));
                        }
                        LineIntersection::Collinear { intersection } => {
                            intersections.push((seg_idx, intersection.start));
                            intersections.push((seg_idx, intersection.end));
                        }
                    }
                }
            }
        }

        intersections
    }

    /// Computes the vertical profile points from the route's legs.
    ///
    /// The profile starts with the origin airport elevation, includes
    /// intermediate navaids at their leg's cruise level, and ends with the
    /// destination airport elevation.
    fn compute_profile(route: &Route) -> Vec<VerticalPoint> {
        let legs = route.legs();

        if legs.is_empty() {
            return Vec::new();
        }

        let mut profile = Vec::new();

        // First point: origin airport elevation at distance 0
        if let Some(origin) = route.origin() {
            profile.push(VerticalPoint::NavAid {
                level: origin.elevation,
                distance: Length::nm(0.0),
                navaid: NavAid::Airport(origin),
            });
        }

        // Intermediate and final points from accumulated leg totals
        let num_legs = legs.len();
        for (i, (leg, totals)) in legs.iter().zip(route.accumulate_legs(None)).enumerate() {
            let is_last = i == num_legs - 1;

            if is_last {
                // Last point: destination airport elevation
                if let Some(dest) = route.destination() {
                    profile.push(VerticalPoint::NavAid {
                        level: dest.elevation,
                        distance: *totals.dist(),
                        navaid: NavAid::Airport(dest),
                    });
                }
            } else if let Some(level) = leg.level() {
                // Intermediate point at the leg's cruise level
                profile.push(VerticalPoint::NavAid {
                    level: *level,
                    distance: *totals.dist(),
                    navaid: leg.to().clone(),
                });
            }
        }

        profile
    }

    /// Returns the vertical profile points.
    pub fn profile(&self) -> &[VerticalPoint] {
        &self.profile
    }

    /// Returns all airspace intersections, sorted by entry distance.
    pub fn intersections(&self) -> &[AirspaceIntersection] {
        &self.intersections
    }

    /// Returns the maximum level along the route.
    ///
    /// If the route contains any level measured in [AGL] or [pressure altitude] are ignored.
    ///
    /// [AGL]: VerticalDistance::Agl
    /// [pressure altitude]: VerticalDistance::PressureAltitude
    pub fn max_level(&self) -> Option<&VerticalDistance> {
        self.profile
            .iter()
            .map(|point| point.level())
            .filter(|level| {
                matches!(
                    level,
                    VerticalDistance::Fl(_)
                        | VerticalDistance::Msl(_)
                        | VerticalDistance::Altitude(_)
                        | VerticalDistance::Gnd
                        | VerticalDistance::Unlimited
                )
            })
            .max_by(|a, b| a.cmp(b))
    }

    /// Returns the number of airspace intersections.
    pub fn len(&self) -> usize {
        self.intersections.len()
    }

    /// Returns true if there are no airspace intersections.
    pub fn is_empty(&self) -> bool {
        self.intersections.is_empty()
    }
}

/// Computes the geodesic distance from the route start to an intersection point
/// on segment `seg_idx`.
///
/// Sums the geodesic lengths of all segments before `seg_idx`, then adds the
/// within-segment fraction (Euclidean `line_locate_point`, acceptable for short
/// individual segments) multiplied by the segment's geodesic length.
fn geodesic_distance_to_intersection(
    seg_idx: usize,
    coord: &geo::Coord<f64>,
    route_line: &LineString<f64>,
    segment_lengths: &[Length],
) -> Length {
    let prior: Length = segment_lengths[..seg_idx].iter().copied().sum();

    // Get the segment as a Line and compute the fraction along it
    let segment = route_line
        .lines()
        .nth(seg_idx)
        .expect("valid segment index");
    let point = Point::new(coord.x, coord.y);
    let fraction = segment.line_locate_point(&point).unwrap_or(0.0) as f32;

    prior + segment_lengths[seg_idx] * fraction
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nd::AirspaceClass;

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

    /// Helper: build segment_lengths and total_length for a route line.
    fn route_lengths(route_line: &LineString<f64>) -> (Vec<Length>, Length) {
        let segment_lengths: Vec<Length> = route_line
            .lines()
            .map(|line| {
                Length::m(Geodesic.distance(Point::from(line.start), Point::from(line.end)) as f32)
            })
            .collect();
        let total_length: Length = segment_lengths.iter().copied().sum();
        (segment_lengths, total_length)
    }

    #[test]
    fn empty_route_produces_empty_profile() {
        let nd = NavigationData::new();
        let route = Route::new();
        let profile = VerticalProfile::new(&route, &nd);
        assert!(profile.is_empty());
    }

    #[test]
    fn profile_finds_airspace_intersection() {
        use crate::nd::NavigationDataBuilder;

        // Create an airspace from lat 53-54, lon 9-10
        let airspace = Airspace {
            name: "Test TMA".to_string(),
            class: AirspaceClass::D,
            ceiling: VerticalDistance::Fl(65),
            floor: VerticalDistance::Msl(1500),
            polygon: {
                let coords: Vec<geo::Coord<f64>> = [
                    (53.0, 9.0),
                    (53.0, 10.0),
                    (54.0, 10.0),
                    (54.0, 9.0),
                    (53.0, 9.0),
                ]
                .iter()
                .map(|&(lat, lon)| geo::Coord { x: lon, y: lat })
                .collect();
                geo::Polygon::new(geo::LineString::from(coords), vec![])
            },
        };

        let mut builder = NavigationDataBuilder::new();
        builder.add_airspace(airspace);
        let nd = builder.build();

        // For this test, we verify the profile computation works with an empty route
        // A full integration test would require setting up waypoints in NavigationData
        let route = Route::new();
        let profile = VerticalProfile::new(&route, &nd);

        // Empty route should produce empty profile
        assert!(profile.is_empty());
    }

    #[test]
    fn route_starting_inside_airspace_has_boundary_exit() {
        //          9.7    9.99   10.2          10.70
        //  53.8     +------+------+--------------+
        //           |      |      |              |
        //           | CTR  |      |              |
        //  53.63    |      EDDH---+-exit-------EDHL
        //           |  Hamburg    |
        //  53.5     +-------------+
        //
        //  Entry = EDDH (route origin, inside CTR)
        //  Exit  = CTR east boundary (~lon 10.2)
        let ctr_hamburg = test_airspace(
            "CTR Hamburg",
            &[
                (53.5, 9.7),
                (53.5, 10.2),
                (53.8, 10.2),
                (53.8, 9.7),
                (53.5, 9.7),
            ],
        );

        let route_line = LineString::new(vec![
            geo::Coord { x: 9.99, y: 53.63 },  // EDDH (inside)
            geo::Coord { x: 10.70, y: 53.81 }, // EDHL (outside)
        ]);

        let (segment_lengths, total_length) = route_lengths(&route_line);

        let intersections = VerticalProfile::compute_intersections(
            ctr_hamburg,
            &route_line,
            &segment_lengths,
            total_length,
        );

        assert_eq!(intersections.len(), 1, "Should find one intersection");
        let intersection = &intersections[0];

        // Entry at EDDH (route origin)
        assert!(
            (intersection.entry_point().x() - 9.99).abs() < 0.01,
            "Entry should be at EDDH lon, got {}",
            intersection.entry_point().x()
        );
        assert!(
            (intersection.entry_point().y() - 53.63).abs() < 0.01,
            "Entry should be at EDDH lat, got {}",
            intersection.entry_point().y()
        );
        assert!(
            *intersection.entry_distance().value() < 0.01,
            "Entry distance should be 0"
        );

        // Exit on CTR east boundary
        assert!(
            (intersection.exit_point().x() - 10.2).abs() < 0.05,
            "Exit should be near CTR east boundary lon 10.2, got {}",
            intersection.exit_point().x()
        );
        assert!(
            *intersection.exit_distance().value() > 1.0,
            "Exit distance should be well past 0 nm, got {} nm",
            intersection.exit_distance().value()
        );
    }

    #[test]
    fn route_ending_inside_airspace_has_boundary_entry() {
        //          9.99          10.5    10.70  10.9
        //  53.95                  +--------+------+
        //                         |        |      |
        //                         |  CTR   |      |
        //  53.81                  |  Lueb. |      |
        //  53.63   EDDH-----------+-entry--EDHL   |
        //                         |               |
        //  53.7                   +---------------+
        //
        //  Entry = CTR west boundary (~lon 10.5)
        //  Exit  = EDHL (route destination, inside CTR)
        let ctr_luebeck = test_airspace(
            "CTR Luebeck",
            &[
                (53.7, 10.5),
                (53.7, 10.9),
                (53.95, 10.9),
                (53.95, 10.5),
                (53.7, 10.5),
            ],
        );

        let route_line = LineString::new(vec![
            geo::Coord { x: 9.99, y: 53.63 },  // EDDH (outside)
            geo::Coord { x: 10.70, y: 53.81 }, // EDHL (inside)
        ]);

        let (segment_lengths, total_length) = route_lengths(&route_line);

        let intersections = VerticalProfile::compute_intersections(
            ctr_luebeck,
            &route_line,
            &segment_lengths,
            total_length,
        );

        assert_eq!(intersections.len(), 1, "Should find one intersection");
        let intersection = &intersections[0];

        // Entry on CTR west boundary
        assert!(
            (intersection.entry_point().x() - 10.5).abs() < 0.05,
            "Entry should be near CTR west boundary lon 10.5, got {}",
            intersection.entry_point().x()
        );
        assert!(
            *intersection.entry_distance().value() > 1.0,
            "Entry distance should be well past 0 nm, got {} nm",
            intersection.entry_distance().value()
        );

        // Exit at EDHL (route destination)
        assert!(
            (intersection.exit_point().x() - 10.70).abs() < 0.01,
            "Exit should be at EDHL lon, got {}",
            intersection.exit_point().x()
        );
        assert!(
            (intersection.exit_point().y() - 53.81).abs() < 0.01,
            "Exit should be at EDHL lat, got {}",
            intersection.exit_point().y()
        );
    }

    #[test]
    fn cross_through_leg_finds_intersection() {
        //       8.0       9.0             10.0      11.0
        //  54.0            +---------------+
        //                  |               |
        //  53.5  A---------+-entry----exit-+---------B
        //                  |               |
        //  53.0            +---------------+
        //
        //  A = route start (outside, west)
        //  B = route end   (outside, east)
        //  Entry ~1/3 of route, exit ~2/3 of route
        let airspace = test_airspace(
            "Cross-Through Test",
            &[
                (53.0, 9.0),
                (53.0, 10.0),
                (54.0, 10.0),
                (54.0, 9.0),
                (53.0, 9.0),
            ],
        );

        let route_line = LineString::new(vec![
            geo::Coord { x: 8.0, y: 53.5 },
            geo::Coord { x: 11.0, y: 53.5 },
        ]);

        let (segment_lengths, total_length) = route_lengths(&route_line);

        let intersections = VerticalProfile::compute_intersections(
            airspace.clone(),
            &route_line,
            &segment_lengths,
            total_length,
        );

        assert_eq!(
            intersections.len(),
            1,
            "Should detect cross-through intersection"
        );
        let intersection = &intersections[0];

        let entry_fraction = 1.0 / 3.0;
        let exit_fraction = 2.0 / 3.0;

        // Entry near west boundary (lon 9.0)
        assert!(
            (intersection.entry_point().x() - 9.0).abs() < 0.01,
            "Entry longitude should be ~9.0, got {}",
            intersection.entry_point().x()
        );
        assert!(
            (intersection.entry_point().y() - 53.5).abs() < 0.01,
            "Entry latitude should be ~53.5, got {}",
            intersection.entry_point().y()
        );

        // Exit near east boundary (lon 10.0)
        assert!(
            (intersection.exit_point().x() - 10.0).abs() < 0.01,
            "Exit longitude should be ~10.0, got {}",
            intersection.exit_point().x()
        );
        assert!(
            (intersection.exit_point().y() - 53.5).abs() < 0.01,
            "Exit latitude should be ~53.5, got {}",
            intersection.exit_point().y()
        );

        // Distances: entry ~1/3, exit ~2/3 of total
        let expected_entry = total_length * entry_fraction as f32;
        let expected_exit = total_length * exit_fraction as f32;
        let tolerance = Length::nm(1.0);

        assert!(
            (*intersection.entry_distance() - expected_entry).abs() < tolerance,
            "Entry distance should be ~{}, got {}",
            expected_entry,
            intersection.entry_distance()
        );
        assert!(
            (*intersection.exit_distance() - expected_exit).abs() < tolerance,
            "Exit distance should be ~{}, got {}",
            expected_exit,
            intersection.exit_distance()
        );

        // Intersection length ~1/3 of total route
        let expected_length = expected_exit - expected_entry;
        assert!(
            (intersection.length() - expected_length).abs() < tolerance,
            "Intersection length should be ~{}, got {}",
            expected_length,
            intersection.length()
        );
    }
}
