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

use std::ffi::{c_char, CString};
use std::ptr::null_mut;

use efb::aircraft::Station;
use efb::measurements::Length;

/// Returns the stations arm in reference to the aircraft's datum.
#[no_mangle]
pub extern "C" fn efb_station_arm(station: &Station) -> &Length {
    station.arm()
}

/// Returns the stations description or null if undefined.
///
/// # Safety
///
/// The returned value, if not null, needs to be freed by [`efb_string_free`].
#[no_mangle]
pub extern "C" fn efb_station_description(station: &Station) -> *mut c_char {
    match station.description() {
        Some(description) => CString::new(description.clone())
            .expect("Invalid station description!")
            .into_raw(),
        None => null_mut::<c_char>(),
    }
}
