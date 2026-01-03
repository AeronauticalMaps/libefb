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

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::fp::{FlightPlanning, FlightPlanningBuilder};
use crate::nd::NavigationData;
use crate::route::Route;

mod printer;
pub use printer::*;

#[derive(Clone, PartialEq, Debug, Default)]
struct Context {
    route: String,
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
        EvalPipeline::default()
            .inspect_err(EvalStage::Route, |_, fms| fms.route.clear())
            .eval(self)
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
        self.context.route = self.route.to_string();
        EvalPipeline::default().eval(self)
    }

    pub fn decode(&mut self, route: String) -> Result<()> {
        self.context.route = route;
        EvalPipeline::default().eval(self)
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
                EvalPipeline::default().eval(self)
            }
            None => Err(Error::UnknownIdent(ident.to_string())),
        }
    }

    pub fn set_flight_planning(&mut self, builder: FlightPlanningBuilder) -> Result<()> {
        self.context.flight_planning_builder = Some(builder);
        EvalPipeline::default()
            .skip_until(EvalStage::FlightPlanning)
            .eval(self)
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
}

/////////////////////////////////////////////////////////////////////////////
// Evaluation pipeline
/////////////////////////////////////////////////////////////////////////////

type Inspector = Box<dyn FnOnce(&Error, &mut FMS)>;

/// Evaluates the FMS in a defined order.
///
/// The FMS is evaluated in stages, where each stage can fail. If a stage fails,
/// it can be inspected to run e.g. clean-up tasks on the FMS. If a certain
/// action doesn't require an update of the entire pipeline, stages can be
/// skipped to start at a specific stage.
struct EvalPipeline {
    stages: [EvalStage; 2],
    stage_range: std::ops::Range<usize>,
    inspectors: HashMap<EvalStage, Inspector>,
}

impl EvalPipeline {
    fn skip_until(mut self, stage: EvalStage) -> Self {
        if let Some(i) = self.stages[self.stage_range.clone()]
            .iter()
            .position(|s| s == &stage)
        {
            self.stage_range.start += i;
        }
        self
    }

    /// Adds an error inspector for a specific stage.
    ///
    /// The inspector is called if that stage fails, before the error is propagated.
    fn inspect_err<F>(mut self, stage: EvalStage, f: F) -> Self
    where
        F: FnOnce(&Error, &mut FMS) + 'static,
    {
        self.inspectors.insert(stage, Box::new(f));
        self
    }

    /// Executes the evaluation pipeline.
    fn eval(mut self, fms: &mut FMS) -> Result<()> {
        // TODO: Return stage errors and continue evaluation even if one stage fails.
        for stage in &self.stages[self.stage_range] {
            let result = stage.eval(fms);

            if let Err(ref e) = result {
                if let Some(inspector) = self.inspectors.remove(stage) {
                    inspector(e, fms);
                }
            }

            result?;
        }

        Ok(())
    }
}

impl Default for EvalPipeline {
    fn default() -> Self {
        Self {
            stages: [EvalStage::Route, EvalStage::FlightPlanning],
            stage_range: 0..2,
            inspectors: HashMap::new(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
enum EvalStage {
    Route,
    FlightPlanning,
}

impl EvalStage {
    fn eval(&self, fms: &mut FMS) -> Result<()> {
        match self {
            EvalStage::Route => {
                fms.route.decode(&fms.context.route, &fms.nd)?;
            }
            EvalStage::FlightPlanning => {
                if let Some(builder) = &fms.context.flight_planning_builder.clone() {
                    let flight_planning = builder.build(&fms.route)?;
                    fms.flight_planning = Some(flight_planning);
                }
            }
        }

        Ok(())
    }
}
