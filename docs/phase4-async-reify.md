# Phase 4: Async Computation Inspector

## What this phase covers

Tooling to extract the *shape* of an `async fn` execution as data: which await points were polled, in what order, how many times, and whether each returned `Ready` or `Pending`.

`async fn` in Rust compiles down to a state machine, and most of the time you don't have to think about that. But when something hangs, stalls, or is mysteriously slow, you really want to see the structure of what is happening. This phase wraps futures so that polling them records a stream of `PollEvent`s into a shared `Trace`, then reifies that trace into an `AsyncStepGraph` you can serialize, inspect, or render as Graphviz DOT.

The `#[trace_async]` proc macro automates the per-await wrapping: every `.await` in the function body becomes a `LabeledFuture`, with labels of the form `"<expr> @ file.rs:42"` so each step in the graph points back to the source line that produced it.

## Crates introduced

- [`async-reify`](https://docs.rs/async-reify) (with [`trace_workflow` example](../async-reify/examples/trace_workflow.rs))
- [`async-reify-macros`](https://docs.rs/async-reify-macros) (the `#[trace_async]` proc macro)

## Design decisions

### Arc<Mutex<Vec<PollEvent>>> for shared logging

Poll events are collected in a shared `Arc<Mutex<Vec<PollEvent>>>`. This
enables multiple `LabeledFuture` instances to contribute to a single trace
log, which is essential for tracing multi-step async workflows. The mutex
overhead is negligible since logging is the instrumentation cost we're
already accepting.

### Label-based step grouping

`reify_execution` groups consecutive events with the same label into a
single `StepNode`. This produces a clean step graph where each node
represents a logical operation (e.g., "fetch_data") rather than individual
poll calls. Label changes mark step boundaries.

### TracedFuture::run() convenience

`TracedFuture::run()` provides an ergonomic one-shot API that returns
`(Output, Trace)`. For more complex multi-future workflows, the lower-level
`LabeledFuture` + shared log pattern gives full control.

### labeled_await! macro for auto-labeling

The `labeled_await!` macro uses `stringify!` and `file!()`/`line!()` to
automatically generate labels like `"fetch_data() @ lib.rs:42"`. This
eliminates manual label management for the common case.

### DOT output, not full replay

The `to_dot()` function provides immediate visualization value. Full
deterministic replay would require a controlled executor (capturing waker
scheduling, timer state, etc.) and is noted as a future milestone in the
docs rather than a half-implemented feature.

## Next

- [Phase 5: Const-generic bridge](phase5-const-reify.md)
- [Documentation index](README.md)
