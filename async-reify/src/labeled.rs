//! [`LabeledFuture`]: attach source-location labels to await points.

use crate::traced::{PollResult, Trace};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

/// A future that records poll events with a source label into a shared
/// [`Trace`].
///
/// Unlike [`TracedFuture`](crate::TracedFuture), `LabeledFuture` pushes
/// events into a caller-supplied [`Trace`] (wrapped in `Arc<Mutex<_>>`),
/// allowing multiple labeled futures to contribute to a single trace
/// with a shared time origin.
///
/// If the wrapped future is dropped before completing, a final
/// [`PollResult::Cancelled`] event is recorded.
///
/// # Examples
///
/// ```
/// use async_reify::{LabeledFuture, Trace};
///
/// # tokio_test::block_on(async {
/// let trace = Trace::shared();
/// let fut = LabeledFuture::new(async { 42 }, "fetch_data", trace.clone());
/// let val = fut.await;
/// assert_eq!(val, 42);
/// assert_eq!(
///     trace.lock().unwrap().events[0].label.as_deref(),
///     Some("fetch_data"),
/// );
/// # });
/// ```
pub struct LabeledFuture<F> {
    inner: Pin<Box<F>>,
    label: String,
    trace: Arc<Mutex<Trace>>,
    completed: bool,
}

impl<F: Future> LabeledFuture<F> {
    /// Create a labeled future that logs to the shared [`Trace`].
    ///
    /// # Examples
    ///
    /// ```
    /// use async_reify::{LabeledFuture, Trace};
    ///
    /// let trace = Trace::shared();
    /// let _fut = LabeledFuture::new(async { 1 }, "step_1", trace);
    /// ```
    pub fn new(inner: F, label: &str, trace: Arc<Mutex<Trace>>) -> Self {
        Self {
            inner: Box::pin(inner),
            label: label.to_string(),
            trace,
            completed: false,
        }
    }
}

impl<F: Future> Future for LabeledFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let poll_result = this.inner.as_mut().poll(cx);

        let result = match &poll_result {
            Poll::Pending => PollResult::Pending,
            Poll::Ready(_) => PollResult::Ready,
        };

        if matches!(result, PollResult::Ready) {
            this.completed = true;
        }

        this.trace
            .lock()
            .expect("trace mutex should not be poisoned")
            .push(result, Some(this.label.clone()));

        poll_result
    }
}

impl<F> Drop for LabeledFuture<F> {
    fn drop(&mut self) {
        if !self.completed {
            if let Ok(mut trace) = self.trace.lock() {
                let last_was_pending = trace
                    .events
                    .last()
                    .is_some_and(|e| matches!(e.result, PollResult::Pending));
                if last_was_pending {
                    trace.push(PollResult::Cancelled, Some(self.label.clone()));
                }
            }
        }
    }
}

/// Helper macro to create a [`LabeledFuture`] with automatic source labeling.
///
/// The label is derived from the expression text and the file/line number.
///
/// # Examples
///
/// ```
/// use async_reify::{labeled_await, LabeledFuture, Trace};
///
/// # tokio_test::block_on(async {
/// let trace = Trace::shared();
/// let val = labeled_await!(async { 42 }, trace).await;
/// assert_eq!(val, 42);
/// let label = trace.lock().unwrap().events[0].label.as_ref().unwrap().clone();
/// assert!(label.contains("labeled.rs")); // contains source file
/// # });
/// ```
#[macro_export]
macro_rules! labeled_await {
    ($fut:expr, $trace:expr) => {{
        let label = format!("{} @ {}:{}", stringify!($fut), file!(), line!());
        $crate::LabeledFuture::new($fut, &label, $trace.clone())
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn labeled_future_records_label() {
        let trace = Trace::shared();
        let fut = LabeledFuture::new(async { "hello" }, "greet_step", trace.clone());
        let val = fut.await;
        assert_eq!(val, "hello");
        let trace = trace.lock().unwrap();
        assert!(!trace.events.is_empty());
        assert_eq!(
            trace.events.last().unwrap().label.as_deref(),
            Some("greet_step")
        );
    }

    #[tokio::test]
    async fn multiple_labeled_futures_share_log() {
        let trace = Trace::shared();

        let fut1 = LabeledFuture::new(async { 1 }, "step_a", trace.clone());
        let _ = fut1.await;

        let fut2 = LabeledFuture::new(async { 2 }, "step_b", trace.clone());
        let _ = fut2.await;

        let trace = trace.lock().unwrap();
        assert_eq!(trace.events.len(), 2);
        assert_eq!(trace.events[0].label.as_deref(), Some("step_a"));
        assert_eq!(trace.events[1].label.as_deref(), Some("step_b"));
    }

    #[tokio::test]
    async fn labeled_await_macro() {
        let trace = Trace::shared();
        let val = labeled_await!(async { 42 }, trace).await;
        assert_eq!(val, 42);
        let trace = trace.lock().unwrap();
        assert!(!trace.events.is_empty());
        let label = trace.events[0].label.as_ref().unwrap();
        assert!(label.contains("labeled.rs"));
    }
}
