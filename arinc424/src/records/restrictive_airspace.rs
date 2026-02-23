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

use crate::fields::*;
use crate::Alphanumeric;
use crate::Record;

#[derive(Record)]
pub struct RestrictiveAirspace<'a> {
    pub record_type: RecordType,
    pub cust_area: CustArea<'a>,
    pub sec_code: SecCode,
    pub sub_code: SubCode<'a>,
    pub icao_code: IcaoCode<'a>,
    pub restrictive_type: RestrictiveType,
    pub restrictive_designation: Alphanumeric<'a, 10>,
    pub multi_cd: Alphanumeric<'a, 1>,
    pub seq_nr: SequenceNumber<'a, 4>,
    pub cont_nr: ContNr<'a>,
    pub level: Option<Level>,
    pub time_cd: Option<TimeCode>,
    pub notam: NOTAM,
    pub uav: UAV,
    #[arinc424(skip(1))]
    pub bdry_via: BoundaryVia,
    pub latitude: Option<Latitude<'a>>,
    pub longitude: Option<Longitude<'a>>,
    pub arc_origin_latitude: Option<Latitude<'a>>,
    pub arc_origin_longitude: Option<Longitude<'a>>,
    pub arc_dist: Option<ArcDistance<'a>>,
    pub arc_brg: Option<ArcBearing<'a>>,
    #[arinc424(skip(3))]
    pub lower_limit: Option<LowerUpperLimit>,
    pub lower_unit_indicator: Option<UnitIndicator>,
    pub upper_limit: Option<LowerUpperLimit>,
    pub upper_unit_indicator: Option<UnitIndicator>,
    pub arsp_name: Option<Alphanumeric<'a, 30>>,
    pub frn: FileRecordNumber<'a>,
    pub cycle: Cycle<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const AIRSPACE: &[u8] = b"SUSAURK1MEEL A     A00101L    H N46200000W124215300                              11000M17999MEEL A MOA                     715681713";

    #[test]
    fn restrictive_airspace_record() {
        let arsp = RestrictiveAirspace::try_from(AIRSPACE).expect("airspace should parse");

        assert_eq!(arsp.record_type, RecordType::Standard);
        assert_eq!(arsp.cust_area, CustArea::USA);
        assert_eq!(arsp.sec_code, SecCode::Airspace);
        assert_eq!(
            arsp.sub_code.kind(&arsp.sec_code),
            Ok(SubCodeKind::RestrictiveAirspace)
        );

        assert_eq!(arsp.icao_code.as_str(), "K1");
        assert_eq!(arsp.restrictive_type, RestrictiveType::MOA);
        assert_eq!(arsp.restrictive_designation.as_str(), "EEL A");
        assert_eq!(arsp.multi_cd.as_str(), "A");
        assert_eq!(arsp.seq_nr.as_u16(), Ok(10));
        assert_eq!(arsp.cont_nr.as_str(), "1");
        assert_eq!(arsp.level, Some(Level::LowLevelAirwaysAltitudes));
        assert_eq!(arsp.time_cd, None);
        assert_eq!(bool::from(arsp.notam), false);
        assert_eq!(bool::from(arsp.uav), false);
        assert_eq!(arsp.bdry_via.path, BoundaryPath::RhumbLine);
        assert!(!arsp.bdry_via.return_to_origin);
        assert_eq!(
            arsp.latitude.map(|lat| lat.as_decimal()),
            Some(Ok(46.333333333333336))
        );
        assert_eq!(
            arsp.longitude.map(|lon| lon.as_decimal()),
            Some(Ok(-124.36472222222221))
        );
        assert_eq!(arsp.arc_origin_latitude, None);
        assert_eq!(arsp.arc_origin_longitude, None);
        assert!(arsp.arc_dist.is_none());
        assert!(arsp.arc_brg.is_none());
        assert_eq!(arsp.lower_limit, Some(LowerUpperLimit::Altitude(11000)));
        assert_eq!(
            arsp.lower_unit_indicator,
            Some(UnitIndicator::MeanSeaLevel)
        );
        assert_eq!(arsp.upper_limit, Some(LowerUpperLimit::Altitude(17999)));
        assert_eq!(
            arsp.upper_unit_indicator,
            Some(UnitIndicator::MeanSeaLevel)
        );
        assert_eq!(
            arsp.arsp_name.map(|name| name.as_str()),
            Some("EEL A MOA")
        );
        assert_eq!(arsp.frn.as_u32(), Ok(71568));
        assert_eq!(arsp.cycle.year(), Ok(17));
        assert_eq!(arsp.cycle.cycle(), Ok(13));
    }
}
