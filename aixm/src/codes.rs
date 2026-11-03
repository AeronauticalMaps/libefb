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

//! The AIXM code lists and units of measurement that the features read.
//!
//! Each list names the values that are read today. A value outside the
//! list, such as an `OTHER:` extension, is kept as `Other`.

use std::fmt;

use crate::Error;

/// A value of an AIXM code list.
pub(crate) trait Code {
    fn from_code(code: &str) -> Self;
}

/// Reads a code from its text, for [`crate::stream::Stream::read_text`].
pub(crate) fn code<T: Code>(text: &str) -> Result<T, Error> {
    Ok(T::from_code(text))
}

macro_rules! code_list {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $code:literal,)*
        }
    ) => {
        $(#[$meta])*
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Hash, Debug)]
        pub enum $name {
            $($(#[$variant_meta])* $variant,)*
            /// A value outside the list, e.g. an `OTHER:` extension.
            Other(String),
        }

        impl $name {
            /// Returns the code as AIXM writes it.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $code,)*
                    Self::Other(code) => code,
                }
            }
        }

        impl Code for $name {
            fn from_code(code: &str) -> Self {
                match code {
                    $($code => Self::$variant,)*
                    other => Self::Other(other.to_string()),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

/////////////////////////////////////////////////////////////////////////////
// Code Lists
/////////////////////////////////////////////////////////////////////////////

code_list! {
    /// The type of a designated point (CodeDesignatedPointType).
    CodeDesignatedPointType {
        /// A five-letter name-code designator assigned by ICAO.
        ICAO = "ICAO",
        /// A point designed for a procedure.
        DESIGNED = "DESIGNED",
        /// A point in a terminal area.
        TERMINAL = "TERMINAL",
    }
}

code_list! {
    /// The purpose of a note (CodeNotePurposeType).
    CodeNotePurposeType {
        /// A remark.
        REMARK = "REMARK",
        /// A description.
        DESCRIPTION = "DESCRIPTION",
    }
}

/////////////////////////////////////////////////////////////////////////////
// Units of Measurement
/////////////////////////////////////////////////////////////////////////////

code_list! {
    /// The unit of a distance or an elevation (UomDistanceType).
    UomDistanceType {
        /// Nautical miles.
        NM = "NM",
        /// Kilometres.
        KM = "KM",
        /// Metres.
        M = "M",
        /// Feet.
        FT = "FT",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_code_of_the_list() {
        assert_eq!(
            CodeDesignatedPointType::from_code("ICAO"),
            CodeDesignatedPointType::ICAO
        );
    }

    #[test]
    fn keeps_code_outside_the_list() {
        let kind = CodeDesignatedPointType::from_code("OTHER:ADHP");
        assert_eq!(
            kind,
            CodeDesignatedPointType::Other("OTHER:ADHP".to_string())
        );
        assert_eq!(kind.as_str(), "OTHER:ADHP");
    }
}
