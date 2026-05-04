# Documentation index

This folder is the long-form documentation. The crate-level rustdoc on [docs.rs](https://docs.rs/reify-reflect) covers the API surface; here you'll find the conceptual walkthroughs and forward-looking RFCs.

If you're new, start with the [guides](guides/) for tutorial-paced walkthroughs.

## Guides (tutorial pace)

The guides build on each other in order. Read them sequentially the first time through.

1. [Reflect basics](guides/01-reflect-basics.md). Type-level values, the `Reflect` trait, reflecting Peano naturals and HLists.
2. [Branded reify](guides/02-branded-reify.md). Lifting any runtime value into a scoped type-level context with `reify` and the `'brand` lifetime.
3. [const-reify](guides/03-const-reify.md). Using a runtime `u64` as a real `const N: u64` via match-table dispatch.
4. [The `#[reifiable]` macro](guides/04-reifiable-macro.md). Generating `NatCallback` boilerplate for entire traits at once.

## Forward-looking RFCs

Sketches for what the project might grow into, plus one design doc for a feature that already shipped.

- [0001: Rank-2 const quantification](rfcs/0001-rank2-const-quantification.md). What `for<const N: u64>` would look like as a language feature, and why we'd want it.
- [0002: Bounded existential const types](rfcs/0002-bounded-existential-const-types.md). A possible `#[derive(ConstEnum)]` for storing a `Modular<N>` with runtime `N` and recovering `N` later.
- [0003: `#[reifiable]` proc macro](rfcs/0003-reifiable-proc-macro.md). The original design for `#[reifiable]`. Now implemented as the [`const-reify-derive`](https://docs.rs/const-reify-derive) crate; the doc is kept for design rationale.

## Examples

Runnable code in the source tree:

- [`examples/roundtrip.rs`](../examples/roundtrip.rs). Type-level to value to type-level to value, end to end. `cargo run --example roundtrip --features full`.
- [`reify-graph/examples/serialize_graph.rs`](../reify-graph/examples/serialize_graph.rs). Serialize and deserialize a cyclic graph through `serde_json`.
- [`async-reify/examples/trace_workflow.rs`](../async-reify/examples/trace_workflow.rs). Trace a multi-step async workflow and render it as Graphviz DOT.
