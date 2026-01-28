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

mod airport;
mod controlled_airspace;
mod runway;
mod waypoint;

pub use airport::Airport;
pub use controlled_airspace::ControlledAirspace;
pub use runway::Runway;
pub use waypoint::Waypoint;

use crate::record::RECORD_LENGTH;

#[derive(Debug)]
pub enum RecordKind {
    Airport,
    ControlledAirspace,
    Waypoint,
    Runway,
}

pub struct Records<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Records<'a> {
    /// Creates a new record iterator from a byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::arinc424::records::{Airport, RecordKind, Records};
    /// # use crate::arinc424::Error;
    /// #
    /// # fn parse_records(data: &[u8]) -> Result<(), Error> {
    /// for (kind, bytes) in Records::new(data) {
    ///     match kind {
    ///         RecordKind::Airport => {
    ///             let arpt = Airport::try_from(bytes)?;
    ///             // now you can read the airport's fields or convert it
    ///             // to some other type
    ///         },
    ///         _ => {},
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Iterator for Records<'a> {
    type Item = (RecordKind, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos + RECORD_LENGTH <= self.data.len() {
            // Standard or tailored record type
            match self.data[self.pos] {
                b'S' | b'T' => {
                    let record = &self.data[self.pos..self.pos + RECORD_LENGTH];
                    self.pos += RECORD_LENGTH;

                    // just a convenience...
                    macro_rules! record {
                        ($t:expr) => {
                            return Some(($t, record))
                        };
                    }

                    let sec_code = record[4];
                    let sub_code = record[5];

                    match (sec_code, sub_code) {
                        (b'E', b'A') | (b'P', b'C') => {
                            record!(RecordKind::Waypoint);
                        }
                        (b'P', b' ') => match record[12] {
                            b'A' => record!(RecordKind::Airport),
                            b'G' => {
                                if record[21] == b'0' {
                                    // primary record
                                    record!(RecordKind::Runway)
                                }
                            }
                            _ => {}
                        },
                        (b'U', b'C') => record!(RecordKind::ControlledAirspace),
                        _ => {}
                    }
                }
                _ => {
                    // Skip byte (likely newline or invalid data)
                    self.pos += 1;
                }
            }
        }

        None
    }
}
