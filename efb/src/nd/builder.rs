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

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::*;

/// Navigation data factory, which is used to build [navigation data].
///
/// [navigation data]: super::NavigationData
#[derive(Default)]
pub struct NavigationDataBuilder {
    airports: HashMap<String, Airport>,
    runways: HashMap<String, Vec<Runway>>,
    airspaces: Vec<Airspace>,
    waypoints: Vec<Rc<Waypoint>>,
    terminal_waypoints: TerminalWaypoints,
    locations: HashSet<LocationIndicator>,
    cycle: Option<AiracCycle>,
    partition_id: u64,
    errors: Vec<Error>,
}

macro_rules! add_navaid {
    ($self:ident, $t:expr) => {
        if let Some(l) = $t.location {
            $self.locations.insert(l);
        }

        if let Some(c) = $t.cycle {
            $self.cycle = Some($self.cycle.map_or(c, |cycle| cycle.min(c)));
        }
    };
}

impl NavigationDataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> NavigationData {
        // add unassigned runways to airports
        self.runways.iter_mut().for_each(|(ident, rwys)| {
            if let Some(arpt) = self.airports.get_mut(ident) {
                arpt.runways.append(rwys);
            }
        });

        NavigationData {
            airports: self.airports.into_values().map(Rc::new).collect(),
            airspaces: self.airspaces,
            waypoints: self.waypoints,
            terminal_waypoints: self.terminal_waypoints,
            locations: self.locations.into_iter().collect(),
            cycle: self.cycle,
            partition_id: self.partition_id,
            partitions: HashMap::new(),
        }
    }

    pub fn add_airport(&mut self, arpt: Airport) {
        add_navaid!(self, arpt);
        self.airports.insert(arpt.ident(), arpt);
    }

    pub fn add_runway(&mut self, ident: String, rwy: Runway) {
        match self.airports.get_mut(&ident) {
            Some(arpt) => arpt.runways.push(rwy),
            // in case we have already a runway but no airport
            None => self.runways.entry(ident).or_default().push(rwy),
        }
    }

    pub fn add_airspace(&mut self, airspace: Airspace) {
        self.airspaces.push(airspace);
    }

    pub fn add_waypoint(&mut self, wp: Waypoint) {
        add_navaid!(self, wp);
        match &wp.region {
            Region::Enroute => self.waypoints.push(Rc::new(wp)),
            Region::TerminalArea(ident) => {
                let ident = str::from_utf8(ident).expect("ident should be valid UTF-8");
                self.terminal_waypoints
                    .entry(ident.to_string())
                    .or_default()
                    .push(Rc::new(wp));
            }
        }
    }

    pub fn with_source(mut self, data: &[u8]) -> Self {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        self.partition_id = hasher.finish();
        self
    }
}

impl Extend<Airspace> for NavigationDataBuilder {
    fn extend<T: IntoIterator<Item = Airspace>>(&mut self, iter: T) {
        self.airspaces.extend(iter);
    }
}
