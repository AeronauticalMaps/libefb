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

use arinc424;

use crate::error::Error;
use crate::nd::*;

mod airspace;
mod fields;
mod records;

use airspace::AirspaceBuilder;

impl NavigationData {
    /// Creates navigation data from an ARINC 424 string.
    pub fn try_from_arinc424(data: &[u8]) -> Result<Self, Error> {
        let mut builder = NavigationData::builder();
        let mut airspace: Option<AirspaceBuilder> = None;

        for (kind, bytes) in arinc424::records::Records::new(data) {
            if let Err(e) = || -> Result<(), arinc424::Error> {
                match kind {
                    arinc424::records::RecordKind::Waypoint => {
                        let record = arinc424::records::Waypoint::try_from(bytes)?;
                        let wp = Waypoint::try_from(record)?;
                        builder.add_waypoint(wp);
                    }

                    arinc424::records::RecordKind::Airport => {
                        let record = arinc424::records::Airport::try_from(bytes)?;
                        let arpt = Airport::try_from(record)?;
                        builder.add_airport(arpt);
                    }

                    arinc424::records::RecordKind::Runway => {
                        let record = arinc424::records::Runway::try_from(bytes)?;
                        let ident = record.arpt_ident.to_string();
                        let rwy = Runway::try_from(record)?;
                        builder.add_runway(ident, rwy);
                    }

                    arinc424::records::RecordKind::ControlledAirspace => {
                        let record = arinc424::records::ControlledAirspace::try_from(bytes)?;
                        let return_to_origin = record.bdry_via.return_to_origin;
                        airspace.get_or_insert_default().add_record(record)?;

                        if return_to_origin {
                            let arsp = airspace
                                .take()
                                .expect("there should be an airspace at this point")
                                .build()?;

                            builder.add_airspace(arsp);
                        }
                    }
                }

                Ok(())
            }() {
                builder.add_error(Error::InvalidA424 {
                    record: bytes.to_vec(),
                    error: e.to_string(),
                });
            }
        }

        Ok(builder.with_source(data).build())
    }
}
