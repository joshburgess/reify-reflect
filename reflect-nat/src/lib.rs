#![deny(unsafe_code)]

//! Ready-made type-level values: Peano naturals, booleans, and
//! heterogeneous lists, all implementing
//! [`Reflect`](reify_reflect_core::Reflect).
//!
//! Where [`reify-reflect-core`](https://docs.rs/reify-reflect-core)
//! defines the [`Reflect`](reify_reflect_core::Reflect) trait and the
//! [`RuntimeValue`](reify_reflect_core::RuntimeValue) vocabulary, this
//! crate provides the most useful concrete instances of that trait, the
//! ones you reach for first when prototyping type-level code.
//!
//! All of these types are zero-sized: they exist only at compile time
//! and disappear at runtime. The [`Reflect`](reify_reflect_core::Reflect)
//! impls are what give you a runtime handle on the value they encode.
//!
//! # What's in here
//!
//! ## Peano naturals
//!
//! - [`Z`] is zero, [`S<N>`] is "successor of `N`".
//! - [`Add`], [`Mul`], and [`Lt`] perform type-level arithmetic.
//! - [`N0`] through [`N8`] are convenience aliases for the small numbers.
//! - The [`Nat`] trait gives you a runtime [`u64`] for any of these types,
//!   and the [`Reflect`](reify_reflect_core::Reflect) impl produces a
//!   [`RuntimeValue::Nat`](reify_reflect_core::RuntimeValue::Nat).
//!
//! ## Type-level booleans
//!
//! - [`True`] and [`False`].
//! - [`Not`], [`And`], [`Or`] perform compile-time boolean logic at the
//!   trait level.
//! - Both reflect to a plain [`prim@bool`].
//!
//! ## Heterogeneous lists
//!
//! - [`HNil`] is the empty list, [`HCons<H, T>`] is a cons cell.
//! - The [`HList`] trait exposes [`len()`](HList::len) and
//!   [`is_empty()`](HList::is_empty) at the type level.
//! - When every element implements
//!   [`Reflect<Value = RuntimeValue>`](reify_reflect_core::Reflect),
//!   the whole HList reflects to a `Vec<RuntimeValue>`.
//!
//! ## Optional bridges (feature-gated)
//!
//! - `frunk`: interoperate with `frunk`'s `HList`.
//! - `typenum`: bridge between [`Nat`] and `typenum`'s `Unsigned`.
//!
//! # Examples
//!
//! ```
//! use reflect_nat::{Z, S, True, HNil, HCons};
//! use reify_reflect_core::{Reflect, RuntimeValue};
//!
//! // Type-level natural: 3
//! type Three = S<S<S<Z>>>;
//! assert_eq!(Three::reflect(), RuntimeValue::Nat(3));
//!
//! // Type-level boolean
//! assert_eq!(True::reflect(), true);
//!
//! // Type-level HList: [3, 0]
//! type MyList = HCons<Three, HCons<Z, HNil>>;
//! assert_eq!(
//!     MyList::reflect(),
//!     vec![RuntimeValue::Nat(3), RuntimeValue::Nat(0)]
//! );
//! ```
//!
//! See also: [`reflect-derive`](https://docs.rs/reflect-derive) for
//! `#[derive(Reflect)]` on your own structs and enums (which can include
//! the types from this crate as fields), and
//! [`const-reify`](https://docs.rs/const-reify) for going the other
//! direction (runtime `u64` to const generic).

mod bool;
mod hlist;
mod nat;

#[cfg(feature = "frunk")]
pub mod frunk_bridge;
#[cfg(feature = "typenum")]
pub mod typenum_bridge;

pub use self::bool::*;
pub use hlist::*;
pub use nat::*;
