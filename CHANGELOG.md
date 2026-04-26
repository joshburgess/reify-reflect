# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - Unreleased

### Phase 1: Foundations (unified Reify/Reflect trait family)
- `reify-reflect-core`: `Reflect` trait, `Reified<'brand, T>` branded token, `reify` scoped function, `RuntimeValue` enum
- `reflect-nat`: Peano naturals (`Z`/`S<N>`), `Add`/`Mul`/`Lt` type-level arithmetic, `N0`–`N8` aliases
- `reflect-nat`: Type-level booleans (`True`/`False`), `Not`/`And`/`Or` operations
- `reflect-nat`: Heterogeneous lists (`HNil`/`HCons`), `HList` trait with `len()`/`is_empty()`
- `reflect-nat`: Optional `frunk` and `typenum` bridges (feature-gated)
- `reflect-derive`: `#[derive(Reflect)]` proc macro for structs (named + tuple) and enums, with `#[reflect(skip)]` support

### Phase 2: Graph reification
- `reify-graph`: `NodeId` from pointer identity, `collect_nodes` with cycle detection
- `reify-graph`: `reify_graph` extracts `ReifiedGraph<T>` from `Rc<RefCell<T>>` graphs
- `reify-graph`: `reify_graph_arc` / `reflect_graph_arc` for `Arc<Mutex<T>>` graphs
- `reify-graph`: `reflect_graph` reconstructs graphs preserving sharing
- `reify-graph`: `serde` feature for `Serialize`/`Deserialize` on `ReifiedGraph`

### Phase 3: Local trait contexts
- `context-trait`: `WithContext<T, Ctx>` wrapper type
- `context-trait`: `OrdContext`, `HashContext`, `DisplayContext` built-in contexts
- `context-trait`: `with_ord!`, `with_hash!`, `with_display!` scoped macros
- `context-trait`: `impl_context_trait!` macro for user-defined contexts

### Phase 4: Async computation inspector
- `async-reify`: `TracedFuture` wrapper recording `PollEvent`s
- `async-reify`: `LabeledFuture` and `labeled_await!` macro for source-labeled traces
- `async-reify`: `#[trace_async]` attribute proc macro (in `async-reify-macros`, re-exported via the `macros` feature) rewrites every `.await` in the function body to a `LabeledFuture` recording into a shared `Trace`. Auto-generates labels of the form `"<expr> @ <file>:<line>"`.
- `async-reify`: `reify_execution` extracts `AsyncStepGraph` from poll events
- `async-reify`: `to_dot` renders step graphs as Graphviz DOT
- `async-reify`: `serde` feature for serialization of `PollEvent`, `Trace`, and `AsyncStepGraph` (timestamps stored as `Duration` since trace start)

### Phase 5: Const-generic bridge
- `const-reify`: `Modular<const N: u64>` and `HasModulus` trait
- `const-reify`: `reify_const` safe match-table dispatch (0..=255)
- `const-reify`: `reify!` convenience macro
- `const-reify`: `DESIGN.md` documenting the match-table approach over vtable fabrication
- `const-reify`: `nat_reify` module with `NatCallback`/`Nat2Callback` for true const-generic dispatch
- `const-reify-derive`: `#[reifiable]` proc macro generating dispatch tables and `NatCallback` wrappers for user traits

### Phase 6: Integration
- `reify-reflect` facade crate re-exporting all components
- Cross-crate integration tests covering all phases
- Feature flags: `serde` (default), `const-reify`, `typenum`, `frunk`, `full`
