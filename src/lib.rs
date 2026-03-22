#![deny(unsafe_code)]

//! # reflect-rs
//!
//! Unified reification and reflection ecosystem for Rust.
//!
//! This facade crate re-exports the core components of the ecosystem:
//!
//! - [`core`] — `Reflect` trait, `reify` function, `Reified` token, `RuntimeValue` enum
//! - [`nat`] — Type-level naturals, booleans, and heterogeneous lists
//! - [`derive`](reflect_derive) — `#[derive(Reflect)]` proc macro
//! - [`graph`] — `Rc`/`Arc` graph reification and reconstruction
//! - [`context`] — Runtime-synthesized trait instances
//! - [`async_trace`] — Async computation step graph extraction
//! - [`const_bridge`] — Runtime-to-const-generic dispatch (behind `const-reify` feature)
//!
//! # Feature Flags
//!
//! - `serde` *(default)* — Enables serde support for `reify-graph` and `async-reify`
//! - `const-reify` — Enables the `const_bridge` module for runtime-to-const-generic dispatch
//! - `full` — Enables all features
//!
//! # Quick Start
//!
//! ```
//! use reflect_rs::core::{Reflect, RuntimeValue};
//! use reflect_rs::nat::{S, Z};
//!
//! type Three = S<S<S<Z>>>;
//! assert_eq!(Three::reflect(), RuntimeValue::Nat(3));
//! ```

/// Core traits and types: [`Reflect`](reflect_core::Reflect),
/// [`reify`](reflect_core::reify), [`Reified`](reflect_core::Reified),
/// [`RuntimeValue`](reflect_core::RuntimeValue).
pub mod core {
    pub use reflect_core::*;
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
