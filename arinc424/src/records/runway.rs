// SPDX-License-Identifier: Apache-2.0
// Copyright 2025, 2026 Joe Pearson
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

use crate::fields::*;
use crate::{Numeric, Record};

// TODO add missing fields and handle different versions
#[derive(Record)]
pub struct Runway<'a> {
    pub record_type: RecordType,
    pub cust_area: CustArea<'a>,
    pub sec_code: SecCode,
    #[arinc424(skip(1))]
    pub arpt_ident: ArptHeliIdent<'a>,
    pub icao_code: IcaoCode<'a>,
    pub sub_code: SubCode<'a>,
    pub runway_id: RunwayId<'a>,
    #[arinc424(skip(3))]
    pub cont_nr: ContNr<'a>,
    /// Runway length in feet.
    pub runway_length: Numeric<'a, 5>,
    pub rwy_brg: RwyBrg,
    pub threshould_source: Option<Source>,
    pub threshould_latitude: Latitude<'a>,
    pub threshould_longitude: Longitude<'a>,
    pub rwy_grad: Option<RwyGrad<'a>>,
    #[arinc424(field = 124)]
    pub frn: FileRecordNumber<'a>,
    pub cycle: Cycle<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUNWAY: &'static [u8] = b"SUSAP KJFKK6GRW04L   0120790440 N40372318W073470505         -0028300012046057200IIHIQ1                                     305541709";

    #[test]
    fn runway_record() {
        let rwy = Runway::try_from(RUNWAY).expect("runway should parse");

        assert_eq!(rwy.record_type, RecordType::Standard);
        assert_eq!(rwy.cust_area, CustArea::USA);
        assert_eq!(rwy.sec_code, SecCode::Airport);
        assert_eq!(rwy.arpt_ident.as_str(), "KJFK");
        assert_eq!(rwy.icao_code.as_str(), "K6");
        assert_eq!(rwy.sub_code.kind(&rwy.sec_code), Ok(SubCodeKind::Runway));
        assert_eq!(rwy.runway_id.designator(), Ok("04L"));
        assert_eq!(rwy.cont_nr.as_str(), "0");
        assert_eq!(rwy.runway_length.as_u32(), Ok(12079u32));
        assert_eq!(rwy.rwy_brg, RwyBrg::MagneticNorth(44.0));
        assert_eq!(rwy.threshould_source, None);
        assert_eq!(rwy.frn.as_u32(), Ok(30554));
        assert_eq!(rwy.cycle.year(), Ok(17));
        assert_eq!(rwy.cycle.cycle(), Ok(9));
    }
}
