// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Joe Pearson
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

use efb::nd::{Fix, NavigationData};
use efb::route::Route;

const ARINC_424_RECORDS: &'static str = r#"SEURP EDDHEDA        0        N N53374900E009591762E002000053                   P    MWGE    HAMBURG                       356462409
SEURP EDDHEDGRW33    0120273330 N53374300E009595081                          151                                           124362502
SEURPCEDDHED N1    ED0    V     N53482105E010015451                                 WGE           NOVEMBER1                359892409
SEURPCEDDHED N2    ED0    V     N53405701E010000576                                 WGE           NOVEMBER2                359902409
SEURP EDHFEDA        0        N N53593300E009343600E000000082                   P    MWGE    ITZEHOE/HUNGRIGER WOLF        320782409
SEURP EDHFEDGRW20    0034122060 N53594752E009344856                          098                                           120792502
"#;

const ROUTE: &'static str = r#"EDDH RWY33 N2 N1 DCT EDHF RWY20"#;

fn route() -> Route {
    let nd = NavigationData::try_from_arinc424(ARINC_424_RECORDS).expect("records should be valid");
    let mut route = Route::new();

    route.decode(ROUTE, &nd).expect("route should decode");

    route
}

#[test]
fn origin_and_destination() {
    let route = route();

    let origin = route.origin().expect("route should have a origin");
    let destination = route
        .destination()
        .expect("route should have a destination");

    assert_eq!(origin.ident(), "EDDH");
    assert_eq!(destination.ident(), "EDHF");
}

#[test]
fn takeoff_rwy() {
    let route = route();
    let designator = route.takeoff_rwy().map(|rwy| rwy.designator.clone());
    assert_eq!(designator, Some(String::from("33")));
}

#[test]
fn landing_rwy() {
    let route = route();
    let designator = route.landing_rwy().map(|rwy| rwy.designator.clone());
    assert_eq!(designator, Some(String::from("20")));
}

#[test]
fn accumulate_legs() {
    let route = route();
    let mut iter = route.accumulate_legs(None);

    // total to N2
    assert_eq!(
        iter.next()
            .expect("route should have leg")
            .dist()
            .value()
            .round(),
        3.0
    );

    // total to N1
    assert_eq!(
        iter.next()
            .expect("route should have leg")
            .dist()
            .value()
            .round(),
        11.0
    );

    // total to EDHF
    assert_eq!(
        iter.next()
            .expect("route should have leg")
            .dist()
            .value()
            .round(),
        30.0
    );
}
