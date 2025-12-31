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

use std::collections::HashMap;

use efb::error::Result;
use efb::nd::{RunwayConditionCode, RunwaySurface};
use efb::prelude::*;
use efb::*;

const ARINC_424_RECORDS: &'static str = r#"SEURP EDDHEDA        0        N N53374900E009591762E002000053                   P    MWGE    HAMBURG                       356462409
SEURP EDDHEDGRW05    0106630500 N53371100E009580180                          151                                           124362502
SEURP EDDHEDGRW23    0106632300 N53380900E009595876                          151                                           124362502
SEURP EDDHEDGRW15    0120271530 N53391500E009583076                          151                                           124362502
SEURP EDDHEDGRW33    0120273330 N53374300E009595081                          151                                           124362502
SEURPCEDDHED N1    ED0    V     N53482105E010015451                                 WGE           NOVEMBER1                359892409
SEURPCEDDHED N2    ED0    V     N53405701E010000576                                 WGE           NOVEMBER2                359902409
SEURP EDHFEDA        0        N N53593300E009343600E000000082                   P    MWGE    ITZEHOE/HUNGRIGER WOLF        320782409
SEURP EDHFEDGRW02    0034120260 N53591751E009342331                          098                                           120792502
SEURP EDHFEDGRW20    0034122060 N53594752E009344856                          098                                           120792502
SEURP EDHFEDGRW09    0023230910 N53592877E009335932                          131                                           120792502
SEURP EDHFEDGRW27    0023232710 N53592838E009344247                          131                                           120792502
"#;

fn main() -> Result<()> {
    // Performance setting with 65% load in cruise. This is the performance
    // profile of a Cessna C172 with an TAE125-02-114 Diesel engine.
    let perf = Performance::from_fn(
        |vd| {
            let tas = if *vd >= VerticalDistance::Altitude(10000) {
                Speed::kt(114.0)
            } else if *vd >= VerticalDistance::Altitude(8000) {
                Speed::kt(112.0)
            } else if *vd >= VerticalDistance::Altitude(6000) {
                Speed::kt(110.0)
            } else if *vd >= VerticalDistance::Altitude(4000) {
                Speed::kt(109.0)
            } else {
                Speed::kt(107.0)
            };

            let ff = FuelFlow::PerHour(diesel!(Volume::l(21.0)));

            (tas, ff)
        },
        // The data end at 10000 ft so we don't need to create the Performance
        // with more values.
        VerticalDistance::Altitude(10000),
    );

    let takeoff_perf = TakeoffLandingPerformance::builder(vec![
        (
            VerticalDistance::PressureAltitude(0),
            Temperature::c(0.0),
            Length::ft(845.0),
            Length::ft(1510.0),
        ),
        (
            VerticalDistance::PressureAltitude(0),
            Temperature::c(10.0),
            Length::ft(910.0),
            Length::ft(1625.0),
        ),
        (
            VerticalDistance::PressureAltitude(0),
            Temperature::c(20.0),
            Length::ft(980.0),
            Length::ft(1745.0),
        ),
        (
            VerticalDistance::PressureAltitude(0),
            Temperature::c(30.0),
            Length::ft(1055.0),
            Length::ft(1875.0),
        ),
        (
            VerticalDistance::PressureAltitude(0),
            Temperature::c(40.0),
            Length::ft(1135.0),
            Length::ft(2015.0),
        ),
    ])
    .factors(vec![
        // Decrease distances 10% for each 9 knots headwind. For operation
        // with tail winds up to 10 knots, increase distances by 10% for
        // each 2 knots.
        AlteringFactor::DecreaseHeadwind(FactorOfEffect::Rate {
            numerator: 0.1,
            denominator: Speed::kt(9.0),
        }),
        AlteringFactor::IncreaseTailwind(FactorOfEffect::Rate {
            numerator: 0.1,
            denominator: Speed::kt(2.0),
        }),
        // For operation on dry, grass runway, increase distances by 15% of
        // the "ground roll" figure.
        AlteringFactor::IncreaseRWYCC(HashMap::from([
            ((None, Some(RunwaySurface::Grass)), 0.15), // we'll add 15% on any grass
        ])),
    ])
    .build();

    let aircraft = Aircraft::builder()
        .registration("N12345".to_string())
        .stations(vec![
            Station::new(Length::m(0.94), Some(String::from("front seats"))),
            Station::new(Length::m(1.85), Some(String::from("back seats"))),
            Station::new(
                Length::m(2.41),
                Some(String::from("first cargo compartment")),
            ),
            Station::new(
                Length::m(3.12),
                Some(String::from("second cargo compartment")),
            ),
        ])
        .empty_mass(Mass::kg(807.0))
        .empty_balance(Length::m(1.0))
        .fuel_type(FuelType::Diesel)
        .tanks(vec![FuelTank::new(Volume::l(168.8), Length::m(1.22))])
        .cg_envelope(vec![
            CGLimit::new(Mass::kg(0.0), Length::m(0.89)),
            CGLimit::new(Mass::kg(885.0), Length::m(0.89)),
            CGLimit::new(Mass::kg(1111.0), Length::m(1.02)),
            CGLimit::new(Mass::kg(1111.0), Length::m(1.20)),
            CGLimit::new(Mass::kg(0.0), Length::m(1.20)),
        ])
        .build()
        .expect("all required aircraft parameter should be configured");

    let mut fms = FMS::new();

    // read the ARINC database
    let ed_nd = NavigationData::try_from_arinc424(ARINC_424_RECORDS)?;

    fms.modify_nd(|nd| nd.append(ed_nd))?;

    // decode a route from EDDH to EDHF with winds at 20 kt from 290° and
    // cruising speed of 107 kt and an altitude of 2500 ft. Takeoff runway in
    // EDDH is runway 33 and landing runway in EDHF is 20.
    fms.decode("29020KT N0107 A0250 EDDH33 N2 N1 DCT EDHF20".to_string())?;

    // Now we can enter some data into the flight planning to get a fuel planning
    // and mass & balance calculation.
    let mut builder = FlightPlanning::builder();

    builder
        .aircraft(aircraft)
        .mass(vec![
            // we're in the front
            Mass::kg(80.0),
            // and no mass on the other stations
            Mass::kg(0.0),
            Mass::kg(0.0),
            Mass::kg(0.0),
        ])
        .policy(FuelPolicy::ManualFuel(diesel!(Volume::l(80.0))))
        .taxi(diesel!(Volume::l(10.0)))
        .reserve(Reserve::Manual(Duration::s(1800))) // 30 min
        .perf(perf)
        .takeoff_perf(takeoff_perf)
        // we use the route's wind so no need to specify it here
        .origin_rwycc(RunwayConditionCode::Six)
        .origin_temperature(Temperature::c(20.0));

    fms.set_flight_planning(builder)?;

    println!("{}", fms.print(40));

    Ok(())
}
