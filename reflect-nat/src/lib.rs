#![deny(unsafe_code)]

//! # reflect-nat
//!
//! Type-level naturals, booleans, and heterogeneous lists with [`Reflect`](reify_reflect_core::Reflect)
//! implementations.
//!
//! ## Type-Level Naturals
//!
//! Natural numbers are represented using Peano encoding:
//! - [`Z`] represents zero
//! - [`S<N>`] represents the successor of `N`
//!
//! ## Type-Level Booleans
//!
//! - [`True`] and [`False`] with boolean operations [`Not`], [`And`], [`Or`]
//!
//! ## Heterogeneous Lists
//!
//! - [`HNil`] and [`HCons<H, T>`] with [`Reflect`](reify_reflect_core::Reflect) implementations
//!
//! # Examples
//!
//! ```
//! use reflect_nat::{Z, S, True, False, HNil, HCons};
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
