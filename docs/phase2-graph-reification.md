# Phase 2: Graph Reification

## Design Decisions

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
