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

use crate::{Alphanumeric, Error, FixedField};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum CustArea<'a> {
    Blank,
    Customer(Alphanumeric<'a, 3>),
    PreferredRoute,
    AFR,
    CAN,
    EEU,
    EUR,
    LAM,
    MES,
    PAC,
    SAM,
    SPA,
    USA,
}

impl<'a> FixedField<'a> for CustArea<'a> {
    const LENGTH: usize = 3;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        Ok(match &bytes[0..3] {
            b"AFR" => Self::AFR,
            b"CAN" => Self::CAN,
            b"EEU" => Self::EEU,
            b"EUR" => Self::EUR,
            b"LAM" => Self::LAM,
            b"MES" => Self::MES,
            b"PAC" => Self::PAC,
            b"SAM" => Self::SAM,
            b"SPA" => Self::SPA,
            b"USA" => Self::USA,
            b"PDR" => Self::PreferredRoute,
            b"   " => Self::Blank,
            code => Self::Customer(Alphanumeric::from_bytes(code)?),
        })
    }
}
