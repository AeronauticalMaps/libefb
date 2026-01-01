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

//! EFB error types.
//!
//! All operations that can fail return [`Result<T>`]. The [`Error`] categorizes
//! errors into:
//!
//! - Route decoding errors (invalid elements, unknown identifiers)
//! - Parsing errors (malformed strings, invalid values)
//! - Navigation data errors (unknown identifiers, invalid runway codes)
//! - Mass & balance planning errors (mismatched dimensions, exceeded capacities)
//! - Aircraft building errors (missing required fields)

use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Error {
    // Errors that can occur while decoding a route:
    //
    /// The entered flight plan does not include a cruise speed as one of the
    /// first two elements.
    ExpectedSpeedOnFPL,
    /// The entered flight plan does not include a cruise level as one of the
    /// first two elements.
    ExpectedLevelOnFPL,
    /// The route prompt includes a token that was not expected.
    UnexpectedRouteToken(String),
    /// The route includes a runway at a position that is not next to an
    /// airport.
    UnexpectedRunwayInRoute(String),
    /// The route includes a runway that is not found on the associated airport.
    UnknownRunwayInRoute { aprt: String, rwy: String },
    /// A terminal waypoint needs to match to exactly one of the terminal areas
    /// in scope.
    AmbiguousTerminalArea { wp: String, a: String, b: String },

    // Errors that are related to parsing of input data:
    //
    /// The string that should be parsed to create some type is malformed.
    UnexpectedString,
    /// The value that should be returned is implausible.
    ImplausibleValue,
    /// The location indicator should be a two-letter code according to ICAO
    /// Document No. 7910.
    UnknownLocationIndicator(String),

    // Errors that relate to navigation data:
    //
    /// The requested identifier is not know.
    UnknownIdent(String),
    /// The RWYCC should be between 0 and 6.
    InvalidRWYCC,

    // Errors that originate from the mass & balance planning:
    //
    /// The number of masses doesn't match the number of stations to which the
    /// masses are assigned.
    UnexpectedMassesForStations,
    /// The number of provided fuel stations doesn't match the aircraft's fuel
    /// stations.
    UnexpectedNumberOfFuelStations,
    /// The planned fuel on ramp exceeds the tank's capacity.
    ExceededFuelCapacityOnRamp,
    /// The planned fuel after landing exceeds the tank's capacity.
    ExceededFuelCapacityAfterLanding,

    // Errors that can occur while building an aircraft:
    //
    /// The aircraft's registration is not set.
    ExpectedRegistration,
    /// The aircraft's empty mass is not set.
    ExpectedEmptyMass,
    /// The aircraft's empty balance is not set.
    ExpectedEmptyBalance,
    /// The aircraft's fuel type is not set.
    ExpectedFuelType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedSpeedOnFPL => write!(f, "FPL is missing cruise speed"),
            Self::ExpectedLevelOnFPL => write!(f, "FPL is missing cruise level"),
            Self::UnexpectedRouteToken(e) => write!(f, "invalid token {e} found in route"),
            Self::UnexpectedRunwayInRoute(rwy) => {
                write!(f, "runway {rwy} should follow an airport")
            }
            Self::UnknownRunwayInRoute { aprt, rwy } => {
                write!(f, "unknown runway {rwy} found for {aprt}")
            }
            Self::AmbiguousTerminalArea { wp, a, b } => {
                write!(f, "waypoint {wp} found in terminal area {a} and {b}")
            }

            Self::UnexpectedString => write!(f, "unexpected string"),
            Self::ImplausibleValue => write!(f, "value seams implausuble"),
            Self::UnknownLocationIndicator(code) => write!(
                f,
                "location {code} should be according to ICAO document no. 7910"
            ),

            Self::UnknownIdent(ident) => write!(f, "unknown ident {ident}"),
            Self::InvalidRWYCC => write!(f, "RWYCC should be between 0 and 6"),

            Self::UnexpectedMassesForStations => {
                write!(f, "mass should match to aircraft's stations")
            }
            Self::UnexpectedNumberOfFuelStations => {
                write!(f, "fuel stations should match to aircraft's tanks")
            }
            Self::ExceededFuelCapacityOnRamp => {
                write!(f, "fuel should not exceed tank capacity on ramp")
            }
            Self::ExceededFuelCapacityAfterLanding => {
                write!(f, "fuel should not exceed tank capacity after landing")
            }

            Self::ExpectedRegistration => write!(f, "aircraft should have a registration"),
            Self::ExpectedEmptyMass => write!(f, "aircraft should have an empty mass"),
            Self::ExpectedEmptyBalance => write!(f, "aircraft should have an empty balance"),
            Self::ExpectedFuelType => write!(f, "aircraft should have a fuel type defined"),
        }
    }
}

impl error::Error for Error {}
