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

//! Flight Management System.
//!
//! [`FMS`] is the type used to manage all different flight systems and that
//! have dependencies on another. For example, to decode a route we need the
//! navigation data and to plan a flight we need a route. The FMS allows to
//! modify e.g. the navigation data and takes care that the route is reevaluated
//! based on the new data.

use crate::error::{Error, Result};
use crate::fp::{FlightPlanning, FlightPlanningBuilder};
use crate::nd::NavigationData;
use crate::route::Route;

mod printer;
pub use printer::*;

#[derive(Clone, PartialEq, Debug, Default)]
struct Context {
    route: Option<String>,
    flight_planning_builder: Option<FlightPlanningBuilder>,
}

/// `FMS` is the type that manages all flight systems.
///
/// See the [module documentation](self) for details.
#[derive(PartialEq, Debug, Default)]
pub struct FMS {
    nd: NavigationData,
    context: Context,
    route: Route,
    flight_planning: Option<FlightPlanning>,
}

impl FMS {
    /// Constructs a new `FMS`.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn nd(&self) -> &NavigationData {
        &self.nd
    }

    /// Modifies the internal [`NavigationData`].
    ///
    /// # Examples
    ///
    /// Append new data created from an ARINC 424 string.
    ///
    /// ```
    /// # use efb::prelude::*;
    /// #
    /// # fn modify_nd(fms: &mut FMS, records: &str) -> Result<(), Error> {
    /// let new_nd = NavigationData::try_from_arinc424(records)?;
    /// fms.modify_nd(|nd| nd.append(new_nd))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn modify_nd<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut NavigationData),
    {
        f(&mut self.nd);
        self.eval_route().inspect_err(|_| self.route.clear())?;
        self.eval_fp()
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    /// Modifies the [`Route`].
    pub fn modify_route<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Route),
    {
        f(&mut self.route);
        self.eval()
    }

    pub fn decode(&mut self, route: String) -> Result<()> {
        self.context.route = Some(route);
        self.eval()
    }

    /// Sets an alternate on the route.
    ///
    /// Returns an [UnknownIdent] error if no [NavAid] is found for the ident
    /// within the navigation data.
    ///
    /// [UnknownIdent]: Error::UnknownIdent
    /// [NavAid]: crate::nd::NavAid
    pub fn set_alternate(&mut self, ident: &str) -> Result<()> {
        match self.nd.find(ident) {
            Some(alternate) => {
                self.route.set_alternate(Some(alternate));
                self.eval()?;
                Ok(())
            }
            None => Err(Error::UnknownIdent(ident.to_string())),
        }
    }

    pub fn set_flight_planning(&mut self, builder: FlightPlanningBuilder) -> Result<()> {
        self.context.flight_planning_builder = Some(builder);
        self.eval()
    }

    pub fn flight_planning(&self) -> Option<&FlightPlanning> {
        self.flight_planning.as_ref()
    }

    /// Prints the route and planning with a defined line length.
    pub fn print(&self, line_length: usize) -> String {
        let printer = Printer { line_length };
        // TODO: Add print errors and return Result.
        printer
            .print(&self.route, self.flight_planning.as_ref())
            .unwrap_or_default()
    }

    fn eval(&mut self) -> Result<()> {
        self.eval_route()?;
        self.eval_fp()?;
        Ok(())
    }

    fn eval_route(&mut self) -> Result<()> {
        if let Some(prompt) = &self.context.route {
            self.route.decode(&prompt, &self.nd)?;
        }

        Ok(())
    }

    fn eval_fp(&mut self) -> Result<()> {
        if let Some(builder) = &self.context.flight_planning_builder.clone() {
            let flight_planning = builder.build(&self.route)?;
            self.flight_planning = Some(flight_planning);
        }

        Ok(())
    }
}
