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

//! ARINC 424 navigation data parser.
//!
//! This crate provides parser for ARINC 424 [records] with their [fields]. Each
//! record is parsed from 132 bytes and references those. The parser tries to
//! copy as little as possible and the fields provide methods to copy or clone
//! values when needed.
//!
//! # Examples
//!
//! Lets parse John F Kennedy Intl airport and print its coordinates:
//!
//! ```
//! use arinc424::records::Airport;
//!
//! # fn main() -> Result<(), arinc424::Error> {
//! let data = b"SUSAP KJFKK6AJFK     0     145YHN40382374W073464329W013000013         1800018000C    MNAR    JOHN F KENNEDY INTL           300671912";
//! let airport = Airport::try_from(data.as_slice())?;
//!
//! // now we can print the ICAO code and the position as decimals
//! let icao = airport.icao_code.as_str();
//! let lat = airport.latitude.as_decimal()?;
//! let lon = airport.longitude.as_decimal()?;
//! println!("{icao} at {lat:.4}, {lon:.4}"); // => "KJFK at 40.6399, -73.7786"
//! #     Ok(())
//! # }
//! ```
//!
//! You can also read an entire navigation database obtained from your
//! authorities or other data provider. The following uses the [`Records`]
//! iterator to print all airports of the FAA's Coded Instrument Flight
//! Procedures (CIFP):
//!
//! ```
//! # use arinc424::records::{Airport, RecordKind, Records};
//! # use arinc424::Error;
//! # fn main() -> Result<(), Error> {
//! // read the navigation database from file
//! let data = std::fs::read("FAACIFP18").expect("file should be readable");
//!
//! // iterate over all records but print only airports
//! for (kind, bytes) in Records::new(&data) {
//!     match kind {
//!         RecordKind::Airport => {
//!             // Airport only references the bytes and gives us access to the fields
//!             let arpt = Airport::try_from(bytes)?;
//!             println!("Airport {} ({})", arpt.arpt_ident, arpt.airport_name);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! [records]: crate::records
//! [fields]: crate::fields
//! [`Records`]: crate::records::Records

#[macro_use]
mod macros;

mod error;
mod field;
mod record;

pub(crate) use field::*;
// Re-export the derive macro for convenience
pub(crate) use arinc424_derive::Record;

pub mod fields;
pub mod records;
pub use error::Error;
