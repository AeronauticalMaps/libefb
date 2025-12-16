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

use std::rc::Rc;

use crate::error::Error;
use crate::fp::Performance;
use crate::measurements::Speed;
use crate::nd::*;
use crate::{VerticalDistance, Wind};

mod accumulator;
mod leg;

pub use accumulator::TotalsToLeg;
pub use leg::Leg;

#[derive(Clone, PartialEq, Debug)]
pub enum RouteElement {
    Speed(Speed),
    Level(VerticalDistance),
    Wind(Wind),
    NavAid(NavAid),
    RunwayDesignator(String),
}

/// A route that goes from an origin to a destination.
///
/// The route is composed of legs where each [`leg`] describes path between two
/// [`fixes`].
///
/// # Decoding
///
/// The route can be decoded from a space separated list of fixes, wind values
/// and performance elements. The route elements
///
/// ```text
/// 13509KT N0107 EDDH DHD HLW EDHL
/// ```
///
/// would create a route from Hamburg to Luebeck via outbound delta routing and
/// inbound whisky routing with a desired TAS of 107kt and a wind of 9kt from
/// south-east. Performance elements can be add at any point but latest before
/// the first leg is defined (we have from and to fix).
///
/// Thus, each leg is computed based on the latest performance elements defined
/// on the route. Extending our route to
///
/// ```text
/// 13509KT N0107 EDDH DHD 18009KT HLW EDHL
/// ```
///
/// we would have wind from south-east (135°) on the leg from EDDH to DHD, but
/// the wind would turn to south (180°) for the remaining legs.
///
/// [`leg`]: Leg
/// [`fixes`]: crate::nd::Fix
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Route {
    elements: Vec<RouteElement>,
    legs: Vec<Leg>,
    speed: Option<Speed>,
    level: Option<VerticalDistance>,
    alternate: Option<NavAid>,
}

impl Route {
    pub fn new() -> Self {
        Self::default()
    }

    /// Decodes a `route` that is composed of a space separated list of fix
    /// idents read from the navigation data `nd`.
    pub fn decode(&mut self, route: &str, nd: &NavigationData) -> Result<(), Error> {
        let mut elements: Vec<RouteElement> = Vec::new();

        // TODO level and speed needs to be properly update from decoder
        for element in route.split_whitespace() {
            if let Some(navaid) = nd.find(element) {
                elements.push(RouteElement::NavAid(navaid));
            } else if let Ok(value) = element.parse::<VerticalDistance>() {
                self.level.get_or_insert(value);
                elements.push(RouteElement::Level(value));
            } else if let Ok(value) = element.parse::<Speed>() {
                self.speed.get_or_insert(value);
                elements.push(RouteElement::Speed(value));
            } else if let Ok(value) = element.parse::<Wind>() {
                elements.push(RouteElement::Wind(value));
            } else if element.starts_with("RWY") {
                if let Some(RouteElement::NavAid(NavAid::Airport(aprt))) = elements.last() {
                    let designator = element.strip_prefix("RWY").unwrap_or_default().to_string();
                    match aprt.runways.iter().find(|rwy| rwy.designator == designator) {
                        Some(_) => elements.push(RouteElement::RunwayDesignator(designator)),
                        None => {
                            return Err(Error::UnknownRunwayInRoute {
                                aprt: aprt.icao_ident.to_string(),
                                rwy: designator,
                            })
                        }
                    }
                } else {
                    return Err(Error::UnexpectedRunwayInRoute(element.to_string()));
                }
            } else {
                return Err(Error::UnexpectedRouteElement(element.to_string()));
            }
        }

        self.elements = elements;
        self.legs = Self::legs_from_elements(&self.elements);

        Ok(())
    }

    pub fn insert(&mut self, index: usize, element: RouteElement) {
        self.elements.insert(index, element);
        self.legs = Self::legs_from_elements(&self.elements);
    }

    pub fn push(&mut self, element: RouteElement) {
        self.elements.push(element);
        self.legs = Self::legs_from_elements(&self.elements);
    }

    /// Clears the route elements, legs and alternate.
    pub fn clear(&mut self) {
        self.elements.clear();
        self.legs.clear();
        self.alternate.take();
    }

    pub fn elements(&self) -> &Vec<RouteElement> {
        &self.elements
    }

    /// Returns the legs of the route.
    pub fn legs(&self) -> &Vec<Leg> {
        &self.legs
    }

    /// Sets the cruise speed and level.
    ///
    /// The cruise speed or level is remove from the route by setting it to
    /// `None`.
    pub fn set_cruise(&mut self, _speed: Option<Speed>, _level: Option<VerticalDistance>) {
        todo!("Add/remove speed and level from the elements")
    }

    pub fn speed(&self) -> Option<Speed> {
        self.speed
    }

    pub fn level(&self) -> Option<VerticalDistance> {
        self.level
    }

    /// Sets an alternate on the route.
    ///
    /// The alternate is remove by setting it to `None`.
    pub fn set_alternate(&mut self, alternate: Option<NavAid>) {
        self.alternate = alternate;
    }

    /// Returns the final leg but going to the alternate.
    pub fn alternate(&self) -> Option<Leg> {
        let final_leg = self.legs[self.legs.len() - 1].clone();
        Some(Leg::new(
            final_leg.from().clone(),
            self.alternate.clone()?,
            final_leg.level().copied(),
            final_leg.tas().copied(),
            final_leg.wind().copied(),
        ))
    }

    /// Returns the origin airport if one is defined in the route.
    pub fn origin(&self) -> Option<Rc<Airport>> {
        self.legs.first().and_then(|leg| match leg.from() {
            NavAid::Airport(aprt) => Some(aprt.clone()),
            _ => None,
        })
    }

    /// Returns the takeoff runway if a defined in the route.
    pub fn takeoff_rwy(&self) -> Option<Runway> {
        let aprt = self.origin()?;
        self.aprt_rwy_from_elements(aprt)
    }

    /// Returns  the destination airport if one is defined in the route.
    pub fn destination(&self) -> Option<Rc<Airport>> {
        self.legs.last().and_then(|leg| match leg.to() {
            NavAid::Airport(aprt) => Some(aprt.clone()),
            _ => None,
        })
    }

    /// Returns the landing runway if a defined in the route.
    pub fn landing_rwy(&self) -> Option<Runway> {
        let aprt = self.destination()?;
        self.aprt_rwy_from_elements(aprt)
    }

    /// Returns an iterator that accumulates totals progressively through each
    /// leg of the route.
    ///
    /// This function provides cumulative [totals] from the route start up to
    /// each leg. Each yielded `TotalsToLeg` represents the accumulated totals
    /// from the beginning of the route to that specific leg. If [`Some`]
    /// performance is provided, the fuel will be accumulated too.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use efb::route::Route;
    /// # use efb::prelude::Performance;
    /// # fn accumulate_legs(route: Route, perf: Performance) {
    /// // Iterate through route showing progressive totals
    /// for (i, totals) in route.accumulate_legs(Some(&perf)).enumerate() {
    ///     println!("Leg {}: Total distance: {}, Total fuel: {:?}",
    ///              i + 1, totals.dist(), totals.fuel());
    /// }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// If any leg in the sequence is missing ETE or fuel data, the cumulative ETE/fuel
    /// will be `None` for that leg and all subsequent legs, following an "all-or-nothing"
    /// approach to ensure data consistency.
    ///
    /// [totals]: `TotalsToLeg`
    pub fn accumulate_legs<'a>(
        &'a self,
        perf: Option<&'a Performance>,
    ) -> impl Iterator<Item = TotalsToLeg> + 'a {
        self.legs
            .iter()
            .scan(None, move |totals_to_leg: &mut Option<TotalsToLeg>, leg| {
                // accumulate totals from previous legs
                *totals_to_leg = Some(match totals_to_leg.as_ref() {
                    None => TotalsToLeg::new(leg, perf),
                    Some(prev) => prev.accumulate(leg, perf),
                });
                // the totals up to this leg
                totals_to_leg.clone()
            })
    }

    /// Returns the totals of the entire route.
    pub fn totals(&self, perf: Option<&Performance>) -> Option<TotalsToLeg> {
        self.accumulate_legs(perf).last()
    }
}

impl Route {
    /// Returns the runway from an airport if a designator is next to the
    /// airport element.
    // TODO: Return Result rather than Option.
    fn aprt_rwy_from_elements(&self, aprt: Rc<Airport>) -> Option<Runway> {
        let designator = self
            .elements
            .iter()
            .position(|element| match element {
                RouteElement::NavAid(NavAid::Airport(other)) => &aprt == other,
                _ => false,
            })
            // the next element to our airport must be the runway
            .and_then(|position| match self.elements.get(position + 1) {
                Some(RouteElement::RunwayDesignator(designator)) => Some(designator),
                _ => None,
            })?;

        aprt.runways
            .iter()
            .find(|rwy| &rwy.designator == designator)
            .cloned()
    }

    fn legs_from_elements(elements: &Vec<RouteElement>) -> Vec<Leg> {
        let mut level: Option<VerticalDistance> = None;
        let mut tas: Option<Speed> = None;
        let mut wind: Option<Wind> = None;
        let mut from: Option<NavAid> = None;
        let mut to: Option<NavAid> = None;
        let mut legs: Vec<Leg> = Vec::new();

        for element in elements {
            match element {
                RouteElement::Speed(value) => tas = Some(*value),
                RouteElement::Level(value) => level = Some(*value),
                RouteElement::Wind(value) => wind = Some(*value),
                RouteElement::NavAid(navaid) => {
                    if from.is_none() {
                        from = Some(navaid.clone());
                    } else if to.is_none() {
                        to = Some(navaid.clone());
                    }
                }
                _ => (),
            }

            match (from.clone(), to.clone()) {
                (Some(from), Some(to)) => {
                    legs.push(Leg::new(from, to, level, tas, wind));
                }
                _ => continue,
            }

            (from, to) = (to, None);
        }

        legs
    }
}
