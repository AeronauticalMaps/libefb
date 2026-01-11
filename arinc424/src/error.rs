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

use std::error;
use std::fmt;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Error {
    InvalidRecordLength {
        actual: usize,
    },
    InvalidFieldLength {
        expected: usize,
        actual: usize,
    },
    InvalidCharacter {
        field: &'static str,
        byte: u8,
        expected: &'static str,
    },
    InvalidVariant {
        field: &'static str,
        bytes: Vec<u8>,
        expected: &'static str,
    },
    NotANumber {
        bytes: Vec<u8>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRecordLength { actual } => {
                write!(f, "record should be 132 byte long but is {actual}")
            }
            Self::InvalidFieldLength { expected, actual } => {
                write!(f, "field should be {expected} byte long but is {actual}")
            }
            Self::InvalidCharacter {
                field,
                byte,
                expected,
            } => {
                write!(
                    f,
                    "{field} is \"{}\" but should be {expected}",
                    *byte as char
                )
            }
            Self::InvalidVariant {
                field,
                bytes,
                expected,
            } => {
                let s = String::from_utf8_lossy(bytes);
                write!(f, "found \"{s}\" in {field} but should be {expected}")
            }
            Self::NotANumber { bytes } => {
                let s = String::from_utf8_lossy(bytes);
                write!(f, "field should be a number but is \"{s}\"")
            }
        }
    }
}

impl error::Error for Error {}
