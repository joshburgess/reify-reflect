# Phase 4 — Async Computation Inspector

## Design Decisions

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
