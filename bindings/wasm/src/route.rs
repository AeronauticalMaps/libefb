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

use std::cell::RefCell;
use std::rc::Rc;

use efb::nd::Fix;
use efb::prelude::*;
use efb::route::{Leg, TotalsToLeg};
use serde::ser::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

use crate::JsPerformance;

#[wasm_bindgen(js_name = Leg)]
pub struct JsLeg {
    inner: Leg,
}

#[wasm_bindgen(js_class = Leg)]
impl JsLeg {
    #[wasm_bindgen(getter)]
    pub fn from(&self) -> String {
        self.inner.from().ident()
    }

    #[wasm_bindgen(getter)]
    pub fn to(&self) -> String {
        self.inner.to().ident()
    }

    #[wasm_bindgen(getter)]
    pub fn level(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.level()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn tas(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.tas()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn wind(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.wind()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn mc(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.mc()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn mh(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.mh()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn dist(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.dist()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn gs(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.gs()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn ete(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.ete()).unwrap_or_default()
    }

    pub fn fuel(&self, perf: &JsPerformance) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.fuel(&perf.clone().into())).unwrap_or_default()
    }
}

#[wasm_bindgen(js_name = Route)]
pub struct JsRoute {
    inner: Rc<RefCell<FMS>>,
}

#[wasm_bindgen(js_class = Route)]
impl JsRoute {
    #[wasm_bindgen(getter)]
    pub fn origin(&self) -> JsValue {
        let fms = self.inner.borrow();
        serde_wasm_bindgen::to_value(&fms.route().origin()).unwrap_or_default()
    }

    #[wasm_bindgen(getter, js_name = takeoffRwy)]
    pub fn takeoff_rwy(&self) -> JsValue {
        let fms = self.inner.borrow();
        serde_wasm_bindgen::to_value(&fms.route().takeoff_rwy()).unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn destination(&self) -> JsValue {
        let fms = self.inner.borrow();
        serde_wasm_bindgen::to_value(&fms.route().destination()).unwrap_or_default()
    }

    #[wasm_bindgen(getter, js_name = landingRwy)]
    pub fn landing_rwy(&self) -> JsValue {
        let fms = self.inner.borrow();
        serde_wasm_bindgen::to_value(&fms.route().landing_rwy()).unwrap_or_default()
    }

    pub fn legs(&self) -> Vec<JsLeg> {
        let fms = self.inner.borrow();
        fms.route()
            .legs()
            .iter()
            .cloned()
            .map(|leg| JsLeg { inner: leg })
            .collect()
    }

    #[wasm_bindgen(js_name = accumulateLegs)]
    pub fn accumulate_legs(&self, perf: Option<JsPerformance>) -> JsValue {
        let fms = self.inner.borrow();
        let perf = perf.clone().map(|js_perf| Performance::from(js_perf));
        let totals: Vec<TotalsToLeg> = fms.route().accumulate_legs(perf.as_ref()).collect();
        serde_wasm_bindgen::to_value(&totals).unwrap_or_default()
    }

    pub fn totals(&self, perf: Option<JsPerformance>) -> JsValue {
        let fms = self.inner.borrow();
        let perf = perf.clone().map(|js_perf| Performance::from(js_perf));
        let totals = fms.route().totals(perf.as_ref());
        serde_wasm_bindgen::to_value(&totals).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = toGeojson)]
    pub fn to_geojson(&self) -> Result<JsValue, JsValue> {
        let fms = self.inner.borrow();
        let value = fms.route().to_geojson();
        let serializer = Serializer::new().serialize_maps_as_objects(true);
        Ok(value.serialize(&serializer)?)
    }
}

impl From<&Rc<RefCell<FMS>>> for JsRoute {
    fn from(value: &Rc<RefCell<FMS>>) -> Self {
        Self {
            inner: Rc::clone(value),
        }
    }
}
