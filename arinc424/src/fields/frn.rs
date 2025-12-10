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

/// File Record Number field (ARINC 424 Spec §5.32).
///
/// Position: 123-128 (5 characters)
/// A sequential number assigned to each record in the file.
#[derive(Debug)]
pub struct FileRecordNumber(u32);

impl FileRecordNumber {
    /// The starting position of the FRN field.
    pub const POSITION: usize = 123;
    /// The length of the FRN field.
    pub const LENGTH: usize = 5;
}

impl Field for FileRecordNumber {}

impl PartialEq<u32> for FileRecordNumber {
    fn eq(&self, other: &u32) -> bool {
        &self.0 == other
    }
}

impl FromStr for FileRecordNumber {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < Self::POSITION + Self::LENGTH {
            return Err(FieldError::invalid_length(
                "FileRecordNumber",
                Self::POSITION,
                Self::LENGTH,
            ));
        }

        let slice = &s[Self::POSITION..Self::POSITION + Self::LENGTH];
        match slice.parse::<u32>() {
            Ok(frn) => Ok(Self(frn)),
            Err(_) => Err(FieldError::not_a_number(
                "FileRecordNumber",
                Self::POSITION,
                Self::LENGTH,
            )
            .with_actual(slice)),
        }
    }
}
