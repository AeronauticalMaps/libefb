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

//! Derive macros for ARINC 424 record types.
//!
//! This crate allows to `derive` a `Record` implementation and implements
//! `TryFrom` on the record too.
//!
//! # Example
//!
//! Deriving an implementation on an airport:
//!
//! ```ignore
//! #[derive(Record)]
//! pub struct Airport<'a> {
//!     pub record_type: RecordType,
//!     pub icao_code: IcaoCode<'a>,
//!     #[arinc424(skip(5))]  // Skip 5 reserved bytes before this field
//!     pub latitude: Latitude<'a>,
//!     #[arinc424(field = 86)]  // Jump to absolute column 86
//!     pub datum: Datum,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, ExprLit, Fields, Lit, Meta};

enum FieldAttribute {
    Skip(usize),
    Position(usize),
}

fn parse_field_attributes(field: &syn::Field) -> Option<FieldAttribute> {
    for attr in &field.attrs {
        if !attr.path().is_ident("arinc424") {
            continue;
        }

        if let Ok(meta) = attr.parse_args::<Meta>() {
            match meta {
                // Handle #[arinc424(skip(n))]
                Meta::List(list) if list.path.is_ident("skip") => {
                    for token in list.tokens {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(int_lit),
                            ..
                        })) = syn::parse2::<Expr>(token.into())
                        {
                            if let Ok(n) = int_lit.base10_parse::<usize>() {
                                return Some(FieldAttribute::Skip(n));
                            }
                        }
                    }
                }
                // Handle #[arinc424(field = n)]
                Meta::NameValue(nv) if nv.path.is_ident("field") => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Int(int_lit),
                        ..
                    }) = nv.value
                    {
                        if let Ok(n) = int_lit.base10_parse::<usize>() {
                            return Some(FieldAttribute::Position(n));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Derive macro for implementing the `Record` trait.
///
/// Generates both the `Record` trait implementation and `TryFrom<&[u8]>` implementation.
#[proc_macro_derive(Record, attributes(arinc424))]
pub fn derive_record(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract lifetime parameter (should be 'a)
    let lifetime = generics
        .lifetimes()
        .next()
        .expect("Record types must have a lifetime parameter (e.g., 'a)");

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Record derive only supports structs with named fields"),
        },
        _ => panic!("Record derive only supports structs"),
    };

    // Generate field parsing code
    let mut field_parsers = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();

        // Parse attributes
        let parse_expr = match parse_field_attributes(field) {
            Some(FieldAttribute::Position(pos)) => {
                quote! {
                    #field_name: fields.get(#pos)?
                }
            }
            Some(FieldAttribute::Skip(n)) => {
                quote! {
                    #field_name: fields.skip(#n).next()?
                }
            }
            None => {
                quote! {
                    #field_name: fields.next()?
                }
            }
        };

        field_parsers.push(parse_expr);
    }

    let expanded = quote! {
        impl #impl_generics crate::record::Record<#lifetime> for #name #ty_generics #where_clause {
            fn parse(mut fields: crate::record::Fields<#lifetime>) -> Result<Self, crate::Error> {
                Ok(Self {
                    #(#field_parsers),*
                })
            }
        }

        impl #impl_generics ::core::convert::TryFrom<&#lifetime [u8]> for #name #ty_generics #where_clause {
            type Error = crate::Error;

            fn try_from(bytes: &#lifetime [u8]) -> Result<Self, crate::Error> {
                <Self as crate::record::Record>::from_bytes(bytes)
            }
        }
    };

    TokenStream::from(expanded)
}
