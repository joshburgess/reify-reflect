//! Type-level heterogeneous lists with [`Reflect`] implementations.

use reflect_core::{Reflect, RuntimeValue};
use std::marker::PhantomData;

/// The empty heterogeneous list.
///
/// # Examples
///
/// ```
/// use reflect_nat::HNil;
/// use reflect_core::Reflect;
///
/// assert_eq!(HNil::reflect(), Vec::<reflect_core::RuntimeValue>::new());
/// ```
pub struct HNil;

/// A cons cell for heterogeneous lists. `HCons<H, T>` prepends head `H` to tail `T`.
///
/// # Examples
///
/// ```
/// use reflect_nat::{HNil, HCons, Z, S};
/// use reflect_core::{Reflect, RuntimeValue};
///
/// // A list containing [1, 0] at the type level
/// type MyList = HCons<S<Z>, HCons<Z, HNil>>;
/// assert_eq!(
///     MyList::reflect(),
///     vec![RuntimeValue::Nat(1), RuntimeValue::Nat(0)]
/// );
/// ```
pub struct HCons<H, T>(PhantomData<(H, T)>);

/// Marker trait for type-level heterogeneous lists.
///
/// # Examples
///
/// ```
/// use reflect_nat::{HNil, HCons, HList, Z, S};
///
/// fn require_hlist<L: HList>() -> usize { L::len() }
/// assert_eq!(require_hlist::<HNil>(), 0);
/// assert_eq!(require_hlist::<HCons<Z, HCons<S<Z>, HNil>>>(), 2);
/// ```
pub trait HList {
    /// The number of elements in this type-level list.
    fn len() -> usize;

    /// Whether this type-level list is empty.
    fn is_empty() -> bool {
        Self::len() == 0
    }
}

impl HList for HNil {
    fn len() -> usize {
        0
    }
}

impl<H, T: HList> HList for HCons<H, T> {
    fn len() -> usize {
        1 + T::len()
    }
}

impl Reflect for HNil {
    type Value = Vec<RuntimeValue>;

    fn reflect() -> Self::Value {
        vec![]
    }
}

impl<H, T> Reflect for HCons<H, T>
where
    H: Reflect<Value = RuntimeValue>,
    T: Reflect<Value = Vec<RuntimeValue>>,
{
    type Value = Vec<RuntimeValue>;

    fn reflect() -> Self::Value {
        let mut list = vec![H::reflect()];
        list.extend(T::reflect());
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{S, Z};

    #[test]
    fn hnil_reflects_to_empty_list() {
        assert_eq!(HNil::reflect(), Vec::<RuntimeValue>::new());
    }

    #[test]
    fn single_element_list() {
        type L = HCons<Z, HNil>;
        assert_eq!(L::reflect(), vec![RuntimeValue::Nat(0)]);
    }

    #[test]
    fn multi_element_list() {
        type L = HCons<S<S<Z>>, HCons<S<Z>, HCons<Z, HNil>>>;
        assert_eq!(
            L::reflect(),
            vec![
                RuntimeValue::Nat(2),
                RuntimeValue::Nat(1),
                RuntimeValue::Nat(0),
            ]
        );
    }

    #[test]
    fn hlist_len() {
        assert_eq!(HNil::len(), 0);
        assert_eq!(<HCons<Z, HNil>>::len(), 1);
        assert_eq!(<HCons<Z, HCons<Z, HCons<Z, HNil>>>>::len(), 3);
    }

    #[test]
    fn hlist_is_empty() {
        assert!(HNil::is_empty());
        assert!(!<HCons<Z, HNil>>::is_empty());
    }

    #[test]
    fn round_trip_type_level_to_runtime() {
        // Construct at type level, reflect, assert runtime values
        type List = HCons<S<S<S<Z>>>, HCons<S<Z>, HCons<Z, HNil>>>;
        let reflected = List::reflect();
        assert_eq!(reflected.len(), 3);
        assert_eq!(reflected[0], RuntimeValue::Nat(3));
        assert_eq!(reflected[1], RuntimeValue::Nat(1));
        assert_eq!(reflected[2], RuntimeValue::Nat(0));
    }
}
