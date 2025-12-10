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

use std::cmp::PartialEq;
use std::fmt;
use std::str;

mod arpt_heli_ident;
mod cont_nr;
mod coordinate;
mod cust_area;
mod cycle;
mod datum;
mod fix_ident;
mod frn;
mod iata;
mod icao_code;
mod mag_true_ind;
mod mag_var;
mod name_desc;
mod name_ind;
mod record_type;
mod regn_code;
mod runway_id;
mod rwy_brg;
mod rwy_grad;
mod sec_sub_code;
mod source;
mod waypoint_type;
mod waypoint_usage;

pub use arpt_heli_ident::ArptHeliIdent;
pub use cont_nr::ContNr;
pub use coordinate::{CardinalDirection, Latitude, Longitude};
pub use cust_area::CustArea;
pub use cycle::Cycle;
pub use datum::Datum;
pub use fix_ident::FixIdent;
pub use frn::FileRecordNumber;
pub use iata::Iata;
pub use icao_code::IcaoCode;
pub use mag_true_ind::MagTrueInd;
pub use mag_var::MagVar;
pub use name_desc::NameDesc;
pub use name_ind::NameInd;
pub use record_type::RecordType;
pub use regn_code::RegnCode;
pub use runway_id::RunwayId;
pub use rwy_brg::RwyBrg;
pub use rwy_grad::RwyGrad;
pub use sec_sub_code::{SecCode, SubCode};
pub use source::Source;
pub use waypoint_type::WaypointType;
pub use waypoint_usage::WaypointUsage;

/// Error context for ARINC 424 field parsing failures.
///
/// Provides detailed information about parsing errors including field position,
/// expected format, and the actual value encountered.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldError {
    /// The kind of error that occurred.
    pub kind: FieldErrorKind,
    /// The name of the field being parsed (e.g., "Latitude", "IcaoCode").
    pub field: &'static str,
    /// The starting position (0-indexed) of the field in the ARINC 424 record.
    pub position: usize,
    /// The expected length of the field.
    pub length: usize,
    /// The actual value that was encountered (if available).
    pub actual: Option<String>,
}

impl FieldError {
    /// Creates a new FieldError with the given context.
    pub fn new(kind: FieldErrorKind, field: &'static str, position: usize, length: usize) -> Self {
        Self {
            kind,
            field,
            position,
            length,
            actual: None,
        }
    }

    /// Adds the actual value that was encountered to the error.
    pub fn with_actual(mut self, actual: impl Into<String>) -> Self {
        self.actual = Some(actual.into());
        self
    }

    /// Creates an InvalidLength error for a field.
    pub fn invalid_length(field: &'static str, position: usize, length: usize) -> Self {
        Self::new(FieldErrorKind::InvalidLength, field, position, length)
    }

    /// Creates an InvalidValue error for a field.
    pub fn invalid_value(
        field: &'static str,
        position: usize,
        length: usize,
        message: &'static str,
    ) -> Self {
        Self::new(
            FieldErrorKind::InvalidValue(message),
            field,
            position,
            length,
        )
    }

    /// Creates an UnexpectedChar error for a field.
    pub fn unexpected_char(
        field: &'static str,
        position: usize,
        length: usize,
        message: &'static str,
    ) -> Self {
        Self::new(
            FieldErrorKind::UnexpectedChar(message),
            field,
            position,
            length,
        )
    }

    /// Creates a NotANumber error for a field.
    pub fn not_a_number(field: &'static str, position: usize, length: usize) -> Self {
        Self::new(FieldErrorKind::NotANumber, field, position, length)
    }

    /// Creates a NumberOutOfRange error for a field.
    pub fn number_out_of_range(field: &'static str, position: usize, length: usize) -> Self {
        Self::new(FieldErrorKind::NumberOutOfRange, field, position, length)
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error parsing {} at position {}..{}: {}",
            self.field,
            self.position,
            self.position + self.length,
            self.kind
        )?;
        if let Some(ref actual) = self.actual {
            write!(f, " (got {:?})", actual)?;
        }
        Ok(())
    }
}

impl std::error::Error for FieldError {}

/// The kind of error that occurred during field parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldErrorKind {
    /// The input string was too short to contain the expected field.
    InvalidLength,
    /// The field value did not match any expected pattern.
    InvalidValue(&'static str),
    /// The field contained an unexpected character.
    UnexpectedChar(&'static str),
    /// A numeric field contained non-numeric characters.
    NotANumber,
    /// A numeric field value was outside the allowed range.
    NumberOutOfRange,
}

impl fmt::Display for FieldErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "input too short for field"),
            Self::InvalidValue(msg) => write!(f, "invalid value: {}", msg),
            Self::UnexpectedChar(msg) => write!(f, "unexpected character: {}", msg),
            Self::NotANumber => write!(f, "expected numeric value"),
            Self::NumberOutOfRange => write!(f, "numeric value out of range"),
        }
    }
}

pub trait Field
where
    Self: Sized + str::FromStr,
{
}

#[derive(Debug)]
pub struct AlphaNumericField<const I: usize, const N: usize>([u8; N]);

impl<const I: usize, const N: usize> AlphaNumericField<I, N> {
    pub fn as_str<'a>(&'a self) -> &'a str {
        str::from_utf8(self.0.as_slice())
            .expect("field should decode to UTF-8")
            .trim_end()
    }

    pub fn into_inner(self) -> [u8; N] {
        self.0
    }
}

impl<const I: usize, const N: usize> Field for AlphaNumericField<I, N> {}

impl<const I: usize, const N: usize> str::FromStr for AlphaNumericField<I, N> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + N {
            return Err(FieldError::invalid_length("AlphaNumericField", I, N));
        }
        match <[u8; N]>::try_from(s[I..I + N].as_bytes()) {
            Ok(b) => Ok(Self(b)),
            Err(_) => Err(FieldError::invalid_length("AlphaNumericField", I, N)),
        }
    }
}

impl<const I: usize, const N: usize> PartialEq<&str> for AlphaNumericField<I, N> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl<const I: usize, const N: usize> fmt::Display for AlphaNumericField<I, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            str::from_utf8(self.0.as_slice())
                .map_err(|_| fmt::Error)?
                .trim_end()
        )
    }
}

#[derive(Debug)]
pub struct NumericField<const I: usize, const N: usize>(u32);

impl<const I: usize, const N: usize> Field for NumericField<I, N> {}

impl<const I: usize, const N: usize> str::FromStr for NumericField<I, N> {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < I + N {
            return Err(FieldError::invalid_length("NumericField", I, N));
        }
        let slice = &s[I..I + N];
        match slice.parse::<u32>() {
            Ok(b) => Ok(Self(b)),
            Err(_) => Err(FieldError::not_a_number("NumericField", I, N).with_actual(slice)),
        }
    }
}

impl<const I: usize, const N: usize> PartialEq<u32> for NumericField<I, N> {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl<const I: usize, const N: usize> fmt::Display for NumericField<I, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const I: usize, const N: usize> From<NumericField<I, N>> for u32 {
    fn from(value: NumericField<I, N>) -> Self {
        value.0
    }
}
