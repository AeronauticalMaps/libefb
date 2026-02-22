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

use log::{debug, info, trace, warn};

use super::*;

use crate::aircraft::Aircraft;
use crate::error::Error;
use crate::measurements::{Mass, Temperature};
use crate::nd::RunwayConditionCode;
use crate::route::Route;
use crate::{Fuel, Wind};

/// Flight planning factory, which is used to build a flight planning.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct FlightPlanningBuilder {
    aircraft: Option<Aircraft>,
    mass: Option<Vec<Mass>>,
    policy: Option<FuelPolicy>,
    taxi: Option<Fuel>,
    reserve: Option<Reserve>,
    perf: Option<Performance>,
    takeoff_perf: Option<TakeoffLandingPerformance>,
    takeoff_factors: Option<AlteringFactors>,
    origin_rwycc: Option<RunwayConditionCode>,
    origin_wind: Option<Wind>,
    origin_temperature: Option<Temperature>,
    landing_perf: Option<TakeoffLandingPerformance>,
    landing_factors: Option<AlteringFactors>,
    destination_rwycc: Option<RunwayConditionCode>,
    destination_wind: Option<Wind>,
    destination_temperature: Option<Temperature>,
}

impl FlightPlanningBuilder {
    /// Creates a new builder.
    pub fn new() -> FlightPlanningBuilder {
        Self::default()
    }

    /// Builds a flight planning for the specified route.
    // TODO: Describe the possible errors.
    pub fn build(&self, route: &Route) -> Result<FlightPlanning, Error> {
        info!("building flight planning");

        let fuel_planning = match (
            &self.aircraft,
            &self.policy,
            self.taxi,
            &self.reserve,
            &self.perf,
        ) {
            (Some(aircraft), Some(policy), Some(taxi), Some(reserve), Some(perf)) => {
                debug!("computing fuel planning (policy={:?})", policy);
                let fp = FuelPlanning::new(aircraft, policy, taxi, route, reserve, perf);
                if fp.is_none() {
                    warn!("fuel planning could not be computed (missing route totals)");
                }
                fp
            }
            _ => {
                trace!("fuel planning skipped: missing aircraft, policy, taxi, reserve, or performance data");
                None
            }
        };

        let mb = match (&self.aircraft, &self.mass, &fuel_planning) {
            (Some(aircraft), Some(mass), Some(fuel_planning)) => {
                debug!("computing mass & balance");
                Some(aircraft.mb_from_const_mass_and_equally_distributed_fuel(
                    mass,
                    fuel_planning.on_ramp(),
                    fuel_planning.after_landing(),
                )?)
            }
            _ => {
                trace!("mass & balance skipped: missing aircraft, mass, or fuel planning");
                None
            }
        };

        let is_balanced = match (&self.aircraft, mb.as_ref()) {
            (Some(aircraft), Some(mb)) => {
                let balanced = aircraft.is_balanced(mb);
                if !balanced {
                    warn!("aircraft is NOT within balance limits");
                } else {
                    debug!("aircraft is within balance limits");
                }
                Some(balanced)
            }
            _ => None,
        };

        let takeoff_rwy_analysis: Option<RunwayAnalysis> = match (
            &route.takeoff_rwy(),
            self.origin_rwycc,
            &self
                .origin_wind
                .or(route.legs().first().and_then(|leg| leg.wind()).cloned()),
            self.origin_temperature,
            &mb,
            &self.takeoff_perf,
        ) {
            (Some(rwy), Some(rwycc), Some(wind), Some(temperature), Some(mb), Some(perf)) => {
                debug!("computing takeoff runway analysis (rwy {})", rwy.designator);
                Some(RunwayAnalysis::takeoff(
                    rwy,
                    rwycc,
                    wind,
                    temperature,
                    mb,
                    perf,
                    self.takeoff_factors.as_ref(),
                ))
            }
            _ => {
                trace!("takeoff runway analysis skipped: missing required parameters");
                None
            }
        };

        let landing_rwy_analysis: Option<RunwayAnalysis> = match (
            &route.landing_rwy(),
            self.destination_rwycc,
            &self
                .destination_wind
                .or(route.legs().last().and_then(|leg| leg.wind()).cloned()),
            self.destination_temperature,
            &mb,
            &self.landing_perf,
        ) {
            (Some(rwy), Some(rwycc), Some(wind), Some(temperature), Some(mb), Some(perf)) => {
                debug!("computing landing runway analysis (rwy {})", rwy.designator);
                Some(RunwayAnalysis::landing(
                    rwy,
                    rwycc,
                    wind,
                    temperature,
                    mb,
                    perf,
                    self.landing_factors.as_ref(),
                ))
            }
            _ => {
                trace!("landing runway analysis skipped: missing required parameters");
                None
            }
        };

        info!(
            "flight planning built: fuel={}, mb={}, takeoff_rwy={}, landing_rwy={}",
            fuel_planning.is_some(),
            mb.is_some(),
            takeoff_rwy_analysis.is_some(),
            landing_rwy_analysis.is_some(),
        );

        Ok(FlightPlanning {
            aircraft: self.aircraft.clone(),
            fuel_planning,
            mb,
            is_balanced,
            takeoff_rwy_analysis,
            landing_rwy_analysis,
        })
    }

    pub fn aircraft(&mut self, aircraft: Aircraft) -> &mut Self {
        self.aircraft = Some(aircraft);
        self
    }

    pub fn mass(&mut self, mass: Vec<Mass>) -> &mut Self {
        self.mass = Some(mass);
        self
    }

    pub fn policy(&mut self, policy: FuelPolicy) -> &mut Self {
        self.policy = Some(policy);
        self
    }

    pub fn taxi(&mut self, taxi: Fuel) -> &mut Self {
        self.taxi = Some(taxi);
        self
    }

    pub fn reserve(&mut self, reserve: Reserve) -> &mut Self {
        self.reserve = Some(reserve);
        self
    }

    pub fn perf(&mut self, perf: Performance) -> &mut Self {
        self.perf = Some(perf);
        self
    }

    pub fn takeoff_perf(&mut self, perf: TakeoffLandingPerformance) -> &mut Self {
        self.takeoff_perf = Some(perf);
        self
    }

    pub fn takeoff_factors(&mut self, factors: AlteringFactors) -> &mut Self {
        self.takeoff_factors = Some(factors);
        self
    }

    pub fn origin_rwycc(&mut self, rwycc: RunwayConditionCode) -> &mut Self {
        self.origin_rwycc = Some(rwycc);
        self
    }

    pub fn origin_wind(&mut self, wind: Wind) -> &mut Self {
        self.origin_wind = Some(wind);
        self
    }

    pub fn origin_temperature(&mut self, temperature: Temperature) -> &mut Self {
        self.origin_temperature = Some(temperature);
        self
    }

    pub fn landing_perf(&mut self, perf: TakeoffLandingPerformance) -> &mut Self {
        self.landing_perf = Some(perf);
        self
    }

    pub fn landing_factors(&mut self, factors: AlteringFactors) -> &mut Self {
        self.landing_factors = Some(factors);
        self
    }

    pub fn destination_rwycc(&mut self, rwycc: RunwayConditionCode) -> &mut Self {
        self.destination_rwycc = Some(rwycc);
        self
    }

    pub fn destination_wind(&mut self, wind: Wind) -> &mut Self {
        self.destination_wind = Some(wind);
        self
    }

    pub fn destination_temperature(&mut self, temperature: Temperature) -> &mut Self {
        self.destination_temperature = Some(temperature);
        self
    }
}
