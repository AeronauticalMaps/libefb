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

//! Procedural macros for ARINC 424 field generation.
//!
//! This crate provides macros to reduce boilerplate when defining ARINC 424 fields.
//!
//! # Example
//!
//! ```ignore
//! use arinc424_macros::arinc424_enum_field;
//!
//! // Generate an enum field with automatic FromStr implementation
//! arinc424_enum_field! {
//!     /// My custom field
//!     pub enum MyField<const I: usize> {
//!         length = 2,
//!         variants = {
//!             "AA" => VariantA,
//!             "BB" => VariantB,
//!         }
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    braced, parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated, Attribute,
    Ident, LitInt, LitStr, Token, Visibility,
};

/// A match arm for an enum field variant.
struct MatchArm {
    pattern: LitStr,
    variant: Ident,
}

impl Parse for MatchArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pattern: LitStr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let variant: Ident = input.parse()?;
        Ok(Self { pattern, variant })
    }
}

/// The input to the `arinc424_enum_field` macro.
struct EnumFieldInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    length: usize,
    error_message: String,
    variants: Punctuated<MatchArm, Token![,]>,
}

impl Parse for EnumFieldInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![enum]>()?;
        let name: Ident = input.parse()?;

        // Parse generic parameter <const I: usize>
        input.parse::<Token![<]>()?;
        input.parse::<Token![const]>()?;
        let _i: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let _usize: Ident = input.parse()?;
        input.parse::<Token![>]>()?;

        let content;
        braced!(content in input);

        // Parse length = N,
        let length_ident: Ident = content.parse()?;
        if length_ident != "length" {
            return Err(syn::Error::new(length_ident.span(), "expected `length`"));
        }
        content.parse::<Token![=]>()?;
        let length_lit: LitInt = content.parse()?;
        let length = length_lit.base10_parse()?;
        content.parse::<Token![,]>()?;

        // Parse optional error_message = "...",
        let error_message = if content.peek(Ident) {
            let next_ident: Ident = content.parse()?;
            if next_ident == "error_message" {
                content.parse::<Token![=]>()?;
                let msg: LitStr = content.parse()?;
                content.parse::<Token![,]>()?;
                msg.value()
            } else if next_ident == "variants" {
                content.parse::<Token![=]>()?;
                format!("invalid {}", name)
            } else {
                return Err(syn::Error::new(
                    next_ident.span(),
                    "expected `error_message` or `variants`",
                ));
            }
        } else {
            format!("invalid {}", name)
        };

        // Parse variants = { ... }
        if !content.peek(Ident) {
            // Already parsed variants above
            let variants_content;
            braced!(variants_content in content);
            let variants = Punctuated::parse_terminated(&variants_content)?;
            return Ok(Self {
                attrs,
                vis,
                name,
                length,
                error_message,
                variants,
            });
        }

        let variants_ident: Ident = content.parse()?;
        if variants_ident != "variants" {
            return Err(syn::Error::new(
                variants_ident.span(),
                "expected `variants`",
            ));
        }
        content.parse::<Token![=]>()?;

        let variants_content;
        braced!(variants_content in content);
        let variants = Punctuated::parse_terminated(&variants_content)?;

        Ok(Self {
            attrs,
            vis,
            name,
            length,
            error_message,
            variants,
        })
    }
}

/// Generates an ARINC 424 enum field with automatic `FromStr` implementation.
///
/// # Syntax
///
/// ```ignore
/// arinc424_enum_field! {
///     /// Optional documentation
///     pub enum FieldName<const I: usize> {
///         length = 2,
///         error_message = "custom error message",  // optional
///         variants = {
///             "AA" => VariantA,
///             "BB" => VariantB,
///         }
///     }
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - An enum with the specified variants
/// - A `LENGTH` associated constant
/// - `Field` trait implementation
/// - `FromStr` trait implementation with proper error handling
#[proc_macro]
pub fn arinc424_enum_field(input: TokenStream) -> TokenStream {
    let EnumFieldInput {
        attrs,
        vis,
        name,
        length,
        error_message,
        variants,
    } = parse_macro_input!(input as EnumFieldInput);

    let name_str = name.to_string();
    let variant_idents: Vec<_> = variants.iter().map(|v| &v.variant).collect();

    let match_arms: Vec<TokenStream2> = variants
        .iter()
        .map(|v| {
            let pattern = &v.pattern;
            let variant = &v.variant;
            quote! { #pattern => Ok(Self::#variant) }
        })
        .collect();

    let expanded = quote! {
        #(#attrs)*
        #[derive(Debug, PartialEq, Clone, Copy)]
        #vis enum #name<const I: usize> {
            #(#variant_idents),*
        }

        impl<const I: usize> #name<I> {
            /// The length of the field in characters.
            pub const LENGTH: usize = #length;
        }

        impl<const I: usize> crate::Field for #name<I> {}

        impl<const I: usize> ::std::str::FromStr for #name<I> {
            type Err = crate::FieldError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() < I + Self::LENGTH {
                    return Err(crate::FieldError::invalid_length(
                        #name_str,
                        I,
                        Self::LENGTH,
                    ));
                }

                let value = &s[I..I + Self::LENGTH];
                match value {
                    #(#match_arms,)*
                    c => Err(crate::FieldError::unexpected_char(
                        #name_str,
                        I,
                        Self::LENGTH,
                        #error_message,
                    ).with_actual(c)),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Creates a type alias for an alphanumeric field at a specific position.
///
/// # Syntax
///
/// ```ignore
/// arinc424_alphanumeric! {
///     /// Airport/Heliport Identifier
///     pub type ArptHeliIdent<const I: usize> = AlphaNumericField<I, 4>;
/// }
/// ```
#[proc_macro]
pub fn arinc424_alphanumeric(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();

    // Just pass through as a type alias - this is more for documentation
    let expanded = quote! {
        #input2
    };

    TokenStream::from(expanded)
}
