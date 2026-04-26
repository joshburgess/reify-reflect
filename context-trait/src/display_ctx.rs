//! [`DisplayContext`]: custom display formatting via function pointer.

use crate::WithContext;
use std::fmt;

/// A context providing a custom display function.
///
/// When used with [`WithContext`], implements `Display` by dispatching
/// through the stored format function.
///
/// # Examples
///
/// ```
/// use context_trait::{WithContext, DisplayContext};
///
/// let ctx = DisplayContext {
///     display: |v: &(i32, i32), f: &mut std::fmt::Formatter| {
///         write!(f, "({}, {})", v.0, v.1)
///     },
/// };
/// let w = WithContext { inner: (1, 2), ctx };
/// assert_eq!(format!("{w}"), "(1, 2)");
/// ```
pub struct DisplayContext<T> {
    /// The display function.
    pub display: fn(&T, &mut fmt::Formatter) -> fmt::Result,
}

impl<T> Clone for DisplayContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for DisplayContext<T> {}
impl<T> std::fmt::Debug for DisplayContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DisplayContext")
            .field("display", &(self.display as usize))
            .finish()
    }
}

impl<T> fmt::Display for WithContext<T, DisplayContext<T>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.ctx.display)(&self.inner, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_display() {
        let ctx = DisplayContext {
            display: |v: &i32, f: &mut fmt::Formatter| write!(f, "value={v}"),
        };
        let w = WithContext { inner: 42, ctx };
        assert_eq!(format!("{w}"), "value=42");
    }

    #[test]
    fn display_struct() {
        struct Point {
            x: f64,
            y: f64,
        }

        let ctx = DisplayContext {
            display: |p: &Point, f: &mut fmt::Formatter| write!(f, "({}, {})", p.x, p.y),
        };
        let w = WithContext {
            inner: Point { x: 1.5, y: 2.5 },
            ctx,
        };
        assert_eq!(format!("{w}"), "(1.5, 2.5)");
    }
}
