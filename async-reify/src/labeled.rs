//! [`LabeledFuture`] — attach source-location labels to await points.

use crate::traced::{PollEvent, PollResult};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Instant;

/// A future that records poll events with a source label.
///
/// Unlike [`TracedFuture`](crate::TracedFuture), `LabeledFuture` pushes
/// events into a shared external event log, allowing multiple labeled
/// futures to contribute to a single trace.
///
/// # Examples
///
/// ```
/// use async_reify::LabeledFuture;
/// use async_reify::PollEvent;
/// use std::sync::{Arc, Mutex};
///
/// # tokio_test::block_on(async {
/// let log = Arc::new(Mutex::new(Vec::<PollEvent>::new()));
/// let fut = LabeledFuture::new(async { 42 }, "fetch_data", log.clone());
/// let val = fut.await;
/// assert_eq!(val, 42);
/// assert_eq!(log.lock().unwrap()[0].label.as_deref(), Some("fetch_data"));
/// # });
/// ```
pub struct LabeledFuture<F> {
    inner: Pin<Box<F>>,
    label: String,
    events: Arc<Mutex<Vec<PollEvent>>>,
}

impl<F: Future> LabeledFuture<F> {
    /// Create a labeled future that logs to the shared `events` vec.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_reify::{LabeledFuture, PollEvent};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let log = Arc::new(Mutex::new(Vec::<PollEvent>::new()));
    /// let _fut = LabeledFuture::new(async { 1 }, "step_1", log);
    /// ```
    pub fn new(inner: F, label: &str, events: Arc<Mutex<Vec<PollEvent>>>) -> Self {
        Self {
            inner: Box::pin(inner),
            label: label.to_string(),
            events,
        }
    }
}

impl<F: Future> Future for LabeledFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let step = this
            .events
            .lock()
            .expect("mutex should not be poisoned")
            .len();

        let poll_result = this.inner.as_mut().poll(cx);

        let result = match &poll_result {
            Poll::Pending => PollResult::Pending,
            Poll::Ready(_) => PollResult::Ready,
        };

        this.events
            .lock()
            .expect("mutex should not be poisoned")
            .push(PollEvent {
                step,
                timestamp: Instant::now(),
                result,
                label: Some(this.label.clone()),
            });

        poll_result
    }
}

/// Helper macro to create a [`LabeledFuture`] with automatic source labeling.
///
/// The label is derived from the expression text and the file/line number.
///
/// # Examples
///
/// ```
/// use async_reify::{labeled_await, LabeledFuture, PollEvent};
/// use std::sync::{Arc, Mutex};
///
/// # tokio_test::block_on(async {
/// let log = Arc::new(Mutex::new(Vec::<PollEvent>::new()));
/// let val = labeled_await!(async { 42 }, log).await;
/// assert_eq!(val, 42);
/// let label = log.lock().unwrap()[0].label.as_ref().unwrap().clone();
/// assert!(label.contains("labeled.rs")); // contains source file
/// # });
/// ```
#[macro_export]
macro_rules! labeled_await {
    ($fut:expr, $log:expr) => {{
        let label = format!("{} @ {}:{}", stringify!($fut), file!(), line!());
        $crate::LabeledFuture::new($fut, &label, $log.clone())
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn labeled_future_records_label() {
        let log = Arc::new(Mutex::new(Vec::<PollEvent>::new()));
        let fut = LabeledFuture::new(async { "hello" }, "greet_step", log.clone());
        let val = fut.await;
        assert_eq!(val, "hello");
        let events = log.lock().unwrap();
        assert!(!events.is_empty());
        assert_eq!(events.last().unwrap().label.as_deref(), Some("greet_step"));
    }

    #[tokio::test]
    async fn multiple_labeled_futures_share_log() {
        let log = Arc::new(Mutex::new(Vec::<PollEvent>::new()));

        let fut1 = LabeledFuture::new(async { 1 }, "step_a", log.clone());
        let _ = fut1.await;

        let fut2 = LabeledFuture::new(async { 2 }, "step_b", log.clone());
        let _ = fut2.await;

        let events = log.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].label.as_deref(), Some("step_a"));
        assert_eq!(events[1].label.as_deref(), Some("step_b"));
    }

    #[tokio::test]
    async fn labeled_await_macro() {
        let log = Arc::new(Mutex::new(Vec::<PollEvent>::new()));
        let val = labeled_await!(async { 42 }, log).await;
        assert_eq!(val, 42);
        let events = log.lock().unwrap();
        assert!(!events.is_empty());
        // Label should contain file and line info
        let label = events[0].label.as_ref().unwrap();
        assert!(label.contains("labeled.rs"));
    }
}
