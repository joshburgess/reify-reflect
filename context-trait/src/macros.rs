//! Declarative macros for scoped context usage and extensibility.

/// Wraps items in [`WithContext<T, OrdContext<T>>`](crate::WithContext) with
/// a custom comparator, runs a callback, and provides the wrapped slice.
///
/// # Examples
///
/// ```
/// use context_trait::{with_ord, WithContext, OrdContext};
///
/// let items = vec![3i32, 1, 4, 1, 5];
/// with_ord!(items, |a: &i32, b: &i32| b.cmp(a), |wrapped: &[WithContext<i32, OrdContext<i32>>]| {
///     let mut sorted = wrapped.to_vec();
///     sorted.sort();
///     let values: Vec<i32> = sorted.into_iter().map(|w| w.inner).collect();
///     assert_eq!(values, vec![5, 4, 3, 1, 1]);
/// });
/// ```
#[macro_export]
macro_rules! with_ord {
    ($items:expr, $cmp:expr, $body:expr) => {{
        let ctx = $crate::OrdContext { compare: $cmp };
        let wrapped: Vec<$crate::WithContext<_, $crate::OrdContext<_>>> = $items
            .iter()
            .map(|v| $crate::WithContext {
                inner: v.clone(),
                ctx,
            })
            .collect();
        let f: &dyn Fn(&[_]) -> _ = &$body;
        f(&wrapped)
    }};
}

/// Wraps items in [`WithContext<T, HashContext<T>>`](crate::WithContext) with
/// a custom hash function, runs a callback, and provides the wrapped slice.
///
/// # Examples
///
/// ```
/// use context_trait::{with_hash, WithContext, HashContext};
/// use std::hash::Hasher;
///
/// let items = vec![(1, 100), (1, 200), (2, 300)];
/// with_hash!(items, |v: &(i32, i32), h: &mut dyn Hasher| { h.write_i32(v.0); },
///     |wrapped: &[WithContext<(i32, i32), HashContext<(i32, i32)>>]| {
///     let _: Vec<_> = wrapped.to_vec();
/// });
/// ```
#[macro_export]
macro_rules! with_hash {
    ($items:expr, $hash_fn:expr, $body:expr) => {{
        let ctx = $crate::HashContext { hash: $hash_fn };
        let wrapped: Vec<$crate::WithContext<_, $crate::HashContext<_>>> = $items
            .iter()
            .map(|v| $crate::WithContext {
                inner: v.clone(),
                ctx,
            })
            .collect();
        let f: &dyn Fn(&[_]) -> _ = &$body;
        f(&wrapped)
    }};
}

/// Wraps items in [`WithContext<T, DisplayContext<T>>`](crate::WithContext) with
/// a custom display function, runs a callback, and provides the wrapped slice.
///
/// # Examples
///
/// ```
/// use context_trait::{with_display, WithContext, DisplayContext};
///
/// let items = vec![1i32, 2, 3];
/// with_display!(
///     items,
///     |v: &i32, f: &mut std::fmt::Formatter| write!(f, "#{v}"),
///     |wrapped: &[WithContext<i32, DisplayContext<i32>>]| {
///         let strs: Vec<String> = wrapped.iter().map(|w| format!("{w}")).collect();
///         assert_eq!(strs, vec!["#1", "#2", "#3"]);
///     }
/// );
/// ```
#[macro_export]
macro_rules! with_display {
    ($items:expr, $display_fn:expr, $body:expr) => {{
        let ctx = $crate::DisplayContext {
            display: $display_fn,
        };
        let wrapped: Vec<$crate::WithContext<_, $crate::DisplayContext<_>>> = $items
            .iter()
            .map(|v| $crate::WithContext {
                inner: v.clone(),
                ctx,
            })
            .collect();
        let f: &dyn Fn(&[_]) -> _ = &$body;
        f(&wrapped)
    }};
}

/// Define a new context type and its trait implementation for [`WithContext`](crate::WithContext).
///
/// This macro generates a context struct holding a function pointer and
/// implements the specified trait for `WithContext<T, YourContext<T>>`.
///
/// # Examples
///
/// ```
/// use context_trait::{impl_context_trait, WithContext};
/// use std::fmt;
///
/// // Define a trait we want to contextualize
/// trait Summarize {
///     fn summarize(&self) -> String;
/// }
///
/// // Generate SummarizeContext and impl Summarize for WithContext
/// impl_context_trait! {
///     /// A context for custom summarization.
///     context SummarizeContext<T> {
///         field summarize_fn: fn(&T) -> String
///     }
///     impl Summarize for WithContext<T, SummarizeContext<T>> {
///         fn summarize(&self) -> String {
///             (self.ctx.summarize_fn)(&self.inner)
///         }
///     }
/// }
///
/// struct Article { title: String, body: String }
///
/// let ctx = SummarizeContext {
///     summarize_fn: |a: &Article| format!("{}...", &a.title),
/// };
/// let wrapped = WithContext {
///     inner: Article { title: "Hello".into(), body: "World".into() },
///     ctx,
/// };
/// assert_eq!(wrapped.summarize(), "Hello...");
/// ```
#[macro_export]
macro_rules! impl_context_trait {
    (
        $(#[$ctx_meta:meta])*
        context $ctx_name:ident<$t:ident> {
            field $field:ident : $field_ty:ty
        }
        impl $trait_name:ident for WithContext<$t2:ident, $ctx_name2:ident<$t3:ident>> {
            $(
                fn $method:ident(&$self:ident $(, $arg:ident : $arg_ty:ty)* ) -> $ret:ty {
                    $($body:tt)*
                }
            )+
        }
    ) => {
        $(#[$ctx_meta])*
        pub struct $ctx_name<$t> {
            /// The function pointer providing the implementation.
            pub $field: $field_ty,
        }

        impl<$t> Clone for $ctx_name<$t> {
            fn clone(&self) -> Self { *self }
        }
        impl<$t> Copy for $ctx_name<$t> {}
        impl<$t> std::fmt::Debug for $ctx_name<$t> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($ctx_name)).finish()
            }
        }

        impl<$t2> $trait_name for $crate::WithContext<$t2, $ctx_name2<$t3>> {
            $(
                fn $method(&$self $(, $arg : $arg_ty)*) -> $ret {
                    $($body)*
                }
            )+
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::BTreeSet;
    use std::hash::Hasher;

    #[test]
    fn with_ord_sort() {
        let items = [3i32, 1, 4, 1, 5, 9];
        with_ord!(
            items,
            |a: &i32, b: &i32| b.cmp(a),
            |wrapped: &[WithContext<i32, OrdContext<i32>>]| {
                let mut sorted = wrapped.to_vec();
                sorted.sort();
                let values: Vec<i32> = sorted.into_iter().map(|w| w.inner).collect();
                assert_eq!(values, vec![9, 5, 4, 3, 1, 1]);
            }
        );
    }

    #[test]
    fn with_ord_btreeset() {
        let items = [3i32, 1, 4, 1, 5];
        with_ord!(
            items,
            |a: &i32, b: &i32| b.cmp(a),
            |wrapped: &[WithContext<i32, OrdContext<i32>>]| {
                let set: BTreeSet<_> = wrapped.iter().cloned().collect();
                let values: Vec<i32> = set.into_iter().map(|w| w.inner).collect();
                // BTreeSet deduplicates, reverse order
                assert_eq!(values, vec![5, 4, 3, 1]);
            }
        );
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn with_hash_custom() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;

        let items = [(1, 100), (1, 200)];
        with_hash!(
            items,
            |v: &(i32, i32), h: &mut dyn Hasher| {
                h.write_i32(v.0);
            },
            |wrapped: &[WithContext<(i32, i32), HashContext<(i32, i32)>>]| {
                let hash_of = |w: &WithContext<(i32, i32), HashContext<(i32, i32)>>| -> u64 {
                    let mut h = DefaultHasher::new();
                    w.hash(&mut h);
                    h.finish()
                };
                assert_eq!(hash_of(&wrapped[0]), hash_of(&wrapped[1]));
            }
        );
    }

    #[test]
    fn with_display_custom() {
        let items = [1i32, 2, 3];
        with_display!(
            items,
            |v: &i32, f: &mut std::fmt::Formatter| write!(f, "[{v}]"),
            |wrapped: &[WithContext<i32, DisplayContext<i32>>]| {
                let strs: Vec<String> = wrapped.iter().map(|w| format!("{w}")).collect();
                assert_eq!(strs, vec!["[1]", "[2]", "[3]"]);
            }
        );
    }

    // Test impl_context_trait! macro
    trait Summarize {
        fn summarize(&self) -> String;
    }

    impl_context_trait! {
        /// Context for custom summarization.
        context SummarizeContext<T> {
            field summarize_fn: fn(&T) -> String
        }
        impl Summarize for WithContext<T, SummarizeContext<T>> {
            fn summarize(&self) -> String {
                (self.ctx.summarize_fn)(&self.inner)
            }
        }
    }

    #[test]
    fn custom_context_trait() {
        struct Article {
            title: String,
        }

        let ctx = SummarizeContext {
            summarize_fn: |a: &Article| format!("Title: {}", a.title),
        };
        let wrapped = WithContext {
            inner: Article {
                title: "Hello".into(),
            },
            ctx,
        };
        assert_eq!(wrapped.summarize(), "Title: Hello");
    }
}
