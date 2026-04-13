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

//! Encoding helpers for types that span multiple SQL columns.

use crate::VerticalDistance;

/// Returns the `kind` column value for a vertical distance.
pub fn vd_kind(vd: &VerticalDistance) -> &'static str {
    match vd {
        VerticalDistance::Agl(_) => "agl",
        VerticalDistance::Altitude(_) => "alt",
        VerticalDistance::PressureAltitude(_) => "pa",
        VerticalDistance::Fl(_) => "fl",
        VerticalDistance::Gnd => "gnd",
        VerticalDistance::Msl(_) => "msl",
        VerticalDistance::Unlimited => "unlimited",
    }
}

/// Returns the `value` column for a vertical distance, or `None` for unit
/// variants (`Gnd`, `Unlimited`) that carry no quantity.
pub fn vd_value(vd: &VerticalDistance) -> Option<i64> {
    match vd {
        VerticalDistance::Agl(n) => Some(*n as i64),
        VerticalDistance::Altitude(n) => Some(*n as i64),
        VerticalDistance::PressureAltitude(n) => Some(*n as i64),
        VerticalDistance::Fl(n) => Some(*n as i64),
        VerticalDistance::Msl(n) => Some(*n as i64),
        VerticalDistance::Gnd | VerticalDistance::Unlimited => None,
    }
}

/// Reconstructs a vertical distance from a `(kind, value)` column pair.
///
/// Returns `None` when `kind` is not recognised.
pub fn vd_from_row(kind: &str, value: Option<i64>) -> Option<VerticalDistance> {
    match (kind, value) {
        ("agl", Some(n)) => u16::try_from(n).ok().map(VerticalDistance::Agl),
        ("alt", Some(n)) => u16::try_from(n).ok().map(VerticalDistance::Altitude),
        ("pa", Some(n)) => i16::try_from(n)
            .ok()
            .map(VerticalDistance::PressureAltitude),
        ("fl", Some(n)) => u16::try_from(n).ok().map(VerticalDistance::Fl),
        ("msl", Some(n)) => u16::try_from(n).ok().map(VerticalDistance::Msl),
        ("gnd", _) => Some(VerticalDistance::Gnd),
        ("unlimited", _) => Some(VerticalDistance::Unlimited),
        _ => None,
    }
}
