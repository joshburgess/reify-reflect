//! [`TracedFuture`] — a future wrapper that records poll events.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Instant;

/// The outcome of a single poll.
///
/// # Examples
///
/// ```
/// use async_reify::PollResult;
///
/// let ready = PollResult::Ready;
/// let pending = PollResult::Pending;
/// assert_ne!(ready, pending);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PollResult {
    /// The future returned `Poll::Pending`.
    Pending,
    /// The future returned `Poll::Ready`.
    Ready,
}

/// A recorded poll event.
///
/// # Examples
///
/// ```
/// use async_reify::{PollEvent, PollResult};
/// use std::time::Instant;
///
/// let event = PollEvent {
///     step: 0,
///     timestamp: Instant::now(),
///     result: PollResult::Ready,
///     label: None,
/// };
/// assert_eq!(event.step, 0);
/// ```
#[derive(Debug, Clone)]
pub struct PollEvent {
    /// Sequential poll index (0-based).
    pub step: usize,
    /// When this poll occurred.
    pub timestamp: Instant,
    /// Whether the poll returned Ready or Pending.
    pub result: PollResult,
    /// Optional label for this await point.
    pub label: Option<String>,
}

/// Collected trace from a [`TracedFuture`].
///
/// # Examples
///
/// ```
/// use async_reify::Trace;
///
/// let trace = Trace { events: vec![] };
/// assert!(trace.events.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Trace {
    /// All poll events in order.
    pub events: Vec<PollEvent>,
}

/// A future wrapper that records each poll as a [`PollEvent`].
///
/// Use [`TracedFuture::run`] for a convenient way to execute a future
/// and collect its trace.
///
/// # Examples
///
/// ```
/// use async_reify::TracedFuture;
///
/// # tokio_test::block_on(async {
/// let (val, trace) = TracedFuture::run(async { 1 + 1 }).await;
/// assert_eq!(val, 2);
/// assert!(!trace.events.is_empty());
/// # });
/// ```
pub struct TracedFuture<F> {
    inner: Pin<Box<F>>,
    events: Arc<Mutex<Vec<PollEvent>>>,
    label: Option<String>,
}

impl<F: Future> TracedFuture<F> {
    /// Create a new traced future wrapping `inner`.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_reify::TracedFuture;
    ///
    /// let traced = TracedFuture::new(async { 42 });
    /// ```
    pub fn new(inner: F) -> Self {
        Self {
            inner: Box::pin(inner),
            events: Arc::new(Mutex::new(Vec::new())),
            label: None,
        }
    }

    /// Create a new traced future with a label.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_reify::TracedFuture;
    ///
    /// let traced = TracedFuture::with_label(async { 42 }, "my_step");
    /// ```
    pub fn with_label(inner: F, label: &str) -> Self {
        Self {
            inner: Box::pin(inner),
            events: Arc::new(Mutex::new(Vec::new())),
            label: Some(label.to_string()),
        }
    }

    /// Run the future to completion, returning the result and the trace.
    ///
    /// This is a convenience wrapper that polls the future through a
    /// [`TracedFuture`] and collects all events.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_reify::{TracedFuture, PollResult};
    ///
    /// # tokio_test::block_on(async {
    /// let (result, trace) = TracedFuture::run(async { "hello" }).await;
    /// assert_eq!(result, "hello");
    /// assert!(matches!(trace.events.last().unwrap().result, PollResult::Ready));
    /// # });
    /// ```
    pub async fn run(inner: F) -> (F::Output, Trace) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let traced = TracedFuture {
            inner: Box::pin(inner),
            events,
            label: None,
        };
        let result = traced.await;
        let events = Arc::try_unwrap(events_clone)
            .expect("trace events arc should have single owner")
            .into_inner()
            .expect("mutex should not be poisoned");
        (result, Trace { events })
    }
}

impl<F: Future> Future for TracedFuture<F> {
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
                label: this.label.clone(),
            });

        poll_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn trace_immediate_future() {
        let (val, trace) = TracedFuture::run(async { 42 }).await;
        assert_eq!(val, 42);
        assert_eq!(trace.events.len(), 1);
        assert_eq!(trace.events[0].result, PollResult::Ready);
        assert_eq!(trace.events[0].step, 0);
    }

    #[tokio::test]
    async fn trace_multi_step() {
        let (val, trace) = TracedFuture::run(async {
            tokio::task::yield_now().await;
            tokio::task::yield_now().await;
            99
        })
        .await;
        assert_eq!(val, 99);
        // At least 2 Pending + 1 Ready
        assert!(trace.events.len() >= 3);
        assert_eq!(trace.events.last().unwrap().result, PollResult::Ready);
    }

    #[tokio::test]
    async fn with_label() {
        let traced = TracedFuture::with_label(async { 1 }, "test_step");
        let events = traced.events.clone();
        let _ = traced.await;
        let events = events.lock().unwrap();
        assert_eq!(events[0].label.as_deref(), Some("test_step"));
    }
}
