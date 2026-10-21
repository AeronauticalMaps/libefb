// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Joe Pearson
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

//! Field-level conversions from AIXM coded values to efb measurement and
//! classification types.

use aixm::AirspaceVolume;

use crate::measurements::{Angle, Length};
use crate::nd::*;
use crate::VerticalDistance;

/// Maps an AIXM airspace type code to an [`AirspaceType`] and optional
/// [`AirspaceClassification`].
pub fn airspace_type_and_class(
    airspace_type: Option<&str>,
) -> (AirspaceType, Option<AirspaceClassification>) {
    match airspace_type {
        Some("A") => (AirspaceType::CTA, Some(AirspaceClassification::A)),
        Some("CLASS_B") => (AirspaceType::CTA, Some(AirspaceClassification::B)),
        Some("CLASS_C" | "C") => (AirspaceType::CTA, Some(AirspaceClassification::C)),
        Some("D" | "CLASS_D") => (AirspaceType::CTA, Some(AirspaceClassification::D)),
        Some("E" | "CLASS_E") => (AirspaceType::CTA, Some(AirspaceClassification::E)),
        Some("F" | "CLASS_F") => (AirspaceType::CTA, Some(AirspaceClassification::F)),
        Some("G" | "CLASS_G") => (AirspaceType::CTA, Some(AirspaceClassification::G)),
        Some("CTA") => (AirspaceType::CTA, None),
        Some("CTR") => (AirspaceType::CTR, None),
        Some("TMA") => (AirspaceType::TMA, None),
        Some("RAS") => (AirspaceType::RadarZone, None),
        Some("TMZ") => (AirspaceType::TMZ, None),
        Some("RMZ") => (AirspaceType::RMZ, None),
        Some("R" | "RESTRICT") => (AirspaceType::Restricted, None),
        Some("D_OTHER" | "DA") => (AirspaceType::Danger, None),
        Some("P" | "PROHIBIT") => (AirspaceType::Prohibited, None),
        _ => (AirspaceType::CTA, None),
    }
}

/// Maps an AIXM vertical limit (value, unit, reference) to a [`VerticalDistance`].
pub fn vertical_distance(
    value: Option<&str>,
    uom: Option<&str>,
    reference: Option<&str>,
) -> VerticalDistance {
    let value = match value {
        Some(v) => v.trim(),
        None => return VerticalDistance::Unlimited,
    };

    match value {
        "GND" | "SFC" => return VerticalDistance::Gnd,
        "UNL" => return VerticalDistance::Unlimited,
        _ => {}
    }

    if let Some(uom) = uom {
        if uom == "FL" {
            if let Ok(fl) = value.parse::<u16>() {
                return VerticalDistance::Fl(fl);
            }
        }
    }

    if let Ok(num) = value.parse::<u16>() {
        match reference {
            Some("MSL") | None => VerticalDistance::Msl(num),
            Some("SFC") | Some("GND") => VerticalDistance::Altitude(num),
            _ => VerticalDistance::Msl(num),
        }
    } else {
        VerticalDistance::Unlimited
    }
}

/// Maps an AIXM surface composition code to a [`RunwaySurface`].
pub fn runway_surface(composition: Option<&str>) -> RunwaySurface {
    match composition {
        Some("ASPH") => RunwaySurface::Asphalt,
        Some("CONC") => RunwaySurface::Concrete,
        Some("GRASS") => RunwaySurface::Grass,
        _ => RunwaySurface::Asphalt,
    }
}

/// Maps a true or magnetic bearing value to an [`Angle`].
///
/// Prefers true bearing when available.
pub fn bearing(true_bearing: Option<f64>, magnetic_bearing: Option<f64>) -> Angle {
    if let Some(tb) = true_bearing {
        Angle::t(tb as f32)
    } else if let Some(mb) = magnetic_bearing {
        Angle::m(mb as f32)
    } else {
        Angle::t(0.0)
    }
}

/// Maps a runway length value and unit to a [`Length`].
pub fn runway_length(value: Option<f64>, uom: Option<&str>) -> Length {
    match (value, uom) {
        (Some(v), Some("M")) => Length::m(v as f32),
        (Some(v), Some("FT")) => Length::ft(v as f32),
        (Some(v), Some("KM")) => Length::km(v as f32),
        (Some(v), _) => Length::m(v as f32),
        (None, _) => Length::m(0.0),
    }
}

/// Maps a field elevation value and unit to a [`VerticalDistance`].
pub fn field_elevation(value: Option<f64>, uom: Option<&str>) -> VerticalDistance {
    match (value, uom) {
        (Some(v), Some("FT")) => VerticalDistance::Msl(v as u16),
        (Some(v), Some("M")) => {
            // Convert meters to feet for the standard altitude representation
            VerticalDistance::Msl((v * 3.28084) as u16)
        }
        (Some(v), _) => VerticalDistance::Msl(v as u16),
        (None, _) => VerticalDistance::Gnd,
    }
}

/// Extracts the ceiling and floor [`VerticalDistance`] from an airspace volume.
pub fn volume_limits(vol: &AirspaceVolume) -> (VerticalDistance, VerticalDistance) {
    let ceiling = vertical_distance(
        vol.upper_limit.as_deref(),
        vol.upper_limit_uom.as_deref(),
        vol.upper_limit_ref.as_deref(),
    );
    let floor = vertical_distance(
        vol.lower_limit.as_deref(),
        vol.lower_limit_uom.as_deref(),
        vol.lower_limit_ref.as_deref(),
    );
    (ceiling, floor)
}
