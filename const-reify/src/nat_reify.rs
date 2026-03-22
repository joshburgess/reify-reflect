//! True value→type reification for natural numbers.
//!
//! This module provides [`NatCallback`], [`Nat2Callback`], [`reify_nat`],
//! and [`reify_nat2`] for genuine runtime-to-type-level dispatch: inside
//! the callback, const generics are fully available as type parameters.
//!
//! ## Why a trait, not a closure?
//!
//! With `&dyn HasModulus`, the const generic is erased. By using a trait
//! with a const-generic method, the full `N` is available inside the
//! callback, enabling construction of types like `Mod<N>`, const-generic
//! arithmetic, and type-safe invariants parameterized by the reified value.
//!
//! ## Composing two reified values
//!
//! [`reify_nat2`] nests two dispatches so both `A` and `B` are known
//! const generics inside the callback.
//!
//! # Examples
//!
//! ```
//! use const_reify::nat_reify::{NatCallback, reify_nat};
//!
//! struct ModMul { a: u64, b: u64 }
//!
//! impl NatCallback<u64> for ModMul {
//!     fn call<const N: u64>(&self) -> u64 {
//!         if N == 0 { return 0; }
//!         (self.a % N) * (self.b % N) % N
//!     }
//! }
//!
//! let result = reify_nat(7, &ModMul { a: 10, b: 20 });
//! assert_eq!(result, (10 % 7) * (20 % 7) % 7);
//! ```
//!
//! Two values:
//!
//! ```
//! use const_reify::nat_reify::{Nat2Callback, reify_nat2};
//!
//! struct Add;
//! impl Nat2Callback<u64> for Add {
//!     fn call<const A: u64, const B: u64>(&self) -> u64 { A + B }
//! }
//!
//! assert_eq!(reify_nat2(5, 3, &Add), 8);
//! ```

/// Callback trait for single-value reification.
///
/// Inside [`call`](NatCallback::call), `N` is a fully known const generic.
///
/// # Examples
///
/// ```
/// use const_reify::nat_reify::{NatCallback, reify_nat};
///
/// struct IsEven;
/// impl NatCallback<bool> for IsEven {
///     fn call<const N: u64>(&self) -> bool { N % 2 == 0 }
/// }
///
/// assert_eq!(reify_nat(4, &IsEven), true);
/// assert_eq!(reify_nat(7, &IsEven), false);
/// ```
pub trait NatCallback<R> {
    /// Called with the const-generic `N` matching the runtime value.
    fn call<const N: u64>(&self) -> R;
}

/// Callback trait for two-value reification.
///
/// Both `A` and `B` are known const generics inside [`call`](Nat2Callback::call).
///
/// # Examples
///
/// ```
/// use const_reify::nat_reify::{Nat2Callback, reify_nat2};
///
/// struct Mul;
/// impl Nat2Callback<u64> for Mul {
///     fn call<const A: u64, const B: u64>(&self) -> u64 { A * B }
/// }
///
/// assert_eq!(reify_nat2(6, 7, &Mul), 42);
/// ```
pub trait Nat2Callback<R> {
    /// Called with both const-generic values.
    fn call<const A: u64, const B: u64>(&self) -> R;
}

/// Reify a runtime `u64` (0..=255) into a const-generic context.
///
/// # Panics
///
/// Panics if `val > 255`.
///
/// # Examples
///
/// ```
/// use const_reify::nat_reify::{NatCallback, reify_nat};
///
/// struct Square;
/// impl NatCallback<u64> for Square {
///     fn call<const N: u64>(&self) -> u64 { N * N }
/// }
///
/// assert_eq!(reify_nat(12, &Square), 144);
/// ```
pub fn reify_nat<C: NatCallback<R>, R>(val: u64, callback: &C) -> R {
    macro_rules! dispatch {
        ($($n:literal),*) => {
            match val {
                $( $n => callback.call::<$n>(), )*
                other => panic!(
                    "const-reify: value {} is out of supported range 0..=255", other
                ),
            }
        };
    }

    dispatch!(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
        131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166,
        167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184,
        185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202,
        203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
        221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238,
        239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255
    )
}

/// Reify two runtime `u64` values (each 0..=255) into a const-generic context.
///
/// Nested dispatch: both `A` and `B` are known const generics in the callback.
///
/// # Panics
///
/// Panics if either value > 255.
///
/// # Examples
///
/// ```
/// use const_reify::nat_reify::{Nat2Callback, reify_nat2};
///
/// struct Lt;
/// impl Nat2Callback<bool> for Lt {
///     fn call<const A: u64, const B: u64>(&self) -> bool { A < B }
/// }
///
/// assert_eq!(reify_nat2(3, 5, &Lt), true);
/// assert_eq!(reify_nat2(5, 3, &Lt), false);
/// ```
pub fn reify_nat2<C: Nat2Callback<R>, R>(a: u64, b: u64, callback: &C) -> R {
    struct Outer<'a, C, R> {
        b: u64,
        inner: &'a C,
        _r: std::marker::PhantomData<R>,
    }

    impl<C: Nat2Callback<R>, R> NatCallback<R> for Outer<'_, C, R> {
        fn call<const A: u64>(&self) -> R {
            struct Inner<'a, const A: u64, C, R> {
                inner: &'a C,
                _r: std::marker::PhantomData<R>,
            }

            impl<const A: u64, C: Nat2Callback<R>, R> NatCallback<R> for Inner<'_, A, C, R> {
                fn call<const B: u64>(&self) -> R {
                    self.inner.call::<A, B>()
                }
            }

            reify_nat(
                self.b,
                &Inner::<A, C, R> {
                    inner: self.inner,
                    _r: std::marker::PhantomData,
                },
            )
        }
    }

    reify_nat(
        a,
        &Outer {
            b,
            inner: callback,
            _r: std::marker::PhantomData,
        },
    )
}

// ---------------------------------------------------------------------------
// Closure-based ergonomic wrappers
// ---------------------------------------------------------------------------

/// Wrapper that adapts a `Fn(u64) -> R` closure into a [`NatCallback`].
///
/// Inside the dispatch, the const generic `N` is passed to the closure as
/// a plain `u64`. This loses the ability to construct types parameterized
/// by `N`, but covers the common case where you just need the value.
///
/// Prefer [`reify_nat_fn`] which uses this internally.
pub struct FnNat<F>(pub F);

impl<F: Fn(u64) -> R, R> NatCallback<R> for FnNat<F> {
    fn call<const N: u64>(&self) -> R {
        (self.0)(N)
    }
}

/// Wrapper that adapts a `Fn(u64, u64) -> R` closure into a [`Nat2Callback`].
///
/// Prefer [`reify_nat2_fn`] which uses this internally.
pub struct FnNat2<F>(pub F);

impl<F: Fn(u64, u64) -> R, R> Nat2Callback<R> for FnNat2<F> {
    fn call<const A: u64, const B: u64>(&self) -> R {
        (self.0)(A, B)
    }
}

/// Ergonomic single-value reification with a closure.
///
/// The closure receives the reified value as a plain `u64`. For cases
/// where you need the actual const generic (e.g., to construct `Mod<N>`),
/// use [`reify_nat`] with a [`NatCallback`] impl instead.
///
/// # Examples
///
/// ```
/// use const_reify::nat_reify::reify_nat_fn;
///
/// assert_eq!(reify_nat_fn(5, |n| n * n), 25);
/// assert_eq!(reify_nat_fn(10, |n| n + 1), 11);
/// assert_eq!(reify_nat_fn(0, |n| n == 0), true);
/// ```
pub fn reify_nat_fn<F: Fn(u64) -> R, R>(val: u64, f: F) -> R {
    reify_nat(val, &FnNat(f))
}

/// Ergonomic two-value reification with a closure.
///
/// The closure receives both reified values as plain `u64`s.
///
/// # Examples
///
/// ```
/// use const_reify::nat_reify::reify_nat2_fn;
///
/// assert_eq!(reify_nat2_fn(5, 3, |a, b| a + b), 8);
/// assert_eq!(reify_nat2_fn(6, 7, |a, b| a * b), 42);
/// assert_eq!(reify_nat2_fn(3, 5, |a, b| a < b), true);
/// ```
pub fn reify_nat2_fn<F: Fn(u64, u64) -> R, R>(a: u64, b: u64, f: F) -> R {
    reify_nat2(a, b, &FnNat2(f))
}

// ---------------------------------------------------------------------------
// Macros for defining callbacks and inline reification
// ---------------------------------------------------------------------------

/// Define a [`NatCallback`] struct with minimal boilerplate.
///
/// Two forms:
///
/// **Stateless** — no captured data, just a const-generic body:
/// ```
/// use const_reify::{def_nat_callback, nat_reify::reify_nat};
///
/// def_nat_callback!(Square -> u64 { N * N });
///
/// assert_eq!(reify_nat(5, &Square), 25);
/// ```
///
/// **With fields** — captures runtime data alongside the const generic:
/// ```
/// use const_reify::{def_nat_callback, nat_reify::reify_nat};
///
/// def_nat_callback!(ModMul { a: u64, b: u64 } -> u64 {
///     |s| if N == 0 { 0 } else { (s.a % N) * (s.b % N) % N }
/// });
///
/// assert_eq!(reify_nat(7, &ModMul { a: 10, b: 20 }), 4);
/// ```
///
/// In both forms, `N` refers to the const-generic `u64` parameter.
#[macro_export]
macro_rules! def_nat_callback {
    // Stateless: def_nat_callback!(Name -> RetType { body using N })
    ($name:ident -> $ret:ty $body:block) => {
        struct $name;

        impl $crate::nat_reify::NatCallback<$ret> for $name {
            #[allow(non_snake_case)]
            fn call<const N: u64>(&self) -> $ret $body
        }
    };

    // With fields: def_nat_callback!(Name { field: Type, ... } -> RetType { |s| body using N, s })
    ($name:ident { $($field:ident : $fty:ty),* $(,)? } -> $ret:ty { |$s:ident| $($body:tt)* }) => {
        struct $name {
            $( $field: $fty, )*
        }

        impl $crate::nat_reify::NatCallback<$ret> for $name {
            #[allow(non_snake_case)]
            fn call<const N: u64>(&self) -> $ret {
                let $s = self;
                $($body)*
            }
        }
    };
}

/// Define a [`Nat2Callback`] struct with minimal boilerplate.
///
/// Two forms:
///
/// **Stateless:**
/// ```
/// use const_reify::{def_nat2_callback, nat_reify::reify_nat2};
///
/// def_nat2_callback!(Add -> u64 { A + B });
///
/// assert_eq!(reify_nat2(5, 3, &Add), 8);
/// ```
///
/// **With fields:**
/// ```
/// use const_reify::{def_nat2_callback, nat_reify::reify_nat2};
///
/// def_nat2_callback!(ScaledSum { scale: u64 } -> u64 { |s| (A + B) * s.scale });
///
/// assert_eq!(reify_nat2(5, 3, &ScaledSum { scale: 10 }), 80);
/// ```
///
/// `A` and `B` refer to the two const-generic `u64` parameters.
#[macro_export]
macro_rules! def_nat2_callback {
    ($name:ident -> $ret:ty $body:block) => {
        struct $name;

        impl $crate::nat_reify::Nat2Callback<$ret> for $name {
            #[allow(non_snake_case)]
            fn call<const A: u64, const B: u64>(&self) -> $ret $body
        }
    };

    ($name:ident { $($field:ident : $fty:ty),* $(,)? } -> $ret:ty { |$s:ident| $($body:tt)* }) => {
        struct $name {
            $( $field: $fty, )*
        }

        impl $crate::nat_reify::Nat2Callback<$ret> for $name {
            #[allow(non_snake_case)]
            fn call<const A: u64, const B: u64>(&self) -> $ret {
                let $s = self;
                $($body)*
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Identity;
    impl NatCallback<u64> for Identity {
        fn call<const N: u64>(&self) -> u64 {
            N
        }
    }

    #[test]
    fn reify_nat_identity() {
        for v in 0..=255u64 {
            assert_eq!(reify_nat(v, &Identity), v);
        }
    }

    struct Square;
    impl NatCallback<u64> for Square {
        fn call<const N: u64>(&self) -> u64 {
            N * N
        }
    }

    #[test]
    fn reify_nat_square() {
        assert_eq!(reify_nat(0, &Square), 0);
        assert_eq!(reify_nat(5, &Square), 25);
        assert_eq!(reify_nat(12, &Square), 144);
    }

    struct IsEven;
    impl NatCallback<bool> for IsEven {
        fn call<const N: u64>(&self) -> bool {
            N % 2 == 0
        }
    }

    #[test]
    fn reify_nat_predicate() {
        assert!(reify_nat(0, &IsEven));
        assert!(!reify_nat(1, &IsEven));
        assert!(reify_nat(42, &IsEven));
    }

    #[test]
    #[should_panic(expected = "out of supported range")]
    fn reify_nat_out_of_range() {
        reify_nat(256, &Identity);
    }

    // --- Two-value tests ---

    struct Add2;
    impl Nat2Callback<u64> for Add2 {
        fn call<const A: u64, const B: u64>(&self) -> u64 {
            A + B
        }
    }

    #[test]
    fn reify_nat2_add() {
        assert_eq!(reify_nat2(5, 3, &Add2), 8);
        assert_eq!(reify_nat2(0, 0, &Add2), 0);
        assert_eq!(reify_nat2(100, 155, &Add2), 255);
    }

    struct Mul2;
    impl Nat2Callback<u64> for Mul2 {
        fn call<const A: u64, const B: u64>(&self) -> u64 {
            A * B
        }
    }

    #[test]
    fn reify_nat2_mul() {
        assert_eq!(reify_nat2(6, 7, &Mul2), 42);
        assert_eq!(reify_nat2(0, 255, &Mul2), 0);
    }

    struct Lt2;
    impl Nat2Callback<bool> for Lt2 {
        fn call<const A: u64, const B: u64>(&self) -> bool {
            A < B
        }
    }

    #[test]
    fn reify_nat2_lt() {
        assert!(reify_nat2(3, 5, &Lt2));
        assert!(!reify_nat2(5, 3, &Lt2));
        assert!(!reify_nat2(5, 5, &Lt2));
    }

    // --- The real power: type-safe modular arithmetic with runtime modulus ---

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Mod<const M: u64> {
        value: u64,
    }

    impl<const M: u64> Mod<M> {
        fn new(v: u64) -> Self {
            Mod {
                value: if M == 0 { 0 } else { v % M },
            }
        }

        fn mul(self, other: Self) -> Self {
            Self::new(self.value * other.value)
        }

        fn pow(self, exp: u64) -> Self {
            let mut result = Self::new(1);
            let mut base = self;
            let mut e = exp;
            while e > 0 {
                if e % 2 == 1 {
                    result = result.mul(base);
                }
                base = base.mul(base);
                e /= 2;
            }
            result
        }
    }

    struct ModPow {
        base: u64,
        exp: u64,
    }

    impl NatCallback<u64> for ModPow {
        fn call<const M: u64>(&self) -> u64 {
            // M is a const generic — we can construct Mod<M> here!
            // The type system ensures all arithmetic stays in the same modulus.
            let b = Mod::<M>::new(self.base);
            b.pow(self.exp).value
        }
    }

    #[test]
    fn modular_exponentiation_with_runtime_modulus() {
        // 3^5 mod 7 = 243 mod 7 = 5
        assert_eq!(reify_nat(7, &ModPow { base: 3, exp: 5 }), 243 % 7);

        // Fermat's little theorem: a^(p-1) ≡ 1 (mod p) for prime p, gcd(a,p)=1
        assert_eq!(reify_nat(7, &ModPow { base: 3, exp: 6 }), 1);
        assert_eq!(reify_nat(11, &ModPow { base: 2, exp: 10 }), 1);
        assert_eq!(reify_nat(13, &ModPow { base: 5, exp: 12 }), 1);
    }

    // --- Closure-based ergonomic API ---

    #[test]
    fn reify_nat_fn_basic() {
        assert_eq!(reify_nat_fn(5, |n| n * n), 25);
        assert_eq!(reify_nat_fn(0, |n| n == 0), true);
        assert_eq!(reify_nat_fn(255, |n| n), 255);
    }

    #[test]
    fn reify_nat_fn_captures_environment() {
        let offset = 100u64;
        assert_eq!(reify_nat_fn(5, |n| n + offset), 105);
    }

    #[test]
    fn reify_nat2_fn_basic() {
        assert_eq!(reify_nat2_fn(5, 3, |a, b| a + b), 8);
        assert_eq!(reify_nat2_fn(6, 7, |a, b| a * b), 42);
        assert_eq!(reify_nat2_fn(3, 5, |a, b| a < b), true);
    }

    #[test]
    fn reify_nat2_fn_captures_environment() {
        let scale = 10u64;
        assert_eq!(reify_nat2_fn(5, 3, |a, b| (a + b) * scale), 80);
    }

    // --- Macro-defined callbacks ---

    def_nat_callback!(Cube -> u64 { N * N * N });

    #[test]
    fn macro_stateless_callback() {
        assert_eq!(reify_nat(3, &Cube), 27);
        assert_eq!(reify_nat(5, &Cube), 125);
    }

    def_nat_callback!(AddOffset { offset: u64 } -> u64 { |s| N + s.offset });

    #[test]
    fn macro_callback_with_fields() {
        assert_eq!(reify_nat(10, &AddOffset { offset: 5 }), 15);
    }

    def_nat2_callback!(Hypotenuse2 -> u64 { A * A + B * B });

    #[test]
    fn macro_nat2_stateless() {
        assert_eq!(reify_nat2(3, 4, &Hypotenuse2), 25); // 3² + 4² = 25
    }

    def_nat2_callback!(ScaledDiff { scale: u64 } -> u64 {
        |s| if A > B { (A - B) * s.scale } else { (B - A) * s.scale }
    });

    #[test]
    fn macro_nat2_with_fields() {
        assert_eq!(reify_nat2(10, 3, &ScaledDiff { scale: 5 }), 35);
    }
}
