//! The core [`WithContext`] wrapper type.

/// Pairs a value with a context that supplies trait implementations.
///
/// By wrapping a value in `WithContext`, you can provide custom
/// implementations of traits like `Ord`, `Hash`, or `Display` without
/// modifying the original type.
///
/// # Examples
///
/// ```
/// use context_trait::{WithContext, OrdContext};
/// use std::cmp::Ordering;
///
/// let ctx = OrdContext { compare: |a: &i32, b: &i32| b.cmp(a) };
/// let a = WithContext { inner: 1, ctx };
/// let b = WithContext { inner: 2, ctx };
/// assert_eq!(a.cmp(&b), Ordering::Greater); // reversed!
/// ```
#[derive(Debug, Clone, Copy)]
pub struct WithContext<T, Ctx> {
    /// The wrapped value.
    pub inner: T,
    /// The context providing trait implementations.
    pub ctx: Ctx,
}
