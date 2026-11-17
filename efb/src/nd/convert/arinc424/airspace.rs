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

//! Builder for constructing airspace polygons from ARINC 424 boundary records.

use arinc424::fields::BoundaryPath;
use arinc424::records::{ControlledAirspace, RestrictiveAirspace};
use geo::{Bearing, Geodesic, Point};

use super::fields::parse_classification;
use crate::measurements::{Angle, Length};
use crate::nd::convert::{interpolate_arc, interpolate_circle};
use crate::nd::{Airspace, AirspaceClassification, AirspaceType};
use crate::VerticalDistance;

/// A boundary segment representing the path from one point to another.
#[derive(Debug)]
struct BoundarySegment {
    /// The path type for this segment.
    path: BoundaryPath,
    /// The endpoint of this segment (lon/lat as geo::Point).
    end_point: Point<f64>,
    /// Arc center point (for arc segments).
    arc_center: Option<Point<f64>>,
    /// Arc radius (for arc segments).
    arc_radius: Option<Length>,
}

/// Builder for constructing an [Airspace] from ARINC 424 airspace records.
///
/// ARINC 424 airspaces are defined as a sequence of records, each describing a
/// boundary segment. This builder accumulates segments and converts them into
/// a polygon when finalized.
#[derive(Debug, Default)]
pub struct AirspaceBuilder {
    name: Option<String>,
    airspace_type: Option<AirspaceType>,
    classification: Option<AirspaceClassification>,
    ceiling: Option<VerticalDistance>,
    floor: Option<VerticalDistance>,
    segments: Vec<BoundarySegment>,
    start_point: Option<Point<f64>>,
}

impl AirspaceBuilder {
    /// Adds a controlled airspace boundary record to the builder.
    pub fn add_controlled_record(
        &mut self,
        record: ControlledAirspace,
    ) -> Result<(), arinc424::Error> {
        let coord = match (record.latitude, record.longitude) {
            (Some(lat), Some(lon)) => {
                // geo uses (x, y) = (longitude, latitude)
                Some(Point::new(lon.as_decimal()?, lat.as_decimal()?))
            }
            _ => None,
        };

        // First record initializes metadata and starting point
        if self.start_point.is_none() {
            self.start_point = coord;
            self.name = record.arsp_name.map(|n| n.to_string());
            self.airspace_type = Some(record.arsp_type.into());
            self.classification =
                parse_classification(record.arsp_type, record.arsp_class.as_ref());
            self.ceiling = record.upper_limit.map(Into::into);
            self.floor = record.lower_limit.map(Into::into);
        }

        self.add_segment(
            coord,
            record.bdry_via.path,
            record.arc_origin_latitude,
            record.arc_origin_longitude,
            record.arc_dist,
        )
    }

    /// Adds a restrictive airspace boundary record to the builder.
    pub fn add_restrictive_record(
        &mut self,
        record: RestrictiveAirspace,
    ) -> Result<(), arinc424::Error> {
        let coord = match (record.latitude, record.longitude) {
            (Some(lat), Some(lon)) => Some(Point::new(lon.as_decimal()?, lat.as_decimal()?)),
            _ => None,
        };

        // First record initializes metadata and starting point
        if self.start_point.is_none() {
            self.start_point = coord;
            self.name = record.arsp_name.map(|n| n.to_string());
            self.airspace_type = Some(record.restrictive_type.into());
            self.classification = None;
            self.ceiling = record.upper_limit.map(Into::into);
            self.floor = record.lower_limit.map(Into::into);
        }

        self.add_segment(
            coord,
            record.bdry_via.path,
            record.arc_origin_latitude,
            record.arc_origin_longitude,
            record.arc_dist,
        )
    }

    fn add_segment(
        &mut self,
        coord: Option<Point<f64>>,
        path: BoundaryPath,
        arc_origin_lat: Option<arinc424::fields::Latitude<'_>>,
        arc_origin_lon: Option<arinc424::fields::Longitude<'_>>,
        arc_dist: Option<arinc424::fields::ArcDistance<'_>>,
    ) -> Result<(), arinc424::Error> {
        let arc_center = match (arc_origin_lat, arc_origin_lon) {
            (Some(lat), Some(lon)) => Some(Point::new(lon.as_decimal()?, lat.as_decimal()?)),
            _ => None,
        };

        let arc_radius = arc_dist.map(|d| d.dist()).transpose()?.map(Length::nm);

        self.segments.push(BoundarySegment {
            path,
            end_point: coord
                .or(arc_center)
                .expect("record should either have coordinates or arc center"),
            arc_center,
            arc_radius,
        });

        Ok(())
    }

    /// Builds the airspace from accumulated segments.
    pub fn build(self) -> Result<Airspace, arinc424::Error> {
        let polygon = self.build_polygon()?;

        Ok(Airspace {
            name: self.name.unwrap_or_default(),
            airspace_type: self.airspace_type.unwrap_or(AirspaceType::CTA),
            classification: self.classification,
            ceiling: self.ceiling.unwrap_or(VerticalDistance::Unlimited),
            floor: self.floor.unwrap_or(VerticalDistance::Gnd),
            polygon,
        })
    }

    /// Builds the polygon from boundary segments.
    fn build_polygon(&self) -> Result<geo::Polygon<f64>, arinc424::Error> {
        let mut coords: Vec<geo::Coord<f64>> = Vec::new();

        // Handle special case: circle (single segment with Circle path)
        if self.segments.len() == 1 && self.segments[0].path == BoundaryPath::Circle {
            return self.build_circle(&self.segments[0]);
        }

        // Process each segment
        for (i, segment) in self.segments.iter().enumerate() {
            let prev_point = if i == 0 {
                // For first segment, previous point is the start point
                self.start_point.unwrap_or(segment.end_point)
            } else {
                self.segments[i - 1].end_point
            };

            match segment.path {
                BoundaryPath::Circle => {
                    // Circle in middle of sequence is unusual, treat as endpoint
                    coords.push(geo::Coord {
                        x: segment.end_point.x(),
                        y: segment.end_point.y(),
                    });
                }
                BoundaryPath::GreatCircle | BoundaryPath::RhumbLine => {
                    // Direct path - just add the endpoint
                    coords.push(geo::Coord {
                        x: segment.end_point.x(),
                        y: segment.end_point.y(),
                    });
                }
                BoundaryPath::ClockwiseArc => {
                    let arc_coords = self.interpolate_arc(prev_point, segment, true)?;
                    coords.extend(arc_coords);
                }
                BoundaryPath::CounterClockwiseArc => {
                    let arc_coords = self.interpolate_arc(prev_point, segment, false)?;
                    coords.extend(arc_coords);
                }
            }
        }

        // Close the polygon by adding start point if not already closed
        if let (Some(first), Some(last)) = (coords.first(), coords.last()) {
            if first != last {
                coords.push(*first);
            }
        }

        Ok(geo::Polygon::new(geo::LineString::from(coords), vec![]))
    }

    /// Builds a circle polygon from a circle segment.
    fn build_circle(
        &self,
        segment: &BoundarySegment,
    ) -> Result<geo::Polygon<f64>, arinc424::Error> {
        let radius = segment.arc_radius.unwrap_or(Length::m(0.0));
        let coords = interpolate_circle(segment.end_point, radius);

        Ok(geo::Polygon::new(geo::LineString::from(coords), vec![]))
    }

    /// Interpolates points along an arc.
    ///
    /// # Arguments
    /// * `start` - Starting point of the arc
    /// * `segment` - The boundary segment containing arc parameters
    /// * `clockwise` - True for clockwise arc, false for counter-clockwise
    fn interpolate_arc(
        &self,
        start: Point<f64>,
        segment: &BoundarySegment,
        clockwise: bool,
    ) -> Result<Vec<geo::Coord<f64>>, arinc424::Error> {
        let (Some(center), Some(radius)) = (segment.arc_center, segment.arc_radius) else {
            // No arc center - fall back to direct line
            return Ok(vec![geo::Coord {
                x: segment.end_point.x(),
                y: segment.end_point.y(),
            }]);
        };

        // Calculate bearings from center to start and end points
        let start_bearing = Angle::t(Geodesic.bearing(center, start) as f32);
        let end_bearing = Angle::t(Geodesic.bearing(center, segment.end_point) as f32);

        Ok(interpolate_arc(
            center,
            radius,
            start_bearing,
            end_bearing,
            clockwise,
        ))
    }
}
