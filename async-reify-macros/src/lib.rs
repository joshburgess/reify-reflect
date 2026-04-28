//! Attribute proc macros for [`async-reify`](https://docs.rs/async-reify).
//!
//! Currently provides [`macro@trace_async`], an attribute that rewrites
//! every `.await` point in an `async fn` body to record into a shared
//! `async_reify::Trace` without you having to wrap each await in
//! `LabeledFuture` by hand.
//!
//! You normally do not depend on this crate directly. Enable the `macros`
//! feature on `async-reify` and the attribute is re-exported as
//! [`async_reify::trace_async`](https://docs.rs/async-reify/latest/async_reify/attr.trace_async.html).
//!
//! # What the macro does
//!
//! `#[trace_async(trace = my_trace)]` on a function rewrites every
//! `.await` inside the body so it is wrapped in a
//! [`LabeledFuture`](https://docs.rs/async-reify/latest/async_reify/struct.LabeledFuture.html)
//! that records into the trace handle named by the `trace = IDENT`
//! argument. Labels are auto-generated as `"<expr> @ <file>:<line>"`, so
//! every step in the resulting trace points back to the source line that
//! produced it.
//!
//! See the [`async-reify` crate docs](https://docs.rs/async-reify) for
//! the recording, inspection, and rendering pipeline this feeds into,
//! and [`docs/phase4-async-reify.md`][phase4] for the design rationale.
//!
//! [phase4]: https://github.com/joshburgess/reify-reflect/blob/main/docs/phase4-async-reify.md

#![deny(unsafe_code)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::visit_mut::VisitMut;
use syn::{parse_macro_input, parse_quote, Expr, Ident, ItemFn, Token};

/// Parsed `#[trace_async(trace = IDENT)]` arguments.
struct TraceAsyncArgs {
    trace_ident: Ident,
}

impl Parse for TraceAsyncArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Single arg: `trace = IDENT`
        let key: Ident = input.parse()?;
        if key != "trace" {
            return Err(syn::Error::new(key.span(), "expected `trace = IDENT`"));
        }
        let _eq: Token![=] = input.parse()?;
        let trace_ident: Ident = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("unexpected tokens after `trace = IDENT`"));
        }
        Ok(TraceAsyncArgs { trace_ident })
    }
}

/// Rewrite every `.await` in the visited body so it goes through a
/// [`async_reify::LabeledFuture`] backed by the named trace handle.
///
/// The label is auto-generated as `"<expr> @ <file>:<line>"`.
struct AwaitRewriter {
    trace_ident: Ident,
}

impl VisitMut for AwaitRewriter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Recurse into children first so nested awaits are rewritten before
        // we wrap their parent. This is safe because we replace `expr` with
        // a block whose `.await` is already inside our wrapper expression
        // and will not be re-visited.
        syn::visit_mut::visit_expr_mut(self, expr);

        if let Expr::Await(await_expr) = expr {
            let inner = &*await_expr.base;
            let label_str = inner_to_label(inner);
            let trace = &self.trace_ident;
            let replacement: Expr = parse_quote! {
                {
                    let __label = format!(
                        "{} @ {}:{}",
                        #label_str,
                        file!(),
                        line!(),
                    );
                    ::async_reify::LabeledFuture::new(#inner, &__label, #trace.clone()).await
                }
            };
            *expr = replacement;
        }
    }

    // Don't descend into nested closures or item definitions; their `.await`
    // (if any, e.g. inside a nested `async fn`) is in a different async
    // context and uses its own trace.
    fn visit_expr_closure_mut(&mut self, _: &mut syn::ExprClosure) {}
    fn visit_item_mut(&mut self, _: &mut syn::Item) {}
}

fn inner_to_label(expr: &Expr) -> String {
    let s = quote!(#expr).to_string();
    // Collapse runs of whitespace from token-stream pretty-printing.
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// Rewrites an async function body so every `.await` records a labeled
/// `async_reify::PollEvent` into the named shared trace.
///
/// The macro takes a single mandatory argument: `trace = IDENT`, where
/// `IDENT` names a value of type
/// `std::sync::Arc<std::sync::Mutex<async_reify::Trace>>` that is in
/// scope inside the function. Typically `IDENT` is a function parameter.
///
/// Each `.await` inside the function body is replaced with an
/// `async_reify::LabeledFuture` that records into the shared trace.
/// The label is `"<expr> @ <file>:<line>"`. Awaits inside nested
/// closures or nested item definitions are left alone (they belong
/// to a different async scope).
///
/// # Examples
///
/// ```ignore
/// use std::sync::{Arc, Mutex};
/// use async_reify::Trace;
/// use async_reify_macros::trace_async;
///
/// #[trace_async(trace = trace)]
/// async fn workflow(trace: Arc<Mutex<Trace>>) -> i32 {
///     fetch().await;
///     compute().await;
///     42
/// }
/// ```
#[proc_macro_attribute]
pub fn trace_async(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TraceAsyncArgs);
    let mut func = parse_macro_input!(item as ItemFn);

    if func.sig.asyncness.is_none() {
        return syn::Error::new_spanned(&func.sig, "#[trace_async] requires an `async fn`")
            .to_compile_error()
            .into();
    }

    let mut rewriter = AwaitRewriter {
        trace_ident: args.trace_ident,
    };
    rewriter.visit_block_mut(&mut func.block);

    let tokens: TokenStream2 = quote! { #func };
    tokens.into()
}
