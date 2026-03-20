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

use efb::prelude::*;
use wasm_bindgen::prelude::*;

use crate::{JsAircraftBuilder, JsFuel, JsMass, JsTemperature};

#[wasm_bindgen(js_name = FlightPlanningBuilder)]
#[derive(Default)]
pub struct JsFlightPlanningBuilder {
    inner: FlightPlanningBuilder,
}

#[wasm_bindgen(js_class = FlightPlanningBuilder)]
impl JsFlightPlanningBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(setter)]
    pub fn set_aircraft(&mut self, aircraft: JsAircraftBuilder) -> Result<(), JsError> {
        let ac = AircraftBuilder::from(aircraft).build()?;
        self.inner.aircraft(ac);
        Ok(())
    }

    #[wasm_bindgen(setter)]
    pub fn set_mass(&mut self, mass: Vec<JsMass>) -> Result<(), JsError> {
        let mass: Vec<Mass> = mass.into_iter().map(|m| m.into()).collect();
        self.inner.mass(mass);
        Ok(())
    }

    #[wasm_bindgen(setter)]
    pub fn set_policy(&mut self, policy: JsFuelPolicy) {
        self.inner.policy(policy.into());
    }

    #[wasm_bindgen(setter)]
    pub fn set_taxi(&mut self, taxi: JsFuel) {
        self.inner.taxi(taxi.into());
    }

    #[wasm_bindgen(setter)]
    pub fn set_reserve(&mut self, reserve: JsReserve) {
        self.inner.reserve(reserve.into());
    }

    #[wasm_bindgen(setter, js_name = takeoffPerf)]
    pub fn set_takeoff_perf(&mut self, perf: JsTakeoffLandingPerformance) {
        self.inner.takeoff_perf(perf.into());
    }

    #[wasm_bindgen(setter, js_name = climbPerf)]
    pub fn set_climb_perf(&mut self, perf: JsClimbDescentPerformance) {
        self.inner.climb_perf(perf.into());
    }

    #[wasm_bindgen(setter)]
    pub fn set_perf(&mut self, perf: JsPerformance) {
        self.inner.perf(perf.into());
    }

    #[wasm_bindgen(setter, js_name = descentPerf)]
    pub fn set_descent_perf(&mut self, perf: JsClimbDescentPerformance) {
        self.inner.descent_perf(perf.into());
    }

    #[wasm_bindgen(setter, js_name = landingPerf)]
    pub fn set_landing_perf(&mut self, perf: JsTakeoffLandingPerformance) {
        self.inner.landing_perf(perf.into());
    }

    #[wasm_bindgen(setter, js_name = originRWYCC)]
    pub fn set_origin_rwycc(&mut self, rwycc: u8) -> Result<(), JsError> {
        self.inner.origin_rwycc(rwycc.try_into()?);
        Ok(())
    }

    #[wasm_bindgen(setter, js_name = originTemperature)]
    pub fn set_origin_temperature(&mut self, temperature: JsTemperature) {
        self.inner.origin_temperature(temperature.into());
    }

    #[wasm_bindgen(setter, js_name = destinationRWYCC)]
    pub fn set_destination_rwycc(&mut self, rwycc: u8) -> Result<(), JsError> {
        self.inner.destination_rwycc(rwycc.try_into()?);
        Ok(())
    }

    #[wasm_bindgen(setter, js_name = destinationTemperature)]
    pub fn set_destination_temperature(&mut self, temperature: JsTemperature) {
        self.inner.destination_temperature(temperature.into());
    }
}

impl From<JsFlightPlanningBuilder> for FlightPlanningBuilder {
    fn from(value: JsFlightPlanningBuilder) -> Self {
        value.inner
    }
}

////////////////////////////////////////////////////////////////////////////////
// Fuel planning
////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = FuelPolicy)]
pub struct JsFuelPolicy {
    inner: FuelPolicy,
}

#[wasm_bindgen(js_class = FuelPolicy)]
impl JsFuelPolicy {
    #[wasm_bindgen(constructor)]
    pub fn new(policy: String, fuel: Option<JsFuel>) -> Result<Self, JsError> {
        let inner = match (policy.as_str(), fuel) {
            ("MinimumFuel", _) => Ok(FuelPolicy::MinimumFuel),
            ("MaximumFuel", _) => Ok(FuelPolicy::MaximumFuel),
            ("ManualFuel", Some(fuel)) => Ok(FuelPolicy::ManualFuel(fuel.into())),
            ("FuelAtLanding", Some(fuel)) => Ok(FuelPolicy::FuelAtLanding(fuel.into())),
            ("ExtraFuel", Some(fuel)) => Ok(FuelPolicy::ExtraFuel(fuel.into())),
            _ => Err(JsError::new("invalid fuel policy or fuel undefined")),
        }?;

        Ok(Self { inner })
    }
}

impl From<JsFuelPolicy> for FuelPolicy {
    fn from(value: JsFuelPolicy) -> Self {
        value.inner
    }
}

#[wasm_bindgen(js_name = Reserve)]
pub struct JsReserve {
    inner: Reserve,
}

#[wasm_bindgen(js_class = Reserve)]
impl JsReserve {
    pub fn manual(duration: u32) -> Self {
        Self {
            inner: Reserve::Manual(Duration::s(duration)),
        }
    }
}

impl From<JsReserve> for Reserve {
    fn from(value: JsReserve) -> Self {
        value.inner
    }
}

////////////////////////////////////////////////////////////////////////////////
// Cruise performance
////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = Performance)]
#[derive(Debug, Clone)]
pub struct JsPerformance {
    inner: Performance,
}

impl From<JsPerformance> for Performance {
    fn from(value: JsPerformance) -> Self {
        value.inner
    }
}

impl From<Performance> for JsPerformance {
    fn from(value: Performance) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Climb & Descent performance
////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = ClimbDescentPerformance)]
#[derive(Debug, Clone)]
pub struct JsClimbDescentPerformance {
    inner: ClimbDescentPerformance,
}

impl From<JsClimbDescentPerformance> for ClimbDescentPerformance {
    fn from(value: JsClimbDescentPerformance) -> Self {
        value.inner
    }
}

impl From<ClimbDescentPerformance> for JsClimbDescentPerformance {
    fn from(value: ClimbDescentPerformance) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Leg performance (JS duck-typed object)
////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "{ cruise?: Performance, climb?: ClimbDescentPerformance, descent?: ClimbDescentPerformance }"
    )]
    pub type JsLegPerformance;

    #[wasm_bindgen(method, getter, structural)]
    fn cruise(this: &JsLegPerformance) -> Option<JsPerformance>;

    #[wasm_bindgen(method, getter, structural)]
    fn climb(this: &JsLegPerformance) -> Option<JsClimbDescentPerformance>;

    #[wasm_bindgen(method, getter, structural)]
    fn descent(this: &JsLegPerformance) -> Option<JsClimbDescentPerformance>;
}

pub struct OwnedLegPerformance {
    cruise: Option<Performance>,
    climb: Option<ClimbDescentPerformance>,
    descent: Option<ClimbDescentPerformance>,
}

impl OwnedLegPerformance {
    /// Returns a borrowed [`LegPerformance`] referencing the owned data.
    pub fn as_leg_perf(&self) -> LegPerformance<'_> {
        LegPerformance::new(
            self.cruise.as_ref(),
            self.climb.as_ref(),
            self.descent.as_ref(),
        )
    }

    pub fn climb(&self) -> Option<&ClimbDescentPerformance> {
        self.climb.as_ref()
    }

    pub fn descent(&self) -> Option<&ClimbDescentPerformance> {
        self.descent.as_ref()
    }
}

impl From<JsLegPerformance> for OwnedLegPerformance {
    fn from(js: JsLegPerformance) -> Self {
        Self {
            cruise: js.cruise().map(|p| p.clone().into()),
            climb: js.climb().map(|c| c.clone().into()),
            descent: js.descent().map(|d| d.clone().into()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Takeoff & Landing performance
////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = TakeoffLandingPerformance)]
#[derive(Debug, Clone)]
pub struct JsTakeoffLandingPerformance {
    inner: TakeoffLandingPerformance,
}

impl From<JsTakeoffLandingPerformance> for TakeoffLandingPerformance {
    fn from(value: JsTakeoffLandingPerformance) -> Self {
        value.inner
    }
}

impl From<TakeoffLandingPerformance> for JsTakeoffLandingPerformance {
    fn from(value: TakeoffLandingPerformance) -> Self {
        Self { inner: value }
    }
}
