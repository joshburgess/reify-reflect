# Documentation index

This folder is the long-form documentation. The crate-level rustdoc on [docs.rs](https://docs.rs/reify-reflect) covers the API surface; here you'll find the conceptual walkthroughs, design notes, and forward-looking RFCs.

If you're new, start with the [narrative blog post](blog-post.md) for the big picture, then move to the [guides](guides/) for tutorial-paced walkthroughs.

## Big picture

- [**Narrative blog post**](blog-post.md). What problem this solves, how reification and reflection fit together, and a worked end-to-end example using modular arithmetic.

## Guides (tutorial pace)

The guides build on each other in order. Read them sequentially the first time through.

1. [Reflect basics](guides/01-reflect-basics.md). Type-level values, the `Reflect` trait, reflecting Peano naturals and HLists.
2. [Branded reify](guides/02-branded-reify.md). Lifting any runtime value into a scoped type-level context with `reify` and the `'brand` lifetime.
3. [const-reify](guides/03-const-reify.md). Using a runtime `u64` as a real `const N: u64` via match-table dispatch.
4. [The `#[reifiable]` macro](guides/04-reifiable-macro.md). Generating `NatCallback` boilerplate for entire traits at once.

## Phase design notes

One per major chunk of the project. These explain *why* each piece is shaped the way it is, the tradeoffs considered, and what was rejected.

1. [Foundations](phase1-foundations.md). The `Reflect` trait, `RuntimeValue`, `reify`, type-level naturals/booleans/HLists, `#[derive(Reflect)]`.
2. [Graph reification](phase2-graph-reification.md). Flat node+edge form for `Rc<RefCell<T>>` / `Arc<Mutex<T>>` graphs, with cycle detection and serde.
3. [Local trait contexts](phase3-context-trait.md). `WithContext<T, Ctx>` for swapping `Ord`/`Hash`/`Display` for one block of code.
4. [Async computation inspector](phase4-async-reify.md). Tracing `async fn` execution as a step graph, with `#[trace_async]`.
5. [Const-generic bridge](phase5-const-reify.md). Safe match-table dispatch from runtime `u64` to `const N: u64`.
6. [Integration](phase6-integration.md). The `reify-reflect` facade crate, feature flags, benchmarks.

## Forward-looking RFCs

Sketches for what the project might grow into. None of these are in 0.1.0; they're shaping the post-0.1 roadmap.

- [0001: Rank-2 const quantification](rfcs/0001-rank2-const-quantification.md). What `for<const N: u64>` would look like as a language feature, and why we'd want it.
- [0002: Bounded existential const types](rfcs/0002-bounded-existential-const-types.md). A possible `#[derive(ConstEnum)]` for storing a `Modular<N>` with runtime `N` and recovering `N` later.
- [0003: `#[reifiable]` proc macro](rfcs/0003-reifiable-proc-macro.md). The design that became phase 5's macro.

## Examples

Runnable code in the source tree:

- [`examples/roundtrip.rs`](../examples/roundtrip.rs). Type-level to value to type-level to value, end to end. `cargo run --example roundtrip --features full`.
- [`reify-graph/examples/serialize_graph.rs`](../reify-graph/examples/serialize_graph.rs). Serialize and deserialize a cyclic graph through `serde_json`.
- [`async-reify/examples/trace_workflow.rs`](../async-reify/examples/trace_workflow.rs). Trace a multi-step async workflow and render it as Graphviz DOT.
