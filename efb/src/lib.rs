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

//! # The Electronic Flight Bag (EFB) Library
//!
//! The EFB Library is the foundation for flight planning applications,
//! providing functionality to plan a flight and get navigational aids
//! in-flight.
//!
//! ## Overview
//!
//! At the core of the EFB is the [`FMS`]. It holds the [navigation data],
//! [route] and delegates the [flight planning]. The following example shows the
//! simplest planning:
//!
//! ```
//! use efb::prelude::*;
//!
//! # fn main() -> Result<(), Error> {
//! // create the FMS
//! let mut fms = FMS::new();
//!
//! // Read navigation data from ARINC 424 records. Here we have the two airports
//! // EDDH (Hamburg) with the runway 33 and EDHF (Itzehoe) with runway 20.
//! let records = r#"SEURP EDDHEDA        0        N N53374900E009591762E002000053                   P    MWGE    HAMBURG                       356462409
//! SEURP EDDHEDGRW33    0120273330 N53374300E009595081                          151                                           124362502
//! SEURP EDHFEDA        0        N N53593300E009343600E000000082                   P    MWGE    ITZEHOE/HUNGRIGER WOLF        320782409
//! SEURP EDHFEDGRW20    0034122060 N53594752E009344856                          098                                           120792502
//! "#;
//! let example_nd = NavigationData::try_from_arinc424(records)?;
//! fms.modify_nd(|nd| nd.append(example_nd))?;
//!
//! // Now we can decode a Route from EDDH to EDHF with takeoff runway 33 and
//! // landing runway 20. Cruise speed is 107kt at an cruise altitude of 2500ft.
//! // The wind is 20kt from 290°.
//! fms.decode("29020KT N0107 A0250 EDDH33 EDHF20".to_string())?;
//! #     Ok(())
//! # }
//! ```
//!
//! From here we can start to define a [`FlightPlanningBuilder`] that holds all
//! information required to let the FMS build a flight planning.
//!
//! [`FMS`]: fms::FMS
//! [navigation data]: nd::NavigationData
//! [route]: route::Route
//! [flight planning]: fp::FlightPlanning
//! [`FlightPlanningBuilder`]: fp::FlightPlanningBuilder
//!
//! # Acronyms & Abbreviations
//!
//! Aviation if full of Acronyms. To not lose track between FMS and RWY, the
//! following section covers acronyms used within this crate.
//!
//! ## A
//!
//! - **AFM** Aircraft Flight Manual
//! - **ARINC** Aeronautical Radio Incorporated
//!
//! ## E
//!
//! - **EFB** Electronic Flight Bag
//! - **Elev** Elevation
//!
//! ## F
//!
//! - **FMS** Flight Management System
//!
//! ## I
//!
//! - **ISA** International Standard Atmosphere
//!
//! ## M
//!
//! - **MSL** Mean Sea Level
//!
//! ## P
//!
//! - **POH** Pilot Operation Handbook
//!
//! ## Q
//!
//! - **QNH** Pressure measured at a location and reduced down to MSL
//!
//! ## R
//!
//! - **RWY** Runway
//! - **RWYCC** Runway Condition Code
//!
//! ## V
//!
//! - **VRP** Visual Reporting Point

////////////////////////////////////////////////////////////////////////////////

#![cfg_attr(docsrs, feature(doc_cfg))]

////////////////////////////////////////////////////////////////////////////////

#[macro_use]
pub mod macros;

mod core;
pub use core::*;

pub mod aircraft;
pub mod algorithm;
pub mod error;
pub mod fc;
pub mod fms;
pub mod fp;
pub mod geom;
pub mod measurements;
pub mod nd;
pub mod route;

pub mod prelude {
    pub use crate::aircraft::{Aircraft, AircraftBuilder, CGLimit, FuelTank, Station};
    pub use crate::core::{Fuel, FuelFlow, FuelType, VerticalDistance};
    pub use crate::error::Error;
    pub use crate::fms::FMS;
    pub use crate::fp::{
        AlteringFactor, AlteringFactors, FactorOfEffect, FlightPlanning, FlightPlanningBuilder,
        FuelPolicy, Performance, Reserve, TakeoffLandingPerformance,
    };
    pub use crate::measurements::*;
    pub use crate::nd::{Fix, InputFormat, NavigationData};
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "geojson")]
pub mod geojson;

#[cfg(feature = "handbook")]
pub mod handbook;
