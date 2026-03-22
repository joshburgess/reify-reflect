//! # reflect-derive
//!
//! Proc macro crate providing `#[derive(Reflect)]`.
//!
//! Generates a `Reflect` implementation for structs that emits a
//! `RuntimeValue::List` of `(field_name, reflected_value)` pairs.
//! Fields whose types implement `Reflect<Value = RuntimeValue>` are
//! reflected; fields annotated with `#[reflect(skip)]` are omitted.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derives the `Reflect` trait for a struct.
///
/// Each field whose type implements `Reflect<Value = RuntimeValue>` is
/// included in the output as a `RuntimeValue::List` of two-element lists
/// `[RuntimeValue::List([field_name_bytes...]), field_value]`.
///
/// Fields annotated with `#[reflect(skip)]` are excluded from the output.
///
/// # Examples
///
/// ```ignore
/// use reflect_derive::Reflect;
/// use reflect_core::{Reflect, RuntimeValue};
///
/// #[derive(Reflect)]
/// struct Point {
///     #[reflect(skip)]
///     label: String,
///     x: MyNat,
///     y: MyNat,
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

fn derive_reflect_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "Reflect can only be derived for structs",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "Reflect can only be derived for structs",
            ));
        }
    };

    let field_entries = build_field_entries(fields)?;

    let expanded = quote! {
        impl #impl_generics reflect_core::Reflect for #name #ty_generics #where_clause {
            type Value = reflect_core::RuntimeValue;

            fn reflect() -> Self::Value {
                reflect_core::RuntimeValue::List(vec![#(#field_entries),*])
            }
        }
    };

    Ok(expanded)
}

fn build_field_entries(fields: &Fields) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut entries = Vec::new();

    let named_fields = match fields {
        Fields::Named(f) => &f.named,
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                fields,
                "Reflect derive requires named fields (not tuple structs)",
            ));
        }
        Fields::Unit => return Ok(entries),
    };

    for field in named_fields {
        if has_skip_attr(field)? {
            continue;
        }

        let field_name = field.ident.as_ref().expect("named field must have ident");
        let field_name_str = field_name.to_string();
        let field_ty = &field.ty;

        entries.push(quote! {
            reflect_core::RuntimeValue::List(vec![
                reflect_core::RuntimeValue::List(
                    #field_name_str.bytes()
                        .map(|b| reflect_core::RuntimeValue::Nat(b as u64))
                        .collect()
                ),
                <#field_ty as reflect_core::Reflect>::reflect(),
            ])
        });
    }

    Ok(entries)
}

fn has_skip_attr(field: &syn::Field) -> syn::Result<bool> {
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
