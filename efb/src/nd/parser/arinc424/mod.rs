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

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::str::FromStr;

use crate::error::Error;
use crate::nd::*;

mod from;

pub struct Arinc424Record {
    pub(crate) airports: Vec<Rc<Airport>>,
    pub(crate) waypoints: Vec<Rc<Waypoint>>,
    pub(crate) terminal_waypoints: TerminalWaypoints,
    pub(crate) locations: Vec<LocationIndicator>,
    pub(crate) cycle: Option<AiracCycle>,
}

impl FromStr for Arinc424Record {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut airports: Vec<Airport> = Vec::new();
        let mut rwy_record_lines: Vec<&str> = Vec::new();
        let mut waypoints: Vec<Rc<Waypoint>> = Vec::new();
        let mut terminal_waypoints: TerminalWaypoints = HashMap::new();
        let mut locations: HashSet<LocationIndicator> = HashSet::new();
        let mut cycle: Option<AiracCycle> = None;

        // TODO add some nice error handling
        s.lines().for_each(|line| match &line[4..6] {
            "EA" | "PC" => {
                if let Ok(waypoint_record) = arinc424::Waypoint::from_str(line) {
                    let wp = Waypoint::from(waypoint_record);
                    if let Some(l) = wp.location {
                        locations.insert(l);
                    }
                    if let Some(c) = wp.cycle {
                        cycle = Some(cycle.map_or(c, |cycle| cycle.min(c)));
                    }

                    match &wp.region {
                        Region::Enroute => waypoints.push(Rc::new(wp)),
                        Region::TerminalArea(ident) => {
                            // TODO: Add a new e.g. FourLetterCode type which is used as key so no conversions are needed.
                            let ident = str::from_utf8(ident).expect("ident should be valid UTF-8");
                            terminal_waypoints
                                .entry(ident.to_string())
                                .or_default()
                                .push(Rc::new(wp));
                        }
                    }
                }
            }
            "P " => match &line[12..13] {
                "A" => {
                    if let Ok(airport_record) = arinc424::Airport::from_str(line) {
                        let aprt = Airport::from(airport_record);
                        if let Some(l) = aprt.location {
                            locations.insert(l);
                        }
                        if let Some(c) = aprt.cycle {
                            cycle = Some(cycle.map_or(c, |cycle| cycle.min(c)));
                        }
                        airports.push(aprt);
                    }
                }
                "G" => rwy_record_lines.push(line),
                _ => {}
            },
            _ => {}
        });

        // now that we know all airports, we can assign the runways
        rwy_record_lines.iter().for_each(|line| {
            if let Ok(rwy_record) = arinc424::Runway::from_str(line) {
                if let Some(aprt) = airports
                    .iter_mut()
                    .find(|aprt| rwy_record.arpt_ident == aprt.icao_ident.as_str())
                {
                    aprt.runways.push(rwy_record.into());
                }
            }
        });

        Ok(Self {
            airports: airports.into_iter().map(Rc::new).collect(),
            waypoints,
            terminal_waypoints,
            locations: locations.into_iter().collect(),
            cycle, // TODO: Get cycle from HDR section if available
        })
    }
}
