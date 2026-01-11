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

use crate::{Alphanumeric, Error};

pub type RwyGrad<'a> = Alphanumeric<'a, 6>;

impl<'a> RwyGrad<'a> {
    /// Returns the gradient as decimal number.
    ///
    /// # Errors
    ///
    /// Returns an error if the field can not be parsed as number.
    pub fn as_decimal(&self) -> Result<f32, Error> {
        let slope = parse_numeric!(5, u32, self.0[1..])? as f32 / 1000.0;

        match self.first() {
            b'+' => Ok(slope),
            b'-' => Ok(-slope),
            byte => Err(Error::InvalidCharacter {
                field: "Runway Gradient",
                byte,
                expected: "+ or -",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedField;

    use super::*;

    #[test]
    fn parse_upwward_gradient() {
        assert_eq!(
            RwyGrad::from_bytes(b"+10000").and_then(|v| v.as_decimal()),
            Ok(10.0)
        );
    }

    #[test]
    fn parse_downwward_gradient() {
        assert_eq!(
            RwyGrad::from_bytes(b"-00450").and_then(|v| v.as_decimal()),
            Ok(-0.45)
        );
    }
}
