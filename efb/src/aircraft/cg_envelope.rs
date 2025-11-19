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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::algorithm;
use crate::fp::MassAndBalance;
use crate::measurements::{Length, Mass};

/// A point that spawns the CG envelope.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CGLimit {
    mass: Mass,
    distance: Length,
}

impl CGLimit {
    pub fn new(mass: Mass, distance: Length) -> Self {
        Self { mass, distance }
    }

    pub fn mass(&self) -> &Mass {
        &self.mass
    }

    pub fn distance(&self) -> &Length {
        &self.distance
    }
}

/// An aircraft's center of gravity (CG) envelope.
///
/// The envelope draws a polygon in a coordinate system with the mass and
/// balance as axis. It contains a CG for a mass if the aircraft is balanced on
/// ramp and after landing.
///
/// # Examples
///
/// This is how an envelope of a Cessna 172 might look like:
///
/// ```
/// # use efb::measurements::{Mass, Length};
/// # use efb::aircraft::{CGEnvelope, CGLimit, LoadedStation, Station};
/// # use efb::fp::MassAndBalance;
/// #
/// // M     2--------------3
/// // a    /               |
/// // s   /                |
/// // s  1                 |
/// //    |                 |
/// //    |                 |
/// //    0-----------------4
/// //
/// //               Length
/// let cg_envelope = CGEnvelope::new(vec![
///     CGLimit::new(Mass::kg(0.0), Length::m(0.89)),    // 0
///     CGLimit::new(Mass::kg(885.0), Length::m(0.89)),  // 1
///     CGLimit::new(Mass::kg(1111.0), Length::m(1.02)), // 2
///     CGLimit::new(Mass::kg(1111.0), Length::m(1.20)), // 3
///     CGLimit::new(Mass::kg(0.0), Length::m(1.20)),    // 4
/// ]);
///
/// // now we calculate the mass & balance which we want to check against our envelope
/// let mb = MassAndBalance::new(&vec![
///     // just for this example we simplify our aircraft as one station
///     LoadedStation {
///         // we and the fuel have an arm of 1.1 m from the reference datum
///         station: Station::new(Length::m(1.1), None),
///         // we start our journey with the pilot and some fuel on board
///         on_ramp: Mass::kg(897.0),
///         // and we burned 10 kg on our little sight seeing trip
///         after_landing: Mass::kg(887.0),
///     },
/// ]);
///
/// // finally we can check if our CG is within the envelope
/// assert!(cg_envelope.contains(&mb));
/// ```
#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CGEnvelope {
    limits: Vec<CGLimit>,
}

impl CGEnvelope {
    /// Creates a new envelope from the limits.
    pub fn new(limits: Vec<CGLimit>) -> Self {
        Self { limits }
    }

    /// Tests if the mass & balance is within this envelope.
    ///
    /// Returns `false` if one of the limits on ramp or after landing is outside
    /// of the envelope.
    pub fn contains(&self, mb: &MassAndBalance) -> bool {
        // We see the envelope as a polygon where the mass describes the y-axis
        // and the balance the x-axis. The M&B on ramp and after landing is
        // considered to be a point within this envelope (polygon).
        let envelope: Vec<algorithm::Point> = self
            .limits
            .iter()
            .map(|mb| algorithm::Point {
                x: mb.distance.to_si(),
                y: mb.mass.to_si(),
            })
            .collect();

        let wn = |mass: &Mass, balance: &Length| -> i32 {
            algorithm::winding_number(
                &algorithm::Point {
                    x: balance.to_si(),
                    y: mass.to_si(),
                },
                &envelope,
            )
        };

        let wn_on_ramp = wn(mb.mass_on_ramp(), mb.balance_on_ramp());
        let wn_after_landing = wn(mb.mass_after_landing(), mb.balance_after_landing());

        // The envelope's winding number around the point is 0 if the point is
        // outside the envelope.
        wn_on_ramp != 0 && wn_after_landing != 0
    }

    /// The limits of the envelope.
    pub fn limits(&self) -> &[CGLimit] {
        &self.limits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aircraft::{LoadedStation, Station};

    #[test]
    fn contains_point() {
        // Lets test an envelope with the limits 1 which is within the envelope
        // and balanced and point 2 which is without the envelope and
        // unbalanced:
        //
        //   2  +--------+
        //     /         |
        //    /          |
        //   +     1     |
        //   |           |
        //   +-----------+
        //
        let envelope = CGEnvelope::new(vec![
            CGLimit {
                mass: Mass::kg(0.0),
                distance: Length::m(0.0),
            },
            CGLimit {
                mass: Mass::kg(0.5),
                distance: Length::m(0.0),
            },
            CGLimit {
                mass: Mass::kg(1.0),
                distance: Length::m(0.25),
            },
            CGLimit {
                mass: Mass::kg(1.0),
                distance: Length::m(1.0),
            },
            CGLimit {
                mass: Mass::kg(0.0),
                distance: Length::m(1.0),
            },
        ]);

        let balanced = MassAndBalance::new(&vec![LoadedStation {
            station: Station::new(Length::m(0.5), None),
            on_ramp: Mass::kg(0.5),
            after_landing: Mass::kg(0.5),
        }]);

        let unbalanced = MassAndBalance::new(&vec![LoadedStation {
            station: Station::new(Length::m(0.0), None),
            on_ramp: Mass::kg(1.0),
            after_landing: Mass::kg(1.0),
        }]);

        assert!(
            envelope.contains(&balanced),
            "envelope didn't contain the balanced M&B"
        );
        assert!(
            !envelope.contains(&unbalanced),
            "envelope contain the unbalanced M&B"
        );
    }
}
