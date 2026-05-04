#![deny(unsafe_code)]

//! Use a runtime `u64` as a `const N: u64`, safely.
//!
//! Const generics in Rust have to be known at compile time. That's
//! frustrating when the value you actually want to parameterize on
//! (a modulus, a buffer size, a feature flag) only becomes known at
//! runtime. The orthodox workarounds are to drop the const generic
//! (and lose the type safety), or to write a giant `match` by hand.
//!
//! This crate is the giant `match`, generated for you, with three
//! progressively more powerful APIs on top.
//!
//! Everything is `#![deny(unsafe_code)]`. There is no vtable
//! fabrication, no `transmute`, no UB. The dispatch is a flat 256-arm
//! `match` that the compiler optimizes well. The tradeoff: the runtime
//! value must lie in `0..=255` per dispatch (and the trait this crate
//! ships with is just one example; you can build your own with a wider
//! range). For the full safety analysis and why we chose this over the
//! original "fabricate a vtable" approach, see [`DESIGN.md`][design].
//!
//! [design]: https://github.com/joshburgess/reify-reflect/blob/main/const-reify/DESIGN.md
//!
//! # Three APIs, in order of power
//!
//! ## 1. [`reify_const`] / [`reify!`]
//!
//! Smallest surface area. You get a `&dyn HasModulus` whose
//! [`modulus()`](HasModulus::modulus) returns your runtime value. The
//! const generic is "real" inside the dispatch (each arm calls
//! `Modular::<N>::new()`), but you can only see it through the
//! [`HasModulus`] trait. Useful for testing the wiring.
//!
//! ```
//! use const_reify::{reify_const, HasModulus};
//!
//! let result = reify_const(17, |m| m.modulus());
//! assert_eq!(result, 17);
//! ```
//!
//! ## 2. [`reify_nat_fn`] / [`reify_nat2_fn`]
//!
//! When you only need the runtime value as a plain `u64` inside the
//! callback (no const generic gymnastics needed). The closure form is
//! the easiest entry point and is enough for most ad-hoc uses.
//!
//! ```
//! use const_reify::reify_nat_fn;
//!
//! let squared = reify_nat_fn(12, |n| n * n);
//! assert_eq!(squared, 144);
//! ```
//!
//! ## 3. [`NatCallback`] / [`reify_nat`]
//!
//! The full power form. Implement [`NatCallback`] on a type, and inside
//! [`call::<const N: u64>()`](NatCallback::call) the value `N` is a
//! genuine const generic that you can use in `const N: u64` positions.
//!
//! ```
//! use const_reify::nat_reify::{NatCallback, reify_nat};
//!
//! struct Square;
//! impl NatCallback<u64> for Square {
//!     fn call<const N: u64>(&self) -> u64 { N * N }
//! }
//!
//! assert_eq!(reify_nat(7, &Square), 49);
//! ```
//!
//! For traits with multiple const-generic methods, the
//! [`#[reifiable]`](https://docs.rs/const-reify-derive) proc macro
//! generates the `NatCallback` plumbing automatically. See
//! [Guide 4][guide4].
//!
//! [guide4]: https://github.com/joshburgess/reify-reflect/blob/main/docs/guides/04-reifiable-macro.md
//!
//! # See also
//!
//! - [Guide 3: const-reify][guide3] for a tutorial-paced walkthrough
//!   covering why the range is 256 and why this is fully safe despite
//!   the "vtable fabrication" reputation of the underlying technique.
//!
//! [guide3]: https://github.com/joshburgess/reify-reflect/blob/main/docs/guides/03-const-reify.md

mod dispatch;
pub mod nat_reify;

pub use dispatch::{reify_const, HasModulus, Modular};
pub use nat_reify::{
    reify_nat, reify_nat2, reify_nat2_fn, reify_nat_fn, FnNat, FnNat2, Nat2Callback, NatCallback,
};

/// Maximum supported value for [`reify_const`] dispatch.
pub const MAX_REIFY_VALUE: u64 = 255;

/// Convenience macro for runtime-to-const-generic dispatch.
///
/// # Examples
///
/// ```
/// use const_reify::{reify, HasModulus};
///
/// reify!(10u64, |m: &dyn HasModulus| {
///     assert_eq!(m.modulus(), 10);
/// });
/// ```
///
/// # Panics
///
/// Panics if the value exceeds [`MAX_REIFY_VALUE`] (255).
#[macro_export]
macro_rules! reify {
    ($val:expr, $f:expr) => {{
        $crate::reify_const($val, $f)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reify_zero() {
        let result = reify_const(0, |m| m.modulus());
        assert_eq!(result, 0);
    }

    #[test]
    fn reify_max() {
        let result = reify_const(255, |m| m.modulus());
        assert_eq!(result, 255);
    }

    #[test]
    fn reify_arbitrary() {
        for v in [1, 17, 42, 100, 200, 255] {
            let result = reify_const(v, |m| m.modulus());
            assert_eq!(result, v);
        }
    }

    #[test]
    #[should_panic(expected = "out of supported range")]
    fn reify_out_of_range() {
        reify_const(256, |m| m.modulus());
    }

    #[test]
    fn reify_macro() {
        reify!(42u64, |m: &dyn HasModulus| { assert_eq!(m.modulus(), 42) });
    }

    #[test]
    fn reify_returns_value() {
        let doubled = reify_const(21, |m| m.modulus() * 2);
        assert_eq!(doubled, 42);
    }

    #[test]
    fn reify_all_values() {
        for v in 0..=255u64 {
            assert_eq!(reify_const(v, |m| m.modulus()), v);
        }
    }
}
