//! [`HashContext`] — custom hashing via function pointer.

use crate::WithContext;
use std::hash::{Hash, Hasher};

/// A context providing a custom hash function.
///
/// When used with [`WithContext`], implements `Hash` by dispatching
/// through the stored hash function. The function receives a `&mut dyn Hasher`
/// so it can work with any hasher type.
///
/// # Examples
///
/// ```
/// use context_trait::{WithContext, HashContext};
/// use std::collections::hash_map::DefaultHasher;
/// use std::hash::{Hash, Hasher};
///
/// let ctx = HashContext { hash: |v: &(i32, i32), h: &mut dyn Hasher| {
///     h.write_i32(v.0);
/// }};
/// let a = WithContext { inner: (1, 100), ctx };
///
/// let mut hasher = DefaultHasher::new();
/// a.hash(&mut hasher);
/// let hash_a = hasher.finish();
///
/// let b = WithContext { inner: (1, 200), ctx };
/// let mut hasher = DefaultHasher::new();
/// b.hash(&mut hasher);
/// let hash_b = hasher.finish();
///
/// // Same hash because only first element is hashed
/// assert_eq!(hash_a, hash_b);
/// ```
pub struct HashContext<T> {
    /// The hash function.
    pub hash: fn(&T, &mut dyn Hasher),
}

impl<T> Clone for HashContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for HashContext<T> {}
impl<T> std::fmt::Debug for HashContext<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashContext")
            .field("hash", &(self.hash as usize))
            .finish()
    }
}

impl<T> Hash for WithContext<T, HashContext<T>> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.ctx.hash)(&self.inner, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn custom_hash_ignores_second_field() {
        let ctx = HashContext {
            hash: |v: &(i32, i32), h: &mut dyn Hasher| {
                h.write_i32(v.0);
            },
        };

        let a = WithContext {
            inner: (1, 100),
            ctx,
        };
        let b = WithContext {
            inner: (1, 999),
            ctx,
        };

        let hash_of = |w: &WithContext<(i32, i32), HashContext<(i32, i32)>>| {
            let mut h = DefaultHasher::new();
            w.hash(&mut h);
            h.finish()
        };

        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn different_values_different_hash() {
        let ctx = HashContext {
            hash: |v: &i32, h: &mut dyn Hasher| {
                h.write_i32(*v);
            },
        };

        let a = WithContext { inner: 1, ctx };
        let b = WithContext { inner: 2, ctx };

        let hash_of = |w: &WithContext<i32, HashContext<i32>>| {
            let mut h = DefaultHasher::new();
            w.hash(&mut h);
            h.finish()
        };

        assert_ne!(hash_of(&a), hash_of(&b));
    }
}
