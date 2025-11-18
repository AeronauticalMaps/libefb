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

use crate::{JsMass, JsVolume};

#[wasm_bindgen(js_name = Fuel)]
pub struct JsFuel {
    inner: Fuel,
}

#[wasm_bindgen(js_class = Fuel)]
impl JsFuel {
    #[wasm_bindgen(constructor)]
    pub fn new(mass: JsMass, fuel_type: JsFuelType) -> Self {
        Self {
            inner: Fuel::new(mass.into(), fuel_type.into()),
        }
    }

    #[wasm_bindgen(js_name = fromVolume)]
    pub fn from_volume(volume: JsVolume, fuel_type: JsValue) -> Result<Self, JsError> {
        let fuel_type: FuelType = serde_wasm_bindgen::from_value(fuel_type)?;

        Ok(Self {
            inner: Fuel::from_volume(volume.into(), fuel_type),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn volume(&self) -> Result<JsValue, JsValue> {
        let v = self.inner.volume();
        Ok(serde_wasm_bindgen::to_value(&v)?)
    }
}

impl From<JsFuel> for Fuel {
    fn from(value: JsFuel) -> Self {
        value.inner
    }
}

#[wasm_bindgen(js_name = FuelType)]
pub struct JsFuelType {
    inner: FuelType,
}

#[wasm_bindgen(js_class = FuelType)]
impl JsFuelType {
    #[wasm_bindgen(constructor)]
    pub fn new(fuel_type: String) -> Result<Self, JsError> {
        let inner = match fuel_type.as_ref() {
            "AvGas" => FuelType::AvGas,
            "Diesel" => FuelType::Diesel,
            "JetA" => FuelType::JetA,
            _ => return Err(JsError::new(&format!("invalid fuel type: {fuel_type}"))),
        };

        Ok(Self { inner })
    }

    #[wasm_bindgen(js_name = avGas)]
    pub fn av_gas() -> Self {
        Self {
            inner: FuelType::AvGas,
        }
    }

    pub fn diesel() -> Self {
        Self {
            inner: FuelType::Diesel,
        }
    }

    #[wasm_bindgen(js_name = jetA)]
    pub fn jet_a() -> Self {
        Self {
            inner: FuelType::JetA,
        }
    }
}

impl From<JsFuelType> for FuelType {
    fn from(value: JsFuelType) -> Self {
        value.inner
    }
}

impl From<FuelType> for JsFuelType {
    fn from(value: FuelType) -> Self {
        Self { inner: value }
    }
}
