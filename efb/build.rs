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

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let handbook = Path::new(&out_dir).join("handbook.rs");

    let handbook_sections = vec![
        ("ROUTE", "../handbook/src/Route.md"),
        ("ROUTE_PROMPT", "../handbook/src/RoutePrompt.md"),
    ];

    let mut content = String::new();

    for (name, file_path) in handbook_sections {
        // Tell Cargo to rerun if the file changes
        println!("cargo:rerun-if-changed={}", file_path);

        // Read the markdown file, defaulting to an empty string when the handbook
        // sources are not available (e.g. when building from a published crate).
        let section = fs::read_to_string(file_path).unwrap_or_default();

        // Generate a const string for this section
        content.push_str(&format!(
            "pub const {}: &str = r#\"{}\"#;\n\n",
            name, section
        ));
    }

    // Write the generated code
    fs::write(handbook, content).unwrap();
}
