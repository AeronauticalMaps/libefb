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

use pyo3::prelude::*;

use efb::fms::FMS;
use efb::nd::{InputFormat, NavigationData};

mod flight_planning_builder;
use flight_planning_builder::*;

/// Input format of navigation data.
#[pyclass(module = "efb", name = "InputFormat", eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum PyInputFormat {
    #[pyo3(name = "ARINC_424")]
    Arinc424,

    #[pyo3(name = "OPEN_AIR")]
    OpenAir,
}

impl From<PyInputFormat> for InputFormat {
    fn from(fmt: PyInputFormat) -> InputFormat {
        match fmt {
            PyInputFormat::Arinc424 => InputFormat::Arinc424,
            PyInputFormat::OpenAir => InputFormat::OpenAir,
        }
    }
}

/// Flight Management System (FMS).
///
/// The FMS is the central part of this library. It loads the navigation data, a
/// route and a flight planning builder to e.g. build a flight planning.
#[pyclass(module = "efb", name = "FMS", unsendable)]
pub struct PyFMS {
    fms: FMS,
}

#[pymethods]
impl PyFMS {
    #[new]
    pub fn new() -> Self {
        Self { fms: FMS::new() }
    }

    /// Reads the navigation data from a string.
    ///
    /// :param str s: The data as string.
    /// :param InputFormat fmt: The format of the string.
    pub fn nd_read(&mut self, s: &str, fmt: PyInputFormat) {
        let new_nd = match fmt {
            PyInputFormat::Arinc424 => NavigationData::try_from_arinc424(s.as_bytes()),
            PyInputFormat::OpenAir => NavigationData::try_from_openair(s),
        };

        if let Ok(new_nd) = new_nd {
            let _ = self.fms.modify_nd(|nd| nd.append(new_nd));
        }
    }

    /// Decode a route from a string.
    ///
    /// :param str route: The route string to decode.
    pub fn decode(&mut self, route: String) {
        let _ = self.fms.decode(route);
    }

    /// Sets the flight planning.
    ///
    /// :param FlightPlanningBuilder builder:
    pub fn set_flight_planning(&mut self, builder: PyFlightPlanningBuilder) {
        let _ = self.fms.set_flight_planning(builder.into());
    }

    /// Prints the flight planning.
    ///
    /// :param int line_length: The length of the printed lines.
    /// :return: The planning as printable string.
    /// :rtype: str
    pub fn print(&mut self, line_length: usize) -> String {
        self.fms.print(line_length)
    }
}

pub fn register_fms_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFlightPlanningBuilder>()?;
    m.add_class::<PyInputFormat>()?;
    m.add_class::<PyFMS>()?;
    Ok(())
}
