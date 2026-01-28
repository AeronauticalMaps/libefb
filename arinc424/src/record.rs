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

use crate::{Error, FixedField};

/// The fixed length of an ARINC 424 record in bytes.
pub const RECORD_LENGTH: usize = 132;

/// An ARINC 424 record type.
pub trait Record<'a>: Sized {
    /// Parse this record type from a field iterator.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short or contains invalid data.
    fn parse(fields: Fields<'a>) -> Result<Self, Error>;

    /// Parse this record type from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if there are not 132 bytes or any error returned by
    /// [`parse`][Record::parse].
    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() == RECORD_LENGTH {
            Self::parse(Fields::new(bytes))
        } else {
            Err(Error::InvalidRecordLength {
                actual: bytes.len(),
            })
        }
    }
}

pub struct Fields<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Fields<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    /// Reads the next field, and advances the position by the field's length.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing the field fails.
    #[inline]
    pub fn next<F>(&mut self) -> Result<F, Error>
    where
        F: FixedField<'a>,
    {
        let field = F::from_bytes(&self.bytes[self.index..])?;
        self.index += F::LENGTH;
        Ok(field)
    }

    /// Skips `n` bytes, advancing the position without parsing.
    #[inline]
    pub fn skip(&mut self, n: usize) -> &mut Self {
        self.index += n;
        self
    }

    /// Jumps to the position and get the field.
    ///
    /// The next field will be the one following this field's position.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing the field fails.
    #[inline]
    pub fn get<F>(&mut self, position: usize) -> Result<F, Error>
    where
        F: FixedField<'a>,
    {
        self.index = position - 1;
        self.next()
    }
}
