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
use crate::Alphanumeric;
use crate::Record;

#[derive(Record)]
pub struct ControlledAirspace<'a> {
    pub record_type: RecordType,
    pub cust_area: CustArea<'a>,
    pub sec_code: SecCode,
    pub sub_code: SubCode<'a>,
    pub icao_code: IcaoCode<'a>,
    pub arsp_type: ArspType,
    pub arsp_cntr: Alphanumeric<'a, 5>,
    #[arinc424(field = 17)]
    pub arsp_class: Option<Alphanumeric<'a, 1>>,
    #[arinc424(skip(2))]
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
    pub rnp: Option<RequiredNavigationPerformance<'a>>,
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

    const AIRSPACE: &'static [u8] = b"SUSAUCK6TKJFK PAB  A00100     R N40394857W074144423N40413000W07409590000402450   GND  A07000MNEW YORK AREA A               676061703";

    #[test]
    fn airport_record() {
        let arsp = ControlledAirspace::try_from(AIRSPACE).expect("airspace should parse");

        assert_eq!(arsp.record_type, RecordType::Standard);
        assert_eq!(arsp.cust_area, CustArea::USA);
        assert_eq!(arsp.sec_code, SecCode::Airspace);
        assert_eq!(
            arsp.sub_code.kind(&arsp.sec_code),
            Ok(SubCodeKind::ControlledAirspace)
        );

        assert_eq!(arsp.icao_code.as_str(), "K6");
        assert_eq!(arsp.arsp_type, ArspType::ClassB);
        assert_eq!(arsp.arsp_cntr.as_str(), "KJFK");
        assert_eq!(arsp.arsp_class.map(|class| class.as_str()), Some("B"));
        assert_eq!(arsp.multi_cd.as_str(), "A");
        assert_eq!(arsp.seq_nr.as_u16(), Ok(10));
        assert_eq!(arsp.cont_nr.as_str(), "0");
        assert_eq!(arsp.level, None);
        assert_eq!(arsp.time_cd, None);
        assert_eq!(bool::from(arsp.notam), false);
        assert_eq!(bool::from(arsp.uav), false);
        assert_eq!(arsp.bdry_via.path, BoundaryPath::ClockwiseArc);
        assert_eq!(
            arsp.latitude.map(|lat| lat.as_decimal()),
            Some(Ok(40.663491666666665))
        );
        assert_eq!(
            arsp.longitude.map(|long| long.as_decimal()),
            Some(Ok(-74.24561944444444))
        );
        assert_eq!(
            arsp.arc_origin_latitude.map(|lat| lat.as_decimal()),
            Some(Ok(40.69166666666666))
        );
        assert_eq!(
            arsp.arc_origin_longitude.map(|lon| lon.as_decimal()),
            Some(Ok(-74.16638888888889))
        );
        assert_eq!(arsp.arc_dist.map(|d| d.dist()), Some(Ok(4.0)));
        assert_eq!(arsp.arc_brg.map(|b| b.deg()), Some(Ok(245.0)));
        assert_eq!(arsp.rnp, None);
        assert_eq!(arsp.lower_limit, Some(LowerUpperLimit::Ground));
        assert_eq!(
            arsp.lower_unit_indicator,
            Some(UnitIndicator::AboveGroundLevel)
        );
        assert_eq!(arsp.upper_limit, Some(LowerUpperLimit::Altitude(7000)));
        assert_eq!(arsp.upper_unit_indicator, Some(UnitIndicator::MeanSeaLevel));
        assert_eq!(
            arsp.arsp_name.map(|name| name.as_str()),
            Some("NEW YORK AREA A")
        );
        assert_eq!(arsp.frn.as_u32(), Ok(67606));
        assert_eq!(arsp.cycle.year(), Ok(17));
        assert_eq!(arsp.cycle.cycle(), Ok(3));
    }
}
