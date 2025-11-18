// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Joe Pearson
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

use crate::{JsFuelType, JsLength, JsMass, JsVolume};

#[wasm_bindgen(js_name = AircraftBuilder)]
#[derive(Default)]
pub struct JsAircraftBuilder {
    inner: AircraftBuilder,
}

#[wasm_bindgen(js_class = AircraftBuilder)]
impl JsAircraftBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(setter)]
    pub fn set_registration(&mut self, registration: String) {
        self.inner.registration(registration);
    }

    #[wasm_bindgen(setter, js_name = icaoType)]
    pub fn set_icao_type(&mut self, icao_type: String) {
        self.inner.icao_type(icao_type);
    }

    #[wasm_bindgen(setter)]
    pub fn set_stations(&mut self, stations: Vec<JsStation>) {
        let stations: Vec<Station> = stations.into_iter().map(|s| s.into()).collect();
        self.inner.stations(stations);
    }

    #[wasm_bindgen(setter, js_name = emptyMass)]
    pub fn set_empty_mass(&mut self, empty_mass: JsMass) {
        self.inner.empty_mass(empty_mass.into());
    }

    #[wasm_bindgen(setter, js_name = emptyBalance)]
    pub fn set_empty_balance(&mut self, empty_balance: JsLength) {
        self.inner.empty_balance(empty_balance.into());
    }

    #[wasm_bindgen(setter, js_name = fuelType)]
    pub fn set_fuel_type(&mut self, fuel_type: JsFuelType) {
        self.inner.fuel_type(fuel_type.into());
    }

    #[wasm_bindgen(setter)]
    pub fn set_tanks(&mut self, tanks: Vec<JsFuelTank>) {
        let tanks: Vec<FuelTank> = tanks.into_iter().map(|t| t.into()).collect();
        self.inner.tanks(tanks);
    }

    #[wasm_bindgen(setter, js_name = cgEnvelope)]
    pub fn set_cg_envelope(&mut self, cg_envelope: Vec<JsCGLimit>) {
        let cg_envelope: Vec<CGLimit> = cg_envelope.into_iter().map(|c| c.into()).collect();
        self.inner.cg_envelope(cg_envelope);
    }

    #[wasm_bindgen(setter)]
    pub fn set_notes(&mut self, notes: String) {
        self.inner.notes(notes);
    }

    pub fn build(&self) -> Result<JsValue, JsError> {
        let ac = self.inner.build()?;
        let value = serde_wasm_bindgen::to_value(&ac)?;
        Ok(value)
    }
}

impl From<JsAircraftBuilder> for AircraftBuilder {
    fn from(value: JsAircraftBuilder) -> Self {
        value.inner
    }
}

impl From<AircraftBuilder> for JsAircraftBuilder {
    fn from(value: AircraftBuilder) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = Station)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct JsStation {
    inner: Station,
}

#[wasm_bindgen(js_class = Station)]
impl JsStation {
    #[wasm_bindgen(constructor)]
    pub fn new(arm: JsLength, description: Option<String>) -> Self {
        Self {
            inner: Station::new(arm.into(), description),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn arm(&self) -> JsLength {
        (*self.inner.arm()).into()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.inner.description().cloned()
    }
}

impl From<JsStation> for Station {
    fn from(value: JsStation) -> Self {
        value.inner
    }
}

impl From<Station> for JsStation {
    fn from(value: Station) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = FuelTank)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct JsFuelTank {
    inner: FuelTank,
}

#[wasm_bindgen(js_class = FuelTank)]
impl JsFuelTank {
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: JsVolume, arm: JsLength) -> Self {
        Self {
            inner: FuelTank::new(capacity.into(), arm.into()),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn capacity(&self) -> JsVolume {
        (*self.inner.capacity()).into()
    }

    #[wasm_bindgen(getter)]
    pub fn arm(&self) -> JsLength {
        (*self.inner.arm()).into()
    }
}

impl From<JsFuelTank> for FuelTank {
    fn from(value: JsFuelTank) -> Self {
        value.inner
    }
}

impl From<FuelTank> for JsFuelTank {
    fn from(value: FuelTank) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = CGLimit)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct JsCGLimit {
    inner: CGLimit,
}

#[wasm_bindgen(js_class = CGLimit)]
impl JsCGLimit {
    #[wasm_bindgen(constructor)]
    pub fn new(mass: JsMass, distance: JsLength) -> Self {
        Self {
            inner: CGLimit::new(mass.into(), distance.into()),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn mass(&self) -> JsMass {
        (*self.inner.mass()).into()
    }

    #[wasm_bindgen(getter)]
    pub fn distance(&self) -> JsLength {
        (*self.inner.distance()).into()
    }
}

impl From<JsCGLimit> for CGLimit {
    fn from(value: JsCGLimit) -> Self {
        value.inner
    }
}

impl From<CGLimit> for JsCGLimit {
    fn from(value: CGLimit) -> Self {
        Self { inner: value }
    }
}
