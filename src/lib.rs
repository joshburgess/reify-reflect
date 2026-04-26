#![deny(unsafe_code)]

//! # reify-reflect
//!
//! Unified reification and reflection ecosystem for Rust.
//!
//! This facade crate re-exports the core components of the ecosystem:
//!
//! - [`core`]: `Reflect` trait, `reify` function, `Reified` token, `RuntimeValue` enum
//! - [`nat`]: type-level naturals, booleans, and heterogeneous lists
//! - [`derive`](reflect_derive): `#[derive(Reflect)]` proc macro
//! - [`graph`]: `Rc`/`Arc` graph reification and reconstruction
//! - [`context`]: runtime-synthesized trait instances
//! - [`async_trace`]: async computation step graph extraction
//! - [`const_bridge`]: runtime-to-const-generic dispatch (behind `const-reify` feature)
//!
//! # Feature Flags
//!
//! - `serde` *(default)*: enables serde support for `reify-graph` and `async-reify`
//! - `const-reify`: enables the `const_bridge` module for runtime-to-const-generic dispatch
//! - `full`: enables all features
//!
//! # Quick Start
//!
//! ```
//! use reify_reflect::core::{Reflect, RuntimeValue};
//! use reify_reflect::nat::{S, Z};
//!
//! type Three = S<S<S<Z>>>;
//! assert_eq!(Three::reflect(), RuntimeValue::Nat(3));
//! ```

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
