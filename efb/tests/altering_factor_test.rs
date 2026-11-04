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

use efb::fp::{AlteringFactor, AlteringFactors, FactorOfEffect, Influences};
use efb::measurements::{Angle, Length, Mass, Speed, Temperature};
use efb::nd::{Runway, RunwayConditionCode, RunwaySurface};
use efb::{VerticalDistance, Wind};

fn west_grass_rwy(elev: VerticalDistance) -> Runway {
    Runway {
        designator: String::from("27"),
        bearing: Angle::t(270.0),
        length: Length::m(1100.0),
        tora: Some(Length::m(900.0)),
        toda: Some(Length::m(900.0)),
        lda: Some(Length::m(900.0)),
        displaced_threshold: None,
        surface: Some(RunwaySurface::Grass),
        slope: Some(0.0),
        elev,
    }
}

#[test]
fn factor_for_winds() {
    fn influences_with_wind(wind: &str) -> Influences {
        Influences::new(
            Mass::kg(0.0),
            &west_grass_rwy(VerticalDistance::Gnd),
            &Wind::from_str(wind).unwrap(),
            Temperature::c(15.0),
            RunwayConditionCode::Six,
        )
    }

    // decrease distance 10% for each 9 knots headwind
    let headwind_factor = AlteringFactor::DecreaseHeadwind(FactorOfEffect::Rate {
        numerator: 0.1,
        denominator: Speed::kt(9.0),
    });

    assert_eq!(
        headwind_factor.ground_roll_factor(&influences_with_wind("27018KT")),
        0.8 // 20% decrease for 18kt headwind
    );

    // increase distance 10% for each 2 knots tailwind
    let tailwind_factor = AlteringFactor::IncreaseTailwind(FactorOfEffect::Rate {
        numerator: 0.1,
        denominator: Speed::kt(2.0),
    });

    assert_eq!(
        tailwind_factor.ground_roll_factor(&influences_with_wind("09006KT")),
        1.3 // 30% increase for 6kt tailwind
    );
}

#[test]
fn factor_for_rwycc_and_surface() {
    fn influences_with_rwycc(rwycc: RunwayConditionCode) -> Influences {
        Influences::new(
            Mass::kg(0.0),
            &west_grass_rwy(VerticalDistance::Gnd),
            &Wind::from_str("00000KT").unwrap(),
            Temperature::c(15.0),
            rwycc,
        )
    }

    let grass_factor = AlteringFactor::IncreaseRWYCC(HashMap::from([
        (
            (Some(RunwayConditionCode::Six), Some(RunwaySurface::Grass)),
            0.2,
        ),
        (
            (Some(RunwayConditionCode::Five), Some(RunwaySurface::Grass)),
            0.33,
        ),
        ((None, Some(RunwaySurface::Grass)), 0.2),
        ((Some(RunwayConditionCode::Four), None), 0.4),
    ]));

    // we have a factor of +20% for RWYCC 6 on grass
    assert!(
        grass_factor.ground_roll_factor(&influences_with_rwycc(RunwayConditionCode::Six)) - 1.2
            <= f32::EPSILON
    );

    // we have a factor of +33% for RWYCC 5 on grass
    assert!(
        grass_factor.ground_roll_factor(&influences_with_rwycc(RunwayConditionCode::Five)) - 1.33
            <= f32::EPSILON
    );

    // we have a factor of +20% on grass in any case
    assert!(
        grass_factor.ground_roll_factor(&influences_with_rwycc(RunwayConditionCode::One)) - 1.2
            <= f32::EPSILON
    );

    // and finally, we have a factor of +40% for RWYCC 4 on any surface
    assert!(
        grass_factor.ground_roll_factor(&influences_with_rwycc(RunwayConditionCode::Four)) - 1.4
            <= f32::EPSILON
    );
}

#[test]
fn factor_for_pressure_altitude() {
    fn influences_with_elev(elev: VerticalDistance) -> Influences {
        Influences::new(
            Mass::kg(0.0),
            &west_grass_rwy(elev),
            &Wind::from_str("00000KT").unwrap(),
            Temperature::c(15.0),
            RunwayConditionCode::Six,
        )
    }

    // ... 1000ft: 10%
    // ... 3000ft: 13%
    // 3000ft ...: 18%
    let factor = AlteringFactor::IncreaseAltitude(FactorOfEffect::Range(vec![
        (..=VerticalDistance::PressureAltitude(1000), 0.1),
        (..=VerticalDistance::PressureAltitude(3000), 0.13),
        (..=VerticalDistance::Unlimited, 0.18),
    ]));

    assert!(
        factor.ground_roll_factor(&influences_with_elev(VerticalDistance::PressureAltitude(
            1000
        ))) - 1.1
            <= f32::EPSILON
    );

    assert!(
        factor.ground_roll_factor(&influences_with_elev(VerticalDistance::PressureAltitude(
            2000
        ))) - 1.13
            <= f32::EPSILON
    );

    assert!(
        factor.ground_roll_factor(&influences_with_elev(VerticalDistance::PressureAltitude(
            20_000
        ))) - 1.18
            <= f32::EPSILON
    );
}

#[test]
fn product_of_factors() {
    let influences = Influences::new(
        Mass::kg(0.0),
        &west_grass_rwy(VerticalDistance::Gnd),
        &Wind::from_str("27010KT").unwrap(),
        Temperature::c(15.0),
        RunwayConditionCode::Six,
    );

    let factors = AlteringFactors::new(vec![
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
    ]);

    // For 10kt headwind and -10%/9kt the ground roll decreases by -11.1111111%.
    // For the grass runway, we increase the distance by 15%.
    // This increases the ground roll in total by 2.22%.
    assert!(factors.ground_roll_factor(&influences) - 1.0222222 <= f32::EPSILON);
}
