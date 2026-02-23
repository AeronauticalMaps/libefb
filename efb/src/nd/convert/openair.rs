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

//! Parser for the OpenAir airspace and terrain description language.
//!
//! The current implementation parses only the airspace descriptions as defined
//! by the [user guide]. For each command, an [OpenAirElement] is created which
//! is than converted to an [Airspace].
//!
//! [user guide]: http://www.winpilot.com/UsersGuide/UserAirspace.asp

use std::str::FromStr;

use crate::error::Error;
use crate::fc;
use crate::nd::{Airspace, AirspaceClassification, AirspaceType, NavigationData};
use crate::VerticalDistance;
use geo::Point;

impl NavigationData {
    pub fn try_from_openair(s: &str) -> Result<Self, Error> {
        // TODO: Move OpenAir parser into dedicated crate and optimize parsing.
        let mut builder = NavigationData::builder();
        let mut element = OpenAirElement::new();

        s.lines().for_each(|command| {
            if let Some(airspace) = Self::parse_command(command, &mut element) {
                builder.add_airspace(airspace);
            }
        });

        builder.add_airspace((&mut element).into());

        Ok(builder.with_source(s.as_bytes()).build())
    }

    fn parse_command(command: &str, element: &mut OpenAirElement) -> Option<Airspace> {
        let record_type = command.get(0..2);
        let record = command.get(3..);
        let mut airspace = None;

        // TODO: Flag invalid airspaces!
        match record_type {
            Some("AC") => {
                if element.ac.is_some() {
                    airspace = Some(element.into());
                    *element = OpenAirElement::new();
                }

                element.ac = record?.parse::<String>().ok();
            }
            Some("AN") => element.an = record?.parse::<String>().ok(),
            Some("AH") => element.ah = record?.parse::<OpenAirVerticalDistance>().ok(),
            Some("AL") => element.al = record?.parse::<OpenAirVerticalDistance>().ok(),
            Some("DP") => {
                if let Ok(coordinate) = record?.parse::<OpenAirCoordinate>() {
                    element.dp.push(coordinate.into_inner());
                }
            }
            _ => {}
        }

        airspace
    }
}

/// An element representing an airspace.
struct OpenAirElement {
    /// Airspace class.
    ac: Option<String>,

    /// Airspace name.
    an: Option<String>,

    /// Airspace ceiling.
    ah: Option<OpenAirVerticalDistance>,

    /// Airspace floor.
    al: Option<OpenAirVerticalDistance>,

    /// Polygon points as geo::Coord (x=lon, y=lat).
    dp: Vec<geo::Coord<f64>>,
}

/// Parses an OpenAir `AC` (airspace class) value into an airspace type and
/// optional classification.
///
/// For bare class letters (A-G), the type defaults to [`AirspaceType::CTA`]
/// since OpenAir doesn't distinguish structural types.
fn parse_openair_class(ac: &str) -> (AirspaceType, Option<AirspaceClassification>) {
    match ac {
        "A" => (AirspaceType::CTA, Some(AirspaceClassification::A)),
        "B" => (AirspaceType::CTA, Some(AirspaceClassification::B)),
        "C" => (AirspaceType::CTA, Some(AirspaceClassification::C)),
        "D" => (AirspaceType::CTA, Some(AirspaceClassification::D)),
        "E" => (AirspaceType::CTA, Some(AirspaceClassification::E)),
        "F" => (AirspaceType::CTA, Some(AirspaceClassification::F)),
        "G" => (AirspaceType::CTA, Some(AirspaceClassification::G)),
        "CTR" => (AirspaceType::CTR, None),
        "TMA" => (AirspaceType::TMA, None),
        "R" => (AirspaceType::Restricted, None),
        "Q" => (AirspaceType::Danger, None),
        "P" => (AirspaceType::Prohibited, None),
        "TMZ" => (AirspaceType::TMZ, None),
        "RMZ" => (AirspaceType::RMZ, None),
        _ => (AirspaceType::CTA, None),
    }
}

impl OpenAirElement {
    fn new() -> Self {
        Self {
            ac: None,
            an: None,
            ah: None,
            al: None,
            dp: Vec::new(),
        }
    }
}

impl From<&mut OpenAirElement> for Airspace {
    fn from(element: &mut OpenAirElement) -> Self {
        let mut coords = std::mem::take(&mut element.dp);

        if let Some(first) = coords.first() {
            if coords.last() != Some(first) {
                coords.push(*first);
            }
        }

        let (airspace_type, classification) =
            parse_openair_class(&element.ac.take().unwrap_or_default());

        Self {
            name: element.an.take().unwrap_or_default(),
            airspace_type,
            classification,
            ceiling: element.ah.take().unwrap_or_default().into_inner(),
            floor: element.al.take().unwrap_or_default().into_inner(),
            polygon: geo::Polygon::new(geo::LineString::from(coords), vec![]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseOpenAirCoordinateError;

#[derive(Debug, PartialEq)]
struct OpenAirCoordinate(Point<f64>);

impl OpenAirCoordinate {
    pub fn into_inner(self) -> geo::Coord<f64> {
        geo::Coord {
            x: self.0.x(),
            y: self.0.y(),
        }
    }
}

impl FromStr for OpenAirCoordinate {
    type Err = ParseOpenAirCoordinateError;

    // 37:53:00 N 116:55:30 W
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(&[' ', ':'][..]);

        // parse latitude
        let d = iter.next().and_then(|s| s.parse::<u8>().ok());
        let m = iter.next().and_then(|s| s.parse::<u8>().ok());
        let s = iter.next().and_then(|s| s.parse::<u8>().ok());
        let ns = iter.next();

        let latitude = match (d, m, s, ns) {
            (Some(d), Some(m), Some(s), Some(ns)) => match ns {
                "N" => Some(fc::dms_to_decimal(d, m, s)),
                "S" => Some(-fc::dms_to_decimal(d, m, s)),
                _ => None,
            },
            _ => None,
        };

        // parse longitude
        let d = iter.next().and_then(|s| s.parse::<u8>().ok());
        let m = iter.next().and_then(|s| s.parse::<u8>().ok());
        let s = iter.next().and_then(|s| s.parse::<u8>().ok());
        let ew = iter.next();

        let longitude = match (d, m, s, ew) {
            (Some(d), Some(m), Some(s), Some(ew)) => match ew {
                "E" => Some(fc::dms_to_decimal(d, m, s)),
                "W" => Some(-fc::dms_to_decimal(d, m, s)),
                _ => None,
            },
            _ => None,
        };

        match (latitude, longitude) {
            // geo uses (x, y) = (longitude, latitude)
            (Some(latitude), Some(longitude)) => Ok(Self(Point::new(longitude, latitude))),
            _ => Err(ParseOpenAirCoordinateError),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseOpenAirVerticalDistanceError;

#[derive(Debug, PartialEq)]
struct OpenAirVerticalDistance(VerticalDistance);

impl OpenAirVerticalDistance {
    pub fn into_inner(self) -> VerticalDistance {
        self.0
    }
}

impl Default for OpenAirVerticalDistance {
    fn default() -> Self {
        Self(VerticalDistance::Gnd)
    }
}

impl FromStr for OpenAirVerticalDistance {
    type Err = ParseOpenAirVerticalDistanceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value_fromstr = s
            .trim()
            .replace(' ', "")
            .trim_matches(char::is_alphabetic)
            .parse::<u16>()
            .map_err(|_| ParseOpenAirVerticalDistanceError);

        let suffix_fromstr = s.trim_matches(char::is_numeric).trim().to_uppercase();

        value_fromstr.map_or(
            // for value less distances we get an Err as value
            match suffix_fromstr.as_str() {
                "UNLIM" | "UNLIMITED" => Ok(OpenAirVerticalDistance(VerticalDistance::Unlimited)),
                "GND" | "SFC" => Ok(OpenAirVerticalDistance(VerticalDistance::Gnd)),
                _ => Err(ParseOpenAirVerticalDistanceError),
            },
            |value| match suffix_fromstr.as_str() {
                "FL" => Ok(OpenAirVerticalDistance(VerticalDistance::Fl(value))),
                "FT AGL" | "AGL" => Ok(OpenAirVerticalDistance(VerticalDistance::Agl(value))),
                "FT MSL" | "MSL" => Ok(OpenAirVerticalDistance(VerticalDistance::Msl(value))),
                "FT" => Ok(OpenAirVerticalDistance(VerticalDistance::Altitude(value))),
                _ => Err(ParseOpenAirVerticalDistanceError),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::fc;

    #[test]
    fn parses_command() {
        let record = r#"AC D
AN TMA BREMEN A
AH FL 65
AL 1500msl
DP 53:06:04 N 8:58:30 E
DP 53:06:10 N 9:04:45 E
DP 52:58:13 N 9:05:04 E
DP 52:58:08 N 8:58:56 E
DP 53:06:04 N 8:58:30 E
"#;

        let nd = NavigationData::try_from_openair(record).expect("OpenAir should parse");

        let tma_bremen_a = Rc::new(Airspace {
            name: String::from("TMA BREMEN A"),
            airspace_type: AirspaceType::CTA,
            classification: Some(AirspaceClassification::D),
            ceiling: VerticalDistance::Fl(65),
            floor: VerticalDistance::Msl(1500),
            polygon: polygon![
                (fc::dms_to_decimal(53, 6, 4), fc::dms_to_decimal(8, 58, 30)),
                (fc::dms_to_decimal(53, 6, 10), fc::dms_to_decimal(9, 4, 45)),
                (fc::dms_to_decimal(52, 58, 13), fc::dms_to_decimal(9, 5, 4)),
                (fc::dms_to_decimal(52, 58, 8), fc::dms_to_decimal(8, 58, 56)),
                (fc::dms_to_decimal(53, 6, 4), fc::dms_to_decimal(8, 58, 30))
            ],
        });

        assert_eq!(nd.airspaces, vec!(tma_bremen_a));
    }

    #[test]
    fn parses_coordinate() {
        let north_west = "37:53:00 N 116:55:30 W".parse::<OpenAirCoordinate>();
        let coord = north_west.unwrap().into_inner();
        assert!((coord.y - fc::dms_to_decimal(37, 53, 0)).abs() < 0.0001);
        assert!((coord.x - (-fc::dms_to_decimal(116, 55, 30))).abs() < 0.0001);

        let south_east = "50:34:00 S 16:55:30 E".parse::<OpenAirCoordinate>();
        let coord = south_east.unwrap().into_inner();
        assert!((coord.y - (-fc::dms_to_decimal(50, 34, 0))).abs() < 0.0001);
        assert!((coord.x - fc::dms_to_decimal(16, 55, 30)).abs() < 0.0001);

        let invalid = "50.1202 X 16.214 Y".parse::<OpenAirCoordinate>();
        assert_eq!(invalid, Err(ParseOpenAirCoordinateError),);
    }

    #[test]
    fn parses_vertical_distance() {
        let agl = "1500 ft agl".parse::<OpenAirVerticalDistance>();
        assert_eq!(agl.unwrap().into_inner(), VerticalDistance::Agl(1500));

        let altitude = "6400ft".parse::<OpenAirVerticalDistance>();
        assert_eq!(
            altitude.unwrap().into_inner(),
            VerticalDistance::Altitude(6400)
        );

        let fl = "FL95".parse::<OpenAirVerticalDistance>();
        assert_eq!(fl.unwrap().into_inner(), VerticalDistance::Fl(95));

        let gnd = "GND".parse::<OpenAirVerticalDistance>();
        assert_eq!(gnd.unwrap().into_inner(), VerticalDistance::Gnd);

        let msl = "2500msl".parse::<OpenAirVerticalDistance>();
        assert_eq!(msl.unwrap().into_inner(), VerticalDistance::Msl(2500));

        let unlimited = "UNLIM".parse::<OpenAirVerticalDistance>(); // UNLIM (Mon-Fri)
        assert_eq!(unlimited.unwrap().into_inner(), VerticalDistance::Unlimited);

        let err = "1500 foo".parse::<OpenAirVerticalDistance>();
        assert_eq!(err, Err(ParseOpenAirVerticalDistanceError));
    }
}
