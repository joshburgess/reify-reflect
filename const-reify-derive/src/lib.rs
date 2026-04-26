//! # const-reify-derive
//!
//! Proc macro providing `#[reifiable]` for automatic const-generic dispatch.
//!
//! Annotate a trait with `#[reifiable(range = 0..=255)]` and the macro
//! generates match-table dispatch functions for each const-generic method,
//! plus `NatCallback` wrapper structs for integration with `const_reify::reify_nat`.

#![deny(unsafe_code)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    ConstParam, FnArg, GenericParam, Ident, ItemTrait, LitInt, Pat, ReturnType, Token, TraitItem,
    TraitItemFn, Type, Visibility,
};

// ---------------------------------------------------------------------------
// Attribute argument parsing
// ---------------------------------------------------------------------------

/// Parsed `#[reifiable(range = START..=END)]` arguments.
struct ReifiableArgs {
    range_start: u64,
    range_end: u64,
}

impl Parse for ReifiableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: range = START..=END
        let ident: Ident = input.parse()?;
        if ident != "range" {
            return Err(syn::Error::new(ident.span(), "expected `range`"));
        }
        let _eq: Token![=] = input.parse()?;
        let start: LitInt = input.parse()?;
        let _dots: Token![..] = input.parse()?;
        let _eq2: Token![=] = input.parse()?;
        let end: LitInt = input.parse()?;

        Ok(ReifiableArgs {
            range_start: start.base10_parse()?,
            range_end: end.base10_parse()?,
        })
    }
}

// ---------------------------------------------------------------------------
// Method analysis
// ---------------------------------------------------------------------------

/// A const-generic method extracted from the trait.
struct ConstMethod {
    /// Method name.
    name: Ident,
    /// The const generic parameter (name and type).
    _const_param_name: Ident,
    const_param_ty: Type,
    /// Whether the method takes &self or &mut self.
    is_mut: bool,
    /// Non-self, non-const-generic parameters: (name, type) pairs.
    params: Vec<(Ident, Type)>,
    /// Lifetime parameters on the method.
    lifetime_params: Vec<syn::LifetimeParam>,
    /// Type parameters on the method (non-const generics).
    type_params: Vec<syn::TypeParam>,
    /// Return type (None = ()).
    return_type: ReturnType,
}

/// Check if a return type mentions a given identifier (the const param).
fn type_mentions_ident(ty: &Type, ident: &Ident) -> bool {
    let ty_str = quote!(#ty).to_string();
    let ident_str = ident.to_string();
    // Simple heuristic: check if the ident appears as a token in the type.
    // A proper implementation would walk the type AST, but this catches
    // the common cases like [u8; N] and Foo<N>.
    ty_str
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|word| word == ident_str)
}

fn analyze_method(method: &TraitItemFn) -> Option<Result<ConstMethod, syn::Error>> {
    // Find const generic parameters
    let const_params: Vec<&ConstParam> = method
        .sig
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Const(cp) => Some(cp),
            _ => None,
        })
        .collect();

    if const_params.is_empty() {
        return None; // Not a const-generic method, skip
    }

    if const_params.len() > 1 {
        return Some(Err(syn::Error::new_spanned(
            &method.sig,
            "#[reifiable] V1 only supports a single const generic parameter per method",
        )));
    }

    let cp = const_params[0];

    // Check receiver
    let receiver = method.sig.receiver();
    let is_mut = match receiver {
        Some(r) => r.mutability.is_some(),
        None => {
            return Some(Err(syn::Error::new_spanned(
                &method.sig,
                "#[reifiable] requires methods with &self or &mut self receiver",
            )));
        }
    };

    // Check return type doesn't depend on N
    if let ReturnType::Type(_, ref ty) = method.sig.output {
        if type_mentions_ident(ty, &cp.ident) {
            return Some(Err(syn::Error::new_spanned(
                ty,
                format!(
                    "#[reifiable] V1 does not support return types that depend on \
                     the const parameter `{}`. Use NatCallback manually for this case.",
                    cp.ident
                ),
            )));
        }
    }

    // Extract non-self parameters
    let params: Vec<(Ident, Type)> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => {
                let name = match pat_type.pat.as_ref() {
                    Pat::Ident(pi) => pi.ident.clone(),
                    _ => Ident::new("_arg", Span::call_site()),
                };
                Some((name, (*pat_type.ty).clone()))
            }
            FnArg::Receiver(_) => None,
        })
        .collect();

    // Extract lifetime and type params (non-const)
    let lifetime_params: Vec<syn::LifetimeParam> = method
        .sig
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Lifetime(lp) => Some(lp.clone()),
            _ => None,
        })
        .collect();

    let type_params: Vec<syn::TypeParam> = method
        .sig
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => Some(tp.clone()),
            _ => None,
        })
        .collect();

    Some(Ok(ConstMethod {
        name: method.sig.ident.clone(),
        _const_param_name: cp.ident.clone(),
        const_param_ty: cp.ty.clone(),
        is_mut,
        params,
        lifetime_params,
        type_params,
        return_type: method.sig.output.clone(),
    }))
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn generate_dispatch_fn(
    trait_name: &Ident,
    trait_generics: &syn::Generics,
    trait_vis: &Visibility,
    method: &ConstMethod,
    range_start: u64,
    range_end: u64,
) -> TokenStream2 {
    let fn_name = format_ident!("reify_{}", method.name);
    let method_name = &method.name;
    let const_ty = &method.const_param_ty;
    let return_type = &method.return_type;

    // Build range literals
    let range_lits: Vec<LitInt> = (range_start..=range_end)
        .map(|n| LitInt::new(&n.to_string(), Span::call_site()))
        .collect();

    // Parameter names and types for the dispatch function signature
    let param_names: Vec<&Ident> = method.params.iter().map(|(n, _)| n).collect();
    let _param_types: Vec<&Type> = method.params.iter().map(|(_, t)| t).collect();
    let param_decls: Vec<TokenStream2> =
        method.params.iter().map(|(n, t)| quote!(#n: #t)).collect();

    // Trait generic params and where clause
    let _trait_generic_params = &trait_generics.params;
    let _trait_where_clause = &trait_generics.where_clause;

    // Build the trait bound: T: TraitName<GenericArgs>
    let trait_generic_args: Punctuated<TokenStream2, Comma> = trait_generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let ident = &tp.ident;
                quote!(#ident)
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                quote!(#lt)
            }
            GenericParam::Const(cp) => {
                let ident = &cp.ident;
                quote!(#ident)
            }
        })
        .collect();

    let trait_bound = if trait_generic_args.is_empty() {
        quote!(#trait_name)
    } else {
        quote!(#trait_name<#trait_generic_args>)
    };

    // Method lifetime and type params
    let method_lifetime_params: Vec<TokenStream2> = method
        .lifetime_params
        .iter()
        .map(|lp| quote!(#lp))
        .collect();
    let method_type_params: Vec<TokenStream2> =
        method.type_params.iter().map(|tp| quote!(#tp)).collect();
    let method_type_args: Vec<TokenStream2> = method
        .type_params
        .iter()
        .map(|tp| {
            let ident = &tp.ident;
            quote!(#ident)
        })
        .collect();

    // All generic params for the dispatch function
    let mut all_fn_generics: Vec<TokenStream2> = Vec::new();
    for lp in &method_lifetime_params {
        all_fn_generics.push(lp.clone());
    }
    // Trait's own generics
    for p in trait_generics.params.iter() {
        all_fn_generics.push(quote!(#p));
    }
    for tp in &method_type_params {
        all_fn_generics.push(tp.clone());
    }
    all_fn_generics.push(quote!(__ReifyT: #trait_bound));

    let fn_generics = if all_fn_generics.is_empty() {
        quote!()
    } else {
        quote!(<#(#all_fn_generics),*>)
    };

    // Self receiver
    let obj_param = if method.is_mut {
        quote!(obj: &mut __ReifyT)
    } else {
        quote!(obj: &__ReifyT)
    };

    // Match arms — each calls obj.method::<N>(args...) with optional type args
    let match_arms: Vec<TokenStream2> = range_lits
        .iter()
        .map(|n| {
            if method_type_args.is_empty() {
                quote!(#n => obj.#method_name::<#n>(#(#param_names),*))
            } else {
                quote!(#n => obj.#method_name::<#n, #(#method_type_args),*>(#(#param_names),*))
            }
        })
        .collect();

    let range_end_display = range_end;

    quote! {
        /// Auto-generated dispatch function for [`#trait_name::#method_name`].
        ///
        /// Dispatches a runtime `val` to the corresponding const-generic
        /// instantiation of the method.
        #trait_vis fn #fn_name #fn_generics(
            val: #const_ty,
            #obj_param,
            #(#param_decls),*
        ) #return_type {
            match val {
                #(#match_arms,)*
                other => panic!(
                    concat!(
                        "#[reifiable] dispatch for ",
                        stringify!(#trait_name),
                        "::",
                        stringify!(#method_name),
                        ": value {} out of range 0..={}",
                    ),
                    other,
                    #range_end_display,
                ),
            }
        }
    }
}

fn generate_callback_wrapper(
    trait_name: &Ident,
    trait_generics: &syn::Generics,
    trait_vis: &Visibility,
    method: &ConstMethod,
) -> TokenStream2 {
    let wrapper_name = format_ident!(
        "{}{}Callback",
        trait_name,
        pascal_case(&method.name.to_string())
    );
    let method_name = &method.name;
    let return_type_inner = match &method.return_type {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    // Fields: obj reference + each parameter
    let param_names: Vec<&Ident> = method.params.iter().map(|(n, _)| n).collect();
    let _param_types: Vec<&Type> = method.params.iter().map(|(_, t)| t).collect();

    // Trait generic params
    let trait_generic_params = &trait_generics.params;
    let trait_generic_args: Punctuated<TokenStream2, Comma> = trait_generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let ident = &tp.ident;
                quote!(#ident)
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                quote!(#lt)
            }
            GenericParam::Const(cp) => {
                let ident = &cp.ident;
                quote!(#ident)
            }
        })
        .collect();

    let trait_bound = if trait_generic_args.is_empty() {
        quote!(#trait_name)
    } else {
        quote!(#trait_name<#trait_generic_args>)
    };

    // Struct generics include a lifetime, the trait's generics, and T
    let has_trait_generics = !trait_generics.params.is_empty();

    let obj_ref = if method.is_mut {
        // Can't have &mut in a NatCallback (call takes &self), so skip wrapper for mut methods
        return quote!();
    } else {
        quote!(&'__reify_a __ReifyT)
    };

    let struct_fields: Vec<TokenStream2> = std::iter::once(quote! {
        /// The trait implementor.
        pub obj: #obj_ref
    })
    .chain(method.params.iter().map(|(n, t)| quote!(pub #n: #t)))
    .collect();

    let struct_generics = if has_trait_generics {
        quote!(<'__reify_a, #trait_generic_params, __ReifyT: #trait_bound>)
    } else {
        quote!(<'__reify_a, __ReifyT: #trait_bound>)
    };

    let impl_generics = if has_trait_generics {
        quote!(<#trait_generic_params, __ReifyT: #trait_bound>)
    } else {
        quote!(<__ReifyT: #trait_bound>)
    };

    // Method type params for the call
    let method_type_args: Vec<TokenStream2> = method
        .type_params
        .iter()
        .map(|tp| {
            let ident = &tp.ident;
            quote!(#ident)
        })
        .collect();

    let call_expr = if method_type_args.is_empty() {
        quote!(self.obj.#method_name::<N>(#(self.#param_names),*))
    } else {
        quote!(self.obj.#method_name::<N, #(#method_type_args),*>(#(self.#param_names),*))
    };

    quote! {
        /// Auto-generated [`const_reify::NatCallback`] wrapper for
        /// [`#trait_name::#method_name`].
        #trait_vis struct #wrapper_name #struct_generics {
            #(#struct_fields,)*
        }

        impl #impl_generics const_reify::NatCallback<#return_type_inner>
            for #wrapper_name<'_, #trait_generic_args __ReifyT>
        {
            fn call<const N: u64>(&self) -> #return_type_inner {
                #call_expr
            }
        }
    }
}

fn pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Generates const-generic dispatch functions for a trait's methods.
///
/// Annotate a trait with `#[reifiable(range = 0..=255)]` and the macro
/// generates a `reify_<method>` dispatch function for each method that
/// has a const generic parameter.
///
/// # Examples
///
/// ```ignore
/// #[reifiable(range = 0..=255)]
/// trait ModArith {
///     fn mul_mod<const N: u64>(&self, a: u64, b: u64) -> u64;
/// }
///
/// // Generated:
/// // fn reify_mul_mod<T: ModArith>(val: u64, obj: &T, a: u64, b: u64) -> u64
/// ```
///
/// # Limitations (V1)
///
/// - Only supports a single const generic parameter per method
/// - Return types must not depend on the const parameter
/// - `&mut self` methods get dispatch functions but not NatCallback wrappers
#[proc_macro_attribute]
pub fn reifiable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ReifiableArgs);
    let trait_def = parse_macro_input!(item as ItemTrait);

    match reifiable_impl(args, &trait_def) {
        Ok(tokens) => tokens.into(),
        Err(e) => {
            let trait_tokens = quote!(#trait_def);
            let err = e.to_compile_error();
            // Emit the original trait so downstream code doesn't break,
            // plus the error.
            TokenStream::from(quote! {
                #trait_tokens
                #err
            })
        }
    }
}

fn reifiable_impl(args: ReifiableArgs, trait_def: &ItemTrait) -> syn::Result<TokenStream2> {
    let trait_name = &trait_def.ident;
    let trait_vis = &trait_def.vis;
    let trait_generics = &trait_def.generics;

    // Validate range
    if args.range_end > 1023 {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "#[reifiable] range 0..={} would generate {} monomorphizations per method. \
                 Maximum is 1024. Use a smaller range.",
                args.range_end,
                args.range_end + 1,
            ),
        ));
    }

    let mut dispatch_fns = Vec::new();
    let mut callback_wrappers = Vec::new();

    for item in &trait_def.items {
        if let TraitItem::Fn(method) = item {
            if let Some(result) = analyze_method(method) {
                let cm = result?;

                dispatch_fns.push(generate_dispatch_fn(
                    trait_name,
                    trait_generics,
                    trait_vis,
                    &cm,
                    args.range_start,
                    args.range_end,
                ));

                let wrapper = generate_callback_wrapper(trait_name, trait_generics, trait_vis, &cm);
                if !wrapper.is_empty() {
                    callback_wrappers.push(wrapper);
                }
            }
        }
    }

    // Emit the original trait unchanged, plus generated code
    Ok(quote! {
        #trait_def

        #(#dispatch_fns)*

        #(#callback_wrappers)*
    })
}
