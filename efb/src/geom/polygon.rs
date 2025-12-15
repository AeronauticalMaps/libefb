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

use std::ops::Index;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Coordinate;
use crate::algorithm;

/// A polygon spawned by coordinates.
#[derive(Clone, PartialEq, PartialOrd, Debug, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polygon {
    coords: Vec<Coordinate>,
}

impl Polygon {
    /// Constructs a new, empty Polygon.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a coordinate to the back of the coordinates.
    pub fn push(&mut self, coord: Coordinate) {
        self.coords.push(coord);
    }

    /// Returns `true` if the given point is within the polygon's area.
    pub fn contains(&self, point: &Coordinate) -> bool {
        algorithm::winding_number(
            &algorithm::Point {
                x: point.longitude,
                y: point.latitude,
            },
            &self
                .coords
                .iter()
                .map(|coord| algorithm::Point {
                    x: coord.longitude,
                    y: coord.latitude,
                })
                .collect::<Vec<algorithm::Point>>(),
        ) != 0
    }

    /// Consumes the Polygon, returning its inner vector of coordinates.
    pub fn into_inner(self) -> Vec<Coordinate> {
        self.coords
    }
}

impl From<Vec<Coordinate>> for Polygon {
    fn from(coords: Vec<Coordinate>) -> Self {
        Polygon { coords }
    }
}

impl Index<usize> for Polygon {
    type Output = Coordinate;

    fn index(&self, i: usize) -> &Self::Output {
        &self.coords[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_is_in_polygon() {
        let point = coord!(15.0, 15.0);
        let polygon = polygon![
            (10.0, 10.0),
            (20.0, 10.0),
            (20.0, 20.0),
            (10.0, 20.0),
            (10.0, 10.0)
        ];
        assert!(polygon.contains(&point));
    }

    #[test]
    fn point_is_not_in_polygon() {
        let point = coord!(20.0, 0.0);
        let polygon = polygon![
            (-10.0, 10.0),
            (10.0, 10.0),
            (10.0, -10.0),
            (-10.0, -10.0),
            (-10.0, 10.0)
        ];
        assert!(!polygon.contains(&point));
    }
}
