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

use std::fmt;

#[derive(Clone, Debug)]
pub enum Error {
    /// An XML parsing error from the underlying parser.
    Xml(String),
    /// A required element or attribute is missing.
    MissingField(&'static str),
    /// A value could not be parsed (e.g. a coordinate or elevation).
    InvalidValue { field: &'static str, value: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "XML error: {e}"),
            Self::MissingField(field) => write!(f, "missing required field: {field}"),
            Self::InvalidValue { field, value } => {
                write!(f, "invalid value for {field}: {value}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<quick_xml::Error> for Error {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e.to_string())
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Xml(e.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(e: quick_xml::events::attributes::AttrError) -> Self {
        Self::Xml(e.to_string())
    }
}

impl From<quick_xml::DeError> for Error {
    fn from(e: quick_xml::DeError) -> Self {
        Self::Xml(e.to_string())
    }
}
