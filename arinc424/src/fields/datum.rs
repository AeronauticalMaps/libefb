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

use crate::{Error, FixedField};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Datum {
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

impl<'a> FixedField<'a> for Datum {
    const LENGTH: usize = 3;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match &bytes[0..3] {
            b"ADI" => Ok(Self::ADI),
            b"AFG" => Ok(Self::AFG),
            b"AIN" => Ok(Self::AIN),
            b"AMA" => Ok(Self::AMA),
            b"ANO" => Ok(Self::ANO),
            b"AIA" => Ok(Self::AIA),
            b"ARF" => Ok(Self::ARF),
            b"ARS" => Ok(Self::ARS),
            b"ASC" => Ok(Self::ASC),
            b"ATF" => Ok(Self::ATF),
            b"SHB" => Ok(Self::SHB),
            b"TRN" => Ok(Self::TRN),
            b"ASQ" => Ok(Self::ASQ),
            b"AUA" => Ok(Self::AUA),
            b"AUG" => Ok(Self::AUG),
            b"PHA" => Ok(Self::PHA),
            b"IBE" => Ok(Self::IBE),
            b"BER" => Ok(Self::BER),
            b"BID" => Ok(Self::BID),
            b"BOO" => Ok(Self::BOO),
            b"BUR" => Ok(Self::BUR),
            b"CAZ" => Ok(Self::CAZ),
            b"CAI" => Ok(Self::CAI),
            b"CAO" => Ok(Self::CAO),
            b"CAP" => Ok(Self::CAP),
            b"CAC" => Ok(Self::CAC),
            b"CGE" => Ok(Self::CGE),
            b"CHI" => Ok(Self::CHI),
            b"CHU" => Ok(Self::CHU),
            b"EST" => Ok(Self::EST),
            b"COA" => Ok(Self::COA),
            b"DAL" => Ok(Self::DAL),
            b"DAN" => Ok(Self::DAN),
            b"DID" => Ok(Self::DID),
            b"BAT" => Ok(Self::BAT),
            b"GIZ" => Ok(Self::GIZ),
            b"EAS" => Ok(Self::EAS),
            b"EUR" => Ok(Self::EUR),
            b"FOT" => Ok(Self::FOT),
            b"GAA" => Ok(Self::GAA),
            b"GAN" => Ok(Self::GAN),
            b"GEO" => Ok(Self::GEO),
            b"GRA" => Ok(Self::GRA),
            b"GRX" => Ok(Self::GRX),
            b"GSE" => Ok(Self::GSE),
            b"DOB" => Ok(Self::DOB),
            b"HEN" => Ok(Self::HEN),
            b"HER" => Ok(Self::HER),
            b"HJO" => Ok(Self::HJO),
            b"HKD" => Ok(Self::HKD),
            b"HTN" => Ok(Self::HTN),
            b"IND" => Ok(Self::IND),
            b"INF" => Ok(Self::INF),
            b"ING" => Ok(Self::ING),
            b"INH" => Ok(Self::INH),
            b"IDN" => Ok(Self::IDN),
            b"IRL" => Ok(Self::IRL),
            b"ISG" => Ok(Self::ISG),
            b"IST" => Ok(Self::IST),
            b"JOH" => Ok(Self::JOH),
            b"KAN" => Ok(Self::KAN),
            b"KEG" => Ok(Self::KEG),
            b"KEA" => Ok(Self::KEA),
            b"KUS" => Ok(Self::KUS),
            b"LCF" => Ok(Self::LCF),
            b"LEH" => Ok(Self::LEH),
            b"LIB" => Ok(Self::LIB),
            b"LUZ" => Ok(Self::LUZ),
            b"MPO" => Ok(Self::MPO),
            b"MIK" => Ok(Self::MIK),
            b"MCN" => Ok(Self::MCN),
            b"MAS" => Ok(Self::MAS),
            b"MER" => Ok(Self::MER),
            b"MID" => Ok(Self::MID),
            b"MIN" => Ok(Self::MIN),
            b"MOL" => Ok(Self::MOL),
            b"ASM" => Ok(Self::ASM),
            b"NAH" => Ok(Self::NAH),
            b"NAN" => Ok(Self::NAN),
            b"NAP" => Ok(Self::NAP),
            b"NAS" => Ok(Self::NAS),
            b"NAR" => Ok(Self::NAR),
            b"NSD" => Ok(Self::NSD),
            b"FLO" => Ok(Self::FLO),
            b"OEG" => Ok(Self::OEG),
            b"OHA" => Ok(Self::OHA),
            b"FAH" => Ok(Self::FAH),
            b"OGB" => Ok(Self::OGB),
            b"PAM" => Ok(Self::PAM),
            b"PLN" => Ok(Self::PLN),
            b"PIT" => Ok(Self::PIT),
            b"PTB" => Ok(Self::PTB),
            b"PTN" => Ok(Self::PTN),
            b"POS" => Ok(Self::POS),
            b"PDM" => Ok(Self::PDM),
            b"PRP" => Ok(Self::PRP),
            b"HIT" => Ok(Self::HIT),
            b"PUR" => Ok(Self::PUR),
            b"PUK" => Ok(Self::PUK),
            b"QAT" => Ok(Self::QAT),
            b"QUO" => Ok(Self::QUO),
            b"REU" => Ok(Self::REU),
            b"MOD" => Ok(Self::MOD),
            b"RTS" => Ok(Self::RTS),
            b"SPK" => Ok(Self::SPK),
            b"SAE" => Ok(Self::SAE),
            b"SAO" => Ok(Self::SAO),
            b"SAP" => Ok(Self::SAP),
            b"SCK" => Ok(Self::SCK),
            b"SGM" => Ok(Self::SGM),
            b"SRL" => Ok(Self::SRL),
            b"CCD" => Ok(Self::CCD),
            b"SAN" => Ok(Self::SAN),
            b"SOA" => Ok(Self::SOA),
            b"STO" => Ok(Self::STO),
            b"SYO" => Ok(Self::SYO),
            b"TAN" => Ok(Self::TAN),
            b"TIL" => Ok(Self::TIL),
            b"TOY" => Ok(Self::TOY),
            b"TRI" => Ok(Self::TRI),
            b"TDC" => Ok(Self::TDC),
            b"U  " => Ok(Self::Unknown),
            b"MVS" => Ok(Self::MVS),
            b"VOI" => Ok(Self::VOI),
            b"VOR" => Ok(Self::VOR),
            b"WAK" => Ok(Self::WAK),
            b"ENW" => Ok(Self::ENW),
            b"WGA" => Ok(Self::WGA),
            b"WGB" => Ok(Self::WGB),
            b"WGC" => Ok(Self::WGC),
            b"WGE" => Ok(Self::WGE),
            b"YAC" => Ok(Self::YAC),
            b"ZAN" => Ok(Self::ZAN),
            _ => Err(Error::InvalidVariant {
                field: "Datum",
                bytes: Vec::from(bytes),
                expected: "datum according to ARINC 424-17 attachment 2",
            }),
        }
    }
}
