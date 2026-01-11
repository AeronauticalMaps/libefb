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

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum NameInd {
    AbeamFix,
    BearingDistanceFix,
    AirportNameAsFix,
    FIRFix,
    PhoneticLetterNameFix,
    AirportIdentFix,
    LatitudeLongitudeFix,
    MultipleWordNameFix,
    NavaidIdentFix,
    PublishedFiveLetterNameFix,
    PublishedNameFixLessThanFiveLetters,
    PublishedNameFixMoreThanFiveLetters,
    AirportRwyRelatedFix,
    UIRFix,
    VFRReportingPointFix,
    LocalizerMarkerWithPublishedFiveLetter,
    LocalizerMarkerWithoutPublishedFiveLetter,
}

impl FixedField<'_> for NameInd {
    const LENGTH: usize = 3;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match &bytes[0..3] {
            b"A  " => Ok(Self::AbeamFix),
            b"B  " => Ok(Self::BearingDistanceFix),
            b"D  " => Ok(Self::AirportNameAsFix),
            b"F  " => Ok(Self::FIRFix),
            b"H  " => Ok(Self::PhoneticLetterNameFix),
            b"I  " => Ok(Self::AirportIdentFix),
            b"L  " => Ok(Self::LatitudeLongitudeFix),
            b"M  " => Ok(Self::MultipleWordNameFix),
            b"N  " => Ok(Self::NavaidIdentFix),
            b"P  " => Ok(Self::PublishedFiveLetterNameFix),
            b"Q  " => Ok(Self::PublishedNameFixLessThanFiveLetters),
            b"R  " => Ok(Self::PublishedNameFixMoreThanFiveLetters),
            b"T  " => Ok(Self::AirportRwyRelatedFix),
            b"U  " => Ok(Self::UIRFix),
            b"V  " => Ok(Self::VFRReportingPointFix),
            b" O " => Ok(Self::LocalizerMarkerWithPublishedFiveLetter),
            b" M " => Ok(Self::LocalizerMarkerWithoutPublishedFiveLetter),
            _ => Err(Error::InvalidVariant {
                field: "Name Format Indicator",
                bytes: Vec::from(bytes),
                expected: "according to ARINC 424-17 5.196",
            }),
        }
    }
}
