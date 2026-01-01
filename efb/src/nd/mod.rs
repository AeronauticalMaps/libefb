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

//! Navigation Data.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::geom::Coordinate;
use crate::MagneticVariation;

mod airac_cycle;
mod airport;
mod airspace;
mod fix;
mod location;
mod navaid;
mod parser;
mod runway;
mod waypoint;

pub use airac_cycle::{AiracCycle, CycleValidity};
pub use airport::Airport;
pub use airspace::{Airspace, AirspaceClass, Airspaces};
pub use fix::Fix;
pub use location::LocationIndicator;
pub use navaid::NavAid;
use parser::*;
pub use runway::*;
pub use waypoint::*;

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum InputFormat {
    Arinc424,
    OpenAir,
}

type TerminalWaypoints = HashMap<String, Vec<Rc<Waypoint>>>;

#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavigationData {
    airports: Vec<Rc<Airport>>,
    airspaces: Airspaces,
    waypoints: Vec<Rc<Waypoint>>,
    terminal_waypoints: TerminalWaypoints,
    locations: Vec<LocationIndicator>,
    cycle: Option<AiracCycle>,
    partition_id: u64,
    partitions: HashMap<u64, NavigationData>,
}

impl NavigationData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates navigation data from an ARINC 424 string.
    pub fn try_from_arinc424(s: &str) -> Result<Self, Error> {
        let record: Arinc424Record = s.parse()?;

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let partition_id = hasher.finish();

        Ok(Self {
            airports: record.airports,
            airspaces: Vec::new(),
            waypoints: record.waypoints,
            terminal_waypoints: record.terminal_waypoints,
            locations: record.locations,
            cycle: record.cycle,
            partition_id,
            partitions: HashMap::new(),
        })
    }

    /// Creates navigation data from an OpenAir string.
    pub fn try_from_openair(s: &str) -> Result<Self, Error> {
        let record: OpenAirRecord = s.parse()?;

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let partition_id = hasher.finish();

        Ok(Self {
            airports: Vec::new(),
            airspaces: record.airspaces,
            waypoints: Vec::new(),
            terminal_waypoints: HashMap::new(),
            locations: Vec::new(),
            cycle: None,
            partition_id,
            partitions: HashMap::new(),
        })
    }

    pub fn locations(&self) -> &[LocationIndicator] {
        self.locations.as_slice()
    }

    pub fn cycle(&self) -> Option<&AiracCycle> {
        self.cycle.as_ref()
    }

    /// Returns the identifier of the navigation data.
    pub fn partition_id(&self) -> u64 {
        self.partition_id
    }

    /// Returns all airspaces that contain the given point.
    ///
    /// This performs a 2D spatial query using only the airspace polygons.
    /// Vertical bounds (floor and ceiling) are not checked. Returns an empty
    /// vector if the point is not within any airspace.
    ///
    /// # Examples
    ///
    /// ```
    /// # use efb::nd::NavigationData;
    /// # use efb::geom::Coordinate;
    /// # fn check_airspace(nd: &NavigationData) {
    /// let position = Coordinate::new(53.03759, 9.00533);
    /// let airspaces = nd.at(&position);
    ///
    /// if airspaces.is_empty() {
    ///     println!("Outside controlled airspace");
    /// } else {
    ///     println!("Inside {} airspace(s)", airspaces.len());
    /// }
    /// # }
    /// ```
    pub fn at(&self, point: &Coordinate) -> Vec<&Airspace> {
        self.airspaces()
            .filter(|airspace| airspace.polygon.contains(point))
            .collect()
    }

    /// Searches for a navigation aid by identifier.
    ///
    /// Searches waypoints first, then airports. Returns the first match found.
    /// The search is case-sensitive and does not perform partial matching.
    ///
    /// # Examples
    ///
    /// ```
    /// # use efb::prelude::*;
    /// # fn search(nd: &NavigationData) -> Result<(), Error> {
    /// // Search for Hamburg airport
    /// match nd.find("EDDH") {
    ///     Some(navaid) => println!("Found: {}", navaid.ident()),
    ///     None => return Err(Error::UnknownIdent("EDDH".to_string())),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find(&self, ident: &str) -> Option<NavAid> {
        self.waypoints()
            .find(|&wp| wp.ident() == ident)
            .map(|wp| NavAid::Waypoint(Rc::clone(wp)))
            .or(self
                .airports()
                .find(|&aprt| aprt.ident() == ident)
                .map(|aprt| NavAid::Airport(Rc::clone(aprt))))
    }

    /// Searches for a waypoint within a terminal area.
    ///
    /// # Examples
    ///
    /// ```
    /// # use efb::prelude::*;
    /// # fn search(nd: &NavigationData) {
    /// // Find visual reporting point N1 (NOVEMBER 1) in the EDDH terminal area
    /// if let Some(waypoint) = nd.find_terminal_waypoint("EDDH", "N1") {
    ///     println!("Found VRP: {}", waypoint.ident());
    /// }
    /// # }
    /// ```
    pub fn find_terminal_waypoint(&self, airport_ident: &str, fix_ident: &str) -> Option<NavAid> {
        self.terminal_waypoints(airport_ident)
            .find(|&wp| wp.fix_ident == fix_ident)
            .map(|wp| NavAid::Waypoint(Rc::clone(wp)))
    }

    /// Appends other navigation data.
    ///
    /// The other navigation data can be [removed] using it's [partition ID].
    ///
    /// [removed]: Self::remove
    /// [partition ID]: Self::partition_id
    pub fn append(&mut self, other: NavigationData) {
        self.partitions.insert(other.partition_id(), other);
    }

    /// Removes the navigation data partition.
    pub fn remove(&mut self, partition_id: &u64) {
        self.partitions.remove(partition_id);
    }

    /// Returns the IDs of the expired navigation data partitions.
    pub fn expired_partitions(&self) -> Vec<u64> {
        self.partitions
            .iter()
            .filter_map(|(&id, nd)| {
                nd.cycle
                    .and_then(|cycle| cycle.now_valid())
                    .filter(|&validity| validity == CycleValidity::Expired)
                    .map(|_| id)
            })
            .collect()
    }

    pub(crate) fn airports(&self) -> impl Iterator<Item = &Rc<Airport>> {
        self.airports.iter().chain(
            self.partitions
                .values()
                .flat_map(|partition| partition.airports.iter()),
        )
    }

    pub(crate) fn waypoints(&self) -> impl Iterator<Item = &Rc<Waypoint>> {
        self.waypoints.iter().chain(
            self.partitions
                .values()
                .flat_map(|partition| partition.waypoints.iter()),
        )
    }

    pub(crate) fn terminal_waypoints<'a>(
        &'a self,
        ident: &'a str,
    ) -> impl Iterator<Item = &'a Rc<Waypoint>> + 'a {
        self.terminal_waypoints
            .get(ident)
            .into_iter()
            .flatten()
            .chain(
                self.partitions
                    .values()
                    .filter_map(move |partition| partition.terminal_waypoints.get(ident))
                    .flatten(),
            )
    }

    pub(crate) fn airspaces(&self) -> impl Iterator<Item = &Airspace> {
        self.airspaces.iter().chain(
            self.partitions
                .values()
                .flat_map(|partition| partition.airspaces.iter()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::Polygon;
    use crate::VerticalDistance;

    use super::*;

    #[test]
    fn airspace_at_point() {
        let inside = coord!(53.03759, 9.00533);
        let outside = coord!(53.04892, 8.90907);

        let nd = NavigationData {
            airspaces: vec![Airspace {
                name: String::from("TMA BREMEN A"),
                class: AirspaceClass::D,
                ceiling: VerticalDistance::Fl(65),
                floor: VerticalDistance::Msl(1500),
                polygon: polygon![
                    (53.10111, 8.974999),
                    (53.102776, 9.079166),
                    (52.97028, 9.084444),
                    (52.96889, 8.982222),
                    (53.10111, 8.974999)
                ],
            }],
            airports: Vec::new(),
            waypoints: Vec::new(),
            terminal_waypoints: HashMap::new(),
            locations: vec!["ED".try_into().expect("ED should be a valid location")],
            cycle: None,
            partition_id: u64::default(),
            partitions: HashMap::new(),
        };

        assert_eq!(nd.at(&inside), vec![&nd.airspaces[0]]);
        assert!(nd.at(&outside).is_empty());
    }
}
