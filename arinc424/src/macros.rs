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

/// Validate that all bytes are ASCII digits and return Result
macro_rules! parse_numeric {
    // check that all bytes are digits
    ($n:tt, $t:ty, $b:expr, $calc:expr) => {{
        if $b.iter().all(|&byte| byte.is_ascii_digit()) {
            Ok($calc)
        } else {
            Err($crate::Error::NotANumber { bytes: $b.to_vec() })
        }
    }};

    (1, $t:ty, $b:expr) => {
        parse_numeric!(1, $t, $b, ($b[0] & 0x0F) as $t)
    };

    (2, $t:ty, $b:expr) => {
        parse_numeric!(2, $t, $b, ($b[0] & 0x0F) as $t * 10 + ($b[1] & 0x0F) as $t)
    };

    (3, $t:ty, $b:expr) => {
        parse_numeric!(
            3,
            $t,
            $b,
            ($b[0] & 0x0F) as $t * 100 + ($b[1] & 0x0F) as $t * 10 + ($b[2] & 0x0F) as $t
        )
    };

    (4, $t:ty, $b:expr) => {
        parse_numeric!(
            4,
            $t,
            $b,
            ($b[0] & 0x0F) as $t * 1000
                + ($b[1] & 0x0F) as $t * 100
                + ($b[2] & 0x0F) as $t * 10
                + ($b[3] & 0x0F) as $t
        )
    };

    (5, $t:ty, $b:expr) => {
        parse_numeric!(
            5,
            $t,
            $b,
            ($b[0] & 0x0F) as $t * 10000
                + ($b[1] & 0x0F) as $t * 1000
                + ($b[2] & 0x0F) as $t * 100
                + ($b[3] & 0x0F) as $t * 10
                + ($b[4] & 0x0F) as $t
        )
    };
}
