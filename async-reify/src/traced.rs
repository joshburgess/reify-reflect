//! [`TracedFuture`]: a future wrapper that records poll events.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// The outcome of a single poll.
///
/// `Cancelled` is recorded when a future is dropped before completing
/// (its last poll returned `Pending` and no `Ready` event was ever emitted).
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
    /// The future was dropped before completing.
    Cancelled,
}

/// A recorded poll event.
///
/// `offset` is the elapsed time since the start of the [`Trace`] this
/// event belongs to, expressed as a [`Duration`]. This makes events
/// serializable and replayable across processes.
///
/// # Examples
///
/// ```
/// use async_reify::{PollEvent, PollResult};
/// use std::time::Duration;
///
/// let event = PollEvent {
///     step: 0,
///     offset: Duration::from_micros(150),
///     result: PollResult::Ready,
///     label: None,
/// };
/// assert_eq!(event.step, 0);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PollEvent {
    /// Sequential poll index (0-based).
    pub step: usize,
    /// Time elapsed since the start of the parent [`Trace`].
    pub offset: Duration,
    /// Whether the poll returned Ready, Pending, or was Cancelled by drop.
    pub result: PollResult,
    /// Optional label for this await point.
    pub label: Option<String>,
}

/// Collected trace from a [`TracedFuture`].
///
/// A `Trace` holds the recorded events plus a reference [`Instant`] used
/// to compute event offsets. The reference instant is not serialized;
/// when a `Trace` is deserialized the `start` field is reset to the
/// deserialization time and only the per-event `offset` values are
/// authoritative.
///
/// # Examples
///
/// ```
/// use async_reify::Trace;
///
/// let trace = Trace::new();
/// assert!(trace.events.is_empty());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trace {
    /// All poll events in order.
    pub events: Vec<PollEvent>,
    /// Reference instant against which each event's `offset` was measured.
    /// Not serialized: only the offsets persist across (de)serialization.
    #[cfg_attr(feature = "serde", serde(skip, default = "Instant::now"))]
    pub start: Instant,
}

impl Trace {
    /// Construct an empty trace anchored at the current instant.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_reify::Trace;
    ///
    /// let t = Trace::new();
    /// assert!(t.events.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start: Instant::now(),
        }
    }

    /// Return a fresh shared, mutex-protected trace suitable for use
    /// with [`LabeledFuture`](crate::LabeledFuture).
    ///
    /// # Examples
    ///
    /// ```
    /// use async_reify::Trace;
    ///
    /// let shared = Trace::shared();
    /// assert!(shared.lock().unwrap().events.is_empty());
    /// ```
    pub fn shared() -> Arc<Mutex<Trace>> {
        Arc::new(Mutex::new(Trace::new()))
    }

    /// Append an event to the trace, computing its `offset` from the
    /// trace's reference instant.
    pub(crate) fn push(&mut self, result: PollResult, label: Option<String>) {
        let step = self.events.len();
        let offset = Instant::now().saturating_duration_since(self.start);
        self.events.push(PollEvent {
            step,
            offset,
            result,
            label,
        });
    }
}

impl Default for Trace {
    fn default() -> Self {
        Trace::new()
    }
}

/// A future wrapper that records each poll as a [`PollEvent`].
///
/// Use [`TracedFuture::run`] for a convenient way to execute a future
/// and collect its trace.
///
/// If the wrapped future is dropped before it completes, a final event
/// with [`PollResult::Cancelled`] is appended to the trace.
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
    trace: Arc<Mutex<Trace>>,
    label: Option<String>,
    completed: bool,
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
            trace: Trace::shared(),
            label: None,
            completed: false,
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
            trace: Trace::shared(),
            label: Some(label.to_string()),
            completed: false,
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
        let trace = Trace::shared();
        let traced = TracedFuture {
            inner: Box::pin(inner),
            trace: trace.clone(),
            label: None,
            completed: false,
        };
        let result = traced.await;
        let trace = Arc::try_unwrap(trace)
            .expect("trace arc should have single owner")
            .into_inner()
            .expect("trace mutex should not be poisoned");
        (result, trace)
    }
}

impl<F: Future> Future for TracedFuture<F> {
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
            .push(result, this.label.clone());

        poll_result
    }
}

impl<F> Drop for TracedFuture<F> {
    fn drop(&mut self) {
        if !self.completed {
            if let Ok(mut trace) = self.trace.lock() {
                let last_was_pending = trace
                    .events
                    .last()
                    .is_some_and(|e| matches!(e.result, PollResult::Pending));
                if last_was_pending {
                    trace.push(PollResult::Cancelled, self.label.clone());
                }
            }
        }
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
        assert!(trace.events.len() >= 3);
        assert_eq!(trace.events.last().unwrap().result, PollResult::Ready);
    }

    #[tokio::test]
    async fn with_label() {
        let traced = TracedFuture::with_label(async { 1 }, "test_step");
        let trace = traced.trace.clone();
        let _ = traced.await;
        let trace = trace.lock().unwrap();
        assert_eq!(trace.events[0].label.as_deref(), Some("test_step"));
    }

    #[tokio::test]
    async fn dropped_pending_future_is_cancelled() {
        let trace_shared = Trace::shared();

        // Build a future that yields once (recording Pending) then would return.
        // Drop it after the first poll to simulate cancellation.
        struct YieldOnce {
            yielded: bool,
        }
        impl Future for YieldOnce {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.yielded {
                    Poll::Ready(())
                } else {
                    self.yielded = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

        let mut traced = TracedFuture {
            inner: Box::pin(YieldOnce { yielded: false }),
            trace: trace_shared.clone(),
            label: Some("drop_me".into()),
            completed: false,
        };

        // Manually poll once (record Pending) without driving to completion.
        let waker = futures_task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let _ = Pin::new(&mut traced).poll(&mut cx);
        drop(traced);

        let trace = trace_shared.lock().unwrap();
        assert!(trace.events.iter().any(|e| e.result == PollResult::Pending));
        assert!(
            trace
                .events
                .iter()
                .any(|e| e.result == PollResult::Cancelled),
            "expected a Cancelled event after drop, got {:?}",
            trace.events
        );
    }

    #[cfg(feature = "serde")]
    #[tokio::test]
    async fn trace_round_trip_serde() {
        let (_, trace) = TracedFuture::run(async {
            tokio::task::yield_now().await;
            7
        })
        .await;
        let json = serde_json::to_string(&trace).expect("serialize");
        let restored: Trace = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.events.len(), trace.events.len());
        for (a, b) in trace.events.iter().zip(restored.events.iter()) {
            assert_eq!(a.step, b.step);
            assert_eq!(a.offset, b.offset);
            assert_eq!(a.result, b.result);
            assert_eq!(a.label, b.label);
        }
    }
}
