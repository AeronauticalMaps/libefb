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

//! The AIXM code lists and units of measurement that the features read.
//!
//! Each list names the values that are read today. A value outside the
//! list, such as an `OTHER:` extension, is kept as `Other`.

use std::fmt;

use crate::Error;

/// A value of an AIXM code list.
pub(crate) trait Code {
    fn from_code(code: &str) -> Self;
}

/// Reads a code from its text, for [`crate::stream::Stream::read_text`].
pub(crate) fn code<T: Code>(text: &str) -> Result<T, Error> {
    Ok(T::from_code(text))
}

macro_rules! code_list {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $code:literal,)*
        }
    ) => {
        $(#[$meta])*
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Hash, Debug)]
        pub enum $name {
            $($(#[$variant_meta])* $variant,)*
            /// A value outside the list, e.g. an `OTHER:` extension.
            Other(String),
        }

        impl $name {
            /// Returns the code as AIXM writes it.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $code,)*
                    Self::Other(code) => code,
                }
            }
        }

        impl Code for $name {
            fn from_code(code: &str) -> Self {
                match code {
                    $($code => Self::$variant,)*
                    other => Self::Other(other.to_string()),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

/////////////////////////////////////////////////////////////////////////////
// Code Lists
/////////////////////////////////////////////////////////////////////////////

code_list! {
    /// The type of a designated point (CodeDesignatedPointType).
    CodeDesignatedPointType {
        /// A five-letter name-code designator assigned by ICAO.
        ICAO = "ICAO",
        /// A point designed for a procedure.
        DESIGNED = "DESIGNED",
        /// A point in a terminal area.
        TERMINAL = "TERMINAL",
    }
}

code_list! {
    /// The type of a runway (CodeRunwayType).
    CodeRunwayType {
        /// A runway.
        RWY = "RWY",
        /// A final approach and take-off area of helicopters.
        FATO = "FATO",
    }
}

code_list! {
    /// The role of a point on the centreline of a runway
    /// (CodeRunwayPointRoleType).
    CodeRunwayPointRoleType {
        /// The threshold.
        THR = "THR",
        /// The displaced threshold.
        DISTHR = "DISTHR",
        /// The midpoint of the runway.
        MID = "MID",
        /// The end of the runway.
        END = "END",
        /// The start of the take-off run.
        START_RUN = "START_RUN",
    }
}

code_list! {
    /// The type of a declared distance (CodeDeclaredDistanceType).
    CodeDeclaredDistanceType {
        /// The take-off run available.
        TORA = "TORA",
        /// The take-off distance available.
        TODA = "TODA",
        /// The accelerate-stop distance available.
        ASDA = "ASDA",
        /// The landing distance available.
        LDA = "LDA",
        /// The distance of a displaced threshold.
        DTHR = "DTHR",
    }
}

code_list! {
    /// The composition of a surface (CodeSurfaceCompositionType).
    CodeSurfaceCompositionType {
        /// Asphalt.
        ASPH = "ASPH",
        /// Concrete.
        CONC = "CONC",
        /// Concrete and asphalt.
        CONC_ASPH = "CONC_ASPH",
        /// Grass.
        GRASS = "GRASS",
    }
}

code_list! {
    /// The service a navaid provides (CodeNavaidServiceType).
    CodeNavaidServiceType {
        /// A VOR.
        VOR = "VOR",
        /// A DME.
        DME = "DME",
        /// An NDB.
        NDB = "NDB",
        /// A TACAN.
        TACAN = "TACAN",
        /// A VOR with a DME.
        VOR_DME = "VOR_DME",
        /// A VOR with a TACAN.
        VORTAC = "VORTAC",
        /// An instrument landing system.
        ILS = "ILS",
        /// An instrument landing system with a DME.
        ILS_DME = "ILS_DME",
        /// A localizer.
        LOC = "LOC",
        /// A marker beacon.
        MKR = "MKR",
    }
}

code_list! {
    /// The type of a VOR (CodeVORType).
    CodeVORType {
        /// A conventional VOR.
        VOR = "VOR",
        /// A Doppler VOR.
        DVOR = "DVOR",
    }
}

code_list! {
    /// The use of an NDB (CodeNDBUsageType).
    CodeNDBUsageType {
        /// An NDB used en route.
        ENR = "ENR",
        /// A locator.
        L = "L",
    }
}

code_list! {
    /// The type of an airspace (CodeAirspaceType).
    CodeAirspaceType {
        /// A national airspace system.
        NAS = "NAS",
        /// A flight information region.
        FIR = "FIR",
        /// A part of a flight information region.
        FIR_P = "FIR_P",
        /// An upper flight information region.
        UIR = "UIR",
        /// A part of an upper flight information region.
        UIR_P = "UIR_P",
        /// A control area.
        CTA = "CTA",
        /// A part of a control area.
        CTA_P = "CTA_P",
        /// A part of an oceanic control area.
        OCA_P = "OCA_P",
        /// An oceanic control area.
        OCA = "OCA",
        /// An upper control area.
        UTA = "UTA",
        /// A part of an upper control area.
        UTA_P = "UTA_P",
        /// A terminal control area.
        TMA = "TMA",
        /// A part of a terminal control area.
        TMA_P = "TMA_P",
        /// A control zone.
        CTR = "CTR",
        /// A part of a control zone.
        CTR_P = "CTR_P",
        /// An oceanic transition area.
        OTA = "OTA",
        /// A sector of a control area.
        SECTOR = "SECTOR",
        /// A temporary sector that combines several sectors.
        SECTOR_C = "SECTOR_C",
        /// A temporary segregated area.
        TSA = "TSA",
        /// A cross-border area.
        CBA = "CBA",
        /// A reduced coordination airspace.
        RCA = "RCA",
        /// A regulated airspace not covered by another type.
        RAS = "RAS",
        /// An airway, a control area in the shape of a corridor.
        AWY = "AWY",
        /// A military training route.
        MTR = "MTR",
        /// A prohibited area.
        P = "P",
        /// A restricted area.
        R = "R",
        /// A danger area.
        D = "D",
        /// An air defence identification zone.
        ADIZ = "ADIZ",
        /// An area without a flight information region.
        NO_FIR = "NO_FIR",
        /// A part of a larger airspace.
        PART = "PART",
        /// An airspace of one ICAO classification.
        CLASS = "CLASS",
        /// A political or administrative area.
        POLITICAL = "POLITICAL",
        /// An area of dangerous activities that is no danger area.
        D_OTHER = "D_OTHER",
        /// A temporary reserved area.
        TRA = "TRA",
        /// An alert area.
        A = "A",
        /// A warning area.
        W = "W",
        /// An airspace that protects a navaid or a procedure.
        PROTECT = "PROTECT",
        /// A minimum altitude area.
        AMA = "AMA",
        /// An altimeter setting region.
        ASR = "ASR",
        /// An advisory area.
        ADV = "ADV",
        /// An upper advisory area.
        UADV = "UADV",
        /// An aerodrome traffic zone.
        ATZ = "ATZ",
        /// A part of an aerodrome traffic zone.
        ATZ_P = "ATZ_P",
        /// A helicopter traffic zone.
        HTZ = "HTZ",
        /// A part of a national airspace system.
        NAS_P = "NAS_P",
    }
}

code_list! {
    /// The activity in an airspace (CodeAirspaceActivityType).
    CodeAirspaceActivityType {
        /// Aerodrome traffic.
        AD_TFC = "AD_TFC",
        /// Helicopter traffic.
        HELI_TFC = "HELI_TFC",
        /// Training flights.
        TRAINING = "TRAINING",
        /// Aerobatics.
        AEROBATICS = "AEROBATICS",
        /// An air show.
        AIRSHOW = "AIRSHOW",
        /// Air sports, e.g. air racing.
        SPORT = "SPORT",
        /// Ultralight flights.
        ULM = "ULM",
        /// Gliding.
        GLIDING = "GLIDING",
        /// Paragliding.
        PARAGLIDER = "PARAGLIDER",
        /// Hang gliding.
        HANGGLIDING = "HANGGLIDING",
        /// Parachute jumping.
        PARACHUTE = "PARACHUTE",
        /// Dropping of troops, supplies or other items.
        AIR_DROP = "AIR_DROP",
        /// Balloon flights.
        BALLOON = "BALLOON",
        /// Radiosondes carried aloft by balloons.
        RADIOSONDE = "RADIOSONDE",
        /// Launches of space flights.
        SPACE_FLIGHT = "SPACE_FLIGHT",
        /// Unmanned aerial vehicles.
        UAV = "UAV",
        /// Aerial work, e.g. photography or surveying.
        AERIAL_WORK = "AERIAL_WORK",
        /// Crop dusting.
        CROP_DUSTING = "CROP_DUSTING",
        /// Fire fighting from the air.
        FIRE_FIGHTING = "FIRE_FIGHTING",
        /// Military operations.
        MILOPS = "MILOPS",
        /// Air-to-air refuelling.
        REFUEL = "REFUEL",
        /// Jets climbing on common tracks.
        JET_CLIMBING = "JET_CLIMBING",
        /// Interception exercises.
        EXERCISE = "EXERCISE",
        /// Towing of aerial targets.
        TOWING = "TOWING",
        /// Naval firing exercises.
        NAVAL_EXER = "NAVAL_EXER",
        /// Guided missiles.
        MISSILES = "MISSILES",
        /// Air-to-air or air-to-ground weapons.
        AIR_GUN = "AIR_GUN",
        /// Artillery firing.
        ARTILLERY = "ARTILLERY",
        /// Shooting, e.g. on a range or a hunting area.
        SHOOTING = "SHOOTING",
        /// Blasting with explosives.
        BLASTING = "BLASTING",
        /// Underwater blasting.
        WATER_BLASTING = "WATER_BLASTING",
        /// Rockets fired against hail.
        ANTI_HAIL = "ANTI_HAIL",
        /// A bird hazard.
        BIRD = "BIRD",
        /// Bird migration.
        BIRD_MIGRATION = "BIRD_MIGRATION",
        /// Fireworks.
        FIREWORK = "FIREWORK",
        /// High-intensity radio transmissions.
        HI_RADIO = "HI_RADIO",
        /// High-intensity lights.
        HI_LIGHT = "HI_LIGHT",
        /// Lasers, e.g. a laser show.
        LASER = "LASER",
        /// A nature reserve.
        NATURE = "NATURE",
        /// Animals sensitive to aircraft noise, e.g. a farm or a zoo.
        FAUNA = "FAUNA",
        /// Noise abatement.
        NO_NOISE = "NO_NOISE",
        /// The investigation of an accident.
        ACCIDENT = "ACCIDENT",
        /// A densely populated area.
        POPULATION = "POPULATION",
        /// A very important person.
        VIP = "VIP",
        /// A head of state.
        VIP_PRES = "VIP_PRES",
        /// A vice head of state.
        VIP_VICE = "VIP_VICE",
        /// An oil field.
        OIL = "OIL",
        /// A gas field or gas plant.
        GAS = "GAS",
        /// A refinery.
        REFINERY = "REFINERY",
        /// A chemical plant.
        CHEMICAL = "CHEMICAL",
        /// A nuclear plant.
        NUCLEAR = "NUCLEAR",
        /// A technical activity that affects air traffic.
        TECHNICAL = "TECHNICAL",
        /// Air traffic services.
        ATS = "ATS",
        /// A special procedure, e.g. an overhead approach.
        PROCEDURE = "PROCEDURE",
    }
}

code_list! {
    /// The ICAO classification of an airspace
    /// (CodeAirspaceClassificationType).
    CodeAirspaceClassificationType {
        /// Class A.
        A = "A",
        /// Class B.
        B = "B",
        /// Class C.
        C = "C",
        /// Class D.
        D = "D",
        /// Class E.
        E = "E",
        /// Class F.
        F = "F",
        /// Class G.
        G = "G",
    }
}

code_list! {
    /// How an airspace volume takes part in the airspace
    /// (CodeAirspaceAggregationType).
    CodeAirspaceAggregationType {
        /// The volume the operations start from.
        BASE = "BASE",
        /// The volume is added.
        UNION = "UNION",
        /// The volume is intersected.
        INTERS = "INTERS",
        /// The volume is subtracted.
        SUBTR = "SUBTR",
    }
}

code_list! {
    /// What an airspace volume takes from another airspace
    /// (CodeAirspaceDependencyType).
    CodeAirspaceDependencyType {
        /// The horizontal projection and the vertical limits.
        FULL_GEOMETRY = "FULL_GEOMETRY",
        /// The horizontal projection only.
        HORZ_PROJECTION = "HORZ_PROJECTION",
    }
}

code_list! {
    /// The reference of a vertical limit (CodeVerticalReferenceType).
    CodeVerticalReferenceType {
        /// The surface of the earth.
        SFC = "SFC",
        /// The mean sea level.
        MSL = "MSL",
        /// The WGS-84 ellipsoid.
        W84 = "W84",
        /// The standard pressure of 1013.25 hPa.
        STD = "STD",
    }
}

code_list! {
    /// The purpose of a note (CodeNotePurposeType).
    CodeNotePurposeType {
        /// A remark.
        REMARK = "REMARK",
        /// A description.
        DESCRIPTION = "DESCRIPTION",
    }
}

/////////////////////////////////////////////////////////////////////////////
// Units of Measurement
/////////////////////////////////////////////////////////////////////////////

code_list! {
    /// The unit of a distance or an elevation (UomDistanceType).
    UomDistanceType {
        /// Nautical miles.
        NM = "NM",
        /// Nautical miles in UCUM, as GML writes the radius of an arc.
        NMI_I = "[nmi_i]",
        /// Kilometres.
        KM = "KM",
        /// Metres.
        M = "M",
        /// Feet.
        FT = "FT",
    }
}

code_list! {
    /// The unit of a vertical distance (UomDistanceVerticalType).
    UomDistanceVerticalType {
        /// Feet.
        FT = "FT",
        /// Metres.
        M = "M",
        /// Flight levels.
        FL = "FL",
        /// Standard flight levels in tens of metres.
        SM = "SM",
    }
}

code_list! {
    /// The unit of a frequency (UomFrequencyType).
    UomFrequencyType {
        /// Kilohertz.
        KHZ = "KHZ",
        /// Megahertz.
        MHZ = "MHZ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_code_of_the_list() {
        assert_eq!(
            CodeDesignatedPointType::from_code("ICAO"),
            CodeDesignatedPointType::ICAO
        );
    }

    #[test]
    fn keeps_code_outside_the_list() {
        let kind = CodeDesignatedPointType::from_code("OTHER:ADHP");
        assert_eq!(
            kind,
            CodeDesignatedPointType::Other("OTHER:ADHP".to_string())
        );
        assert_eq!(kind.as_str(), "OTHER:ADHP");
    }
}
