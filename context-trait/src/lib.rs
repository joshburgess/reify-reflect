#![deny(unsafe_code)]

//! Swap out a type's `Ord`, `Hash`, `Display` (or any user-defined trait)
//! for one block of code, without newtype boilerplate.
//!
//! Sometimes you want to sort a `Vec<i32>` descending, hash a `String`
//! case-insensitively, or render a number with a custom prefix, but only
//! for one operation. The orthodox Rust answer is to wrap the value in a
//! newtype that implements the trait differently. That works, but it
//! requires a new struct, manual impls, and you lose access to all the
//! existing impls on the inner type.
//!
//! This crate offers a lighter alternative. [`WithContext<T, Ctx>`] is a
//! wrapper that pairs a value with a *context*, a small `Copy` struct of
//! function pointers that supplies the relevant trait implementation.
//! Standard library APIs that take `Ord` / `Hash` / `Display` work
//! unchanged; the context decides the behavior.
//!
//! Three built-in contexts cover the common cases:
//!
//! - [`OrdContext<T>`] supplies a custom comparator.
//! - [`HashContext<T>`] supplies a custom hasher.
//! - [`DisplayContext<T>`] supplies a custom formatter.
//!
//! And three corresponding macros wrap up the lift / call / project dance:
//!
//! - [`with_ord!`]
//! - [`with_hash!`]
//! - [`with_display!`]
//!
//! For traits beyond these three, [`impl_context_trait!`] generates a new
//! context type for an arbitrary trait of yours.
//!
//! # Why function pointers?
//!
//! Contexts hold `fn` pointers, not `Box<dyn Fn>`. This makes the wrapper
//! `Copy` regardless of `T`, which is what lets `BTreeSet`,
//! `slice::sort`, and `HashMap` accept it without complaint. It also
//! means the comparator must be stateless. If you need captured state,
//! reach for a newtype.
//!
//! See [`docs/phase3-context-trait.md`][phase3] for the full design
//! rationale.
//!
//! [phase3]: https://github.com/joshburgess/reify-reflect/blob/main/docs/phase3-context-trait.md
//!
//! # Examples
//!
//! Sort a slice descending without a newtype:
//!
//! ```
//! use context_trait::{with_ord, OrdContext, WithContext};
//!
//! let items = vec![3i32, 1, 4, 1, 5];
//! with_ord!(items, |a: &i32, b: &i32| b.cmp(a),
//!     |wrapped: &[WithContext<i32, OrdContext<i32>>]| {
//!         let mut sorted = wrapped.to_vec();
//!         sorted.sort();   // uses the descending comparator
//!         let values: Vec<i32> = sorted.into_iter().map(|w| w.inner).collect();
//!         assert_eq!(values, vec![5, 4, 3, 1, 1]);
//!     });
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
