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

use core::f32;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;
use std::ops::Div;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::measurements::{Altitude, Length, LengthUnit, Pressure};

mod constants {
    pub const METER_IN_FEET: f32 = 3.28084;
}

/// A vertical distance.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(C)]
pub enum VerticalDistance {
    /// Absolute Altitude as distance above ground level in feet.
    Agl(u16),

    /// Altitude in feet with reference to a local air pressure.
    Altitude(u16), // TODO does it make sense to have ALT?

    /// Pressure altitude in feet.
    PressureAltitude(i16),

    /// Flight level in hundreds of feet as altitude at standard air pressure.
    Fl(u16),

    /// Ground level.
    Gnd,

    /// True Altitude as distance above mean sea level.
    Msl(u16),

    /// An unlimited vertical distance.
    Unlimited,
}

impl VerticalDistance {
    /// Resolves this vertical distance to an altitude above mean sea level.
    ///
    /// Returns `None` for [`VerticalDistance::Unlimited`] which has no finite
    /// altitude representation.
    ///
    /// # Conversion rules
    ///
    /// | Variant               | Resolved as                           |
    /// |-----------------------|---------------------------------------|
    /// | `Gnd`                 | `elevation`                           |
    /// | `Agl(n)`              | `elevation + n ft`                    |
    /// | `Msl(n)`              | `n ft`                                |
    /// | `Altitude(n)`         | `n ft` (QNH-referenced, equal to MSL) |
    /// | `Fl(n)`               | `n × 100 ft`, corrected for QNH       |
    /// | `PressureAltitude(n)` | `n ft`, corrected for QNH             |
    /// | `Unlimited`           | `None`                                |
    ///
    /// The QNH correction for flight level and pressure altitude uses the standard
    /// lapse rate approximation of 27 ft/hPa, valid for normal QNH ranges.
    pub fn to_msl(&self, qnh: Pressure, elevation: Length) -> Option<Altitude> {
        // Correction in feet: positive when QNH is above standard (denser air
        // means the same FL is at a higher true altitude).
        //
        // The 27 ft/hPa factor is derived from the hydrostatic equation at ISA
        // sea-level conditions (ρ = 1.225 kg/m³, g = 9.80665 m/s²), giving
        // dP/dh ≈ −1 hPa per 8.3 m ≈ −1 hPa per 27 ft.
        // See: https://www.weather.gov/media/epz/wxcalc/pressureAltitude.pdf
        let qnh_correction_ft = (qnh - Pressure::STD).to_si() / 100.0 * 27.0;
        let ground_ft = *elevation.convert_to(LengthUnit::Feet).value();

        Some(Altitude::ft(match self {
            Self::Gnd => ground_ft,
            Self::Agl(n) => ground_ft + *n as f32,
            Self::Msl(n) => *n as f32,
            Self::Altitude(n) => *n as f32,
            Self::Fl(n) => *n as f32 * 100.0 + qnh_correction_ft,
            Self::PressureAltitude(n) => *n as f32 + qnh_correction_ft,
            Self::Unlimited => return None,
        }))
    }

    /// Returns the pressure altitude based on the elevation and the QNH.
    ///
    /// # Errors
    ///
    /// Will return [`ImplausibleValue`] if the QNH is implausible causing the
    /// pressure altitude to overflow.
    ///
    /// [`ImplausibleValue`]: Error::ImplausibleValue
    pub fn pa(elevation: Length, qnh: Pressure) -> Result<Self, Error> {
        // https://www.weather.gov/media/epz/wxcalc/pressureAltitude.pdf
        let elevation_ft = *elevation.convert_to(LengthUnit::Feet).value() as i16;
        let (pa, overflowed) = elevation_ft.overflowing_add(
            (145366.45 * (1.0 - (qnh / Pressure::STD).powf(0.190284))).round() as i16,
        );

        if overflowed {
            Err(Error::ImplausibleValue)
        } else {
            Ok(Self::PressureAltitude(pa))
        }
    }
}

impl FromStr for VerticalDistance {
    type Err = Error;

    /// Parses a string `s` to return a VerticalDistance.
    ///
    /// The string should be according to ICAO Doc. 4444 Annex 2:
    /// - Flight level, expressed as F followed by 3 figures e.g. `F085`
    /// - Standard metric level in tens of metres, expressed by S followed by 4
    ///   figures e.g. `S1130`
    /// - Altitude in hundreds of feet, expressed as A followed by 3 figures
    ///   e.g. `A045`
    /// - Altitude in tens of metres, expressed as M followed by 4 figures e.g.
    ///   `M0840`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! value {
            ($s:expr, $index:expr) => {
                $s.get($index)
                    .and_then(|s| s.parse::<u16>().ok())
                    .ok_or(Error::UnexpectedString)
            };
        }

        match s.get(0..1).unwrap_or_default() {
            // first character is the unit
            "F" => Ok(Self::Fl(value!(s, 1..4)?)),
            "S" => Ok(Self::Fl(
                // value in tens of meter or hundreds of feet
                (value!(s, 1..5)? as f32 * constants::METER_IN_FEET / 10.0).round() as u16,
            )),
            "A" => Ok(Self::Altitude(value!(s, 1..4)? * 100)), // value in hundredth of feet
            "M" => Ok(Self::Altitude(
                // value in tens of meter
                (value!(s, 1..5)? as f32 * constants::METER_IN_FEET).round() as u16,
            )),
            _ => Err(Error::UnexpectedString),
        }
    }
}

impl fmt::Display for VerticalDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerticalDistance::Gnd => write!(f, "GND"),
            VerticalDistance::Fl(value) => write!(f, "FL{value}"),
            VerticalDistance::Agl(value) => write!(f, "{value} AGL"),
            VerticalDistance::Msl(value) => write!(f, "{value} MSL"),
            VerticalDistance::Altitude(value) => write!(f, "{value} ALT"),
            VerticalDistance::PressureAltitude(value) => write!(f, "PA {value}"),
            VerticalDistance::Unlimited => write!(f, "unlimited"),
        }
    }
}

/// # Panics
///
/// Explain why and when we panic...
impl Ord for VerticalDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // ground is always less
            (Self::Gnd, Self::Gnd) => Ordering::Equal,
            (Self::Gnd, _) => Ordering::Less,
            (_, Self::Gnd) => Ordering::Greater,

            // and unlimited is always greater
            (Self::Unlimited, Self::Unlimited) => Ordering::Equal,
            (Self::Unlimited, _) => Ordering::Greater,
            (_, Self::Unlimited) => Ordering::Less,

            // now compare what can only be compared to the same type
            (Self::Agl(v), Self::Agl(o)) => v.cmp(o),
            (Self::PressureAltitude(v), Self::PressureAltitude(o)) => v.cmp(o),

            _ => {
                fn to_msl(vd: &VerticalDistance) -> u16 {
                    match vd {
                        VerticalDistance::Fl(v) => v * 100,
                        VerticalDistance::Msl(v) => *v,
                        VerticalDistance::Altitude(v) => *v,
                        _ => panic!(
                            "We can't compare {vd} here, since it doesn't reference to common datum."
                        ),
                    }
                }

                to_msl(self).cmp(&to_msl(other))
            }
        }
    }
}

impl PartialOrd for VerticalDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Div for VerticalDistance {
    type Output = f32;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Gnd, Self::Gnd) => 1.0,
            (Self::Fl(a), Self::Fl(b)) => (a / b).into(),
            (Self::Agl(a), Self::Agl(b)) => (a / b).into(),
            (Self::Msl(a), Self::Msl(b)) => (a / b).into(),
            (Self::Altitude(a), Self::Altitude(b)) => (a / b).into(),
            (Self::PressureAltitude(a), Self::PressureAltitude(b)) => (a / b).into(),
            (Self::Unlimited, Self::Unlimited) => 1.0,
            _ => unimplemented!(
                "Division of vertical distances of different types is not yet supported!"
            ),
        }
    }
}

impl From<VerticalDistance> for f32 {
    fn from(value: VerticalDistance) -> Self {
        match value {
            VerticalDistance::Gnd => 0.0,
            VerticalDistance::Fl(value) => value.into(),
            VerticalDistance::Agl(value) => value.into(),
            VerticalDistance::Msl(value) => value.into(),
            VerticalDistance::Altitude(value) => value.into(),
            VerticalDistance::PressureAltitude(value) => value.into(),
            VerticalDistance::Unlimited => f32::INFINITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_distance_from_str() {
        assert_eq!(
            "F085".parse::<VerticalDistance>(),
            Ok(VerticalDistance::Fl(85))
        );
        assert_eq!(
            "S1130".parse::<VerticalDistance>(),
            Ok(VerticalDistance::Fl(371))
        );
        assert_eq!(
            "A025".parse::<VerticalDistance>(),
            Ok(VerticalDistance::Altitude(2500))
        );
        assert_eq!(
            "M0762".parse::<VerticalDistance>(),
            Ok(VerticalDistance::Altitude(2500))
        );
        assert_eq!(
            "F08".parse::<VerticalDistance>(),
            Err(Error::UnexpectedString)
        );
    }

    #[test]
    fn gnd_is_least() {
        assert!(VerticalDistance::Gnd < VerticalDistance::Agl(1000));
        assert!(VerticalDistance::Gnd < VerticalDistance::Altitude(1000));
        assert!(VerticalDistance::Gnd < VerticalDistance::Fl(10));
        assert!(VerticalDistance::Gnd == VerticalDistance::Gnd);
        assert!(VerticalDistance::Gnd < VerticalDistance::Msl(100));
        assert!(VerticalDistance::Gnd < VerticalDistance::Unlimited);
    }

    #[test]
    fn unlimited_is_greatest() {
        assert!(VerticalDistance::Unlimited > VerticalDistance::Agl(1000));
        assert!(VerticalDistance::Unlimited > VerticalDistance::Altitude(1000));
        assert!(VerticalDistance::Unlimited > VerticalDistance::Fl(10));
        assert!(VerticalDistance::Unlimited > VerticalDistance::Gnd);
        assert!(VerticalDistance::Unlimited > VerticalDistance::Msl(100));
        assert!(VerticalDistance::Unlimited == VerticalDistance::Unlimited);
    }

    #[test]
    fn cmp_vertical_distances() {
        assert!(VerticalDistance::Agl(1000) < VerticalDistance::Agl(2000));
        assert!(VerticalDistance::Altitude(1000) < VerticalDistance::Altitude(2000));
        assert!(VerticalDistance::Msl(1000) < VerticalDistance::Fl(100));
    }

    #[test]
    fn to_msl_at_standard_pressure() {
        let std_qnh = Pressure::STD;
        let zero_elev = Length::m(0.0);

        // FL100 at standard QNH = 10 000 ft MSL
        let alt = VerticalDistance::Fl(100)
            .to_msl(std_qnh, zero_elev)
            .unwrap();
        assert!((alt.to_si() - Length::ft(10_000.0).to_si()).abs() < 1.0);

        // MSL 5 000 ft is always 5 000 ft
        let alt = VerticalDistance::Msl(5_000)
            .to_msl(std_qnh, zero_elev)
            .unwrap();
        assert!((alt.to_si() - Length::ft(5_000.0).to_si()).abs() < 1.0);

        // Unlimited has no MSL representation
        assert!(VerticalDistance::Unlimited
            .to_msl(std_qnh, zero_elev)
            .is_none());
    }

    #[test]
    fn to_msl_qnh_correction() {
        // High QNH (1033 hPa, +20 hPa above std): FL100 should read ~540 ft higher
        let high_qnh = Pressure::STD + Pressure::h_pa(20.0);
        let expected_correction = 20.0 * 27.0; // +540 ft
        let alt = VerticalDistance::Fl(100)
            .to_msl(high_qnh, Length::m(0.0))
            .unwrap();
        let expected_ft = 10_000.0 + expected_correction;
        assert!((alt.to_si() - Length::ft(expected_ft).to_si()).abs() < 2.0);
    }

    #[test]
    fn pa_at_standard_qnh_equals_elevation() {
        // At standard QNH the correction is zero so PA equals the field elevation.
        let elev = Length::ft(1000.0);
        assert_eq!(
            VerticalDistance::pa(elev, Pressure::STD),
            Ok(VerticalDistance::PressureAltitude(1000))
        );
    }

    #[test]
    fn pa_sea_level_std_qnh_is_zero() {
        assert_eq!(
            VerticalDistance::pa(Length::m(0.0), Pressure::STD),
            Ok(VerticalDistance::PressureAltitude(0))
        );
    }

    #[test]
    fn to_msl_agl_adds_ground_elevation() {
        let std_qnh = Pressure::STD;
        let ground = Length::ft(500.0);

        // 1 000 ft AGL above 500 ft ground = 1 500 ft MSL
        let alt = VerticalDistance::Agl(1_000)
            .to_msl(std_qnh, ground)
            .unwrap();
        assert!((alt.to_si() - Length::ft(1_500.0).to_si()).abs() < 1.0);

        // Gnd at a 500 ft airfield = 500 ft MSL
        let alt = VerticalDistance::Gnd.to_msl(std_qnh, ground).unwrap();
        assert!((alt.to_si() - Length::ft(500.0).to_si()).abs() < 1.0);
    }
}
