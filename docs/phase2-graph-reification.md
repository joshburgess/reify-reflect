# Phase 2: Graph Reification

## What this phase covers

A small library for converting `Rc<RefCell<T>>` and `Arc<Mutex<T>>` pointer graphs into a flat node+edge form, and back.

This solves three problems that come up whenever pointer-shaped data needs to leave the program:

- Naively walking a graph with shared subtrees duplicates them.
- Naively walking a cyclic graph loops forever.
- Hand-rolled visitors get tangled up with `RefCell` borrow rules.

`reify_graph` handles all three with cycle detection, pointer-identity deduplication, and a closure-based children extractor that keeps the library non-invasive (no traits to implement on your `T`).

`reflect_graph` rebuilds the original pointer graph, restoring the same sharing topology exactly.

With the default `serde` feature, the flat representation serializes cleanly. That gives you JSON / postcard / etc. round-trips of arbitrary cyclic data essentially for free.

## Crate introduced

- [`reify-graph`](https://docs.rs/reify-graph) (with [`serialize_graph` example](../reify-graph/examples/serialize_graph.rs))

## Design decisions

### Pointer identity via Rc::as_ptr

Node identity is derived from `Rc::as_ptr` cast to `usize`. This is the
simplest reliable way to detect shared nodes without requiring user types
to implement `Hash` or `Eq`. The cast is safe (no dereference), and pointer
stability is guaranteed for the lifetime of the `Rc`.

### User-provided children closure

Rather than requiring a trait like `GraphNode`, the API takes a closure
`Fn(&T) -> Vec<Rc<RefCell<T>>>` to extract children. This keeps the library
non-invasive: users don't need to modify their types to use it.

### Clone-based extraction

Node data is cloned into the `ReifiedGraph`. This avoids lifetime complexity
and makes the graph freely serializable. For large node types, users can
wrap data in `Arc` to make cloning cheap.

### Reconstruction preserves sharing

`reflect_graph` builds one `Rc<RefCell<T>>` per `NodeId` and wires up
children by looking up the same `Rc` for shared node IDs. This ensures
that the original sharing topology is preserved exactly.

## Next

- [Phase 3: Local trait contexts](phase3-context-trait.md)
- [Documentation index](README.md)
