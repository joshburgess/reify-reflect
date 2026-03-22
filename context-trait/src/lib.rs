#![deny(unsafe_code)]

//! # context-trait
//!
//! Runtime-synthesized trait instances using function pointer tables,
//! scoped to a callback.
//!
//! This crate provides [`WithContext`], a wrapper that pairs a value with
//! a context supplying trait implementations. Built-in contexts include
//! [`OrdContext`], [`HashContext`], and [`DisplayContext`].
//!
//! Declarative macros [`with_ord!`], [`with_hash!`], and [`with_display!`]
//! make it easy to use non-default trait implementations in a scoped block.
//!
//! The [`impl_context_trait!`] macro lets users define new context types
//! for arbitrary traits.
//!
//! # Examples
//!
//! ```
//! use context_trait::{with_ord, OrdContext, WithContext};
//!
//! let items = vec![3i32, 1, 4, 1, 5];
//! with_ord!(items, |a: &i32, b: &i32| b.cmp(a), |wrapped: &[WithContext<i32, OrdContext<i32>>]| {
//!     let mut sorted = wrapped.to_vec();
//!     sorted.sort();
//!     let values: Vec<i32> = sorted.into_iter().map(|w| w.inner).collect();
//!     assert_eq!(values, vec![5, 4, 3, 1, 1]);
//! });
//! ```

mod context;
mod display_ctx;
mod hash_ctx;
mod macros;
mod ord_ctx;

pub use context::WithContext;
pub use display_ctx::DisplayContext;
pub use hash_ctx::HashContext;
pub use ord_ctx::OrdContext;
