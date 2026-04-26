//! [`OrdContext`]: custom ordering via function pointer.

use crate::WithContext;
use std::cmp::Ordering;

/// A context providing a custom comparison function.
///
/// When used with [`WithContext`], implements `PartialEq`, `Eq`,
/// `PartialOrd`, and `Ord` by dispatching through the stored comparator.
///
/// # Examples
///
/// ```
/// use context_trait::{WithContext, OrdContext};
///
/// // Sort in reverse order
/// let ctx = OrdContext { compare: |a: &i32, b: &i32| b.cmp(a) };
/// let mut items: Vec<_> = [3, 1, 2].iter()
///     .map(|&v| WithContext { inner: v, ctx })
///     .collect();
/// items.sort();
/// let values: Vec<i32> = items.into_iter().map(|w| w.inner).collect();
/// assert_eq!(values, vec![3, 2, 1]);
/// ```
pub struct OrdContext<T> {
    /// The comparison function.
    pub compare: fn(&T, &T) -> Ordering,
}

// Manual impls: the fn pointer is always Copy regardless of T.
impl<T> Clone for OrdContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for OrdContext<T> {}
impl<T> std::fmt::Debug for OrdContext<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrdContext")
            .field("compare", &(self.compare as usize))
            .finish()
    }
}

impl<T> PartialEq for WithContext<T, OrdContext<T>> {
    fn eq(&self, other: &Self) -> bool {
        (self.ctx.compare)(&self.inner, &other.inner) == Ordering::Equal
    }
}

impl<T> Eq for WithContext<T, OrdContext<T>> {}

impl<T> PartialOrd for WithContext<T, OrdContext<T>> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for WithContext<T, OrdContext<T>> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.ctx.compare)(&self.inner, &other.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_with_custom_ord() {
        let ctx = OrdContext {
            compare: |a: &i32, b: &i32| b.cmp(a),
        };
        let mut items: Vec<_> = [3, 1, 4, 1, 5]
            .iter()
            .map(|&v| WithContext { inner: v, ctx })
            .collect();
        items.sort();
        let values: Vec<i32> = items.into_iter().map(|w| w.inner).collect();
        assert_eq!(values, vec![5, 4, 3, 1, 1]);
    }

    #[test]
    fn equality_through_context() {
        let ctx = OrdContext {
            compare: |a: &(i32, i32), b: &(i32, i32)| a.0.cmp(&b.0),
        };
        let a = WithContext {
            inner: (1, 100),
            ctx,
        };
        let b = WithContext {
            inner: (1, 200),
            ctx,
        };
        // Equal by first element only
        assert_eq!(a, b);
    }

    #[test]
    fn btreeset_with_custom_ord() {
        use std::collections::BTreeSet;

        let ctx = OrdContext {
            compare: |a: &String, b: &String| a.len().cmp(&b.len()),
        };

        let mut set = BTreeSet::new();
        set.insert(WithContext {
            inner: "hello".to_string(),
            ctx,
        });
        set.insert(WithContext {
            inner: "hi".to_string(),
            ctx,
        });
        set.insert(WithContext {
            inner: "world".to_string(),
            ctx,
        }); // same len as "hello"

        // "world" collides with "hello" (same length), so only 2 entries
        assert_eq!(set.len(), 2);
    }
}
