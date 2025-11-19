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

//! Core aviation related types and utilities.
//!
//! This module provides fundamental types used throughout the library for
//! representing aviation-specific concepts such as fuel, wind, vertical
//! distances, and magnetic variation.

mod fuel;
mod mag_var;
mod vertical_distance;
mod wind;

pub use fuel::*;
pub use mag_var::*;
pub use vertical_distance::VerticalDistance;
pub use wind::*;
