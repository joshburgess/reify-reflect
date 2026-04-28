//! `#[derive(Reflect)]` for structs and enums.
//!
//! This crate provides a single proc macro that generates a
//! `reify_reflect_core::Reflect` implementation describing the
//! *structural shape* of a type as a `RuntimeValue` tree.
//!
//! It pairs naturally with
//! [`reflect-nat`](https://docs.rs/reflect-nat): use type-level naturals,
//! booleans, or HLists as fields, and the derive will recursively reflect
//! them into a single value-level description of the type.
//!
//! Note that this reflects the *type schema*, not a particular runtime
//! instance. The generated `reflect()` is a static method (matching the
//! `reify_reflect_core::Reflect` trait), so it walks every variant and
//! every variant's field types.
//!
//! # Encoding
//!
//! The generated `reflect()` returns:
//!
//! - **Named struct** => `RuntimeValue::List` of
//!   `(field_name_bytes, field_value)` pairs.
//! - **Tuple struct** => `RuntimeValue::List` of positional field values.
//! - **Unit struct** (`struct X;`) => `RuntimeValue::Unit`.
//! - **Empty named struct** (`struct X {}`) => `RuntimeValue::List(vec![])`.
//! - **Enum** => `RuntimeValue::List` of variant entries, each
//!   `(variant_name_bytes, variant_payload)`, where the payload mirrors
//!   the per-variant shape (unit / tuple / named).
//!
//! Field names are encoded as a `RuntimeValue::List` of byte values to
//! keep `RuntimeValue` minimal, rather than introducing a new string
//! variant. See [`docs/phase1-foundations.md`][phase1] for the rationale.
//!
//! [phase1]: https://github.com/joshburgess/reify-reflect/blob/main/docs/phase1-foundations.md
//!
//! # Field attributes
//!
//! - `#[reflect(skip)]` omits a field from the generated output. Fields
//!   whose types do not implement `Reflect<Value = RuntimeValue>` must be
//!   skipped (or the type changed) for the derive to compile.
//!
//! # Examples
//!
//! ```ignore
//! use reflect_derive::Reflect;
//! use reify_reflect_core::{Reflect, RuntimeValue};
//! use reflect_nat::{S, Z};
//!
//! #[derive(Reflect)]
//! struct Point {
//!     #[reflect(skip)]
//!     label: String,
//!     x: S<S<Z>>,        // type-level 2
//!     y: S<S<S<Z>>>,     // type-level 3
//! }
//!
//! // Reflects to a List of (b"x", Nat(2)), (b"y", Nat(3)).
//! let _shape: RuntimeValue = <Point as Reflect>::reflect();
//! ```

#![deny(unsafe_code)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Variant};

/// Derives the `Reflect` trait for a struct or enum.
///
/// See the crate-level documentation for a description of the encoding.
///
/// Fields annotated with `#[reflect(skip)]` are excluded from the output.
///
/// # Examples
///
/// ```ignore
/// use reflect_derive::Reflect;
/// use reify_reflect_core::{Reflect, RuntimeValue};
///
/// #[derive(Reflect)]
/// struct Point {
///     #[reflect(skip)]
///     label: String,
///     x: MyNat,
///     y: MyNat,
/// }
///
/// #[derive(Reflect)]
/// struct Pair(MyNat, MyNat);
///
/// #[derive(Reflect)]
/// enum Shape {
///     Point,
///     Line(MyNat, MyNat),
///     Box { w: MyNat, h: MyNat },
/// }
/// ```
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_reflect_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_reflect_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data) => struct_body(data)?,
        Data::Enum(data) => enum_body(data)?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "Reflect cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics reify_reflect_core::Reflect for #name #ty_generics #where_clause {
            type Value = reify_reflect_core::RuntimeValue;

            fn reflect() -> Self::Value {
                #body
            }
        }
    })
}

fn struct_body(data: &DataStruct) -> syn::Result<TokenStream2> {
    match &data.fields {
        Fields::Named(named) => {
            let entries = named_field_entries(&named.named)?;
            Ok(quote! {
                reify_reflect_core::RuntimeValue::List(vec![#(#entries),*])
            })
        }
        Fields::Unnamed(unnamed) => {
            let entries = positional_field_entries(&unnamed.unnamed)?;
            Ok(quote! {
                reify_reflect_core::RuntimeValue::List(vec![#(#entries),*])
            })
        }
        Fields::Unit => Ok(quote! {
            reify_reflect_core::RuntimeValue::Unit
        }),
    }
}

fn enum_body(data: &DataEnum) -> syn::Result<TokenStream2> {
    let mut variant_entries = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        variant_entries.push(variant_entry(variant)?);
    }
    Ok(quote! {
        reify_reflect_core::RuntimeValue::List(vec![#(#variant_entries),*])
    })
}

fn variant_entry(variant: &Variant) -> syn::Result<TokenStream2> {
    let name_str = variant.ident.to_string();
    let name_lit = name_bytes_literal(&name_str);

    let payload = match &variant.fields {
        Fields::Unit => quote! { reify_reflect_core::RuntimeValue::Unit },
        Fields::Named(named) => {
            let entries = named_field_entries(&named.named)?;
            quote! {
                reify_reflect_core::RuntimeValue::List(vec![#(#entries),*])
            }
        }
        Fields::Unnamed(unnamed) => {
            let entries = positional_field_entries(&unnamed.unnamed)?;
            quote! {
                reify_reflect_core::RuntimeValue::List(vec![#(#entries),*])
            }
        }
    };

    Ok(quote! {
        reify_reflect_core::RuntimeValue::List(vec![
            #name_lit,
            #payload,
        ])
    })
}

fn named_field_entries(
    fields: &syn::punctuated::Punctuated<Field, syn::Token![,]>,
) -> syn::Result<Vec<TokenStream2>> {
    let mut entries = Vec::new();
    for field in fields {
        if has_skip_attr(field)? {
            continue;
        }
        let name_str = field
            .ident
            .as_ref()
            .expect("named field must have ident")
            .to_string();
        let name_lit = name_bytes_literal(&name_str);
        let ty = &field.ty;
        entries.push(quote! {
            reify_reflect_core::RuntimeValue::List(vec![
                #name_lit,
                <#ty as reify_reflect_core::Reflect>::reflect(),
            ])
        });
    }
    Ok(entries)
}

fn positional_field_entries(
    fields: &syn::punctuated::Punctuated<Field, syn::Token![,]>,
) -> syn::Result<Vec<TokenStream2>> {
    let mut entries = Vec::new();
    for field in fields {
        if has_skip_attr(field)? {
            continue;
        }
        let ty = &field.ty;
        entries.push(quote! {
            <#ty as reify_reflect_core::Reflect>::reflect()
        });
    }
    Ok(entries)
}

fn name_bytes_literal(s: &str) -> TokenStream2 {
    quote! {
        reify_reflect_core::RuntimeValue::List(
            #s.bytes()
                .map(|b| reify_reflect_core::RuntimeValue::Nat(b as u64))
                .collect()
        )
    }
}

fn has_skip_attr(field: &Field) -> syn::Result<bool> {
    for attr in &field.attrs {
        if attr.path().is_ident("reflect") {
            let mut skip = false;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                    Ok(())
                } else {
                    Err(meta.error("expected `skip`"))
                }
            })?;
            if skip {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
