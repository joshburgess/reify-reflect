#![deny(unsafe_code)]

//! # async-reify
//!
//! Instrument async functions to extract their continuation structure
//! as an inspectable, serializable step graph.
//!
//! The core workflow:
//! 1. Wrap futures in [`TracedFuture`] to record [`PollEvent`]s
//! 2. Use [`LabeledFuture`] to attach source labels to await points
//! 3. Extract an [`AsyncStepGraph`] from the collected trace
//! 4. Optionally serialize to JSON or render as Graphviz DOT
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
