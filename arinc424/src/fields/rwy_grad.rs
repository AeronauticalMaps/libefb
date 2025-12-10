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

use std::str::FromStr;

use super::{Field, FieldError};

/// Runway Gradient field (ARINC 424 Spec §5.212).
///
/// Position I (6 characters):
/// - Character 1: Sign (+ or -)
/// - Characters 2-3: Integer degrees (00-99)
/// - Characters 4-6: Decimal degrees (000-999, representing thousandths)
#[derive(Debug, Default, PartialEq)]
pub struct RwyGrad<const I: usize> {
    pub degree: f32,
}

impl<const I: usize> RwyGrad<I> {
    /// The length of the runway gradient field.
    pub const LENGTH: usize = 6;
}

impl<const I: usize> Field for RwyGrad<I> {}

impl<const I: usize> FromStr for RwyGrad<I> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + Self::LENGTH {
            return Err(FieldError::invalid_length("RwyGrad", I, Self::LENGTH));
        }

        let sign_char = &s[I..I + 1];
        let sign = match sign_char {
            "+" => Ok(1.0),
            "-" => Ok(-1.0),
            c => Err(
                FieldError::unexpected_char("RwyGrad.sign", I, 1, "expected + or -").with_actual(c),
            ),
        }?;

        let int_slice = &s[I + 1..I + 3];
        let dec_slice = &s[I + 3..I + 6];

        let degree = {
            let degree = int_slice.parse::<f32>().map_err(|_| {
                FieldError::not_a_number("RwyGrad.integer", I + 1, 2).with_actual(int_slice)
            })?;
            let decimal = dec_slice.parse::<f32>().map_err(|_| {
                FieldError::not_a_number("RwyGrad.decimal", I + 3, 3).with_actual(dec_slice)
            })?;
            Ok(degree + decimal / 1000.0)
        }?;

        Ok(Self {
            degree: degree * sign,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_upwward_gradient() {
        assert_eq!("+10000".parse::<RwyGrad<0>>(), Ok(RwyGrad { degree: 10.0 }));
    }

    #[test]
    fn parse_downwward_gradient() {
        assert_eq!(
            "-00450".parse::<RwyGrad<0>>(),
            Ok(RwyGrad { degree: -0.45 })
        );
    }
}
