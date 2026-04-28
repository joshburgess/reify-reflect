#![deny(unsafe_code)]

//! Core traits and types for type-level reification and reflection in Rust.
//!
//! This is the foundation crate. It defines:
//!
//! - [`Reflect`]: a trait for types that carry a compile-time value, with
//!   [`reflect()`](Reflect::reflect) extracting that value at runtime.
//! - [`reify`]: a function that takes any runtime value and lifts it into
//!   a scoped type-level context, where it is available through a
//!   [`Reified`] token branded with an invariant lifetime that cannot
//!   escape the callback.
//! - [`RuntimeValue`]: a small, structural enum used as the standard
//!   payload type for [`Reflect`] implementations across the workspace.
//!
//! Everything is fully safe (`#![deny(unsafe_code)]`), with scoping
//! enforced by the borrow checker.
//!
//! # When should I use this crate?
//!
//! - You're writing a library where types encode values (Peano numbers,
//!   type-level flags, dimensional units, etc.) and want a uniform way to
//!   surface those values at runtime: implement [`Reflect`] for them.
//! - You want a Haskell-`reflection`-style scoping primitive in Rust: use
//!   [`reify`] to thread a runtime value into a callback as if it were a
//!   compile-time fact.
//! - You're writing a downstream crate (such as
//!   [`reflect-nat`](https://docs.rs/reflect-nat) or
//!   [`reflect-derive`](https://docs.rs/reflect-derive)) that produces
//!   [`Reflect`] implementations: depend on this crate for the trait and
//!   the [`RuntimeValue`] vocabulary.
//!
//! # The reification / reflection pattern
//!
//! This implements the pattern from Kiselyov & Shan's *Functional Pearl:
//! Implicit Configurations*, popularized by Kmett's Haskell `reflection`
//! library, adapted to Rust with branded lifetimes for scoping safety.
//!
//! In Haskell:
//!
//! ```haskell
//! reify   :: a -> (forall s. Reifies s a => Proxy s -> r) -> r
//! reflect :: Reifies s a => proxy s -> a
//! ```
//!
//! In Rust, the `forall s` is modeled by an invariant lifetime `'brand`
//! on [`Reified`]. The higher-rank bound `for<'brand>` on the callback
//! ensures the token cannot escape, just as Haskell's rank-2 type
//! prevents `s` from escaping.
//!
//! Unlike Haskell's `reflection` library (which uses `unsafeCoerce` to
//! fabricate typeclass dictionaries from GHC internals), this
//! implementation is safe all the way down: no unsafe code, no
//! compiler-internal assumptions. Scoping is enforced mechanically by
//! the borrow checker.
//!
//! # Examples
//!
//! Lift a runtime value into a scoped type-level context:
//!
//! ```
//! use reify_reflect_core::reify;
//!
//! let result = reify(&42i32, |token| {
//!     let val: &i32 = token.reflect();
//!     *val + 1
//! });
//! assert_eq!(result, 43);
//! ```
//!
//! Implement [`Reflect`] for a type that carries a compile-time value:
//!
//! ```
//! use reify_reflect_core::{Reflect, RuntimeValue};
//!
//! struct Pi;
//!
//! impl Reflect for Pi {
//!     type Value = RuntimeValue;
//!     fn reflect() -> Self::Value {
//!         RuntimeValue::Nat(3)  // close enough
//!     }
//! }
//!
//! assert_eq!(Pi::reflect(), RuntimeValue::Nat(3));
//! ```
//!
//! See also: [`reflect-nat`](https://docs.rs/reflect-nat) for ready-made
//! [`Reflect`] implementations on Peano naturals, booleans, and HLists,
//! and [`reflect-derive`](https://docs.rs/reflect-derive) for
//! `#[derive(Reflect)]` on user types.

use std::marker::PhantomData;

/// Converts a type-level value into a runtime value.
///
/// Types implementing `Reflect` carry a compile-time value that can be
/// extracted at runtime via [`Reflect::reflect`].
///
/// # Examples
///
/// ```
/// use reify_reflect_core::{Reflect, RuntimeValue};
///
/// struct MyZero;
///
/// impl Reflect for MyZero {
///     type Value = RuntimeValue;
///     fn reflect() -> Self::Value {
///         RuntimeValue::Nat(0)
///     }
/// }
///
/// assert_eq!(MyZero::reflect(), RuntimeValue::Nat(0));
/// ```
pub trait Reflect {
    /// The runtime representation of this type-level value.
    type Value;

    /// Produce the runtime value corresponding to this type-level value.
    fn reflect() -> Self::Value;
}

/// A branded token carrying a reified value.
///
/// The lifetime `'brand` is existential, created fresh by each call to
/// [`reify`] and prevented from escaping the callback by the higher-rank
/// bound `for<'brand>`. This mirrors Haskell's
/// `forall s. Reifies s a => Proxy s -> r` scoping.
///
/// The [`PhantomData`] carrying `fn(&'brand ()) -> &'brand ()`
/// makes `'brand` *invariant*, preventing the compiler from shrinking
/// or growing it to unify with any other lifetime. This is what makes
/// the brand unique.
///
/// # Examples
///
/// ```
/// use reify_reflect_core::reify;
///
/// reify(&"hello", |token| {
///     assert_eq!(*token.reflect(), "hello");
/// });
/// ```
///
/// The token cannot escape:
///
/// ```compile_fail
/// use reify_reflect_core::reify;
///
/// let escaped = reify(&42, |token| {
///     token // ERROR: borrowed data escapes the closure
/// });
/// ```
pub struct Reified<'brand, T: ?Sized> {
    value: &'brand T,
    // Invariant in 'brand: prevents lifetime coercion
    _brand: PhantomData<fn(&'brand ()) -> &'brand ()>,
}

impl<'brand, T: ?Sized> Reified<'brand, T> {
    /// Reflect the reified value back to a runtime reference.
    ///
    /// This is the Rust equivalent of Haskell's
    /// `reflect :: Reifies s a => proxy s -> a`.
    ///
    /// The branded lifetime ensures this reference cannot outlive the
    /// [`reify`] callback that created this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use reify_reflect_core::reify;
    ///
    /// let doubled = reify(&21i32, |token| {
    ///     token.reflect() * 2
    /// });
    /// assert_eq!(doubled, 42);
    /// ```
    pub fn reflect(&self) -> &T {
        self.value
    }
}

/// Reify a runtime value into a scoped type-level context.
///
/// This is the Rust equivalent of Haskell's
/// `reify :: a -> (forall s. Reifies s a => Proxy s -> r) -> r`.
///
/// The callback receives a [`Reified`] token branded with a fresh lifetime.
/// The `for<'brand>` bound ensures this lifetime cannot escape the closure,
/// so the token (and any references derived from it) are confined to the
/// callback's scope.
///
/// **No unsafe code.** Unlike Haskell's `reflection` library (which uses
/// `unsafeCoerce` to fabricate typeclass dictionaries), this implementation
/// relies only on Rust's borrow checker for scoping safety.
///
/// # Examples
///
/// Basic reification and reflection:
///
/// ```
/// use reify_reflect_core::reify;
///
/// let result = reify(&100u64, |token| {
///     token.reflect() + 1
/// });
/// assert_eq!(result, 101);
/// ```
///
/// Works with any type:
///
/// ```
/// use reify_reflect_core::reify;
///
/// let result = reify(&vec![1, 2, 3], |token| {
///     token.reflect().iter().sum::<i32>()
/// });
/// assert_eq!(result, 6);
/// ```
///
/// Nested reification:
///
/// ```
/// use reify_reflect_core::reify;
///
/// let result = reify(&10i32, |outer| {
///     reify(&20i32, |inner| {
///         outer.reflect() + inner.reflect()
///     })
/// });
/// assert_eq!(result, 30);
/// ```
///
/// Composing with [`Reflect`]:
///
/// ```
/// use reify_reflect_core::{reify, Reflect, RuntimeValue};
///
/// struct Three;
/// impl Reflect for Three {
///     type Value = RuntimeValue;
///     fn reflect() -> RuntimeValue { RuntimeValue::Nat(3) }
/// }
///
/// let result = reify(&Three::reflect(), |token| {
///     match token.reflect() {
///         RuntimeValue::Nat(n) => *n * 2,
///         _ => panic!("expected Nat"),
///     }
/// });
/// assert_eq!(result, 6);
/// ```
pub fn reify<T: ?Sized, F, R>(val: &T, f: F) -> R
where
    F: for<'brand> FnOnce(Reified<'brand, T>) -> R,
{
    // The lifetime of `val` is erased into the fresh `'brand`.
    // The `for<'brand>` bound on F ensures no `Reified<'brand, T>`
    // can escape — the caller must be parametric in 'brand.
    f(Reified {
        value: val,
        _brand: PhantomData,
    })
}

/// Runtime representation of type-level values.
///
/// This enum provides a uniform way to inspect type-level values at runtime,
/// regardless of their original type-level encoding.
///
/// # Examples
///
/// ```
/// use reify_reflect_core::RuntimeValue;
///
/// let nat = RuntimeValue::Nat(42);
/// let boolean = RuntimeValue::Bool(true);
/// let list = RuntimeValue::List(vec![
///     RuntimeValue::Nat(1),
///     RuntimeValue::Nat(2),
/// ]);
/// let unit = RuntimeValue::Unit;
///
/// assert_eq!(nat, RuntimeValue::Nat(42));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// A natural number.
    Nat(u64),
    /// A boolean.
    Bool(bool),
    /// A list of runtime values.
    List(Vec<RuntimeValue>),
    /// The unit value.
    Unit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_value_nat() {
        let val = RuntimeValue::Nat(42);
        assert_eq!(val, RuntimeValue::Nat(42));
    }

    #[test]
    fn runtime_value_bool() {
        let val = RuntimeValue::Bool(true);
        assert_eq!(val, RuntimeValue::Bool(true));
    }

    #[test]
    fn runtime_value_list() {
        let val = RuntimeValue::List(vec![RuntimeValue::Nat(1), RuntimeValue::Nat(2)]);
        assert_eq!(
            val,
            RuntimeValue::List(vec![RuntimeValue::Nat(1), RuntimeValue::Nat(2)])
        );
    }

    #[test]
    fn runtime_value_unit() {
        assert_eq!(RuntimeValue::Unit, RuntimeValue::Unit);
    }

    #[test]
    fn runtime_value_clone() {
        let val = RuntimeValue::List(vec![RuntimeValue::Bool(false)]);
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }

    struct TestZero;

    impl Reflect for TestZero {
        type Value = RuntimeValue;
        fn reflect() -> Self::Value {
            RuntimeValue::Nat(0)
        }
    }

    #[test]
    fn reflect_trait_works() {
        assert_eq!(TestZero::reflect(), RuntimeValue::Nat(0));
    }

    #[test]
    fn reify_basic() {
        let result = reify(&42i32, |token| *token.reflect() + 1);
        assert_eq!(result, 43);
    }

    #[test]
    fn reify_string() {
        let s = String::from("hello");
        let len = reify(&s, |token| token.reflect().len());
        assert_eq!(len, 5);
    }

    #[test]
    fn reify_nested() {
        let result = reify(&10i32, |outer| {
            reify(&20i32, |inner| outer.reflect() + inner.reflect())
        });
        assert_eq!(result, 30);
    }

    #[test]
    fn reify_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let sum = reify(&data, |token| token.reflect().iter().sum::<i32>());
        assert_eq!(sum, 15);
    }

    #[test]
    fn reify_with_reflect_trait() {
        let reflected = TestZero::reflect();
        let result = reify(&reflected, |token| match token.reflect() {
            RuntimeValue::Nat(n) => *n,
            _ => panic!("expected Nat"),
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn reify_dyn_trait() {
        // Works with unsized types via &dyn
        trait Greet {
            fn greet(&self) -> &str;
        }
        struct Hello;
        impl Greet for Hello {
            fn greet(&self) -> &str {
                "hi"
            }
        }

        let obj: &dyn Greet = &Hello;
        let result = reify(obj, |token| token.reflect().greet().to_string());
        assert_eq!(result, "hi");
    }

    #[test]
    fn reify_slice() {
        let data = [1, 2, 3];
        let sum = reify(&data[..], |token| token.reflect().iter().sum::<i32>());
        assert_eq!(sum, 6);
    }
}
