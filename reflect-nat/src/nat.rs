//! Peano-encoded type-level natural numbers with type-level arithmetic.

use reflect_core::{Reflect, RuntimeValue};
use std::marker::PhantomData;

/// Type-level zero.
///
/// # Examples
///
/// ```
/// use reflect_nat::Z;
/// use reflect_core::{Reflect, RuntimeValue};
///
/// assert_eq!(Z::reflect(), RuntimeValue::Nat(0));
/// ```
pub struct Z;

/// Type-level successor. `S<N>` represents `N + 1`.
///
/// # Examples
///
/// ```
/// use reflect_nat::{Z, S};
/// use reflect_core::{Reflect, RuntimeValue};
///
/// type One = S<Z>;
/// type Two = S<S<Z>>;
/// assert_eq!(One::reflect(), RuntimeValue::Nat(1));
/// assert_eq!(Two::reflect(), RuntimeValue::Nat(2));
/// ```
pub struct S<N>(PhantomData<N>);

/// Marker trait for types that represent type-level natural numbers.
///
/// # Examples
///
/// ```
/// use reflect_nat::{Z, S, Nat};
///
/// fn require_nat<N: Nat>() {}
/// require_nat::<Z>();
/// require_nat::<S<Z>>();
/// ```
pub trait Nat {
    /// The runtime `u64` value of this type-level natural.
    fn to_u64() -> u64;
}

impl Nat for Z {
    fn to_u64() -> u64 {
        0
    }
}

impl<N: Nat> Nat for S<N> {
    fn to_u64() -> u64 {
        1 + N::to_u64()
    }
}

impl Reflect for Z {
    type Value = RuntimeValue;

    fn reflect() -> Self::Value {
        RuntimeValue::Nat(0)
    }
}

impl<N: Nat> Reflect for S<N> {
    type Value = RuntimeValue;

    fn reflect() -> Self::Value {
        RuntimeValue::Nat(<S<N> as Nat>::to_u64())
    }
}

// ---------------------------------------------------------------------------
// Type-level arithmetic
// ---------------------------------------------------------------------------

/// Type-level addition. `Add<A, B>` computes `A + B`.
///
/// # Examples
///
/// ```
/// use reflect_nat::{Z, S, Add};
/// use reflect_core::{Reflect, RuntimeValue};
///
/// // 2 + 3 = 5
/// type Two = S<S<Z>>;
/// type Three = S<S<S<Z>>>;
/// type Five = <Two as Add<Three>>::Result;
/// assert_eq!(Five::reflect(), RuntimeValue::Nat(5));
/// ```
pub trait Add<Rhs> {
    /// The resulting type-level natural.
    type Result: Nat;
}

// Z + N = N
impl<N: Nat> Add<N> for Z {
    type Result = N;
}

// S<M> + N = S<M + N>
impl<M: Nat + Add<N>, N: Nat> Add<N> for S<M>
where
    <M as Add<N>>::Result: Nat,
{
    type Result = S<<M as Add<N>>::Result>;
}

/// Type-level multiplication. `Mul<A, B>` computes `A * B`.
///
/// # Examples
///
/// ```
/// use reflect_nat::{Z, S, Mul};
/// use reflect_core::{Reflect, RuntimeValue};
///
/// // 2 * 3 = 6
/// type Two = S<S<Z>>;
/// type Three = S<S<S<Z>>>;
/// type Six = <Two as Mul<Three>>::Result;
/// assert_eq!(Six::reflect(), RuntimeValue::Nat(6));
/// ```
pub trait Mul<Rhs> {
    /// The resulting type-level natural.
    type Result: Nat;
}

// Z * N = Z
impl<N: Nat> Mul<N> for Z {
    type Result = Z;
}

// S<M> * N = N + (M * N)
impl<M, N> Mul<N> for S<M>
where
    M: Nat + Mul<N>,
    N: Nat + Add<<M as Mul<N>>::Result>,
    <M as Mul<N>>::Result: Nat,
    <N as Add<<M as Mul<N>>::Result>>::Result: Nat,
{
    type Result = <N as Add<<M as Mul<N>>::Result>>::Result;
}

/// Type-level less-than comparison. `Lt<A, B>` is true when `A < B`.
///
/// # Examples
///
/// ```
/// use reflect_nat::{Z, S, Lt};
///
/// // 0 < 1 is true
/// assert!(<Z as Lt<S<Z>>>::VALUE);
///
/// // 2 < 1 is false
/// type Two = S<S<Z>>;
/// assert!(!<Two as Lt<S<Z>>>::VALUE);
/// ```
pub trait Lt<Rhs> {
    /// `true` if `Self < Rhs` at the type level.
    const VALUE: bool;
}

// Z < Z = false
impl Lt<Z> for Z {
    const VALUE: bool = false;
}

// Z < S<N> = true
impl<N: Nat> Lt<S<N>> for Z {
    const VALUE: bool = true;
}

// S<M> < Z = false
impl<M: Nat> Lt<Z> for S<M> {
    const VALUE: bool = false;
}

// S<M> < S<N> = M < N
impl<M: Nat + Lt<N>, N: Nat> Lt<S<N>> for S<M> {
    const VALUE: bool = <M as Lt<N>>::VALUE;
}

// ---------------------------------------------------------------------------
// Convenience type aliases
// ---------------------------------------------------------------------------

/// Type alias for 0.
pub type N0 = Z;
/// Type alias for 1.
pub type N1 = S<N0>;
/// Type alias for 2.
pub type N2 = S<N1>;
/// Type alias for 3.
pub type N3 = S<N2>;
/// Type alias for 4.
pub type N4 = S<N3>;
/// Type alias for 5.
pub type N5 = S<N4>;
/// Type alias for 6.
pub type N6 = S<N5>;
/// Type alias for 7.
pub type N7 = S<N6>;
/// Type alias for 8.
pub type N8 = S<N7>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_reflects_to_nat_0() {
        assert_eq!(Z::reflect(), RuntimeValue::Nat(0));
    }

    #[test]
    fn successor_reflects_correctly() {
        assert_eq!(S::<Z>::reflect(), RuntimeValue::Nat(1));
        assert_eq!(<S<S<Z>>>::reflect(), RuntimeValue::Nat(2));
        assert_eq!(<S<S<S<Z>>>>::reflect(), RuntimeValue::Nat(3));
    }

    #[test]
    fn nat_to_u64() {
        assert_eq!(Z::to_u64(), 0);
        assert_eq!(<S<Z>>::to_u64(), 1);
        assert_eq!(<S<S<S<S<S<Z>>>>>>::to_u64(), 5);
    }

    #[test]
    fn type_aliases() {
        assert_eq!(N0::to_u64(), 0);
        assert_eq!(N1::to_u64(), 1);
        assert_eq!(N5::to_u64(), 5);
        assert_eq!(N8::to_u64(), 8);
    }

    #[test]
    fn addition() {
        // 0 + 3 = 3
        assert_eq!(<<Z as Add<N3>>::Result as Nat>::to_u64(), 3);
        // 2 + 3 = 5
        assert_eq!(<<N2 as Add<N3>>::Result as Nat>::to_u64(), 5);
        // 1 + 0 = 1
        assert_eq!(<<N1 as Add<N0>>::Result as Nat>::to_u64(), 1);
    }

    #[test]
    fn multiplication() {
        // 0 * 3 = 0
        assert_eq!(<<Z as Mul<N3>>::Result as Nat>::to_u64(), 0);
        // 2 * 3 = 6
        assert_eq!(<<N2 as Mul<N3>>::Result as Nat>::to_u64(), 6);
        // 1 * 5 = 5
        assert_eq!(<<N1 as Mul<N5>>::Result as Nat>::to_u64(), 5);
        // 3 * 1 = 3
        assert_eq!(<<N3 as Mul<N1>>::Result as Nat>::to_u64(), 3);
    }

    #[test]
    fn less_than() {
        assert!(!<Z as Lt<Z>>::VALUE);
        assert!(<Z as Lt<S<Z>>>::VALUE);
        assert!(!<S<Z> as Lt<Z>>::VALUE);
        assert!(<N2 as Lt<N5>>::VALUE);
        assert!(!<N5 as Lt<N2>>::VALUE);
        assert!(!<N3 as Lt<N3>>::VALUE);
    }

    #[test]
    fn reflect_returns_runtime_value() {
        assert_eq!(N5::reflect(), RuntimeValue::Nat(5));
        assert_eq!(N0::reflect(), RuntimeValue::Nat(0));
    }
}
