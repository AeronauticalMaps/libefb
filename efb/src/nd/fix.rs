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

use crate::MagneticVariation;
use geo::Point;

/// A fix location with coordinates.
pub trait Fix: Sized {
    fn ident(&self) -> String;

    fn coordinate(&self) -> Point<f64>;

    fn mag_var(&self) -> MagneticVariation {
        self.coordinate().into()
    }
}
