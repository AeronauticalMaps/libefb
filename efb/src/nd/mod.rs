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

//! Navigation Data.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use geo::{Contains, Point};
use rstar::AABB;

use crate::error::Error;
use crate::measurements::Length;
use crate::MagneticVariation;

mod airac_cycle;
mod airport;
mod airspace;
mod builder;
mod convert;
mod fix;
mod index;
mod location;
mod navaid;
mod runway;
mod waypoint;

pub use airac_cycle::{AiracCycle, CycleValidity};
pub use airport::Airport;
pub use airspace::{Airspace, AirspaceClassification, AirspaceType};
pub use fix::Fix;
pub use location::LocationIndicator;
pub use navaid::NavAid;
pub use runway::*;
pub use waypoint::*;

pub(crate) use builder::NavigationDataBuilder;
pub(crate) use index::{AirspaceIndex, NavAidIndex};

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum InputFormat {
    Arinc424,
    OpenAir,
}

type TerminalWaypoints = HashMap<String, Vec<Rc<Waypoint>>>;

/// Results from a spatial query at a given point.
///
/// Contains airspaces that contain the point and navaids (airports and
/// waypoints) within the specified search radius.
#[derive(Clone, Debug, Default)]
pub struct Nearby {
    /// Airspaces that contain the query point.
    pub airspaces: Vec<Rc<Airspace>>,
    /// Navaids within the search radius.
    pub navaids: Vec<NavAid>,
}

impl Nearby {
    /// Returns true if no results were found.
    pub fn is_empty(&self) -> bool {
        self.airspaces.is_empty() && self.navaids.is_empty()
    }

    /// Returns the total number of results.
    pub fn len(&self) -> usize {
        self.airspaces.len() + self.navaids.len()
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NavigationData {
    airports: Vec<Rc<Airport>>,
    airspaces: Vec<Rc<Airspace>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    airspace_index: AirspaceIndex,
    #[cfg_attr(feature = "serde", serde(skip))]
    navaid_index: NavAidIndex,
    waypoints: Vec<Rc<Waypoint>>,
    terminal_waypoints: TerminalWaypoints,
    locations: Vec<LocationIndicator>,
    cycle: Option<AiracCycle>,
    partition_id: u64,
    partitions: HashMap<u64, NavigationData>,
    errors: Vec<Error>,
}

impl NavigationData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a factory to build navigation data.
    pub(super) fn builder() -> NavigationDataBuilder {
        NavigationDataBuilder::new()
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

    /// Returns all airspaces containing the point and navaids within the radius.
    ///
    /// Performs a spatial query that:
    /// - Finds airspaces whose polygons contain the point (2D containment)
    /// - Finds airports and waypoints within the specified radius
    ///
    /// Vertical bounds (floor and ceiling) of airspaces are not checked.
    ///
    /// # Examples
    ///
    /// ```
    /// # use efb::nd::NavigationData;
    /// # use efb::measurements::Length;
    /// # use geo::Point;
    /// # fn nearby(nd: &NavigationData) {
    /// let position = Point::new(9.99, 53.63); // (lon, lat)
    /// let nearby = nd.at(&position, Length::nm(10.0));
    ///
    /// println!("Airspaces: {}", nearby.airspaces.len());
    /// println!("Navaids: {}", nearby.navaids.len());
    /// # }
    /// ```
    pub fn at(&self, point: &Point<f64>, radius: Length) -> Nearby {
        // Find airspaces containing the point
        let airspaces: Vec<_> = self
            .airspace_index
            .candidates_at(point.x(), point.y())
            .filter(|airspace| airspace.polygon.contains(point))
            .cloned()
            .collect();

        // Find navaids within radius
        let navaids: Vec<_> = self
            .navaid_index
            .within_radius(point, radius)
            .cloned()
            .collect();

        Nearby { airspaces, navaids }
    }

    /// Returns candidate airspaces whose bounding boxes intersect the given
    /// envelope.
    pub(crate) fn candidate_airspaces_for_envelope(
        &self,
        envelope: &AABB<Point<f64>>,
    ) -> Vec<Rc<Airspace>> {
        self.airspace_index
            .candidates_intersecting(envelope)
            .cloned()
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
                .find(|&arpt| arpt.ident() == ident)
                .map(|arpt| NavAid::Airport(Rc::clone(arpt))))
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
        self.reindex();
    }

    /// Removes the navigation data partition.
    pub fn remove(&mut self, partition_id: &u64) {
        self.partitions.remove(partition_id);
        self.reindex();
    }

    /// Indexes the navigation data partitions.
    fn reindex(&mut self) {
        self.airspace_index = AirspaceIndex::new(self.airspaces());
        self.navaid_index = NavAidIndex::new(self.airports(), self.waypoints());
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

    /// Returns all possible data errors.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub(crate) fn airports(&self) -> impl Iterator<Item = &Rc<Airport>> {
        self.airports.iter().chain(
            self.partitions
                .values()
                .flat_map(|partition| partition.airports.iter()),
        )
    }

    pub(crate) fn airspaces(&self) -> impl Iterator<Item = &Rc<Airspace>> {
        self.airspaces.iter().chain(
            self.partitions
                .values()
                .flat_map(|partition| partition.airspaces.iter()),
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
}

#[cfg(test)]
mod tests {
    use crate::VerticalDistance;

    use super::*;

    #[test]
    fn airspace_at_point() {
        let mut builder = NavigationData::builder();
        let inside = coord!(53.03759, 9.00533);
        let outside = coord!(53.04892, 8.90907);

        builder.add_airspace(Airspace {
            name: String::from("TMA BREMEN A"),
            airspace_type: AirspaceType::CTA,
            classification: Some(AirspaceClassification::D),
            ceiling: VerticalDistance::Fl(65),
            floor: VerticalDistance::Msl(1500),
            polygon: polygon![
                (53.10111, 8.974999),
                (53.102776, 9.079166),
                (52.97028, 9.084444),
                (52.96889, 8.982222),
                (53.10111, 8.974999)
            ],
        });

        let nd = builder.build();
        let nearby_inside = nd.at(&inside, Length::nm(1.0));
        let nearby_outside = nd.at(&outside, Length::nm(1.0));

        assert_eq!(nearby_inside.airspaces, vec![Rc::clone(&nd.airspaces[0])]);
        assert!(nearby_outside.airspaces.is_empty());
    }

    #[test]
    fn navaids_within_radius() {
        let mut builder = NavigationData::builder();

        // Add an airport
        builder.add_airport(Airport {
            icao_ident: "EDDH".to_string(),
            iata_designator: "HAM".to_string(),
            name: "Hamburg".to_string(),
            coordinate: Point::new(9.99, 53.63), // (lon, lat)
            mag_var: None,
            elevation: VerticalDistance::Gnd,
            runways: vec![],
            location: None,
            cycle: None,
        });

        // Add a waypoint nearby
        builder.add_waypoint(Waypoint {
            fix_ident: "DHN1".to_string(),
            desc: "Delta November 1".to_string(),
            usage: WaypointUsage::VFROnly,
            coordinate: Point::new(9.95, 53.60), // (lon, lat)
            mag_var: None,
            region: Region::Enroute,
            location: None,
            cycle: None,
        });

        // Add a waypoint far away
        builder.add_waypoint(Waypoint {
            fix_ident: "FAR1".to_string(),
            desc: "Far Away".to_string(),
            usage: WaypointUsage::Unknown,
            coordinate: Point::new(10.5, 54.5), // (lon, lat)
            mag_var: None,
            region: Region::Enroute,
            location: None,
            cycle: None,
        });

        let nd = builder.build();
        let center = Point::new(9.97, 53.62); // (lon, lat)

        // Small radius - should find airport and nearby waypoint
        let nearby = nd.at(&center, Length::nm(5.0));
        assert_eq!(nearby.navaids.len(), 2);

        // Large radius - should find everything
        let nearby = nd.at(&center, Length::nm(100.0));
        assert_eq!(nearby.navaids.len(), 3);
    }
}
