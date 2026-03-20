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

use efb::fp::FuelPlanning;
use efb::Fuel;

#[no_mangle]
pub extern "C" fn efb_fuel_planning_taxi(planning: &FuelPlanning) -> &Fuel {
    planning.taxi()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_trip(planning: &FuelPlanning) -> &Fuel {
    planning.trip()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_alternate(planning: &FuelPlanning) -> Option<&Fuel> {
    planning.alternate()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_reserve(planning: &FuelPlanning) -> &Fuel {
    planning.reserve()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_total(planning: &FuelPlanning) -> &Fuel {
    planning.total()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_min(planning: &FuelPlanning) -> &Fuel {
    planning.min()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_extra(planning: &FuelPlanning) -> Option<&Fuel> {
    planning.extra()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_on_ramp(planning: &FuelPlanning) -> &Fuel {
    planning.on_ramp()
}

#[no_mangle]
pub extern "C" fn efb_fuel_planning_after_landing(planning: &FuelPlanning) -> &Fuel {
    planning.after_landing()
}
