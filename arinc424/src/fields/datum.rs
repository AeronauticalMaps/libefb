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

use super::{Field, FieldError};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Datum<const I: usize> {
    /// Adindan
    ADI,
    /// Afgooye
    AFG,
    /// Ain El Abd 1970
    AIN,
    /// American Samoa 1962
    AMA,
    /// Anna 1 Astro 1965
    ANO,
    /// Antigua Island Astro 1943
    AIA,
    /// Arc 1950
    ARF,
    /// Arc 1960
    ARS,
    /// Ascension Island 1958
    ASC,
    /// Astro Beacon E 1945
    ATF,
    /// Astro DOS 71/4
    SHB,
    /// Astro Tern Island (Frig) 1961
    TRN,
    /// Astronomical Station 1952
    ASQ,
    /// Australian Geodetic 1966
    AUA,
    /// Australian Geodetic 1984
    AUG,
    /// Ayabelle Lighthouse
    PHA,
    /// Bellevue (IGN)
    IBE,
    /// Bermuda 1957
    BER,
    /// Bissau
    BID,
    /// Bogota Observatory
    BOO,
    /// Bukit Rimpah
    BUR,
    /// Camp Area Astro
    CAZ,
    /// Campo Inchauspe 1969
    CAI,
    /// Canton Astro 1966
    CAO,
    /// Cape
    CAP,
    /// Cape Canaveral
    CAC,
    /// Carthage
    CGE,
    /// Chatham Island Astro 1971
    CHI,
    /// Chua Astro
    CHU,
    /// Co-Ordinate System 1937 of Estonia
    EST,
    /// Corrego Alegre
    COA,
    /// Dabola
    DAL,
    /// Danish Geodetic Institute 1934 System
    DAN,
    /// Deception Island
    DID,
    /// Djakarta (Batavia)
    BAT,
    /// DOS 1968
    GIZ,
    /// Easter Island 1967
    EAS,
    /// European 1950
    EUR,
    /// Fort Thomas 1955
    FOT,
    /// Gan 1970
    GAA,
    /// Gandajika Base
    GAN,
    /// Geodetic Datum 1949
    GEO,
    /// Graciosa Base SW 1948
    GRA,
    /// Greek Geodetic Reference System 1987
    GRX,
    /// Gunuung Segara
    GSE,
    /// GUX 1 Astro
    DOB,
    /// Herat North
    HEN,
    /// Hermannskogel
    HER,
    /// Hjorsey 1955
    HJO,
    /// Hong Kong 1963
    HKD,
    /// Hu-Tzu-Shan
    HTN,
    /// Indian
    IND,
    /// Indian 1954
    INF,
    /// Indian 1960
    ING,
    /// Indian 1975
    INH,
    /// Indonesian 1974
    IDN,
    /// Ireland 1965
    IRL,
    /// ISTS 061 Astro 1968
    ISG,
    /// ISTS 073 Astro 1969
    IST,
    /// Johnston Island 1961
    JOH,
    /// Kandawala
    KAN,
    /// Kerguelen Island 1949
    KEG,
    /// Kertau 1948
    KEA,
    /// Kusaie Astro 1951
    KUS,
    /// L.C. 5 Astro 1961
    LCF,
    /// Leigon
    LEH,
    /// Liberia 1964
    LIB,
    /// Luzon
    LUZ,
    /// MPoraloko
    MPO,
    /// Mahe 1971
    MIK,
    /// Manchurian Principal System
    MCN,
    /// Massawa
    MAS,
    /// Merchich
    MER,
    /// Midway Astro 1961
    MID,
    /// Minna
    MIN,
    /// Montjong Lowe
    MOL,
    /// Montserrat Island Astro 1958
    ASM,
    /// Nahrwan
    NAH,
    /// Nanking 1960
    NAN,
    /// Naparima, BWI
    NAP,
    /// North American 1927
    NAS,
    /// North American 1983
    NAR,
    /// North Sahara 1959
    NSD,
    /// Observatorio Meteorologico 1939
    FLO,
    /// Old Egyptian 1907
    OEG,
    /// Old Hawaiian
    OHA,
    /// Oman
    FAH,
    /// Ordnance Survey of Great Britain 1936
    OGB,
    /// Palmer Astro
    PAM,
    /// Pico de las Nieves
    PLN,
    /// Pitcairn Astro 1967
    PIT,
    /// Point 58
    PTB,
    /// Point Noire 1948
    PTN,
    /// Porto Santo 1936
    POS,
    /// Potsdam
    PDM,
    /// Provisional South American 1956
    PRP,
    /// Provisional South Chilean 1963
    HIT,
    /// Puerto Rico
    PUR,
    /// Pulkovo 1942
    PUK,
    /// Qatar National
    QAT,
    /// Qornoq
    QUO,
    /// Reunion
    REU,
    /// Rome 1940
    MOD,
    /// RT90
    RTS,
    /// S42Pulkovo1942
    SPK,
    /// SantoDOS1965
    SAE,
    /// Sao Braz
    SAO,
    /// Sapper Hill 1943
    SAP,
    /// Schwarzeck
    SCK,
    /// Selvagem Grande 1938
    SGM,
    /// Sierra Leone 1960
    SRL,
    /// S-JTSK
    CCD,
    /// South American 1969
    SAN,
    /// South Asia
    SOA,
    /// Stockholm 1938
    STO,
    /// Sydney Observatory
    SYO,
    /// Tananarive Observatory 1925
    TAN,
    /// Timbalai 1948
    TIL,
    /// Tokyo
    TOY,
    /// Trinidad Trigonometrical Survey
    TRI,
    /// Tristan Astro 1968
    TDC,
    /// Unknown
    Unknown,
    /// Viti Levu 1916
    MVS,
    /// Voirol 1874
    VOI,
    /// Voirol 1960
    VOR,
    /// Wake Island Astro 1952
    WAK,
    /// Wake-Eniwetok 1960
    ENW,
    /// World Geodetic System 1960
    WGA,
    /// World Geodetic System 1966
    WGB,
    /// World Geodetic System 1972
    WGC,
    /// World Geodetic System 1984
    WGE,
    /// Yacare
    YAC,
    /// Zanderij
    ZAN,
}

impl<const I: usize> Datum<I> {
    /// The length of the datum field.
    pub const LENGTH: usize = 3;
}

impl<const I: usize> Field for Datum<I> {}

impl<const I: usize> FromStr for Datum<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("Datum", I, Self::LENGTH));
        }

        let datum = &s[I..I + 3];
        match datum {
            "ADI" => Ok(Self::ADI),
            "AFG" => Ok(Self::AFG),
            "AIN" => Ok(Self::AIN),
            "AMA" => Ok(Self::AMA),
            "ANO" => Ok(Self::ANO),
            "AIA" => Ok(Self::AIA),
            "ARF" => Ok(Self::ARF),
            "ARS" => Ok(Self::ARS),
            "ASC" => Ok(Self::ASC),
            "ATF" => Ok(Self::ATF),
            "SHB" => Ok(Self::SHB),
            "TRN" => Ok(Self::TRN),
            "ASQ" => Ok(Self::ASQ),
            "AUA" => Ok(Self::AUA),
            "AUG" => Ok(Self::AUG),
            "PHA" => Ok(Self::PHA),
            "IBE" => Ok(Self::IBE),
            "BER" => Ok(Self::BER),
            "BID" => Ok(Self::BID),
            "BOO" => Ok(Self::BOO),
            "BUR" => Ok(Self::BUR),
            "CAZ" => Ok(Self::CAZ),
            "CAI" => Ok(Self::CAI),
            "CAO" => Ok(Self::CAO),
            "CAP" => Ok(Self::CAP),
            "CAC" => Ok(Self::CAC),
            "CGE" => Ok(Self::CGE),
            "CHI" => Ok(Self::CHI),
            "CHU" => Ok(Self::CHU),
            "EST" => Ok(Self::EST),
            "COA" => Ok(Self::COA),
            "DAL" => Ok(Self::DAL),
            "DAN" => Ok(Self::DAN),
            "DID" => Ok(Self::DID),
            "BAT" => Ok(Self::BAT),
            "GIZ" => Ok(Self::GIZ),
            "EAS" => Ok(Self::EAS),
            "EUR" => Ok(Self::EUR),
            "FOT" => Ok(Self::FOT),
            "GAA" => Ok(Self::GAA),
            "GAN" => Ok(Self::GAN),
            "GEO" => Ok(Self::GEO),
            "GRA" => Ok(Self::GRA),
            "GRX" => Ok(Self::GRX),
            "GSE" => Ok(Self::GSE),
            "DOB" => Ok(Self::DOB),
            "HEN" => Ok(Self::HEN),
            "HER" => Ok(Self::HER),
            "HJO" => Ok(Self::HJO),
            "HKD" => Ok(Self::HKD),
            "HTN" => Ok(Self::HTN),
            "IND" => Ok(Self::IND),
            "INF" => Ok(Self::INF),
            "ING" => Ok(Self::ING),
            "INH" => Ok(Self::INH),
            "IDN" => Ok(Self::IDN),
            "IRL" => Ok(Self::IRL),
            "ISG" => Ok(Self::ISG),
            "IST" => Ok(Self::IST),
            "JOH" => Ok(Self::JOH),
            "KAN" => Ok(Self::KAN),
            "KEG" => Ok(Self::KEG),
            "KEA" => Ok(Self::KEA),
            "KUS" => Ok(Self::KUS),
            "LCF" => Ok(Self::LCF),
            "LEH" => Ok(Self::LEH),
            "LIB" => Ok(Self::LIB),
            "LUZ" => Ok(Self::LUZ),
            "MPO" => Ok(Self::MPO),
            "MIK" => Ok(Self::MIK),
            "MCN" => Ok(Self::MCN),
            "MAS" => Ok(Self::MAS),
            "MER" => Ok(Self::MER),
            "MID" => Ok(Self::MID),
            "MIN" => Ok(Self::MIN),
            "MOL" => Ok(Self::MOL),
            "ASM" => Ok(Self::ASM),
            "NAH" => Ok(Self::NAH),
            "NAN" => Ok(Self::NAN),
            "NAP" => Ok(Self::NAP),
            "NAS" => Ok(Self::NAS),
            "NAR" => Ok(Self::NAR),
            "NSD" => Ok(Self::NSD),
            "FLO" => Ok(Self::FLO),
            "OEG" => Ok(Self::OEG),
            "OHA" => Ok(Self::OHA),
            "FAH" => Ok(Self::FAH),
            "OGB" => Ok(Self::OGB),
            "PAM" => Ok(Self::PAM),
            "PLN" => Ok(Self::PLN),
            "PIT" => Ok(Self::PIT),
            "PTB" => Ok(Self::PTB),
            "PTN" => Ok(Self::PTN),
            "POS" => Ok(Self::POS),
            "PDM" => Ok(Self::PDM),
            "PRP" => Ok(Self::PRP),
            "HIT" => Ok(Self::HIT),
            "PUR" => Ok(Self::PUR),
            "PUK" => Ok(Self::PUK),
            "QAT" => Ok(Self::QAT),
            "QUO" => Ok(Self::QUO),
            "REU" => Ok(Self::REU),
            "MOD" => Ok(Self::MOD),
            "RTS" => Ok(Self::RTS),
            "SPK" => Ok(Self::SPK),
            "SAE" => Ok(Self::SAE),
            "SAO" => Ok(Self::SAO),
            "SAP" => Ok(Self::SAP),
            "SCK" => Ok(Self::SCK),
            "SGM" => Ok(Self::SGM),
            "SRL" => Ok(Self::SRL),
            "CCD" => Ok(Self::CCD),
            "SAN" => Ok(Self::SAN),
            "SOA" => Ok(Self::SOA),
            "STO" => Ok(Self::STO),
            "SYO" => Ok(Self::SYO),
            "TAN" => Ok(Self::TAN),
            "TIL" => Ok(Self::TIL),
            "TOY" => Ok(Self::TOY),
            "TRI" => Ok(Self::TRI),
            "TDC" => Ok(Self::TDC),
            "U  " => Ok(Self::Unknown),
            "MVS" => Ok(Self::MVS),
            "VOI" => Ok(Self::VOI),
            "VOR" => Ok(Self::VOR),
            "WAK" => Ok(Self::WAK),
            "ENW" => Ok(Self::ENW),
            "WGA" => Ok(Self::WGA),
            "WGB" => Ok(Self::WGB),
            "WGC" => Ok(Self::WGC),
            "WGE" => Ok(Self::WGE),
            "YAC" => Ok(Self::YAC),
            "ZAN" => Ok(Self::ZAN),
            c => Err(FieldError::unexpected_char(
                "Datum",
                I,
                Self::LENGTH,
                "expected a datum code per ARINC 424-17 attachment 2",
            )
            .with_actual(c)),
        }
    }
}
