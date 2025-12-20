// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Joe Pearson
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

//! AIXM (Aeronautical Information Exchange Model) parser
//!
//! This crate provides Rust types and parsers for AIXM 5.2 data, which is used
//! by aviation authorities to exchange aeronautical information.
//!
//! The types in this crate are automatically generated from the official AIXM XSD schemas.

// Re-export xsd_parser_types that are used by generated code
pub use xsd_parser_types;

// Include the generated modules
#[cfg(not(feature = "no-codegen"))]
mod generated;

#[cfg(not(feature = "no-codegen"))]
pub use generated::*;
