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

use std::collections::HashMap;
use std::str::FromStr;

use efb::aircraft::{LoadedStation, Station};
use efb::fp::{
    AlteringFactor, FactorOfEffect, MassAndBalance, RunwayAnalysis, RunwayAnalysisNote,
    TakeoffLandingPerformance,
};
use efb::nd::{Runway, RunwayConditionCode, RunwaySurface};
use efb::{VerticalDistance, Wind};

use efb::measurements::*;

/// The fictitious grass runway 27 of the runway analysis.
fn west_grass_rwy() -> Runway {
    Runway {
        designator: String::from("27"),
        bearing: Angle::t(270.0),
        length: Length::ft(3600.0),
        tora: Some(Length::ft(2900.0)),
        toda: Some(Length::ft(2900.0)),
        lda: Some(Length::ft(2900.0)),
        displaced_threshold: None,
        surface: Some(RunwaySurface::Grass),
        slope: Some(0.0),
        elev: VerticalDistance::Gnd,
    }
}

/// Runway analysis to test.
///
/// # Conditions
///
/// This analysis uses performance data of a Cessna 172R with altering factors
/// according to the POH. Takeoff is on a fictitious grass runway 27 with
/// following properties:
///
/// - Length: 3600ft
/// - TORA: 2900ft
/// - TODA: 2900ft
/// - LDA: 2900ft
/// - Slope: 0%
/// - Elev: GND
///
/// The influences affecting the takeoff are 10kt wind directly on the RWY at
/// 20°C. The RWYCC is 6.
///
/// # Expected Results
///
/// For wind and runway surface we get the following factors: -11.1111% for 10kt
/// headwind and +15% for dry grass. At 20°C on ground level, the ground roll
/// from the table is 980ft and distance to clear a 50ft obstacle is 1745ft. The
/// grass has only an effect on the ground roll so we get the following
/// estimated lengths:
///
/// - Ground roll: 1001.7777ft
/// - Distance to clear 50ft obstacle: 1744.8888ft
///
/// With a TORA of 2900ft we have a margin of 1898.2223ft which is 65% of the
/// available length.
fn rwy_analysis(rwy: Runway) -> RunwayAnalysis {
    let perf = TakeoffLandingPerformance::builder(vec![
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

    // the mass and balance is irrelevant for this test since we don't have any
    // mass factors
    let mb = MassAndBalance::new(&vec![LoadedStation {
        station: Station::new(Length::m(1.0), None),
        on_ramp: Mass::kg(1111.0),
        after_landing: Mass::kg(1111.0),
    }]);

    RunwayAnalysis::takeoff(
        &rwy,
        RunwayConditionCode::Six,
        &Wind::from_str("27010KT").unwrap(),
        Temperature::c(20.0),
        &mb,
        &perf,
        None,
    )
}

#[test]
fn wind_components() {
    let rwy_analysis = rwy_analysis(west_grass_rwy());
    assert_eq!(rwy_analysis.headwind(), &Speed::kt(10.0));
    assert_eq!(rwy_analysis.crosswind(), &Speed::kt(0.0));
}

#[test]
fn ground_roll_and_distance_to_clear_obstacle() {
    let rwy_analysis = rwy_analysis(west_grass_rwy());

    assert!(
        *(*rwy_analysis.ground_roll() - Length::ft(1001.7777)).value() <= f32::EPSILON,
        "the ground roll estimated with {} wasn't correct!",
        rwy_analysis.ground_roll()
    );

    assert!(
        *(*rwy_analysis.clear_obstacle() - Length::ft(1744.8888)).value() <= f32::EPSILON,
        "the distance to clear a 50ft obstacle estimated with {} wasn't correct!",
        rwy_analysis.ground_roll()
    );
}

#[test]
fn ground_roll_margin() {
    let rwy_analysis = rwy_analysis(west_grass_rwy());
    assert!(*(*rwy_analysis.margin() - Length::ft(1898.2223)).value() <= f32::EPSILON);
    assert_eq!((rwy_analysis.pct_margin() * 100.0).round(), 65.0);
}

#[test]
fn has_no_notes_with_known_inputs() {
    let rwy_analysis = rwy_analysis(west_grass_rwy());
    assert!(rwy_analysis.notes().is_empty());
}

#[test]
fn notes_estimated_tora_and_unknown_surface() {
    let rwy = Runway {
        tora: None,
        surface: None,
        ..west_grass_rwy()
    };

    assert_eq!(
        rwy_analysis(rwy).notes(),
        [
            RunwayAnalysisNote::ToraEstimated,
            RunwayAnalysisNote::SurfaceUnknown
        ]
    );
}
