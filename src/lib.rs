#![deny(unsafe_code)]

//! A unified, fully-safe ecosystem for moving values between Rust's type
//! level and value level.
//!
//! This is the **facade crate**. It re-exports each of the focused crates
//! in the workspace under a single namespace, so a single dependency on
//! `reify-reflect` (with appropriate features) is enough to use everything.
//! If you only need one piece, depending on the focused crate directly
//! gives you a smaller build.
//!
//! # What is "reification" and "reflection"?
//!
//! Two complementary directions:
//!
//! - **Reflection** is type → value. A type like `S<S<S<Z>>>` encodes the
//!   number 3 at compile time; [`core::Reflect::reflect`] hands you `3`
//!   at runtime.
//! - **Reification** is value → type. A `u64` known only at runtime can
//!   be lifted into a callback in which it is genuinely a `const N: u64`.
//!   See [`const_reify`](https://docs.rs/const-reify) (re-exported as
//!   [`const_bridge`] when the `const-reify` feature is enabled).
//!
//! Everything here is `#![deny(unsafe_code)]`. There is no `unsafe`, no
//! compiler-internal layout assumption, and no `unsafeCoerce`-style trick.
//! The Rust borrow checker enforces what GHC's parametricity enforces in
//! Haskell's `reflection` library.
//!
//! # Module map
//!
//! | Module | Crate | Use it for |
//! |---|---|---|
//! | [`core`] | [`reify_reflect_core`] | The [`Reflect`](core::Reflect) trait, the [`reify`](core::reify) scoping function, the [`Reified`](core::Reified) branded token, and the [`RuntimeValue`](core::RuntimeValue) enum. |
//! | [`nat`] | [`reflect_nat`] | Peano naturals (`Z`/`S<N>`), type-level booleans, heterogeneous lists, with optional `frunk`/`typenum` bridges. |
//! | [`graph`] | [`reify_graph`] | Convert `Rc<RefCell<T>>` or `Arc<Mutex<T>>` graphs to and from a flat node+edge form, preserving sharing and cycles. |
//! | [`context`] | [`context_trait`] | Swap out `Ord`, `Hash`, `Display` (or any user-defined trait) for one block of code via `WithContext`. |
//! | [`async_trace`] | [`async_reify`] | Wrap futures to record poll events and turn them into an inspectable async step graph. |
//! | [`const_bridge`] (feature `const-reify`) | [`const_reify`](https://docs.rs/const-reify) | Dispatch a runtime `u64` in `0..=255` to the matching `const N: u64` monomorphization, safely. |
//!
//! For the `#[derive(Reflect)]`, `#[trace_async]`, and `#[reifiable]`
//! proc macros, depend on `reflect-derive`, `async-reify-macros`, and
//! `const-reify-derive` respectively (or enable the relevant features).
//!
//! # Quick start
//!
//! Reflect a type-level number to a runtime value:
//!
//! ```
//! use reify_reflect::core::{Reflect, RuntimeValue};
//! use reify_reflect::nat::{S, Z};
//!
//! type Three = S<S<S<Z>>>;
//! assert_eq!(Three::reflect(), RuntimeValue::Nat(3));
//! ```
//!
//! For deeper walkthroughs, see the [guides] in the source tree.
//!
//! [guides]: https://github.com/joshburgess/reify-reflect/tree/main/docs/guides
//!
//! # Feature flags
//!
//! | Feature | Default | Effect |
//! |---|---|---|
//! | `serde` | yes | Enables `Serialize`/`Deserialize` impls in [`graph`] and [`async_trace`]. |
//! | `const-reify` | no | Enables the [`const_bridge`] module. Adds 256 monomorphizations to compile time. |
//! | `typenum` | no | Bridge between [`nat`] and the `typenum` crate. |
//! | `frunk` | no | Bridge between [`nat`] and `frunk`'s HList. |
//! | `full` | no | All of the above. |

/// Core traits and types: [`Reflect`](reify_reflect_core::Reflect),
/// [`reify`](reify_reflect_core::reify), [`Reified`](reify_reflect_core::Reified),
/// [`RuntimeValue`](reify_reflect_core::RuntimeValue).
pub mod core {
    pub use reify_reflect_core::*;
}

/// Type-level naturals ([`Z`](reflect_nat::Z), [`S`](reflect_nat::S)),
/// booleans ([`True`](reflect_nat::True), [`False`](reflect_nat::False)),
/// and heterogeneous lists ([`HNil`](reflect_nat::HNil), [`HCons`](reflect_nat::HCons)).
pub mod nat {
    pub use reflect_nat::*;
}

/// `Rc<RefCell<T>>` graph reification and reconstruction.
pub mod graph {
    pub use reify_graph::*;
}

/// Runtime-synthesized trait instances scoped to callbacks.
pub mod context {
    pub use context_trait::*;
}

/// Async computation tracing and step graph extraction.
pub mod async_trace {
    pub use async_reify::*;
}

/// Runtime-to-const-generic dispatch (requires `const-reify` feature).
#[cfg(feature = "const-reify")]
pub mod const_bridge {
    pub use const_reify::*;
}
