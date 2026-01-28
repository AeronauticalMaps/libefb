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

use crate::fields::*;
use crate::Record;

// TODO: add missing fields
#[derive(Record)]
pub struct Airport<'a> {
    pub record_type: RecordType,
    pub cust_area: CustArea<'a>,
    pub sec_code: SecCode,
    #[arinc424(skip(1))]
    pub arpt_ident: ArptHeliIdent<'a>,
    pub icao_code: IcaoCode<'a>,
    pub sub_code: SubCode<'a>,
    pub iata: Iata<'a>,
    #[arinc424(skip(5))]
    pub cont_nr: ContNr<'a>,
    #[arinc424(skip(10))]
    pub latitude: Latitude<'a>,
    pub longitude: Longitude<'a>,
    pub mag_var: Option<MagVar>,
    #[arinc424(field = 86)]
    pub mag_true_ind: MagTrueInd,
    pub datum: Datum,
    #[arinc424(field = 94)]
    pub airport_name: NameField<'a>,
    #[arinc424(field = 124)]
    pub frn: FileRecordNumber<'a>,
    pub cycle: Cycle<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const AIRPORT: &'static [u8] = b"SUSAP KJFKK6AJFK     0     145YHN40382374W073464329W013000013         1800018000C    MNAR    JOHN F KENNEDY INTL           300671912";

    #[test]
    fn airport_record() {
        let arpt = Airport::try_from(AIRPORT).expect("airport should parse");

        assert_eq!(arpt.record_type, RecordType::Standard);
        assert_eq!(arpt.cust_area, CustArea::USA);
        assert_eq!(arpt.sec_code, SecCode::Airport);
        assert_eq!(arpt.arpt_ident.as_str(), "KJFK");
        assert_eq!(arpt.icao_code.as_str(), "K6");
        assert_eq!(
            arpt.sub_code.kind(&arpt.sec_code),
            Ok(SubCodeKind::ReferencePoint)
        );
        assert_eq!(arpt.iata.as_str(), "JFK");
        assert_eq!(arpt.cont_nr.as_str(), "0");
        assert_eq!(arpt.latitude.as_decimal(), Ok(40.63992777777778));
        assert_eq!(arpt.longitude.as_decimal(), Ok(-73.77869166666666));
        assert_eq!(arpt.mag_var, Some(MagVar::West(1.3)));
        assert_eq!(arpt.mag_true_ind, MagTrueInd::Magnetic);
        assert_eq!(arpt.datum, Datum::NAR);
        assert_eq!(arpt.airport_name.as_str(), "JOHN F KENNEDY INTL");
        assert_eq!(arpt.frn.as_u32(), Ok(30067));
        assert_eq!(arpt.cycle.year(), Ok(19));
        assert_eq!(arpt.cycle.cycle(), Ok(12));
    }
}
