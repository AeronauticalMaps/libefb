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

use efb::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = NavigationData)]
pub struct JsNavigationData {
    inner: Rc<RefCell<FMS>>,
}

#[wasm_bindgen(js_class = NavigationData)]
impl JsNavigationData {
    pub fn find(&self, ident: &str) -> JsValue {
        let fms = self.inner.borrow();
        serde_wasm_bindgen::to_value(&fms.nd().find(ident)).unwrap()
    }

    pub fn remove(&self, uuid: JsValue) -> Result<(), JsError> {
        let mut fms = self.inner.borrow_mut();
        let uuid: [u8; 16] = serde_wasm_bindgen::from_value(uuid)?;
        fms.modify_nd(|nd| nd.remove(&uuid))?;
        Ok(())
    }
}

impl From<&Rc<RefCell<FMS>>> for JsNavigationData {
    fn from(value: &Rc<RefCell<FMS>>) -> Self {
        Self {
            inner: Rc::clone(value),
        }
    }
}
