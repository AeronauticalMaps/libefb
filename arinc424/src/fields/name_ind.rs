// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Joe Pearson
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

use super::{Field, FieldError};
use std::str::FromStr;

/// Name Format Indicator field (ARINC 424 Spec §5.196).
///
/// Position I (3 characters)
/// Indicates the method used to derive the fix name.
#[derive(Debug, PartialEq)]
pub enum NameInd<const I: usize> {
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
    LocalizerMarkerWithPublishedFiveLetter,
    LocalizerMarkerWithoutPublishedFiveLetter,
    Unspecified, // TODO this is not valid ARINC 424-17
}

impl<const I: usize> NameInd<I> {
    /// The length of the name format indicator field.
    pub const LENGTH: usize = 3;
}

impl<const I: usize> Field for NameInd<I> {}

impl<const I: usize> FromStr for NameInd<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("NameInd", I, Self::LENGTH));
        }

        let indicator = &s[I..I + 3];
        match indicator {
            "A  " => Ok(Self::AbeamFix),
            "B  " => Ok(Self::BearingDistanceFix),
            "D  " => Ok(Self::AirportNameAsFix),
            "F  " => Ok(Self::FIRFix),
            "H  " => Ok(Self::PhoneticLetterNameFix),
            "I  " => Ok(Self::AirportIdentFix),
            "L  " => Ok(Self::LatitudeLongitudeFix),
            "M  " => Ok(Self::MultipleWordNameFix),
            "N  " => Ok(Self::NavaidIdentFix),
            "P  " => Ok(Self::PublishedFiveLetterNameFix),
            "Q  " => Ok(Self::PublishedNameFixLessThanFiveLetters),
            "R  " => Ok(Self::PublishedNameFixMoreThanFiveLetters),
            "T  " => Ok(Self::AirportRwyRelatedFix),
            "U  " => Ok(Self::UIRFix),
            " O " => Ok(Self::LocalizerMarkerWithPublishedFiveLetter),
            " M " => Ok(Self::LocalizerMarkerWithoutPublishedFiveLetter),
            "   " => Ok(Self::Unspecified),
            c => Err(FieldError::unexpected_char(
                "NameInd",
                I,
                Self::LENGTH,
                "unexpected name format indicator",
            )
            .with_actual(c)),
        }
    }
}
