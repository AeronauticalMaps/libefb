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

/// Record Type field (ARINC 424 Spec §5.2).
///
/// Position: 0 (1 character)
/// - "S": Standard record
/// - "T": Tailored record
#[derive(Debug, PartialEq)]
pub enum RecordType {
    Standard,
    Tailored,
}

impl RecordType {
    /// The position of the record type field.
    pub const POSITION: usize = 0;
    /// The length of the record type field.
    pub const LENGTH: usize = 1;
}

impl Field for RecordType {}

impl FromStr for RecordType {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(FieldError::invalid_length(
                "RecordType",
                Self::POSITION,
                Self::LENGTH,
            ));
        }

        let code = &s[0..1];
        match code {
            "S" => Ok(Self::Standard),
            "T" => Ok(Self::Tailored),
            c => Err(FieldError::invalid_value(
                "RecordType",
                Self::POSITION,
                Self::LENGTH,
                "expected S or T",
            )
            .with_actual(c)),
        }
    }
}
