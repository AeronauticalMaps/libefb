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

//! AIXM 5.1 parser for aeronautical navigation data.
//!
//! This crate provides a streaming parser for AIXM 5.1/5.1.1 XML data. Only
//! the feature types relevant for navigation are parsed: airports, runways,
//! designated points, navaids, and airspaces. All other AIXM features are
//! silently skipped.
//!
//! The main entry point is the [`Features`] iterator which yields one
//! [`Feature`] at a time as it streams through the XML document.
//!
//! # Examples
//!
//! ```no_run
//! use aixm::Features;
//!
//! let data = std::fs::read("AIXM_DATA.xml").expect("file should be readable");
//!
//! for feature in Features::new(&data) {
//!     match feature {
//!         Ok(aixm::Feature::AirportHeliport(ahp)) => {
//!             println!("Airport: {} ({})", ahp.designator, ahp.name);
//!         }
//!         Ok(aixm::Feature::DesignatedPoint(dp)) => {
//!             println!("Point: {}", dp.designator);
//!         }
//!         Ok(_) => {}
//!         Err(e) => eprintln!("Parse error: {e}"),
//!     }
//! }
//! ```

mod error;
mod features;
mod parser;

pub use error::Error;
pub use features::*;
pub use parser::Features;
