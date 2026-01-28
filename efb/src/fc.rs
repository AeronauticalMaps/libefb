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

//! Flight Computer.

/// Converts an angle from degree minutes and seconds to decimal.
pub fn dms_to_decimal(degree: u8, minutes: u8, seconds: u8) -> f64 {
    degree as f64 + minutes as f64 / 60.0 + seconds as f64 / 3600.0
}
