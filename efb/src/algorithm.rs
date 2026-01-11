// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Joe Pearson
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

//! Algorithms.

/// A point within a cartesian coordinate system.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Point {
    /// The x coordinate.
    pub x: f64,

    /// The y coordinate.
    pub y: f64,
}

type Line = (Point, Point);

/// Returns the winding number and which is 0 if the point `p` is outside the
/// polygon `v`.
///
/// This algorithm is based on [Dan Sunday][sunday]'s improved winding number algorithm.
///
/// [sunday]: https://web.archive.org/web/20130126163405/http://geomalgorithms.com/a03-_inclusion.html
pub fn winding_number(p: &Point, v: &[Point]) -> i32 {
    let mut wn = 0;

    for i in 0..(v.len() - 1) {
        if v[i].y <= p.y {
            if v[i + 1].y > p.y {
                // an upward crossing
                if is_left_of_line(p, &(v[i], v[i + 1])) > 0.0 {
                    wn += 1;
                }
            }
        } else if v[i + 1].y <= p.y {
            // a downward crossing
            if is_left_of_line(p, &(v[i], v[i + 1])) < 0.0 {
                wn -= 1;
            }
        }
    }

    wn
}

fn is_left_of_line(point: &Point, line: &Line) -> f64 {
    (line.1.x - line.0.x) * (point.y - line.0.y) - (point.x - line.0.x) * (line.1.y - line.0.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_is_left_of_line() {
        let line = (Point { x: 10.0, y: 10.0 }, Point { x: 10.0, y: 20.0 });
        let point = Point { x: 5.0, y: 15.0 };
        assert!(is_left_of_line(&point, &line) > 0.0);
    }

    #[test]
    fn point_is_right_of_line() {
        let line = (Point { x: 10.0, y: 10.0 }, Point { x: 10.0, y: 20.0 });
        let point = Point { x: 15.0, y: 15.0 };
        assert!(is_left_of_line(&point, &line) < 0.0);
    }

    #[test]
    fn point_is_on_line() {
        let line = (Point { x: 10.0, y: 10.0 }, Point { x: 10.0, y: 20.0 });
        let point = Point { x: 10.0, y: 15.0 };
        assert_eq!(is_left_of_line(&point, &line), 0.0);
    }
}
