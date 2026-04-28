#![deny(unsafe_code)]

//! Instrument `async` functions to extract their continuation structure
//! as an inspectable, serializable step graph.
//!
//! `async fn` in Rust compiles down to a state machine that the executor
//! polls. Most of the time you do not need to think about that, but when
//! something is mysteriously slow, hangs, or stalls, you would really
//! like to see the shape of what is happening: which await points were
//! hit, in what order, how often each was polled, and whether each one
//! returned `Ready` or `Pending`.
//!
//! This crate gives you a low-friction way to capture that. Wrap a future
//! in [`TracedFuture`] (or sprinkle [`LabeledFuture`] across specific
//! await points), drive it to completion, and you get a [`Trace`] of
//! [`PollEvent`]s. Pass that to [`reify_execution`] for an
//! [`AsyncStepGraph`] grouped by label, and [`to_dot`] to render it as
//! Graphviz.
//!
//! With the `serde` feature, [`PollEvent`], [`Trace`], and
//! [`AsyncStepGraph`] all serialize cleanly (timestamps are stored as
//! [`Duration`](std::time::Duration) since trace start, so traces are
//! deterministic and portable).
//!
//! With the `macros` feature, the [`macro@trace_async`] attribute proc
//! macro (re-exported from
//! [`async-reify-macros`](https://docs.rs/async-reify-macros)) rewrites
//! every `.await` in a function body into a [`LabeledFuture`] so you do
//! not have to label each one by hand. Labels look like
//! `"<expr> @ file.rs:42"` for easy navigation back to the source.
//!
//! # Workflow
//!
//! 1. Wrap futures in [`TracedFuture`] (or use [`LabeledFuture`] /
//!    `#[trace_async]` for finer-grained labels).
//! 2. Drive them to completion. They behave like ordinary futures; the
//!    only side effect is recording a [`PollEvent`] every time they are
//!    polled.
//! 3. Convert the [`Trace`] to an [`AsyncStepGraph`] with
//!    [`reify_execution`].
//! 4. Inspect, serialize, or render. [`to_dot`] outputs Graphviz DOT.
//!
//! See the [`trace_workflow` example][example] for an end-to-end run, and
//! [`docs/phase4-async-reify.md`][phase4] for the design choices (why
//! `Arc<Mutex<...>>` for shared logging, why label-based step grouping,
//! why DOT rather than full deterministic replay).
//!
//! [example]: https://github.com/joshburgess/reify-reflect/blob/main/async-reify/examples/trace_workflow.rs
//! [phase4]: https://github.com/joshburgess/reify-reflect/blob/main/docs/phase4-async-reify.md
//!
//! # Examples
//!
//! ```
//! use async_reify::{TracedFuture, PollResult};
//!
//! # tokio_test::block_on(async {
//! let (result, trace) = TracedFuture::run(async { 42 }).await;
//! assert_eq!(result, 42);
//! assert!(!trace.events.is_empty());
//! assert!(matches!(trace.events.last().unwrap().result, PollResult::Ready));
//! # });
//! ```

mod graph;
mod labeled;
mod traced;

pub use graph::{reify_execution, to_dot, AsyncStepGraph, StepNode, StepOutcome};
pub use labeled::LabeledFuture;
pub use traced::{PollEvent, PollResult, Trace, TracedFuture};

/// Attribute proc macro that rewrites every `.await` in an async function
/// body into a [`LabeledFuture`] recording into a shared [`Trace`].
///
/// Re-exported from `async-reify-macros` when the `macros` feature is
/// enabled.
#[cfg(feature = "macros")]
pub use async_reify_macros::trace_async;
