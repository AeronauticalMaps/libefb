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

mod aixm;
mod arinc424;
mod openair;

use geo::{Coord, Destination, Geodesic, Point};

use crate::measurements::{Angle, Length};

/// Number of points to interpolate per 90 degrees of arc.
const ARC_POINTS_PER_QUADRANT: usize = 6;

/// Interpolates the arc around `center` with `radius` from the bearing
/// `start` to the bearing `end`. The points exclude the start and include
/// the end of the arc.
fn interpolate_arc(
    center: Point<f64>,
    radius: Length,
    start: Angle,
    end: Angle,
    clockwise: bool,
) -> Vec<Coord<f64>> {
    // Calculate the angular sweep
    let sweep = calculate_arc_sweep(start, end, clockwise);
    let sweep_rad = sweep.to_si();
    let num_points = ((sweep_rad.abs() / std::f32::consts::FRAC_PI_2)
        * ARC_POINTS_PER_QUADRANT as f32)
        .ceil() as usize;
    let num_points = num_points.max(2);

    let mut coords = Vec::with_capacity(num_points);
    let radius_m = radius.to_si() as f64;
    let start_rad = start.to_si();

    for i in 1..=num_points {
        let fraction = i as f32 / num_points as f32;
        let bearing_deg = (start_rad + sweep_rad * fraction).to_degrees() as f64;

        let point = Geodesic.destination(center, bearing_deg, radius_m);
        coords.push(Coord {
            x: point.x(),
            y: point.y(),
        });
    }

    coords
}

/// Interpolates the closed ring of the circle around `center` with
/// `radius`.
fn interpolate_circle(center: Point<f64>, radius: Length) -> Vec<Coord<f64>> {
    let radius_m = radius.to_si() as f64;

    let num_points = ARC_POINTS_PER_QUADRANT * 4;
    let mut coords = Vec::with_capacity(num_points + 1);

    for i in 0..num_points {
        let bearing = Angle::t((i as f32) * 360.0 / (num_points as f32));
        let point = Geodesic.destination(center, *bearing.value() as f64, radius_m);
        coords.push(Coord {
            x: point.x(),
            y: point.y(),
        });
    }

    // Close the circle
    if let Some(first) = coords.first() {
        coords.push(*first);
    }

    coords
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
