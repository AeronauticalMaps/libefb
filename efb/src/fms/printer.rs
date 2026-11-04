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

use std::fmt::{Error, Write as _};

use crate::fp::{FlightPlanning, FuelPlanning, RunwayAnalysis};
use crate::measurements::LengthUnit;
use crate::nd::*;
use crate::route::Route;

/// Prints the flight planning of the FMS.
///
/// The printer can [`print`] the route and if available the fuel and mass & balance
/// of the FMS to a String with a defined line length. The result can be used to
/// print it to a physical sheet of paper as a hard copy of the planning.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Printer {
    /// The line length of the printers output.
    pub(super) line_length: usize,
}

impl Printer {
    /// Prints the flight planning of the FMS.
    pub fn print(
        &self,
        route: &Route,
        flight_planning: Option<&FlightPlanning>,
    ) -> Result<String, Error> {
        let mut buffer = String::new();

        self.write_route(&mut buffer, route)?;

        if let Some(flight_planning) = flight_planning {
            if let Some(fuel_planning) = flight_planning.fuel_planning() {
                self.write_fuel(&mut buffer, fuel_planning)?;
            }

            self.write_mb(&mut buffer, flight_planning)?;

            if let Some(rwy_analysis) = flight_planning.takeoff_rwy_analysis() {
                self.write_takeoff_landing_rwy_analysis(&mut buffer, "TAKEOFF RWY", rwy_analysis)?;
            }

            if let Some(rwy_analysis) = flight_planning.landing_rwy_analysis() {
                self.write_takeoff_landing_rwy_analysis(&mut buffer, "LANDING RWY", rwy_analysis)?;
            }
        }

        Ok(buffer)
    }

    /// Writes a section with title to the buffer.
    fn write_section(&self, buffer: &mut String, title: &str) -> Result<(), Error> {
        writeln!(buffer, "{}", "-".repeat(self.line_length))?;
        writeln!(buffer, "-- {title}")?;
        writeln!(buffer, "{}", "-".repeat(self.line_length))?;
        writeln!(buffer)?;
        Ok(())
    }

    /// Writes the route to the buffer.
    fn write_route(&self, buffer: &mut String, route: &Route) -> Result<(), Error> {
        self.write_section(buffer, "ROUTE")?;

        for leg in route.legs() {
            let space = (self.line_length - 23) / 3;

            let is_heading = leg.mh().is_some();

            writeln!(
                buffer,
                "{:<6}{:space$}{:^6}{:space$}{:>8}{:space$}{:^5}",
                "TO",
                "",
                if is_heading { "HDG" } else { "TRK" },
                "",
                "DIST",
                "",
                "ETE"
            )?;

            writeln!(
                buffer,
                "{:<6}{:space$}{:^6.0}{:space$}{:>8.1}{:space$}{:^5}",
                leg.to().ident(),
                "",
                leg.mh().unwrap_or(leg.mc()),
                "",
                leg.dist().convert_to(LengthUnit::NauticalMiles),
                "",
                leg.ete().map(|d| d.to_string()).unwrap_or("-".to_string()),
            )?;

            writeln!(buffer)?;
        }

        if let Some(totals) = route.totals(None) {
            writeln!(buffer, "DIST {:>1$.1}", totals.dist(), self.line_length - 5)?;

            if let Some(ete) = totals.ete() {
                writeln!(buffer, "ETE {:>1$}", ete, self.line_length - 4)?;
            }
        }

        writeln!(buffer)?;

        Ok(())
    }

    /// Writes the fuel planning to the buffer.
    fn write_fuel(&self, buffer: &mut String, fuel_planning: &FuelPlanning) -> Result<(), Error> {
        self.write_section(buffer, "FUEL")?;

        writeln!(
            buffer,
            "TRIP {:>1$.0}",
            fuel_planning.trip(),
            self.line_length - 5
        )?;

        writeln!(
            buffer,
            "TAXI {:>1$.0}",
            fuel_planning.taxi(),
            self.line_length - 5
        )?;

        if let Some(alternate) = fuel_planning.alternate() {
            writeln!(
                buffer,
                "ALTERNATE {:>1$.0}",
                alternate,
                self.line_length - 10
            )?;
        }

        writeln!(
            buffer,
            "RESERVE {:>1$.0}",
            fuel_planning.reserve(),
            self.line_length - 8
        )?;

        writeln!(
            buffer,
            "MINIMUM {:>1$.0}",
            fuel_planning.min(),
            self.line_length - 8
        )?;

        if let Some(extra) = fuel_planning.extra() {
            writeln!(buffer, "EXTRA {:>1$.0}", extra, self.line_length - 6)?;
        }

        writeln!(
            buffer,
            "TOTAL {:>1$.0}",
            fuel_planning.total(),
            self.line_length - 6
        )?;

        writeln!(buffer)?;

        Ok(())
    }

    /// Writes the mass & balance of the flight planning to the buffer.
    fn write_mb(&self, buffer: &mut String, flight_planning: &FlightPlanning) -> Result<(), Error> {
        self.write_section(buffer, "MASS & BALANCE")?;

        if let Some(mb) = flight_planning.mb() {
            let column_length = (self.line_length - 15) / 2;
            writeln!(
                buffer,
                "              {:^column_length$} {:^column_length$}",
                "MASS", "BALANCE"
            )?;

            writeln!(
                buffer,
                "      ON RAMP {:^column_length$.0} {:^column_length$.1}",
                mb.mass_on_ramp(),
                mb.balance_on_ramp()
            )?;
            writeln!(
                buffer,
                "AFTER LANDING {:^column_length$.0} {:^column_length$.1}",
                mb.mass_after_landing(),
                mb.balance_on_ramp()
            )?;
        }

        let balanced = match flight_planning.is_balanced() {
            Some(true) => "YES",
            Some(false) => "NO",
            None => "N/A",
        };

        writeln!(buffer)?;
        writeln!(buffer, "BALANCED {:>1$}", balanced, self.line_length - 9)?;
        writeln!(buffer)?;

        Ok(())
    }

    fn write_takeoff_landing_rwy_analysis(
        &self,
        buffer: &mut String,
        section: &str,
        rwy_analysis: &RunwayAnalysis,
    ) -> Result<(), Error> {
        self.write_section(buffer, section)?;

        writeln!(
            buffer,
            "HEADWIND {:>1$.0}",
            rwy_analysis.headwind(),
            self.line_length - 9
        )?;

        writeln!(
            buffer,
            "CROSSWIND {:>1$.0}",
            rwy_analysis.crosswind(),
            self.line_length - 10
        )?;

        writeln!(
            buffer,
            "GROUND ROLL {:>1$.0}",
            rwy_analysis.ground_roll(),
            self.line_length - 12
        )?;

        writeln!(
            buffer,
            "MARGIN {:>1$.0}",
            rwy_analysis.margin(),
            self.line_length - 7
        )?;

        writeln!(
            buffer,
            "MARGIN in % {:>1$.2}",
            rwy_analysis.pct_margin(),
            self.line_length - 12
        )?;

        for note in rwy_analysis.notes() {
            writeln!(buffer, "{note}")?;
        }

        writeln!(buffer)?;

        Ok(())
    }
}
