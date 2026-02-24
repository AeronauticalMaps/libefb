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

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use time::OffsetDateTime;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// AIRAC 1901: January 3, 2019
// This is a known AIRAC effective date (Thursday)
const REFERENCE_DATE: NaiveDate =
    NaiveDate::from_ymd_opt(2019, 1, 3).expect("2019-01-03 should be a valid date");

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CycleValidity {
    Valid,
    Expired,
    Future,
}

impl fmt::Display for CycleValidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::Expired => write!(f, "expired"),
            Self::Future => write!(f, "future"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AiracCycle {
    year: u8,
    cycle: u8,
}

impl AiracCycle {
    pub fn new(year: u8, cycle: u8) -> Self {
        Self { year, cycle }
    }

    /// Calculates the effective date when this AIRAC cycle starts.
    ///
    /// Returns `None` if the calculated date is either to far in the future or
    /// past.
    pub fn effective_date(&self) -> Option<NaiveDate> {
        // now we need to get the first Thursday of the year
        let year = self.year as u16 + 2000u16; // TODO: Please update in the year 3000...
        let first_thu = NaiveDate::from_weekday_of_month_opt(year as i32, 1, Weekday::Thu, 1)
            .expect("the year should be before before 262143 CE");

        // align with the 28-day AIRAC cycle pattern
        let days_since_ref = (first_thu - REFERENCE_DATE).num_days();
        let cycle_offset = days_since_ref % 28;

        let first_airac_of_year = if cycle_offset == 0 {
            first_thu
        } else {
            first_thu + Duration::days(28 - cycle_offset)
        };

        // calculate the target cycle date
        let target_date = first_airac_of_year + Duration::days(28 * (self.cycle - 1) as i64);

        // verify the date makes sense (not too far into the future/past)
        if target_date.year() as u16 == year
            || (self.cycle == 1 && target_date.year() as u16 == year - 1)
        {
            Some(target_date)
        } else {
            None
        }
    }

    /// Calculates the end date when this AIRAC cycle expires
    ///
    /// Returns `None` if the effective date is invalid.
    pub fn end_date(&self) -> Option<NaiveDate> {
        self.effective_date()
            .map(|start| start + Duration::days(27)) // 28-day cycle (0-27 days)
    }

    /// Checks if the cycle is valid for the date.
    ///
    /// Returns `None` if the effective date is invalid.
    pub fn valid_for_date(&self, date: NaiveDate) -> Option<CycleValidity> {
        let start_date = self.effective_date()?;
        let end_date = self.end_date()?;

        if date < start_date {
            Some(CycleValidity::Future)
        } else if date > end_date {
            Some(CycleValidity::Expired)
        } else {
            Some(CycleValidity::Valid)
        }
    }

    /// Checks if the cycle is now valid with reference to the UTC date.
    ///
    /// Returns `None` if the effective date is invalid.
    pub fn now_valid(&self) -> Option<CycleValidity> {
        let now = OffsetDateTime::now_utc().date();
        let y = now.year();
        let m: u8 = now.month().into();
        let d = now.day();

        let date =
            NaiveDate::from_ymd_opt(y, m as u32, d as u32).expect("now should be a valid date");

        self.valid_for_date(date)
    }
}

impl fmt::Display for AiracCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}{:02}", self.year, self.cycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_dates() {
        // AIRAC 2509: 2025-09-04 – 2025-10-01
        let cycle = AiracCycle::new(25, 9);
        assert_eq!(cycle.effective_date(), NaiveDate::from_ymd_opt(2025, 9, 4));
        assert_eq!(cycle.end_date(), NaiveDate::from_ymd_opt(2025, 10, 1));

        // AIRAC 2501: 2025-01-23 – 2025-02-19
        let cycle = AiracCycle::new(25, 1);
        assert_eq!(cycle.effective_date(), NaiveDate::from_ymd_opt(2025, 1, 23));
        assert_eq!(cycle.end_date(), NaiveDate::from_ymd_opt(2025, 2, 19));

        // AIRAC 2413: 2024-12-26 – 2025-01-22
        let cycle = AiracCycle::new(24, 13);
        assert_eq!(cycle.effective_date(), NaiveDate::from_ymd_opt(2024, 12, 26));
        assert_eq!(cycle.end_date(), NaiveDate::from_ymd_opt(2025, 1, 22));

        // AIRAC 2601: 2026-01-22 – 2026-02-18
        let cycle = AiracCycle::new(26, 1);
        assert_eq!(cycle.effective_date(), NaiveDate::from_ymd_opt(2026, 1, 22));
        assert_eq!(cycle.end_date(), NaiveDate::from_ymd_opt(2026, 2, 18));

        // AIRAC 2602: 2026-02-19 – 2026-03-18
        let cycle = AiracCycle::new(26, 2);
        assert_eq!(cycle.effective_date(), NaiveDate::from_ymd_opt(2026, 2, 19));
        assert_eq!(cycle.end_date(), NaiveDate::from_ymd_opt(2026, 3, 18));
    }

    #[test]
    fn test_cycle_validity() {
        // AIRAC 2509 goes from 2025-09-04 till 2025-10-01
        let cycle = AiracCycle::new(25, 9);

        let mid_date =
            NaiveDate::from_ymd_opt(2025, 9, 18).expect("2025-09-08 should be a valid date");
        assert_eq!(cycle.valid_for_date(mid_date), Some(CycleValidity::Valid));

        let outdated_date =
            NaiveDate::from_ymd_opt(2025, 10, 18).expect("2025-10-08 should be a valid date");
        assert_eq!(
            cycle.valid_for_date(outdated_date),
            Some(CycleValidity::Expired)
        );
    }

    #[test]
    fn test_identifier_format() {
        let cycle = AiracCycle::new(25, 9);
        assert_eq!(cycle.to_string(), "2509");
    }
}
