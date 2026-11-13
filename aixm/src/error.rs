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

/// Errors that occur while reading an AIXM document.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Error {
    /// The document is not well-formed XML.
    InvalidXml(String),
    /// The root element's namespace is not a supported AIXM 5.x namespace.
    UnknownVersion(String),
    /// The message metadata has no temporal extent, so it is unknown on which
    /// date the features should be valid.
    MissingTemporalExtent,
    /// A record of the document has an invalid value. Only this record is
    /// affected; the rest of the document can still be read.
    InvalidRecord(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "invalid XML: {e}"),
            Self::UnknownVersion(ns) => write!(f, "unsupported AIXM namespace {ns}"),
            Self::MissingTemporalExtent => write!(f, "message metadata has no temporal extent"),
            Self::InvalidRecord(e) => write!(f, "invalid AIXM record: {e}"),
        }
    }
}

impl error::Error for Error {}

impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Self::InvalidXml(err.to_string())
    }
}
