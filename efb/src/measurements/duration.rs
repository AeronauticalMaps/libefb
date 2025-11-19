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

use super::{Measurement, PhysicalQuantity, UnitOfMeasure};

/// Duration unit with s as SI unit.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum DurationUnit {
    Seconds,
}

impl UnitOfMeasure<u32> for DurationUnit {
    fn quantity() -> PhysicalQuantity {
        PhysicalQuantity::Duration
    }

    fn si() -> Self {
        Self::Seconds
    }

    fn symbol(&self) -> &'static str {
        "s"
    }

    fn from_si(value: u32, _: &Self) -> u32 {
        value
    }

    fn to_si(&self, value: &u32) -> u32 {
        *value
    }
}

pub type Duration = Measurement<u32, DurationUnit>;

impl Duration {
    pub fn s(value: u32) -> Self {
        Measurement {
            value,
            unit: DurationUnit::Seconds,
        }
    }

    pub fn hours(&self) -> u32 {
        self.value / 3600 % 24
    }

    pub fn minutes(&self) -> u32 {
        self.value / 60 % 60
    }

    pub fn seconds(&self) -> u32 {
        self.value % 60
    }

    /// Rounds the duration to the nearest minute.
    pub fn round(&self) -> Self {
        let s = self.seconds();

        let rounded_value = if s >= 30 {
            self.value - s + 60
        } else {
            self.value - s
        };

        Self::s(rounded_value)
    }
}

/*
impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tmp = if f.alternate() {
            format!(
                "{:02}:{:02}:{:02}",
                self.hours(),
                self.minutes(),
                self.seconds()
            )
        } else {
            let rounded = self.round();
            format!("{:02}:{:02}", rounded.hours(), rounded.minutes())
        };

        f.pad_integral(true, "", &tmp)
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_seconds() {
        let duration = Duration::s(3661);
        assert_eq!(duration.hours(), 1);
        assert_eq!(duration.minutes(), 1);
        assert_eq!(duration.seconds(), 1);
    }

    #[test]
    fn sum_durations() {
        let sum = Duration::s(3561) + Duration::s(100);
        assert_eq!(sum.hours(), 1);
        assert_eq!(sum.minutes(), 1);
        assert_eq!(sum.seconds(), 1);
    }

    #[test]
    fn round() {
        let duration = Duration::s(29);
        assert_eq!(duration.round().minutes(), 0);

        let duration = Duration::s(30);
        assert_eq!(duration.round().minutes(), 1);
    }
}
