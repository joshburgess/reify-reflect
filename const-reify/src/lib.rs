#![deny(unsafe_code)]

//! # const-reify
//!
//! Runtime-to-const-generic bridge via match-table dispatch.
//!
//! Given a runtime `u64` value in `0..=255`, dispatches to the corresponding
//! monomorphization of a const-generic type, enabling type-level programming
//! with runtime-determined values.
//!
//! See [`DESIGN.md`](https://github.com/joshburgess/reflect-rs/blob/main/const-reify/DESIGN.md)
//! for the design rationale and safety analysis.
//!
//! # Examples
//!
//! ```
//! use const_reify::{reify_const, HasModulus};
//!
//! let result = reify_const(17, |m| m.modulus());
//! assert_eq!(result, 17);
//! ```
//!
//! Using the [`reify!`] macro:
//!
//! ```
//! use const_reify::{reify, HasModulus};
//!
//! reify!(42u64, |m: &dyn HasModulus| {
//!     assert_eq!(m.modulus(), 42);
//! });
//! ```

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
