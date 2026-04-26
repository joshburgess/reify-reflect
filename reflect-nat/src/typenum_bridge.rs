//! Bridge between [`typenum`](https://docs.rs/typenum) type-level naturals
//! and [`reify_reflect_core::RuntimeValue`].
//!
//! Available when the `typenum` feature is enabled. The bridge is provided
//! as helper functions and an extension trait (rather than `From` impls)
//! because both [`reify_reflect_core::RuntimeValue`] and the `typenum::Uxxx`
//! types are foreign to this crate, which would make a direct `From`
//! impl violate the orphan rule.
//!
//! # Examples
//!
//! ```
//! use reflect_nat::typenum_bridge::{reflect_unsigned, reflect_bit};
//! use reify_reflect_core::RuntimeValue;
//! use typenum::{U0, U7, B0, B1};
//!
//! assert_eq!(reflect_unsigned::<U0>(), RuntimeValue::Nat(0));
//! assert_eq!(reflect_unsigned::<U7>(), RuntimeValue::Nat(7));
//! assert_eq!(reflect_bit::<B0>(), false);
//! assert_eq!(reflect_bit::<B1>(), true);
//! ```

use reify_reflect_core::RuntimeValue;
use typenum::{Bit, Unsigned};

/// Reflect a [`typenum::Unsigned`] type-level natural to a
/// [`RuntimeValue::Nat`].
///
/// # Examples
///
/// ```
/// use reflect_nat::typenum_bridge::reflect_unsigned;
/// use reify_reflect_core::RuntimeValue;
/// use typenum::U42;
///
/// assert_eq!(reflect_unsigned::<U42>(), RuntimeValue::Nat(42));
/// ```
pub fn reflect_unsigned<T: Unsigned>() -> RuntimeValue {
    RuntimeValue::Nat(T::U64)
}

/// Reflect a [`typenum::Bit`] (`B0` or `B1`) to a runtime `bool`.
///
/// # Examples
///
/// ```
/// use reflect_nat::typenum_bridge::reflect_bit;
/// use typenum::{B0, B1};
///
/// assert!(!reflect_bit::<B0>());
/// assert!( reflect_bit::<B1>());
/// ```
pub fn reflect_bit<T: Bit>() -> bool {
    T::to_bool()
}

/// Extension trait giving every [`typenum::Unsigned`] a `.reflect()`
/// method that returns a [`RuntimeValue::Nat`]. Implemented blanket-style
/// over all `T: Unsigned`.
///
/// # Examples
///
/// ```
/// use reflect_nat::typenum_bridge::TypenumReflect;
/// use reify_reflect_core::RuntimeValue;
/// use typenum::U5;
///
/// assert_eq!(U5::reflect_runtime(), RuntimeValue::Nat(5));
/// ```
pub trait TypenumReflect {
    /// Reflect this type-level natural to a [`RuntimeValue::Nat`].
    fn reflect_runtime() -> RuntimeValue;
}

impl<T: Unsigned> TypenumReflect for T {
    fn reflect_runtime() -> RuntimeValue {
        reflect_unsigned::<T>()
    }
}

/// Convert one of our [`crate::Nat`] type-level naturals to its runtime
/// `u64` value (the input shape that `typenum`'s `U*` types accept via
/// constants like `T::U64`).
///
/// This is the dual of [`reflect_unsigned`]: it goes our-encoding → runtime;
/// from there callers can pick a `typenum::U*` based on the value if needed.
///
/// # Examples
///
/// ```
/// use reflect_nat::{N3, typenum_bridge::nat_to_u64};
///
/// assert_eq!(nat_to_u64::<N3>(), 3);
/// ```
pub fn nat_to_u64<N: crate::Nat>() -> u64 {
    N::to_u64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::{B0, B1, U0, U1, U255, U8};

    #[test]
    fn unsigned_reflects_to_runtime_value() {
        assert_eq!(reflect_unsigned::<U0>(), RuntimeValue::Nat(0));
        assert_eq!(reflect_unsigned::<U1>(), RuntimeValue::Nat(1));
        assert_eq!(reflect_unsigned::<U8>(), RuntimeValue::Nat(8));
        assert_eq!(reflect_unsigned::<U255>(), RuntimeValue::Nat(255));
    }

    #[test]
    fn bits_reflect() {
        assert!(!reflect_bit::<B0>());
        assert!(reflect_bit::<B1>());
    }

    #[test]
    fn extension_trait_works() {
        assert_eq!(U8::reflect_runtime(), RuntimeValue::Nat(8));
    }

    #[test]
    fn our_nat_to_u64_matches_typenum() {
        use crate::{N0, N5, N8};
        assert_eq!(nat_to_u64::<N0>(), U0::U64);
        assert_eq!(nat_to_u64::<N5>(), 5);
        assert_eq!(nat_to_u64::<N8>(), U8::U64);
    }
}
