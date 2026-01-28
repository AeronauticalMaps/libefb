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

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum RwyBrg {
    MagneticNorth(f32),
    TrueNorth(u32),
}

impl FixedField<'_> for RwyBrg {
    const LENGTH: usize = 4;

    fn from_bytes(bytes: &'_ [u8]) -> Result<Self, Error> {
        match bytes[3] {
            b'T' => {
                let deg = parse_numeric!(3, u32, bytes[0..3])?;
                Ok(Self::TrueNorth(deg))
            }
            _ => {
                let deg = parse_numeric!(4, u32, bytes[0..4])? as f32 / 10.0;
                Ok(Self::MagneticNorth(deg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_true_north() {
        assert_eq!(RwyBrg::from_bytes(b"347T"), Ok(RwyBrg::TrueNorth(347)));
    }

    #[test]
    fn parse_magnetic_north() {
        assert_eq!(
            RwyBrg::from_bytes(b"2302"),
            Ok(RwyBrg::MagneticNorth(230.2))
        );
    }
}
