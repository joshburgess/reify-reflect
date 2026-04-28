# Phase 6: Integration, Polish, and Workspace Crate

## What this phase covers

Pulling everything in phases 1 through 5 together into one umbrella `reify-reflect` facade crate, plus the workspace-level polish that turned a collection of crates into a publishable, benchmarked, integration-tested whole.

This phase is what makes the project usable as a single dependency: pull in `reify-reflect` (with appropriate features), and you have access to every primitive in the workspace under a sensible namespacing. If you only need one piece, you depend on the focused crate directly and pay nothing for the rest.

## Crate introduced

- [`reify-reflect`](https://docs.rs/reify-reflect) (the facade)

Cross-crate integration tests in [`tests/`](../tests/) exercise the full pipeline; benchmarks in [`benches/`](../benches/) validate the cost model.

## Design decisions

### Module-based re-exports over glob re-exports

The facade crate uses named modules (`core`, `nat`, `graph`, `context`,
`async_trace`, `const_bridge`) rather than flat glob re-exports. This
avoids name collisions between crates and provides clear namespacing.

### Feature flag strategy

- `serde` (default): Most users want serialization for graphs and traces.
- `const-reify`: Opt-in because it adds 256 monomorphizations to compile time.
- `full`: Convenience for enabling everything.

### const-reify behind a feature flag

The `const-reify` crate generates 256 monomorphizations at compile time.
While the runtime cost is zero, the compilation cost is non-trivial. Making
it opt-in respects users who don't need runtime-to-const-generic bridging.

### Criterion benchmarks

Three benchmarks cover the performance-critical paths:
1. HList reflection (type-level computation overhead)
2. Graph reification at scale (1,000 nodes with cross-links)
3. WithContext sort vs. plain closure sort (overhead measurement)

The sort benchmark specifically validates that the function-pointer dispatch
in `WithContext` has no measurable overhead compared to a plain closure.

## Next

- [Documentation index](README.md)
- The roadmap RFCs in [`rfcs/`](rfcs/) sketch where the project goes after 0.1.0.
