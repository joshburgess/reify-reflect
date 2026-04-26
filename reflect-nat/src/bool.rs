//! Type-level booleans with [`Reflect`] implementations.

use reify_reflect_core::Reflect;

/// Type-level `true`.
///
/// # Examples
///
/// ```
/// use reflect_nat::True;
/// use reify_reflect_core::Reflect;
///
/// assert_eq!(True::reflect(), true);
/// ```
pub struct True;

/// Type-level `false`.
///
/// # Examples
///
/// ```
/// use reflect_nat::False;
/// use reify_reflect_core::Reflect;
///
/// assert_eq!(False::reflect(), false);
/// ```
pub struct False;

/// Marker trait for type-level booleans.
///
/// # Examples
///
/// ```
/// use reflect_nat::{True, False, Bool};
///
/// fn require_bool<B: Bool>() -> bool { B::to_bool() }
/// assert!(require_bool::<True>());
/// assert!(!require_bool::<False>());
/// ```
pub trait Bool {
    /// The runtime `bool` value of this type-level boolean.
    fn to_bool() -> bool;
}

impl Bool for True {
    fn to_bool() -> bool {
        true
    }
}

impl Bool for False {
    fn to_bool() -> bool {
        false
    }
}

impl Reflect for True {
    type Value = bool;

    fn reflect() -> Self::Value {
        true
    }
}

impl Reflect for False {
    type Value = bool;

    fn reflect() -> Self::Value {
        false
    }
}

/// Type-level boolean NOT.
///
/// # Examples
///
/// ```
/// use reflect_nat::{True, False, Not};
/// use reify_reflect_core::Reflect;
///
/// assert_eq!(<<True as Not>::Result as Reflect>::reflect(), false);
/// assert_eq!(<<False as Not>::Result as Reflect>::reflect(), true);
/// ```
pub trait Not {
    /// The negated type-level boolean.
    type Result: Bool;
}

impl Not for True {
    type Result = False;
}

impl Not for False {
    type Result = True;
}

/// Type-level boolean AND.
///
/// # Examples
///
/// ```
/// use reflect_nat::{True, False, And, Bool};
///
/// assert!( <<True as And<True>>::Result as Bool>::to_bool());
/// assert!(!<<True as And<False>>::Result as Bool>::to_bool());
/// ```
pub trait And<Rhs> {
    /// The result of `Self && Rhs`.
    type Result: Bool;
}

impl And<True> for True {
    type Result = True;
}
impl And<False> for True {
    type Result = False;
}
impl And<True> for False {
    type Result = False;
}
impl And<False> for False {
    type Result = False;
}

/// Type-level boolean OR.
///
/// # Examples
///
/// ```
/// use reflect_nat::{True, False, Or, Bool};
///
/// assert!( <<True as Or<False>>::Result as Bool>::to_bool());
/// assert!(!<<False as Or<False>>::Result as Bool>::to_bool());
/// ```
pub trait Or<Rhs> {
    /// The result of `Self || Rhs`.
    type Result: Bool;
}

impl Or<True> for True {
    type Result = True;
}
impl Or<False> for True {
    type Result = True;
}
impl Or<True> for False {
    type Result = True;
}
impl Or<False> for False {
    type Result = False;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_reflects() {
        assert!(True::reflect());
    }

    #[test]
    fn false_reflects() {
        assert!(!False::reflect());
    }

    #[test]
    fn bool_to_bool() {
        assert!(True::to_bool());
        assert!(!False::to_bool());
    }

    #[test]
    fn not() {
        assert!(!<<True as Not>::Result as Bool>::to_bool());
        assert!(<<False as Not>::Result as Bool>::to_bool());
    }

    #[test]
    fn and() {
        assert!(<<True as And<True>>::Result as Bool>::to_bool());
        assert!(!<<True as And<False>>::Result as Bool>::to_bool());
        assert!(!<<False as And<True>>::Result as Bool>::to_bool());
        assert!(!<<False as And<False>>::Result as Bool>::to_bool());
    }

    #[test]
    fn or() {
        assert!(<<True as Or<True>>::Result as Bool>::to_bool());
        assert!(<<True as Or<False>>::Result as Bool>::to_bool());
        assert!(<<False as Or<True>>::Result as Bool>::to_bool());
        assert!(!<<False as Or<False>>::Result as Bool>::to_bool());
    }
}
