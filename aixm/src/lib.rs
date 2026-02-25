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

//! Streaming parser for AIXM 5.1 aeronautical navigation data.
//!
//! [AIXM] (Aeronautical Information Exchange Model) is an XML-based standard
//! used by aviation authorities to publish navigation databases. This crate
//! reads an AIXM 5.1 document and yields the features needed for flight
//! planning — airports, runways, waypoints, navaids, and airspaces — while
//! skipping everything else.
//!
//! # Usage
//!
//! The entry point is the [`Features`] iterator. Pass it the raw XML bytes and
//! iterate to get one [`Feature`] at a time:
//!
//! ```no_run
//! let data = std::fs::read("navigation_data.xml").unwrap();
//!
//! for result in aixm::Features::new(&data) {
//!     match result.unwrap() {
//!         aixm::Feature::AirportHeliport(ahp) => {
//!             println!("Airport {} – {}", ahp.designator(), ahp.name());
//!         }
//!         aixm::Feature::Navaid(nav) => {
//!             println!("Navaid {} ({})", nav.designator(),
//!                 nav.navaid_type().unwrap_or("unknown"));
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! [AIXM]: https://aixm.aero

mod error;
mod features;
mod parser;

pub use error::Error;
pub use features::*;
pub use parser::Features;
