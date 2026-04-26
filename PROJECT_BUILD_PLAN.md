# Rust Reification & Reflection: Agent Build Plan

This document is a phased implementation plan for building a unified reification/reflection ecosystem in Rust. Each phase is designed to be handed to a Claude Code agent as a self-contained task. Phases build on each other; complete them in order unless noted otherwise.

---

## Guiding Principles for All Agents

- Use a single Cargo workspace (`reify-reflect/`) with one crate per component.
- Every public API must be documented with `///` doc comments and at least one `# Example` block.
- Every crate must have `#[deny(unsafe_code)]` at the top-level **except** the crates explicitly marked as unsafe-core.
- Write tests in `#[cfg(test)]` modules and integration tests in `tests/`. Aim for >80% coverage.
- Use `cargo clippy -- -D warnings` and `cargo fmt` before marking any phase complete.
- Pin Rust edition to `2021`. MSRV target: `1.75.0`.

---

## Phase 1: Foundations (Unified `Reify`/`Reflect` Trait Family)

**Goal:** Establish the core trait definitions and implementations for type-level naturals, booleans, and lists. This is the substrate everything else builds on.

**Crate:** `reify-reflect-core`

### Iteration 1.1: Core Traits

Define the following traits in `reify-reflect-core/src/lib.rs`:

```rust
/// Converts a type-level value into a runtime value.
pub trait Reflect {
    type Value;
    fn reflect() -> Self::Value;
}

/// Witnesses that a runtime value can be lifted into a type-level context
/// and passed to a callback.
pub trait Reify<T> {
    fn reify<F, R>(val: T, f: F) -> R
    where
        F: for<'a> FnOnce(&'a dyn std::any::Any) -> R;
}
```

Also define a `RuntimeValue` enum:

```rust
pub enum RuntimeValue {
    Nat(u64),
    Bool(bool),
    List(Vec<RuntimeValue>),
    Unit,
}
```

Deliverable: Compiles, all traits exported, no implementations yet.

---

### Iteration 1.2: Type-Level Naturals

**Crate:** `reflect-nat` (depends on `reify-reflect-core`)

- Define `Z` (zero) and `S<N>` (successor) types.
- Implement `Reflect` for `Z` (returns `RuntimeValue::Nat(0)`) and for `S<N: Reflect<Value = RuntimeValue>>`.
- Implement `Add`, `Mul`, and `Lt` type-level operations using associated types.
- Provide `typenum`-compatible bridge: `impl From<typenum::UTerm> for RuntimeValue` etc.
- Write tests verifying that `S<S<S<Z>>>::reflect()` returns `RuntimeValue::Nat(3)`.

---

### Iteration 1.3: Type-Level Booleans and Lists

Extend `reflect-nat`:

- Add `True` and `False` types, each implementing `Reflect<Value = bool>`.
- Add `HNil` and `HCons<H, T>` for type-level lists, implementing `Reflect<Value = Vec<RuntimeValue>>`.
- Implement `frunk`-compatible bridge: `From<frunk::HNil>` and `From<frunk::HCons<H,T>>`.
- Test round-trips: construct an HList at type level, reflect to `Vec<RuntimeValue>`, assert values.

---

### Iteration 1.4: `#[derive(Reflect)]` Proc Macro

**Crate:** `reflect-derive`

Write a derive macro that implements `Reflect` for structs, emitting a `RuntimeValue::List` of field names (as strings) paired with their reflected values where the field types implement `Reflect`. Fields that don't implement `Reflect` should be skipped with a warning attribute `#[reflect(skip)]`.

Test with at least two structs, one nested.

---

## Phase 2: Graph Reification (`reify-graph`)

**Goal:** Safe library to convert `Rc`/`Arc`-based pointer graphs into node-indexed adjacency representations and back.

**Crate:** `reify-graph`

### Iteration 2.1: Node Detection via Pointer Identity

- Define `NodeId(usize)` as a stable identifier derived from `Rc::as_ptr` cast to `usize`.
- Implement `fn collect_nodes<T>(root: &Rc<T>) -> HashMap<NodeId, Rc<T>>` using a recursive visitor that detects already-visited pointers.
- Mark internal pointer comparisons as `unsafe` inside a private module; expose only safe public API.
- Test with a simple linked list containing a cycle.

---

### Iteration 2.2: Graph Extraction

Define:

```rust
pub struct ReifiedGraph<T> {
    pub nodes: Vec<(NodeId, T)>,
    pub edges: Vec<(NodeId, NodeId)>,
    pub root: NodeId,
}
```

Implement `fn reify_graph<T: Clone>(root: Rc<RefCell<T>>) -> ReifiedGraph<T>` where `T` exposes its child `Rc` pointers via a user-provided closure:

```rust
pub fn reify_graph<T, F>(root: Rc<RefCell<T>>, children: F) -> ReifiedGraph<T>
where
    F: Fn(&T) -> Vec<Rc<RefCell<T>>>,
    T: Clone;
```

Test with a DAG of at least 5 nodes with shared children.

---

### Iteration 2.3: Graph Reconstruction (`reflect_graph`)

Implement the inverse:

```rust
pub fn reflect_graph<T, F>(
    graph: ReifiedGraph<T>,
    set_children: F,
) -> Rc<RefCell<T>>
where
    F: Fn(&mut T, Vec<Rc<RefCell<T>>>),
    T: Clone;
```

Test a round-trip: reify a graph, serialize node values to JSON (using `serde_json`), deserialize, reconstruct, and verify structural equality.

---

### Iteration 2.4: `serde` Integration

Add a `serde` feature flag. When enabled:
- Implement `Serialize`/`Deserialize` for `ReifiedGraph<T: Serialize + DeserializeOwned>`.
- Add an example in `examples/serialize_graph.rs` demonstrating serializing a cyclic structure to JSON and reconstructing it.

---

## Phase 3: Local Trait Contexts (`context-trait`)

**Goal:** Enable runtime-synthesized trait instances scoped to a callback, using function pointer tables.

**Crate:** `context-trait`

### Iteration 3.1: `WithContext` Wrapper Type

Define:

```rust
pub struct WithContext<T, Ctx> {
    pub inner: T,
    pub ctx: Ctx,
}
```

Define a `OrdContext<T>` struct holding a comparator:

```rust
pub struct OrdContext<T> {
    pub compare: fn(&T, &T) -> std::cmp::Ordering,
}
```

Implement `PartialEq`, `Eq`, `PartialOrd`, `Ord` for `WithContext<T, OrdContext<T>>` by dispatching through `ctx.compare`.

---

### Iteration 3.2: `HashContext` and `DisplayContext`

Following the same pattern as `OrdContext`:

- `HashContext<T>` with `hash: fn(&T, &mut dyn Hasher)`: implement `Hash` for `WithContext<T, HashContext<T>>`.
- `DisplayContext<T>` with `display: fn(&T, &mut fmt::Formatter) -> fmt::Result`: implement `Display`.

Test: sort a `Vec<WithContext<MyStruct, OrdContext<MyStruct>>>` using a custom comparator, assert order.

---

### Iteration 3.3: `with_ord!` and `with_hash!` Macros

Write declarative macros:

```rust
with_ord!(items, |a: &Item, b: &Item| a.score.cmp(&b.score), |wrapped| {
    let mut sorted = wrapped.clone();
    sorted.sort();
    // use sorted...
});
```

The macro should: wrap each item in `WithContext`, run the callback, and unwrap results transparently. Write tests demonstrating use with `BTreeSet` and `sort()`.

---

### Iteration 3.4: Generic `ContextImpl` Macro

Provide a macro `impl_context_trait!` that lets users define new context types for arbitrary traits following the same pattern, so the library is extensible beyond the built-in `Ord`/`Hash`/`Display`.

Document with a full example deriving a custom `Summarize` trait context.

---

## Phase 4: Async Computation Inspector (`async-reify`)

**Goal:** Instrument async functions to extract their continuation structure as an inspectable, serializable step graph.

**Crate:** `async-reify`

### Iteration 4.1: `TracedFuture` Wrapper

Define `TracedFuture<F: Future>` that wraps any future and records each poll event:

```rust
pub struct PollEvent {
    pub step: usize,
    pub timestamp: std::time::Instant,
    pub result: PollResult,  // enum { Pending, Ready }
}
```

Implement `Future for TracedFuture<F>` by delegating to the inner future and appending to an internal `Arc<Mutex<Vec<PollEvent>>>`.

Test: run a simple multi-step async function under a test executor and assert the poll event count.

---

### Iteration 4.2: `.await` Point Labeling

Provide an attribute macro `#[trace_async]` that rewrites an async function body, wrapping each `.await` expression with a labeled `TracedFuture` carrying the source file, line number, and optional user-supplied name:

```rust
#[trace_async]
async fn my_workflow() {
    fetch_data().await;           // auto-labeled "fetch_data @ lib.rs:12"
    #[label = "transform"]
    process().await;
}
```

Deliverable: The macro compiles and labels are captured in `PollEvent`.

---

### Iteration 4.3: Step Graph Extraction

Define:

```rust
pub struct AsyncStepGraph {
    pub steps: Vec<StepNode>,
    pub edges: Vec<(usize, usize)>,  // sequential and branch edges
}

pub struct StepNode {
    pub id: usize,
    pub label: String,
    pub duration_us: u64,
    pub outcome: StepOutcome,  // enum { Completed, Pending, Cancelled }
}
```

Implement `fn reify_execution(trace: Vec<PollEvent>) -> AsyncStepGraph`.

Test: run a branching async function, extract graph, assert correct node and edge counts.

---

### Iteration 4.4: Serialization and Replay Stub

- Add `serde` support for `AsyncStepGraph`.
- Write a `fn to_dot(graph: &AsyncStepGraph) -> String` that outputs a Graphviz DOT representation.
- Add an example in `examples/trace_workflow.rs` that runs an async function, prints the DOT graph, and writes the serialized JSON trace to a temp file.
- Note in docs: full replay (deterministic re-execution) is a future milestone requiring integration with a controlled executor.

---

## Phase 5: Unsafe `reify` Bridge for Const Generics

**Goal:** A runtime-to-const-generic bridge using unsafe vtable fabrication. This is the most experimental phase.

**Crate:** `const-reify`. **Explicitly `unsafe` core. `#[deny(unsafe_code)]` is NOT set here. All unsafe blocks must have `// SAFETY:` comments.**

### Iteration 5.1: Design Spike and Safety Audit Doc

Before writing any code, produce a `DESIGN.md` in the crate root covering:

- Which rustc internals are being relied upon (monomorphization layout, vtable structure).
- Which compiler versions this is known to work on (start with current stable).
- A threat model: what breaks if rustc changes its vtable layout.
- A mitigation plan: compile-time assertions (`static_assertions` crate) to catch layout changes early.

This document must be reviewed/acknowledged before proceeding to 5.2.

---

### Iteration 5.2: Trait Object Layout Introspection

Implement a test harness that:
- Defines a trait `HasValue { fn value(&self) -> u64; }`
- Implements it for `Modular<1>` through `Modular<8>` (const generic struct).
- Uses `std::mem::transmute` and raw pointer arithmetic to inspect the vtable pointers of these trait objects.
- Asserts that the vtable layout is consistent across consecutive const values.

This is a pure research iteration. Deliverable: a passing test that documents vtable layout assumptions.

---

### Iteration 5.3: Runtime Vtable Fabrication

Implement:

```rust
/// SAFETY: caller must ensure `n` fits the valid range of `N` for the impl.
pub unsafe fn reify_const<const N: u64, T, F, R>(val: u64, f: F) -> R
where
    T: HasModulus,
    F: FnOnce(&dyn HasModulus) -> R;
```

Using the vtable introspection from 5.2, fabricate a vtable for the appropriate monomorphization and call `f` with the resulting trait object reference.

Restrict to `u64` values in `0..=255` for the initial implementation (reduces monomorphization explosion).

---

### Iteration 5.4: Safe Public API Wrapper and `reify!` Macro

Wrap the unsafe core in a safe macro:

```rust
reify!(17u64 as Modular, |m: &dyn HasModulus| {
    println!("modulus = {}", m.modulus());
});
```

The macro should perform bounds checking (panic if out of supported range) and call the unsafe internals. Document clearly that this pins to specific rustc versions. Add a `#[cfg(rustc_version)]` gate or a build script that validates the compiler version and emits a warning if unrecognized.

---

## Phase 6: Integration, Polish, and Workspace Crate

**Goal:** Wire all crates together, write cross-crate integration tests, and publish a top-level `reify-reflect` facade crate.

### Iteration 6.1: Workspace Facade Crate

Create `reify-reflect/src/lib.rs` that re-exports:
- `reify_reflect_core::*`
- `reflect_nat::*`
- `reify_graph::*`
- `context_trait::*`
- `async_reify::*`
- `const_reify` behind a `const-reify` feature flag (opt-in due to unsafe nature)

---

### Iteration 6.2: Cross-Crate Integration Tests

Write integration tests in `reify-reflect/tests/` covering:
- A struct using `#[derive(Reflect)]` whose fields are type-level naturals: reflect to `RuntimeValue`, assert.
- A graph of structs serialized via `reify_graph`, transmitted as JSON, and reconstructed.
- A `BTreeSet` sorted via `with_ord!` using a non-default comparator.
- An async workflow traced with `#[trace_async]`, step graph extracted and serialized to DOT.

---

### Iteration 6.3: Benchmarks

Add `benches/` using `criterion`:
- Benchmark `Reflect::reflect()` on a 5-level nested HList.
- Benchmark `reify_graph` on a graph of 1,000 nodes.
- Benchmark `with_ord!` sort vs. a plain closure sort on 10,000 elements (expect parity).

---

### Iteration 6.4: README and Documentation Site

- Write `README.md` at the workspace root with a quick-start, feature overview table, and links to each crate.
- Ensure `cargo doc --all-features --no-deps` builds without warnings.
- Add a `docs/` directory with one markdown file per phase explaining design decisions.

---

## Dependency Map

```
reify-reflect-core
    └── reflect-nat        (+ frunk, typenum)
    └── reflect-derive     (proc-macro, + syn, quote)

reify-graph                (+ serde optional)

context-trait              (no external deps)

async-reify                (+ tokio dev-dep, + serde optional)

const-reify                (unsafe, standalone)

reify-reflect (facade)
    └── all of the above
```

---

## Completion Checklist per Phase

Each phase is complete when all of the following pass:

- [ ] `cargo build --all-features` exits 0
- [ ] `cargo test --all-features` exits 0
- [ ] `cargo clippy --all-features -- -D warnings` exits 0
- [ ] `cargo fmt --check` exits 0
- [ ] `cargo doc --all-features --no-deps` exits 0 with no warnings
- [ ] All public items have doc comments with at least one example
- [ ] All `unsafe` blocks have `// SAFETY:` annotations
- [ ] CHANGELOG.md entry added for the phase
