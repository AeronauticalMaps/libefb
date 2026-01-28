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

#[derive(Record)]
pub struct Waypoint<'a> {
    pub record_type: RecordType,
    pub cust_area: CustArea<'a>,
    pub sec_code: SecCode,
    sub_code_6: Option<SubCode<'a>>,
    pub regn_code: RegnCode<'a>,
    /// The ICAO code can either be in column 11 or 20.
    icao_code_11: Option<IcaoCode<'a>>,
    sub_code_13: Option<SubCode<'a>>,
    pub fix_ident: FixIdent<'a>,
    /// The ICAO code can either be in column 11 or 20.
    #[arinc424(field = 20)]
    icao_code_20: Option<IcaoCode<'a>>,
    #[arinc424(field = 22)]
    pub cont_nr: ContNr<'a>,
    #[arinc424(skip(4))]
    pub waypoint_type: WaypointType<'a>,
    #[arinc424(skip(1))]
    pub waypoint_usage: WaypointUsage,
    #[arinc424(skip(1))]
    pub latitude: Latitude<'a>,
    pub longitude: Longitude<'a>,
    #[arinc424(skip(23))]
    pub mag_var: Option<MagVar>,
    #[arinc424(field = 85)]
    pub datum: Datum,
    #[arinc424(field = 96)]
    pub name_ind: Option<NameInd>,
    pub name_desc: NameDesc<'a>,
    pub frn: FileRecordNumber<'a>,
    pub cycle: Cycle<'a>,
}

impl<'a> Waypoint<'a> {
    /// Returns the subsection code of the waypoint.
    ///
    /// # Panics
    ///
    /// Panics if the code is neither in column 6 nor column 13.
    pub fn sub_code(&self) -> SubCode<'a> {
        self.sub_code_6
            .or(self.sub_code_13)
            .expect("waypoint should have a SUB CODE")
    }

    /// Returns the ICAO code of the waypoint.
    ///
    /// # Panics
    ///
    /// Panics if the code is neither in column 11 nor column 20.
    pub fn icao_code(&self) -> IcaoCode<'a> {
        self.icao_code_11
            .or(self.icao_code_20)
            .expect("waypoint should have an ICAO code")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PC_WAYPOINT: &'static [u8] = b"SEURPCEDDHED W1    ED0    V     N53341894E009404512                                 WGE           WHISKEY1                 122922407";

    #[test]
    fn terminal_waypoint_record() {
        let wp = Waypoint::try_from(PC_WAYPOINT).expect("waypoint should parse");

        assert_eq!(wp.record_type, RecordType::Standard);
        assert_eq!(wp.cust_area, CustArea::EUR);
        assert_eq!(wp.sec_code, SecCode::Airport);
        assert_eq!(
            wp.sub_code().kind(&wp.sec_code),
            Ok(SubCodeKind::TerminalWaypoint)
        );
        assert_eq!(wp.regn_code.as_str(), "EDDH");
        assert_eq!(wp.icao_code().as_str(), "ED");
        assert_eq!(wp.fix_ident.as_str(), "W1");
        assert_eq!(wp.cont_nr.as_str(), "0");
        assert_eq!(wp.waypoint_type.as_str(), "V");
        assert_eq!(wp.waypoint_usage, WaypointUsage::TerminalOnly);
        assert_eq!(wp.latitude.as_decimal(), Ok(53.57192777777778));
        assert_eq!(wp.longitude.as_decimal(), Ok(9.6792));
        assert_eq!(wp.mag_var, None);
        assert_eq!(wp.datum, Datum::WGE);
        assert_eq!(wp.name_ind, None);
        assert_eq!(wp.name_desc.as_str(), "WHISKEY1");
        assert_eq!(wp.frn.as_u32(), Ok(12292));
        assert_eq!(wp.cycle.year(), Ok(24));
        assert_eq!(wp.cycle.cycle(), Ok(7));
    }

    const EA_WAYPOINT: &'static [u8] = b"SUSAEAENRT   AAARG K 0    W   B N32413827W078030466                       W0093     NAR           AAARG                    270862407";

    #[test]
    fn enroute_waypoint_record() {
        let wp = Waypoint::try_from(EA_WAYPOINT).expect("waypoint should parse");

        assert_eq!(wp.record_type, RecordType::Standard);
        assert_eq!(wp.cust_area, CustArea::USA);
        assert_eq!(wp.sec_code, SecCode::Enroute);
        assert_eq!(wp.sub_code().kind(&wp.sec_code), Ok(SubCodeKind::Waypoint));
        assert_eq!(wp.regn_code.as_str(), "ENRT");
        assert_eq!(wp.icao_code().as_str(), "K");
        assert_eq!(wp.fix_ident.as_str(), "AAARG");
        assert_eq!(wp.cont_nr.as_str(), "0");
        assert_eq!(wp.waypoint_type.as_str(), "W");
        assert_eq!(wp.waypoint_usage, WaypointUsage::HiLoAltitude);
        assert_eq!(wp.latitude.as_decimal(), Ok(32.69396388888889));
        assert_eq!(wp.longitude.as_decimal(), Ok(-78.05129444444444));
        assert_eq!(wp.mag_var, Some(MagVar::West(0.93)));
        assert_eq!(wp.datum, Datum::NAR);
        assert_eq!(wp.name_ind, None);
        assert_eq!(wp.name_desc.as_str(), "AAARG");
        assert_eq!(wp.frn.as_u32(), Ok(27086));
        assert_eq!(wp.cycle.year(), Ok(24));
        assert_eq!(wp.cycle.cycle(), Ok(7));
    }
}
