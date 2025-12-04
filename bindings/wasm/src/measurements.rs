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

#[wasm_bindgen(js_name = Duration)]
#[derive(Debug, Clone, Copy)]
pub struct JsDuration {
    inner: Duration,
}

#[wasm_bindgen(js_class = Duration)]
impl JsDuration {
    #[wasm_bindgen(constructor)]
    pub fn new(s: u32) -> Self {
        Self {
            inner: Duration::s(s),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn hours(&self) -> u32 {
        self.inner.hours()
    }

    #[wasm_bindgen(getter)]
    pub fn minutes(&self) -> u32 {
        self.inner.minutes()
    }

    #[wasm_bindgen(getter)]
    pub fn seconds(&self) -> u32 {
        self.inner.seconds()
    }
}

impl From<JsDuration> for Duration {
    fn from(value: JsDuration) -> Self {
        value.inner
    }
}

impl From<Duration> for JsDuration {
    fn from(value: Duration) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = Length)]
#[derive(Debug, Clone, Copy)]
pub struct JsLength {
    inner: Length,
}

#[wasm_bindgen(js_class = Length)]
impl JsLength {
    #[wasm_bindgen(constructor)]
    pub fn new(value: Option<f32>, unit: Option<String>) -> Self {
        let unit = match unit.as_deref() {
            Some("cm") => LengthUnit::Centimeters,
            Some("m") => LengthUnit::Meters,
            Some("km") => LengthUnit::Kilometers,
            Some("in") => LengthUnit::Inches,
            Some("ft") => LengthUnit::Feet,
            Some("NM") => LengthUnit::NauticalMiles,
            Some("SM") => LengthUnit::StatuteMiles,
            _ => serde_wasm_bindgen::from_value(unit.into()).unwrap_or(LengthUnit::si()),
        };

        Self {
            inner: Length::new(value.unwrap_or_default(), unit),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f32 {
        *self.inner.value()
    }

    #[wasm_bindgen(getter)]
    pub fn unit(&self) -> String {
        self.inner.symbol().to_string()
    }
}

impl From<JsLength> for Length {
    fn from(value: JsLength) -> Self {
        value.inner
    }
}

impl From<Length> for JsLength {
    fn from(value: Length) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = Mass)]
#[derive(Debug, Clone, Copy)]
pub struct JsMass {
    inner: Mass,
}

#[wasm_bindgen(js_class = Mass)]
impl JsMass {
    #[wasm_bindgen(constructor)]
    pub fn new(value: Option<f32>, unit: Option<String>) -> Self {
        let unit = match unit.as_deref() {
            Some("kg") => MassUnit::Kilograms,
            Some("lb") => MassUnit::Pounds,
            _ => serde_wasm_bindgen::from_value(unit.into()).unwrap_or(MassUnit::si()),
        };

        Self {
            inner: Mass::new(value.unwrap_or_default(), unit),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f32 {
        *self.inner.value()
    }

    #[wasm_bindgen(getter)]
    pub fn unit(&self) -> String {
        self.inner.symbol().to_string()
    }
}

impl From<JsMass> for Mass {
    fn from(value: JsMass) -> Self {
        value.inner
    }
}

impl From<Mass> for JsMass {
    fn from(value: Mass) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = Temperature)]
#[derive(Debug, Clone, Copy)]
pub struct JsTemperature {
    inner: Temperature,
}

#[wasm_bindgen(js_class = Temperature)]
impl JsTemperature {
    #[wasm_bindgen(constructor)]
    pub fn new(value: Option<f32>, unit: Option<String>) -> Self {
        let unit = match unit.as_deref() {
            Some("K") => TemperatureUnit::Kelvin,
            Some("°C") => TemperatureUnit::Celsius,
            Some("°F") => TemperatureUnit::Fahrenheit,
            _ => serde_wasm_bindgen::from_value(unit.into()).unwrap_or(TemperatureUnit::si()),
        };

        Self {
            inner: Temperature::new(value.unwrap_or_default(), unit),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f32 {
        *self.inner.value()
    }

    #[wasm_bindgen(getter)]
    pub fn unit(&self) -> String {
        self.inner.symbol().to_string()
    }
}

impl From<JsTemperature> for Temperature {
    fn from(value: JsTemperature) -> Self {
        value.inner
    }
}

impl From<Temperature> for JsTemperature {
    fn from(value: Temperature) -> Self {
        Self { inner: value }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(js_name = Volume)]
#[derive(Debug, Clone, Copy)]
pub struct JsVolume {
    inner: Volume,
}

#[wasm_bindgen(js_class = Volume)]
impl JsVolume {
    #[wasm_bindgen(constructor)]
    pub fn new(value: Option<f32>, unit: Option<String>) -> Self {
        let unit = match unit.as_deref() {
            Some("m³") => VolumeUnit::CubicMeters,
            Some("L") => VolumeUnit::Liter,
            Some("gal") => VolumeUnit::USGallon,
            _ => serde_wasm_bindgen::from_value(unit.into()).unwrap_or(VolumeUnit::si()),
        };

        Self {
            inner: Volume::new(value.unwrap_or_default(), unit),
        }
    }
}

impl From<JsVolume> for Volume {
    fn from(value: JsVolume) -> Self {
        value.inner
    }
}

impl From<Volume> for JsVolume {
    fn from(value: Volume) -> Self {
        Self { inner: value }
    }
}
