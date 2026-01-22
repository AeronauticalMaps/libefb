// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Joe Pearson
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

use crate::{Error, FixedField, Numeric};

/// 5.119 Arc Distance (ARC DIST)
pub struct ArcDistance<'a>(Numeric<'a, 4>);

impl<'a> ArcDistance<'a> {
    pub fn dist(&self) -> Result<f32, Error> {
        self.0.as_u32().map(|dist| dist as f32 / 10.0)
    }
}

impl<'a> FixedField<'a> for ArcDistance<'a> {
    const LENGTH: usize = 4;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        Ok(Self(Numeric::from_bytes(bytes)?))
    }
}

/// 5.120 Arc Bearing (ARC BRG)
pub struct ArcBearing<'a>(Numeric<'a, 4>);

impl<'a> ArcBearing<'a> {
    pub fn deg(&self) -> Result<f32, Error> {
        self.0.as_u32().map(|dist| dist as f32 / 10.0)
    }
}

impl<'a> FixedField<'a> for ArcBearing<'a> {
    const LENGTH: usize = 4;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        Ok(Self(Numeric::from_bytes(bytes)?))
    }
}
