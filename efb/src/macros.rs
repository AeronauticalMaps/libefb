// SPDX-License-Identifier: Apache-2.0
// Copyright 2024, 2026 Joe Pearson
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

/// Creates AvGas [`Fuel`] from the [`Volume`] at ISA conditions.
///
/// [`Fuel`]: crate::Fuel
/// [`Volume`]: crate::measurements::Volume
#[macro_export]
macro_rules! avgas {
    ($volume:expr) => {
        Fuel::from_volume($volume, FuelType::AvGas)
    };
}

/// Creates Diesel [`Fuel`] from the [`Volume`] at ISA conditions.
///
/// [`Fuel`]: crate::Fuel
/// [`Volume`]: crate::measurements::Volume
#[macro_export]
macro_rules! diesel {
    ($volume:expr) => {
        Fuel::from_volume($volume, FuelType::Diesel)
    };
}

/// Creates Jet-A [`Fuel`] from the [`Volume`] at ISA conditions.
///
/// [`Fuel`]: crate::Fuel
/// [`Volume`]: crate::measurements::Volume
#[macro_export]
macro_rules! jet_a {
    ($volume:expr) => {
        Fuel::from_volume($volume, FuelType::JetA)
    };
}

/// Creates a [`geo::Point<f64>`] from latitude and longitude.
///
/// Note: This macro accepts (latitude, longitude) but internally creates
/// the geo::Point with (longitude, latitude) to match geo's coordinate order.
#[macro_export]
macro_rules! coord {
    ($latitude:expr, $longitude:expr) => {
        geo::Point::new($longitude, $latitude)
    };
}

/// Creates a [`geo::Polygon<f64>`] containing the coordinates.
///
/// ```
/// use efb::polygon;
///
/// let p = polygon![(0.0, 0.0), (10.0, 10.0), (0.0, 10.0), (0.0, 0.0)];
/// ```
///
/// Note: Coordinates are specified as (latitude, longitude) but internally
/// converted to geo's (longitude, latitude) coordinate order.
#[macro_export]
macro_rules! polygon {
    ( $( ($lat:expr, $lon:expr) ),* $(,)? ) => {{
        geo::Polygon::new(
            geo::LineString::from(vec![ $( geo::Coord { x: $lon, y: $lat }, )* ]),
            vec![]
        )
    }};
}
