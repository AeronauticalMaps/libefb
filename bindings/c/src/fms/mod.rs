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

use std::ffi::{c_char, CStr, CString};

use efb::fms::FMS;
use efb::fp::{FlightPlanning, FlightPlanningBuilder};
use efb::nd::{InputFormat, NavigationData};

use super::EfbRoute;

mod aircraft_builder;
mod flight_planning;
mod flight_planning_builder;

pub use aircraft_builder::*;
pub use flight_planning::*;
pub use flight_planning_builder::*;

/// The Flight Management System (FMS).
///
/// This type wraps the [FMS] which is the integral system of this library. The
/// FMS holds all information like the navigation data or the route.
pub struct EfbFMS {
    inner: FMS,
}

/// Creates and returns a new FMS.
///
/// # Safety
///
/// The caller is responsible to free the allocated FMS by calling efb_fms_free.
#[no_mangle]
pub unsafe extern "C" fn efb_fms_new() -> Box<EfbFMS> {
    let fms = EfbFMS { inner: FMS::new() };
    Box::new(fms)
}

/// Frees the memory of the allocated FMS.
#[no_mangle]
pub extern "C" fn efb_fms_free(fms: Option<Box<EfbFMS>>) {
    drop(fms);
}

/// Reads the string which is in the fmt into the navigation database.
///
/// # Safety
///
/// It is up to the caller to guarantee that `s` points to a valid string.
#[no_mangle]
pub unsafe extern "C" fn efb_fms_nd_read(fms: &mut EfbFMS, s: *const c_char, fmt: InputFormat) {
    // TODO: Shouldn't crash when passing the wrong format!
    if let Ok(s) = unsafe { CStr::from_ptr(s).to_str() } {
        let new_nd = match fmt {
            // TODO: This is quite inefficient. Change the reader function once
            //       OpenAir is parsed from an byte array too.
            InputFormat::Arinc424 => NavigationData::try_from_arinc424(s.as_bytes()),
            InputFormat::OpenAir => NavigationData::try_from_openair(s),
        };

        if let Ok(new_nd) = new_nd {
            let _ = fms.inner.modify_nd(|nd| nd.append(new_nd));
        }
    }
}

/// Decodes the route and enters it into the FMS.
///
/// # Safety
///
/// It is up to the caller to guarantee that `route` points to a valid string.
#[no_mangle]
pub unsafe extern "C" fn efb_fms_decode(fms: &mut EfbFMS, route: *const c_char) {
    if let Ok(route) = unsafe { CStr::from_ptr(route).to_str() } {
        let _ = fms.inner.decode(route.to_string());
    }
}

/// Returns a reference to the FMS route.
///
/// # Safety
///
/// It's up to the caller to unref the returned pointer.
#[no_mangle]
pub unsafe extern "C" fn efb_fms_route_ref(fms: &mut EfbFMS) -> Box<EfbRoute<'_>> {
    Box::new(EfbRoute::from(fms.inner.route()))
}

/// Decreases the reference count of the route.
#[no_mangle]
pub extern "C" fn efb_fms_route_unref(route: Option<Box<EfbRoute>>) {
    drop(route);
}

/// Returns the flight planning.
///
/// The planning is created by the builder returned by
/// [`efb_flight_planning_builder_new`].
#[no_mangle]
pub extern "C" fn efb_fms_flight_planning(fms: &EfbFMS) -> Option<&FlightPlanning> {
    fms.inner.flight_planning()
}

/// Sets the flight planning.
///
/// The planning is created by the builder returned by
/// [`efb_flight_planning_builder_new`].
#[no_mangle]
pub extern "C" fn efb_fms_set_flight_planning(fms: &mut EfbFMS, builder: &FlightPlanningBuilder) {
    // clone here to not mess with the builder's memory
    let _ = fms.inner.set_flight_planning(builder.clone());
}

/// Prints the route and planning of the FMS.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
#[no_mangle]
pub extern "C" fn efb_fms_print(fms: &mut EfbFMS, line_length: usize) -> *mut c_char {
    CString::new(fms.inner.print(line_length))
        .expect("Invalid FMS printer!")
        .into_raw()
}
