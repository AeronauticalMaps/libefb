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

use std::ffi::{c_char, CString};

use efb::measurements::{Angle, Duration, Length, Speed};
use efb::nd::Fix;
use efb::route::{Leg, LegFuel};
use efb::{Fuel, VerticalDistance, Wind};

/// Returns the ident from where the leg starts.
///
/// # Safety
///
/// The returned value needs to be freed by [`efb_string_free`].
#[no_mangle]
pub unsafe extern "C" fn efb_leg_get_from(leg: &Leg) -> *mut c_char {
    CString::new(leg.from().ident()).unwrap().into_raw()
}

/// Returns the ident to where the leg ends.
///
/// # Safety
///
/// The returned value needs to be freed by [`efb_string_free`].
#[no_mangle]
pub unsafe extern "C" fn efb_leg_get_to(leg: &Leg) -> *mut c_char {
    CString::new(leg.to().ident()).unwrap().into_raw()
}

/// Returns the leg's level or null if unknown.
#[no_mangle]
pub extern "C" fn efb_leg_get_level(leg: &Leg) -> Option<&VerticalDistance> {
    leg.level()
}

/// Returns the wind along the leg or null if unknown.
#[no_mangle]
pub extern "C" fn efb_leg_get_wind(leg: &Leg) -> Option<&Wind> {
    leg.wind()
}

/// Returns the leg's true airspeed or null if unknown.
#[no_mangle]
pub extern "C" fn efb_leg_get_tas(leg: &Leg) -> Option<&Speed> {
    leg.tas()
}

/// Returns the true heading considering the WCA or null if unknown.
#[no_mangle]
pub extern "C" fn efb_leg_get_heading(leg: &Leg) -> Option<&Angle> {
    leg.heading()
}

/// Returns the magnetic heading considering the variation at the start of the
/// leg or null if unknown.
#[no_mangle]
pub extern "C" fn efb_leg_get_mh(leg: &Leg) -> Option<&Angle> {
    leg.mh()
}

/// Returns the bearing between the two points.
#[no_mangle]
pub extern "C" fn efb_leg_get_bearing(leg: &Leg) -> &Angle {
    leg.bearing()
}

/// Returns the magnetic course taking the magnetic variation from the starting
/// point into consideration.
#[no_mangle]
pub extern "C" fn efb_leg_get_mc(leg: &Leg) -> &Angle {
    leg.mc()
}

/// Returns the distance between the leg's two points.
#[no_mangle]
pub extern "C" fn efb_leg_get_dist(leg: &Leg) -> &Length {
    leg.dist()
}

/// Returns the ground speed in knots or null if unknown.
#[no_mangle]
pub extern "C" fn efb_leg_get_gs(leg: &Leg) -> Option<&Speed> {
    leg.gs()
}

/// Returns the estimated time enroute the leg or null if unknown.
#[no_mangle]
pub extern "C" fn efb_leg_get_ete(leg: &Leg) -> Option<&Duration> {
    leg.ete()
}

/// Returns the climb fuel from a leg fuel breakdown, or null if the leg has no
/// climb phase.
#[no_mangle]
pub extern "C" fn efb_leg_fuel_climb(leg_fuel: &LegFuel) -> Option<&Fuel> {
    leg_fuel.climb()
}

/// Returns the cruise fuel from a leg fuel breakdown, or null if the leg has no
/// cruise phase.
#[no_mangle]
pub extern "C" fn efb_leg_fuel_cruise(leg_fuel: &LegFuel) -> Option<&Fuel> {
    leg_fuel.cruise()
}

/// Returns the descent fuel from a leg fuel breakdown, or null if the leg has
/// no descent phase.
#[no_mangle]
pub extern "C" fn efb_leg_fuel_descent(leg_fuel: &LegFuel) -> Option<&Fuel> {
    leg_fuel.descent()
}

/// Returns the total fuel from a leg fuel breakdown.
#[no_mangle]
pub extern "C" fn efb_leg_fuel_total(leg_fuel: &LegFuel) -> &Fuel {
    leg_fuel.total()
}
