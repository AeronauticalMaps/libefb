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

use std::fmt;
use std::rc::Rc;

use log::{debug, trace, warn};

use crate::error::Error;
use crate::fp::Performance;
use crate::measurements::Speed;
use crate::nd::*;
use crate::{VerticalDistance, Wind};

mod accumulator;
mod leg;
mod token;

pub use accumulator::TotalsToLeg;
pub use leg::Leg;
use token::Tokens;
pub use token::{Token, TokenKind};

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
/// 13509KT N0107 EDDH D DCT W EDHL
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
/// 13509KT N0107 EDDH D DCT 18009KT DCT W EDHL
/// ```
///
/// we would have wind from south-east (135°) on the leg from EDDH to D (VRP Delta), but
/// the wind would turn to south (180°) for the remaining legs.
///
/// [`leg`]: Leg
/// [`fixes`]: crate::nd::Fix
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Route {
    tokens: Tokens,
    legs: Vec<Leg>,
    speed: Option<Speed>,
    level: Option<VerticalDistance>,
    origin: Option<Rc<Airport>>,
    takeoff_rwy: Option<Runway>,
    destination: Option<Rc<Airport>>,
    landing_rwy: Option<Runway>,
    alternate: Option<NavAid>,
}

impl Route {
    pub fn new() -> Self {
        Self::default()
    }

    /// Decodes a `route` that is composed of a space separated list of fix
    /// idents read from the navigation data `nd`.
    pub fn decode(&mut self, route: &str, nd: &NavigationData) -> Result<(), Error> {
        debug!("route decode: {:?}", route);
        self.tokens = Tokens::new(route, nd);
        self.legs.clear();

        // clear values relevant during parsing of all tokens
        self.origin.take();
        self.destination.take();
        self.takeoff_rwy.take();
        self.landing_rwy.take();

        let mut level: Option<VerticalDistance> = None;
        let mut tas: Option<Speed> = None;
        let mut wind: Option<Wind> = None;
        let mut from: Option<NavAid> = None;
        let mut to: Option<NavAid> = None;

        for token in &self.tokens {
            match token.kind() {
                TokenKind::Speed(value) => {
                    tas = Some(*value);
                    // first speed is cruise speed
                    if self.speed.is_none() {
                        self.speed = Some(*value);
                        debug!("cruise speed set to {:?}", value);
                    }
                }

                TokenKind::Level(value) => {
                    level = Some(*value);
                    // first level is cruise level
                    if self.level.is_none() {
                        self.level = Some(*value);
                        debug!("cruise level set to {:?}", value);
                    }
                }

                TokenKind::Wind(value) => wind = Some(*value),

                TokenKind::Airport { arpt, rwy } => {
                    // Track for leg building
                    if from.is_none() {
                        from = Some(NavAid::Airport(Rc::clone(arpt)));
                    } else if to.is_none() {
                        to = Some(NavAid::Airport(Rc::clone(arpt)));
                    }

                    // First airport is origin, subsequent airports are destinations
                    match &self.origin {
                        None => {
                            // First airport = origin with optional takeoff runway
                            debug!(
                                "origin set to {} (rwy {:?})",
                                arpt.ident(),
                                rwy.as_ref().map(|r| &r.designator)
                            );
                            self.origin = Some(Rc::clone(arpt));
                            self.takeoff_rwy = rwy.clone();
                        }
                        Some(_) => {
                            // Any subsequent airport = destination with optional landing runway
                            debug!(
                                "destination set to {} (rwy {:?})",
                                arpt.ident(),
                                rwy.as_ref().map(|r| &r.designator)
                            );
                            self.destination = Some(Rc::clone(arpt));
                            self.landing_rwy = rwy.clone();
                        }
                    }
                }

                TokenKind::NavAid(navaid) => {
                    // Non-airport navaids (waypoints, VOR, NDB, etc.)
                    if from.is_none() {
                        from = Some(navaid.clone());
                    } else if to.is_none() {
                        to = Some(navaid.clone());
                    }
                }

                TokenKind::Err(err) => {
                    warn!("error token encountered during route decode: {}", err);
                    return Err(err.clone());
                }

                _ => (),
            }

            match (&from, &to) {
                (Some(from), Some(to)) => {
                    trace!(
                        "creating leg: {} -> {}",
                        from.ident(),
                        to.ident()
                    );
                    self.legs
                        .push(Leg::new(from.clone(), to.clone(), level, tas, wind));
                }
                _ => continue,
            }

            (from, to) = (to, None);
        }

        debug!("route decoded: {} leg(s)", self.legs.len());

        Ok(())
    }

    /// Returns the tokens used to build the route.
    pub fn tokens(&self) -> &[Token] {
        self.tokens.tokens()
    }

    /// Clears the route elements, legs and alternate.
    pub fn clear(&mut self) {
        self.tokens.clear();
        self.legs.clear();
        self.alternate.take();
    }

    /// Returns the legs of the route.
    pub fn legs(&self) -> &[Leg] {
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
        let final_leg = self.legs.last()?.clone();
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
        self.origin.as_ref().map(Rc::clone)
    }

    /// Returns the takeoff runway if a defined in the route.
    pub fn takeoff_rwy(&self) -> Option<&Runway> {
        self.takeoff_rwy.as_ref()
    }

    /// Returns  the destination airport if one is defined in the route.
    pub fn destination(&self) -> Option<Rc<Airport>> {
        self.destination.as_ref().map(Rc::clone)
    }

    /// Returns the landing runway if a defined in the route.
    pub fn landing_rwy(&self) -> Option<&Runway> {
        self.landing_rwy.as_ref()
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
                *totals_to_leg
            })
    }

    /// Returns the totals of the entire route.
    pub fn totals(&self, perf: Option<&Performance>) -> Option<TotalsToLeg> {
        self.accumulate_legs(perf).last()
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tokens)
    }
}
