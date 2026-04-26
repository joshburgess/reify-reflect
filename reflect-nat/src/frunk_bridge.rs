//! Bridge between [`frunk`](https://docs.rs/frunk) heterogeneous lists
//! and [`reify_reflect_core::RuntimeValue`].
//!
//! Available when the `frunk` feature is enabled.
//!
//! `frunk::HCons<H, T>` is value-bearing (it stores an actual `H`), while
//! [`crate::HCons<H, T>`] is purely type-level. The bridge therefore
//! exposes a [`FrunkReflect`] trait that walks a value-bearing
//! `frunk::HList` and reflects each element type via
//! [`reify_reflect_core::Reflect`].
//!
//! # Examples
//!
//! ```
//! use frunk::hlist;
//! use reflect_nat::frunk_bridge::FrunkReflect;
//! use reflect_nat::{S, Z};
//! use reify_reflect_core::RuntimeValue;
//!
//! // A frunk HList holding type-level naturals (zero-sized markers).
//! let list = hlist![Z, S::<Z>(Default::default())];
//! let reflected = list.reflect_runtime();
//! assert_eq!(reflected, vec![RuntimeValue::Nat(0), RuntimeValue::Nat(1)]);
//! ```

use reify_reflect_core::{Reflect, RuntimeValue};

/// Walk a value-bearing `frunk::HList` and reflect each element type
/// via [`reify_reflect_core::Reflect`].
///
/// Each element type `H` of the list must implement
/// `Reflect<Value = RuntimeValue>`. The element values themselves are
/// not used: only their types drive reflection.
pub trait FrunkReflect {
    /// Reflect every element of this `frunk::HList` to a single
    /// `Vec<RuntimeValue>`.
    fn reflect_runtime(&self) -> Vec<RuntimeValue>;
}

impl FrunkReflect for frunk::HNil {
    fn reflect_runtime(&self) -> Vec<RuntimeValue> {
        Vec::new()
    }
}

impl<H, T> FrunkReflect for frunk::HCons<H, T>
where
    H: Reflect<Value = RuntimeValue>,
    T: FrunkReflect,
{
    fn reflect_runtime(&self) -> Vec<RuntimeValue> {
        let mut out = Vec::with_capacity(1 + self.tail.reflect_runtime().len());
        out.push(H::reflect());
        out.extend(self.tail.reflect_runtime());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{S, Z};
    use frunk::hlist;
    use std::marker::PhantomData;

    #[test]
    fn empty_frunk_list() {
        let list = frunk::HNil;
        assert_eq!(list.reflect_runtime(), Vec::<RuntimeValue>::new());
    }

    #[test]
    fn frunk_list_of_our_nats() {
        let list = hlist![Z, S::<Z>(PhantomData), S::<S<Z>>(PhantomData)];
        assert_eq!(
            list.reflect_runtime(),
            vec![
                RuntimeValue::Nat(0),
                RuntimeValue::Nat(1),
                RuntimeValue::Nat(2),
            ]
        );
    }
}
