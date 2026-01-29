// SPDX-License-Identifier: Apache-2.0
// Copyright 2025, 2026 Joe Pearson
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

use geo::BoundingRect;
use geojson::{Feature, GeoJson, Geometry, Value};

use super::geom::rect_to_bbox;
use crate::nd::Fix;
use crate::route::Route;

impl Route {
    /// Returns the route's legs as GeoJSON with a line string geometry.
    #[cfg_attr(docsrs, doc(cfg(feature = "geojson")))]
    pub fn to_geojson(&self) -> GeoJson {
        let legs = self.legs();
        let mut coords: Vec<geo::Coord<f64>> = Vec::with_capacity(legs.len() + 1);

        if let Some(origin) = legs.first() {
            coords.push(origin.from().coordinate().into());
        }

        for leg in legs {
            coords.push(leg.to().coordinate().into());
        }

        let line = geo::LineString::from(coords);

        GeoJson::Feature(Feature {
            bbox: line.bounding_rect().map(rect_to_bbox),
            geometry: Some(Geometry::new(Value::from(&line))),
            id: None,
            properties: None,
            foreign_members: None,
        })
    }
}
