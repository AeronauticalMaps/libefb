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

use crate::{Error, FixedField};

/// 5.121 Lower/Upper Limit
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LowerUpperLimit {
    Altitude(u32),
    FlightLevel(u16),
    NotSpecified,
    Unlimited,
    Ground,
    MeanSeaLevel,
    NOTAM,
}

impl FixedField<'_> for LowerUpperLimit {
    const LENGTH: usize = 5;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match &bytes[..Self::LENGTH] {
            [b'F', b'L', d @ ..] if d.iter().all(u8::is_ascii_digit) => {
                let fl = parse_numeric!(3, u16, d)?;
                Ok(Self::FlightLevel(fl))
            }
            digits if digits.iter().all(u8::is_ascii_digit) => {
                let alt = parse_numeric!(5, u32, digits)?;
                Ok(Self::Altitude(alt))
            }
            b"NOTSP" => Ok(Self::NotSpecified),
            b"UNLTD" => Ok(Self::Unlimited),
            b"GND  " => Ok(Self::Ground),
            b"MSL  " => Ok(Self::MeanSeaLevel),
            b"NOTAM" => Ok(Self::NOTAM),
            bytes => Err(Error::InvalidVariant {
                field: "Lower/Upper Limit",
                bytes: bytes.to_vec(),
                expected: "according to ARINC 424-23 5.121",
            }),
        }
    }
}
