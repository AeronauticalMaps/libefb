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

use std::fmt;

use crate::Error;

/// A fixed-length field type.
///
/// This trait is implemented by all ARINC 424 fields. Each field is created
/// [`from_bytes`](FixedField::from_bytes) and stores a reference to those
/// bytes.
pub trait FixedField<'a>: Sized {
    /// The fixed length of this field in bytes.
    const LENGTH: usize;

    /// Parse this field from a byte slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the byte slice is too short or contains invalid data.
    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error>;
}

/////////////////////////////////////////////////////////////////////////////
// Alphanumeric Field
/////////////////////////////////////////////////////////////////////////////

/// A alpha/numeric field (left-justified, space-padded).
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Alphanumeric<'a, const N: usize>(pub(super) &'a [u8; N]);

impl<'a, const N: usize> Alphanumeric<'a, N> {
    /// Returns the field as a bytes.
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }

    /// Returns the field as a UTF-8 string with trailing spaces removed.
    ///
    /// Returns an empty string if the field contains invalid UTF-8.
    #[inline]
    pub fn as_str(&self) -> &'a str {
        std::str::from_utf8(self.0).unwrap_or("").trim_end()
    }

    /// Returns `true` if the field contains only spaces.
    #[inline]
    pub fn is_blank(&self) -> bool {
        self.0.iter().all(|&b| b == b' ')
    }

    /// Returns the first byte of the field.
    #[inline]
    pub fn first(&self) -> u8 {
        self.0[0]
    }
}

impl<'a, const N: usize> FixedField<'a> for Alphanumeric<'a, N> {
    const LENGTH: usize = N;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        // 1. check if there are enough bytes
        if bytes.len() < N {
            return Err(Error::InvalidFieldLength {
                expected: N,
                actual: bytes.len(),
            });
        }

        // 2. now we can cast them unsafe
        let arr = unsafe { &*(bytes.as_ptr() as *const [u8; N]) };
        Ok(Self(arr))
    }
}

impl<const N: usize> fmt::Debug for Alphanumeric<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}

impl<const N: usize> fmt::Display for Alphanumeric<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> From<Alphanumeric<'_, N>> for String {
    fn from(a: Alphanumeric<'_, N>) -> Self {
        a.as_str().to_owned()
    }
}

/////////////////////////////////////////////////////////////////////////////
// Numeric Field
/////////////////////////////////////////////////////////////////////////////

/// A numeric field (right-justified, zero or space-padded).
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Numeric<'a, const N: usize>(&'a [u8; N]);

macro_rules! numeric_impl {
    ($n:tt => $($method:ident : $t:ty),+) => {
        impl<'a> Numeric<'a, $n> {
            $(
                #[inline]
                pub fn $method(&self) -> Result<$t, Error> {
                    parse_numeric!($n, $t, self.0)
                }
            )+
        }
    };
}

// Implement different parser depending on the field's length. For example, a
// two byte long field can't be parsed to a unsigned 32 bit integer.
numeric_impl!(1 => as_u8: u8);
numeric_impl!(2 => as_u8: u8, as_u16: u16);
numeric_impl!(3 => as_u8: u8, as_u16: u16);
numeric_impl!(4 => as_u16: u16, as_u32: u32);
numeric_impl!(5 => as_u32: u32);

impl<'a, const N: usize> Numeric<'a, N> {
    /// Returns `true` if the field contains only spaces.
    #[inline]
    pub fn is_blank(&self) -> bool {
        self.0.iter().all(|&b| b == b' ')
    }
}

impl<'a, const N: usize> FixedField<'a> for Numeric<'a, N> {
    const LENGTH: usize = N;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        // 1. check if there are enough bytes
        if bytes.len() < N {
            return Err(Error::InvalidFieldLength {
                expected: N,
                actual: bytes.len(),
            });
        }

        // 2. now we can cast them unsafe
        let arr = unsafe { &*(bytes.as_ptr() as *const [u8; N]) };
        Ok(Self(arr))
    }
}

impl<const N: usize> fmt::Debug for Numeric<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8_lossy(self.0);
        write!(f, "{s}")
    }
}

/////////////////////////////////////////////////////////////////////////////
// Optional Field Support
/////////////////////////////////////////////////////////////////////////////

/// Optional field that may contain only spaces.
///
/// Allows fields to be `None` when they contain only spaces (i.e., data from an
/// older version).
impl<'a, T> FixedField<'a> for Option<T>
where
    T: FixedField<'a>,
{
    const LENGTH: usize = T::LENGTH;

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        // Check if all bytes in the field length are spaces
        if bytes
            .get(..T::LENGTH)
            .is_some_and(|b| b.iter().all(|&c| c == b' '))
        {
            Ok(None)
        } else {
            T::from_bytes(bytes).map(Some)
        }
    }
}
