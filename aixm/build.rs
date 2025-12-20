// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Joe Pearson
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

use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use xsd_parser::config::{
    GeneratorFlags, InterpreterFlags, OptimizerFlags, RenderStep, Resolver, Schema,
};
use xsd_parser::models::meta::MetaTypes;
use xsd_parser::pipeline::renderer::NamespaceSerialization;
use xsd_parser::{exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render, Config};

const AIXM_FEATURES_URL: &str = "https://www.aixm.aero/schema/5.2/5.2.0/AIXM_Features.xsd";
const AIXM_MESSAGE_URL: &str =
    "https://www.aixm.aero/schema/5.2/5.2.0/message/AIXM_BasicMessage.xsd";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Configure the XSD parser with web resolver
    let mut config = Config::default();
    config.parser.schemas = vec![
        Schema::Url(AIXM_FEATURES_URL.parse().expect("Invalid AIXM features URL")),
        Schema::Url(AIXM_MESSAGE_URL.parse().expect("Invalid AIXM message URL")),
    ];
    config.parser.resolver = vec![Resolver::Web];
    config.interpreter.flags = InterpreterFlags::all();
    config.optimizer.flags = OptimizerFlags::all();
    config.generator.flags = GeneratorFlags::all();
    config.renderer.xsd_parser_types = "xsd_parser_types".into();

    // Add render steps for serialization support
    config = config
        .with_render_step(RenderStep::Types)
        .with_render_step(RenderStep::Defaults)
        .with_render_step(RenderStep::NamespaceConstants)
        .with_render_step(RenderStep::QuickXmlDeserialize {
            boxed_deserializer: false,
        })
        .with_render_step(RenderStep::QuickXmlSerialize {
            namespaces: NamespaceSerialization::Global,
            default_namespace: None,
        });

    // Use custom pipeline to apply AIXM name fixes before code generation
    println!("Parsing AIXM schemas...");
    let schemas = exec_parser(config.parser).expect("Failed to parse schemas");

    println!("Interpreting schemas...");
    let mut types = exec_interpreter(config.interpreter, &schemas).expect("Failed to interpret schemas");

    println!("Applying AIXM-specific name fixes...");
    apply_aixm_name_fixes(&mut types);

    println!("Optimizing types...");
    let types = exec_optimizer(config.optimizer, types).expect("Failed to optimize types");

    println!("Generating Rust types...");
    let data_types = exec_generator(config.generator, &schemas, &types).expect("Failed to generate types");

    println!("Rendering code...");
    let module = exec_render(config.renderer, &data_types).expect("Failed to render code");

    // Convert module to TokenStream
    let code = module.code;

    // Split into modules and write to files
    write_modules(code);
}

/// Apply AIXM-specific name fixes to MetaTypes before code generation
/// This fixes issues like enum variants named "+" or "-"
fn apply_aixm_name_fixes(types: &mut MetaTypes) {
    use xsd_parser::models::meta::MetaTypeVariant;

    let idents: Vec<_> = types.items.keys().cloned().collect();
    for ident in idents {
        match types.get_variant_mut(&ident).expect("Could not get variant") {
            MetaTypeVariant::Enumeration(enum_meta) => {
                for variant in enum_meta.variants.iter_mut() {
                    let name_str = variant.ident.name.as_str();
                    if let Some(fixed_name) = prepare_name(name_str) {
                        variant.display_name = Some(fixed_name);
                    }
                }
            }
            MetaTypeVariant::Union(union_meta) => {
                for union_type in union_meta.types.iter_mut() {
                    let name_str = union_type.type_.name.as_str();
                    if let Some(fixed_name) = prepare_name(name_str) {
                        union_type.display_name = Some(fixed_name);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Prepare a name for AIXM, handling special characters
fn prepare_name(s: &str) -> Option<String> {
    match s {
        "+" => Some("Plus".to_string()),
        "-" => Some("Minus".to_string()),
        s if s.starts_with("UTC+") || s.starts_with("UTC-") => {
            let fixed = s
                .replace("UTC+", "UtcPlus")
                .replace("UTC-", "UtcMinus")
                .replace('+', "_")
                .replace('-', "_");
            Some(to_pascal_case(&fixed))
        }
        s if s.starts_with('+') || s.starts_with('-') => {
            let fixed = s.replace('+', "Plus").replace('-', "Minus");
            Some(to_pascal_case(&fixed))
        }
        s if s.chars().next().map_or(false, |c| c.is_numeric()) => {
            Some(format!("_{}", to_pascal_case(s)))
        }
        _ => None,
    }
}

/// Convert a string to PascalCase, removing invalid characters
fn to_pascal_case(s: &str) -> String {
    s.chars()
        // Replace invalid identifier characters with underscores
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(|c| c.to_lowercase()))
                    .collect(),
            }
        })
        .collect()
}


/// Split the generated code into modules and write to separate files
fn write_modules(tokens: TokenStream) {
    let syntax_tree = syn::parse2::<syn::File>(tokens).expect("Failed to parse generated code");

    // Create the generated directory
    let out_dir = Path::new("src/generated");
    fs::create_dir_all(out_dir).expect("Failed to create generated directory");

    // Split items by module based on their namespace or type
    let mut modules: HashMap<String, Vec<syn::Item>> = HashMap::new();

    for item in syntax_tree.items {
        let module_name = match &item {
            syn::Item::Struct(s) => extract_module_name(&s.ident.to_string()),
            syn::Item::Enum(e) => extract_module_name(&e.ident.to_string()),
            syn::Item::Type(t) => extract_module_name(&t.ident.to_string()),
            syn::Item::Const(_) => "constants".to_string(),
            syn::Item::Impl(_) => "impls".to_string(),
            _ => "common".to_string(),
        };

        modules.entry(module_name).or_default().push(item);
    }

    // Collect module names before consuming modules
    let module_names: Vec<String> = modules.keys().cloned().collect();

    // Write each module to a separate file
    for (module_name, items) in modules {
        let file_path = out_dir.join(format!("{}.rs", module_name));
        let module_tokens = quote! {
            #(#items)*
        };

        let formatted = format_code(module_tokens);
        let mut file = File::create(&file_path).expect("Failed to create module file");
        file.write_all(formatted.as_bytes())
            .expect("Failed to write module file");

        println!("Generated module: {}", file_path.display());
    }

    // Create mod.rs to export all modules
    create_mod_file(&module_names, out_dir);
}

/// Extract module name from a type identifier
fn extract_module_name(ident: &str) -> String {
    // Try to extract a meaningful prefix for module organization
    // AIXM types often have prefixes like "Airspace", "Navaid", etc.

    // Common AIXM prefixes
    let prefixes = vec![
        "Airspace", "Airport", "Runway", "Navaid", "Route", "Procedure",
        "Obstacle", "AirTrafficControl", "Organisation", "Unit", "Service",
        "Frequency", "Guidance", "Vertical", "Horizontal", "Geographic",
        "Time", "Address", "Contact", "Note", "Code", "Value", "Property",
    ];

    for prefix in prefixes {
        if ident.starts_with(prefix) {
            return prefix.to_lowercase();
        }
    }

    // For types starting with "AIXM" or other patterns
    if ident.starts_with("AIXM") {
        return "aixm_types".to_string();
    }

    // Default to common module
    "types".to_string()
}

/// Create the mod.rs file that exports all generated modules
fn create_mod_file(module_names: &[String], out_dir: &Path) {
    let modules: Vec<TokenStream> = module_names
        .iter()
        .map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                pub mod #ident;
            }
        })
        .collect();

    let content = quote! {
        // This file is automatically generated by build.rs
        // Do not edit manually

        #(#modules)*
    };

    let formatted = format_code(content);
    let mod_file_path = out_dir.join("mod.rs");
    let mut file = File::create(&mod_file_path).expect("Failed to create mod.rs");
    file.write_all(formatted.as_bytes())
        .expect("Failed to write mod.rs");

    println!("Generated mod.rs: {}", mod_file_path.display());
}

/// Format Rust code using prettyplease
fn format_code(tokens: TokenStream) -> String {
    let syntax_tree = syn::parse2(tokens).expect("Failed to parse code for formatting");
    prettyplease::unparse(&syntax_tree)
}
