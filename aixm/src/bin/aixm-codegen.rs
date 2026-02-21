// SPDX-License-Identifier: Apache-2.0
// Copyright 2025, 2026 Joe Pearson
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

//! Code generator for AIXM types from XSD schemas.
//!
//! Run with: `cargo run -p aixm --bin aixm-codegen --features codegen`

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use xsd_parser::config::{
    GeneratorFlags, InterpreterFlags, OptimizerFlags, RenderStep, Resolver, Schema,
};
use xsd_parser::models::meta::{MetaTypeVariant, MetaTypes};
use xsd_parser::pipeline::renderer::NamespaceSerialization;
use xsd_parser::{
    exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render, Config, Module,
    SubModules,
};

const AIXM_FEATURES_URL: &str = "https://www.aixm.aero/schema/5.2/5.2.0/AIXM_Features.xsd";
const AIXM_MESSAGE_URL: &str =
    "https://www.aixm.aero/schema/5.2/5.2.0/message/AIXM_BasicMessage.xsd";

fn main() {
    let config = build_config();

    eprintln!("Parsing AIXM schemas (this downloads ~50 XSD files)...");
    let schemas = exec_parser(config.parser).expect("Failed to parse schemas");

    eprintln!("Interpreting schemas...");
    let mut meta_types =
        exec_interpreter(config.interpreter, &schemas).expect("Failed to interpret schemas");

    eprintln!("Applying AIXM-specific name fixes...");
    apply_aixm_name_fixes(&mut meta_types);

    eprintln!("Optimizing types...");
    let meta_types =
        exec_optimizer(config.optimizer, meta_types).expect("Failed to optimize types");

    eprintln!("Generating Rust types...");
    let data_types =
        exec_generator(config.generator, &schemas, &meta_types).expect("Failed to generate types");

    eprintln!("Rendering code...");
    let module = exec_render(config.renderer, &data_types).expect("Failed to render code");

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/generated");
    eprintln!("Writing module tree to {}...", out_dir.display());
    write_module_tree(&module, &out_dir);

    eprintln!("Done.");
}

fn build_config() -> Config {
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

    config
        .with_render_step(RenderStep::Types)
        .with_render_step(RenderStep::Defaults)
        .with_render_step(RenderStep::NamespaceConstants)
        .with_render_step(RenderStep::QuickXmlDeserialize {
            boxed_deserializer: false,
        })
        .with_render_step(RenderStep::QuickXmlSerialize {
            namespaces: NamespaceSerialization::Global,
            default_namespace: None,
        })
}

/// Apply AIXM-specific name fixes to MetaTypes before code generation.
///
/// Some AIXM enum variants use characters like `+` or `-` that are not valid
/// Rust identifiers. This function assigns display names so the generator
/// produces valid code.
fn apply_aixm_name_fixes(types: &mut MetaTypes) {
    for (_ident, ty) in types.items.iter_mut() {
        match &mut ty.variant {
            MetaTypeVariant::Enumeration(enum_meta) => {
                for variant in enum_meta.variants.iter_mut() {
                    if let Some(fixed) = prepare_name(variant.ident.name.as_str()) {
                        variant.display_name = Some(fixed);
                    }
                }
            }
            MetaTypeVariant::Union(union_meta) => {
                for union_type in union_meta.types.iter_mut() {
                    if let Some(fixed) = prepare_name(union_type.type_.name.as_str()) {
                        union_type.display_name = Some(fixed);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Prepare a name for AIXM, handling special characters that would produce
/// invalid Rust identifiers.
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
        s if s.starts_with(|c: char| c.is_ascii_digit()) => Some(format!("_{}", to_pascal_case(s))),
        _ => None,
    }
}

fn to_pascal_case(s: &str) -> String {
    s.chars()
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

/// Write the module tree produced by xsd-parser to the filesystem.
///
/// Level 1 (namespace modules like `aixm`, `gml`, `message`) are already
/// present in `module.modules`. For each namespace module we further split
/// the flat code into per-type files.
fn write_module_tree(root: &Module, out_dir: &Path) {
    // Clean previous output
    if out_dir.exists() {
        fs::remove_dir_all(out_dir).expect("Failed to clean generated directory");
    }
    fs::create_dir_all(out_dir).expect("Failed to create generated directory");

    let mut namespace_names: Vec<String> = Vec::new();

    // Write each namespace sub-module
    for (name, sub_module) in &root.modules {
        let mod_dir = out_dir.join(name);
        fs::create_dir_all(&mod_dir).expect("Failed to create namespace directory");

        write_namespace_module(sub_module, &mod_dir);
        namespace_names.push(name.clone());
    }

    // If the root itself has code (usings, constants), write it into the
    // top-level mod.rs alongside the sub-module declarations.
    let mut root_code = TokenStream::new();
    root.to_code(&mut root_code, SubModules::None);

    // Build the top-level mod.rs
    let mod_decls: Vec<TokenStream> = namespace_names
        .iter()
        .map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                pub mod #ident;
                pub use #ident::*;
            }
        })
        .collect();

    let top_mod = quote! {
        // This file is automatically generated. Do not edit manually.
        #![allow(
            clippy::all,
            unused_imports,
            dead_code,
            non_camel_case_types,
            non_snake_case,
            missing_docs
        )]

        #(#mod_decls)*

        #root_code
    };

    write_formatted(out_dir.join("mod.rs"), top_mod);
}

/// Split a namespace module's code into per-type files and write them.
fn write_namespace_module(module: &Module, dir: &Path) {
    // Get the flat code for this module (no sub-modules inlined)
    let mut tokens = TokenStream::new();
    module.to_code(&mut tokens, SubModules::None);

    let file = syn::parse2::<syn::File>(tokens).expect("Failed to parse generated code");

    // Group items by type name
    let mut type_files: BTreeMap<String, Vec<syn::Item>> = BTreeMap::new();

    for item in file.items {
        let key = item_file_key(&item);
        type_files.entry(key).or_default().push(item);
    }

    // Write each type file
    let file_names: Vec<String> = type_files.keys().cloned().collect();

    for (file_name, items) in &type_files {
        let item_tokens: Vec<TokenStream> = items.iter().map(|i| quote! { #i }).collect();
        let file_code = quote! {
            use super::*;
            #(#item_tokens)*
        };

        write_formatted(dir.join(format!("{file_name}.rs")), file_code);
    }

    // If the namespace module itself has sub-modules, recurse
    for (sub_name, sub_module) in &module.modules {
        let sub_dir = dir.join(sub_name);
        fs::create_dir_all(&sub_dir).expect("Failed to create sub-module directory");
        write_namespace_module(sub_module, &sub_dir);
    }

    // Build mod.rs with pub mod + pub use for each file, plus sub-modules
    let mut mod_items = Vec::new();

    for name in &file_names {
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        mod_items.push(quote! {
            mod #ident;
            pub use #ident::*;
        });
    }

    for sub_name in module.modules.keys() {
        let ident = syn::Ident::new(sub_name, proc_macro2::Span::call_site());
        mod_items.push(quote! {
            pub mod #ident;
            pub use #ident::*;
        });
    }

    let mod_code = quote! {
        // This file is automatically generated. Do not edit manually.
        #![allow(
            clippy::all,
            unused_imports,
            dead_code,
            non_camel_case_types,
            non_snake_case,
            missing_docs
        )]

        #(#mod_items)*
    };

    write_formatted(dir.join("mod.rs"), mod_code);
}

/// Determine which file an item belongs to.
fn item_file_key(item: &syn::Item) -> String {
    let raw = match item {
        syn::Item::Struct(s) => s.ident.to_string().to_snake_case(),
        syn::Item::Enum(e) => e.ident.to_string().to_snake_case(),
        syn::Item::Type(t) => t.ident.to_string().to_snake_case(),
        syn::Item::Impl(imp) => extract_impl_type_name(&imp.self_ty)
            .map(|n| n.to_snake_case())
            .unwrap_or_else(|| "_misc".to_string()),
        syn::Item::Const(_) => "constants".to_string(),
        _ => "_misc".to_string(),
    };
    sanitize_module_name(&raw)
}

/// Check if a name is a Rust keyword that needs `r#` escaping in module declarations.
fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "type" | "struct" | "enum" | "fn" | "mod" | "use" | "pub" | "impl" | "trait" | "const"
        | "static" | "let" | "mut" | "ref" | "self" | "super" | "crate" | "as" | "in"
        | "for" | "if" | "else" | "loop" | "while" | "match" | "return" | "break"
        | "continue" | "where" | "async" | "await" | "dyn" | "move" | "extern" | "unsafe"
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" | "priv"
        | "typeof" | "unsized" | "virtual" | "yield" | "try"
    )
}

/// Sanitize a module/file name: prefix with underscore if it's a Rust keyword.
fn sanitize_module_name(name: &str) -> String {
    if is_rust_keyword(name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

/// Extract the type name from an impl block's self_ty.
fn extract_impl_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(tp) => tp.path.segments.last().map(|seg| seg.ident.to_string()),
        syn::Type::Reference(r) => extract_impl_type_name(&r.elem),
        _ => None,
    }
}

/// Format a TokenStream with prettyplease and write to a file.
fn write_formatted(path: impl AsRef<Path>, tokens: TokenStream) {
    let path = path.as_ref();
    let code = match syn::parse2::<syn::File>(tokens.clone()) {
        Ok(syntax_tree) => prettyplease::unparse(&syntax_tree),
        Err(e) => {
            eprintln!(
                "Warning: failed to parse {} for formatting: {e}. Writing raw tokens.",
                path.display()
            );
            tokens.to_string()
        }
    };

    let mut file = File::create(path).unwrap_or_else(|e| {
        panic!("Failed to create {}: {e}", path.display());
    });
    file.write_all(code.as_bytes()).unwrap_or_else(|e| {
        panic!("Failed to write {}: {e}", path.display());
    });
}
