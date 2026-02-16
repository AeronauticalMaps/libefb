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
use arinc424::records::ControlledAirspace;
use geo::{Bearing, Destination, Geodesic, Point};

use crate::measurements::{Angle, Length};
use crate::nd::{Airspace, AirspaceClass};
use crate::VerticalDistance;

/// Number of points to interpolate per 90 degrees of arc.
const ARC_POINTS_PER_QUADRANT: usize = 6;

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

/// Builder for constructing an [Airspace] from ARINC 424 controlled airspace records.
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
    /// Adds a boundary record to the builder.
    pub fn add_record(&mut self, record: ControlledAirspace) -> Result<(), arinc424::Error> {
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

        // Parse arc parameters if present
        let arc_center = match (record.arc_origin_latitude, record.arc_origin_longitude) {
            (Some(lat), Some(lon)) => {
                // geo uses (x, y) = (longitude, latitude)
                Some(Point::new(lon.as_decimal()?, lat.as_decimal()?))
            }
            _ => None,
        };

        let arc_radius = record
            .arc_dist
            .map(|d| d.dist())
            .transpose()?
            .map(Length::nm);

        // Add segment
        self.segments.push(BoundarySegment {
            path: record.bdry_via.path,
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
            class: self.class.unwrap_or(AirspaceClass::G),
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
    fn build_circle(&self, segment: &BoundarySegment) -> Result<geo::Polygon<f64>, arinc424::Error> {
        let center = segment.end_point;
        let radius_m = segment.arc_radius.map(|r| r.to_si()).unwrap_or(0.0) as f64;

        let num_points = ARC_POINTS_PER_QUADRANT * 4;
        let mut coords = Vec::with_capacity(num_points + 1);

        for i in 0..num_points {
            let bearing = Angle::t((i as f32) * 360.0 / (num_points as f32));
            let point = Geodesic.destination(center, *bearing.value() as f64, radius_m);
            coords.push(geo::Coord {
                x: point.x(),
                y: point.y(),
            });
        }

        // Close the circle
        if let Some(first) = coords.first() {
            coords.push(*first);
        }

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

        // Calculate the angular sweep
        let sweep = calculate_arc_sweep(start_bearing, end_bearing, clockwise);
        let sweep_rad = sweep.to_si();
        let num_points = ((sweep_rad.abs() / std::f32::consts::FRAC_PI_2)
            * ARC_POINTS_PER_QUADRANT as f32)
            .ceil() as usize;
        let num_points = num_points.max(2);

        let mut coords = Vec::with_capacity(num_points);
        let radius_m = radius.to_si() as f64;
        let start_rad = start_bearing.to_si();

        for i in 1..=num_points {
            let fraction = i as f32 / num_points as f32;
            let bearing_deg = (start_rad + sweep_rad * fraction).to_degrees() as f64;

            let point = Geodesic.destination(center, bearing_deg, radius_m);
            coords.push(geo::Coord {
                x: point.x(),
                y: point.y(),
            });
        }

        Ok(coords)
    }
}

/// Calculates the angular sweep for an arc.
///
/// Returns the signed sweep angle from `start` to `end`,
/// going in the specified direction (clockwise = positive).
fn calculate_arc_sweep(start: Angle, end: Angle, clockwise: bool) -> Angle {
    let mut diff = end.value() - start.value();

    if clockwise {
        // For clockwise, we want a positive sweep
        if diff <= 0.0 {
            diff += 360.0;
        }
    } else {
        // For counter-clockwise, we want a negative sweep
        if diff >= 0.0 {
            diff -= 360.0;
        }
    }

    Angle::rad(diff.to_radians())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_arc_sweep_clockwise() {
        // 0° to 90° clockwise = 90°
        let sweep = calculate_arc_sweep(Angle::t(0.0), Angle::t(90.0), true);
        assert!((sweep.to_si().to_degrees() - 90.0).abs() < 0.001);

        // 90° to 0° clockwise = 270°
        let sweep = calculate_arc_sweep(Angle::t(90.0), Angle::t(0.0), true);
        assert!((sweep.to_si().to_degrees() - 270.0).abs() < 0.001);

        // 350° to 10° clockwise = 20°
        let sweep = calculate_arc_sweep(Angle::t(350.0), Angle::t(10.0), true);
        assert!((sweep.to_si().to_degrees() - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_arc_sweep_counterclockwise() {
        // 90° to 0° counter-clockwise = -90°
        let sweep = calculate_arc_sweep(Angle::t(90.0), Angle::t(0.0), false);
        assert!((sweep.to_si().to_degrees() - (-90.0)).abs() < 0.001);

        // 0° to 90° counter-clockwise = -270°
        let sweep = calculate_arc_sweep(Angle::t(0.0), Angle::t(90.0), false);
        assert!((sweep.to_si().to_degrees() - (-270.0)).abs() < 0.001);
    }
}
