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

use crate::{Alphanumeric, Numeric};

mod arc_dist_brg;
mod arsp_type;
mod boundary_via;
mod coordinate;
mod cust_area;
mod cycle;
mod datum;
mod level;
mod lower_upper_limit;
mod mag_true_ind;
mod mag_var;
mod name_ind;
mod notam;
mod record_type;
mod runway_id;
mod rwy_brg;
mod rwy_grad;
mod sec_sub_code;
mod source;
mod time_cd;
mod uav;
mod unit_ind;
mod waypoint_usage;

pub use arc_dist_brg::{ArcBearing, ArcDistance};
pub use arsp_type::ArspType;
pub use boundary_via::{BoundaryPath, BoundaryVia};
pub use coordinate::{Latitude, Longitude};
pub use cust_area::CustArea;
pub use cycle::Cycle;
pub use datum::Datum;
pub use level::Level;
pub use lower_upper_limit::LowerUpperLimit;
pub use mag_true_ind::MagTrueInd;
pub use mag_var::MagVar;
pub use name_ind::NameInd;
pub use notam::NOTAM;
pub use record_type::RecordType;
pub use runway_id::RunwayId;
pub use rwy_brg::RwyBrg;
pub use rwy_grad::RwyGrad;
pub use sec_sub_code::{SecCode, SubCode, SubCodeKind};
pub use source::Source;
pub use time_cd::TimeCode;
pub use uav::UAV;
pub use unit_ind::UnitIndicator;
pub use waypoint_usage::WaypointUsage;

pub type ArptHeliIdent<'a> = Alphanumeric<'a, 4>;
pub type FileRecordNumber<'a> = Numeric<'a, 5>;
pub type FixIdent<'a> = Alphanumeric<'a, 5>;
pub type Iata<'a> = Alphanumeric<'a, 3>;
pub type IcaoCode<'a> = Alphanumeric<'a, 2>;
pub type NameDesc<'a> = Alphanumeric<'a, 25>;
pub type NameField<'a> = Alphanumeric<'a, 30>;
pub type RegnCode<'a> = Alphanumeric<'a, 4>;
pub type WaypointType<'a> = Alphanumeric<'a, 3>;

/// 5.12 Sequence Number
pub type SequenceNumber<'a, const N: usize> = Numeric<'a, N>;

/// 5.16 Continuation Record Number
pub type ContNr<'a> = Alphanumeric<'a, 1>;

/// 5.211 Required Navigation Performance (RNP)
pub type RequiredNavigationPerformance<'a> = Numeric<'a, 3>;

/// 5.215 Controlled Airspace Classification (ARSP CLASS)
pub type AirspaceClassification<'a> = Alphanumeric<'a, 1>;
