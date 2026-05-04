# reify-reflect

[![CI](https://github.com/joshburgess/reify-reflect/actions/workflows/ci.yml/badge.svg)](https://github.com/joshburgess/reify-reflect/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/reify-reflect.svg)](https://crates.io/crates/reify-reflect)
[![Docs.rs](https://docs.rs/reify-reflect/badge.svg)](https://docs.rs/reify-reflect)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.75.0-orange.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

A unified ecosystem of small Rust crates for moving values back and forth between the **type level** and the **value level**, safely and ergonomically.

If you've ever wanted to:

- Read out a number, boolean, or struct shape encoded in the type system as a normal runtime value,
- Take a runtime value (a number from a config file, a length from user input) and "lift" it into the type system as a const generic,
- Reify a graph of `Rc`/`Arc` pointers into something you can serialize, inspect, or rebuild,
- Swap out a type's `Ord`, `Hash`, or `Display` implementation for a single block of code,
- Or extract the structure of an `async` function as an inspectable step graph,

...this is the workspace for that. Everything is `#![deny(unsafe_code)]`. There is no `unsafe`, no compiler-internal layout assumptions, and no `unsafeCoerce`-flavored tricks. Where Haskell's `reflection` library leans on GHC internals, this leans on the borrow checker.

## What "reification" and "reflection" mean here

Two directions, both useful:

- **Reflection**: type → value. A type like `S<S<S<Z>>>` carries the number 3 at compile time. `Three::reflect()` hands you `3` at runtime.
- **Reification**: value → type. You have a `u64` that is only known at runtime, and you want to use it where a `const N: u64` is required. `reify_nat(n, &cb)` enters a callback in which `N` really is a const generic, monomorphized at compile time.

Together they let you move freely between Rust's type system and its values without leaving safe code.

## Quick taste

```rust
use reify_reflect::core::{Reflect, RuntimeValue};
use reify_reflect::nat::{S, Z};

// Type → value
type Three = S<S<S<Z>>>;
assert_eq!(Three::reflect(), RuntimeValue::Nat(3));
```

```rust
use const_reify::reify_nat_fn;

// Value → type → value. Inside the closure, `n` is the const-generic
// monomorphization that matches the runtime input.
let squared = reify_nat_fn(12, |n| n * n);
assert_eq!(squared, 144);
```

Step-by-step guides are in [`docs/guides/`](docs/guides/).

## Where to start

Pick the entry point that matches what you're trying to do:

| You want to... | Start with | Read |
|---|---|---|
| Understand the whole pattern | `reify-reflect-core` + `reflect-nat` | [Guide 1: Reflect basics](docs/guides/01-reflect-basics.md) |
| Use a runtime value as a `const N: u64` | `const-reify` (+ `const-reify-derive`) | [Guide 3: const-reify](docs/guides/03-const-reify.md), [Guide 4: `#[reifiable]`](docs/guides/04-reifiable-macro.md) |
| Lift any value into a scoped type-level context | `reify-reflect-core::reify` | [Guide 2: branded reify](docs/guides/02-branded-reify.md) |
| Serialize an `Rc<RefCell<T>>` graph | `reify-graph` | [`reify-graph/examples/serialize_graph.rs`](reify-graph/examples/serialize_graph.rs) |
| Override `Ord`/`Hash`/`Display` for one block | `context-trait` | docs.rs page |
| Inspect what an `async fn` is doing | `async-reify` | [`async-reify/examples/trace_workflow.rs`](async-reify/examples/trace_workflow.rs) |
| Try the whole stack end to end | facade `reify-reflect` | [`examples/roundtrip.rs`](examples/roundtrip.rs) |

If you just want one dependency that re-exports everything, the facade crate is `reify-reflect`. If you want a leaner build, depend on the individual crates directly.

## Tour of the crates

```
reify-reflect                facade: re-exports everything below
├── reify-reflect-core       Reflect trait, reify(), Reified token, RuntimeValue
├── reflect-nat              Peano naturals, type-level booleans, HLists
├── reflect-derive           #[derive(Reflect)] for structs and enums
├── reify-graph              Rc<RefCell<T>> / Arc<Mutex<T>> graph reify+reflect
├── context-trait            Scoped Ord/Hash/Display swaps via WithContext
├── async-reify              Trace and reify async execution as a step graph
├── async-reify-macros       #[trace_async] attribute proc macro
├── const-reify              Safe runtime u64 → const generic dispatch (0..=255)
└── const-reify-derive       #[reifiable] proc macro for trait dispatch tables
```

Every crate has its own [docs.rs](https://docs.rs) page with a runnable example at the top. The [guides](docs/guides/) walk through the same material at tutorial pace.

## A few more flavors

### Derive `Reflect` on your own types

```rust
use reflect_derive::Reflect;
use reify_reflect_core::{Reflect, RuntimeValue};
use reflect_nat::{S, Z};

#[derive(Reflect)]
struct Config {
    width: S<S<Z>>,        // reflects to Nat(2)
    #[reflect(skip)]
    notes: String,         // not part of the type-level shape
}
```

### Reify a pointer graph for serialization

```rust
use reify_graph::{reify_graph, reflect_graph};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
struct Node { value: i32, children: Vec<Rc<RefCell<Node>>> }

let leaf = Rc::new(RefCell::new(Node { value: 1, children: vec![] }));
let root = Rc::new(RefCell::new(Node {
    value: 0,
    children: vec![leaf.clone()],
}));

let graph = reify_graph(root, |n| n.children.clone());
// `graph` is now a flat node+edge structure you can serialize, inspect, or transform
let restored = reflect_graph(graph, |n, kids| n.children = kids);
```

### Swap a trait implementation for one scope

```rust
use context_trait::{with_ord, OrdContext, WithContext};

let items = vec![3i32, 1, 4, 1, 5];
with_ord!(items, |a: &i32, b: &i32| b.cmp(a),  // descending
    |wrapped: &[WithContext<i32, OrdContext<i32>>]| {
        let mut sorted = wrapped.to_vec();
        sorted.sort();  // uses the descending Ord we just supplied
        let values: Vec<i32> = sorted.into_iter().map(|w| w.inner).collect();
        assert_eq!(values, vec![5, 4, 3, 1, 1]);
    });
```

### Trace an async workflow

```rust
use async_reify::{TracedFuture, reify_execution, to_dot};

# tokio_test::block_on(async {
let (result, trace) = TracedFuture::run(async { 42 }).await;
let graph = reify_execution(trace.events);
println!("{}", to_dot(&graph));    // Graphviz output you can render
# });
```

### Use a runtime value as a const generic

```rust
use const_reify::nat_reify::{NatCallback, reify_nat};

struct Square;
impl NatCallback<u64> for Square {
    fn call<const N: u64>(&self) -> u64 { N * N }
}

let n: u64 = 7;                                  // runtime input
assert_eq!(reify_nat(n, &Square), 49);           // dispatched into call::<7>
```

For traits with multiple const-generic methods, `#[reifiable(range = 0..=255)]` on the trait declaration generates the dispatch wrappers automatically. See [Guide 4](docs/guides/04-reifiable-macro.md).

## Feature flags

| Feature | Default | Effect |
|---|---|---|
| `serde` | yes | `Serialize`/`Deserialize` for `reify-graph` and `async-reify` types |
| `const-reify` | no | Re-export the `const_bridge` module (adds 256 monomorphizations to compile time) |
| `typenum` | no | Bridge between `reflect-nat` and the `typenum` crate |
| `frunk` | no | Bridge between `reflect-nat` and `frunk`'s HList |
| `full` | no | All of the above |

## Documentation

- API docs: `cargo doc --features full --no-deps --open`
- Step-by-step tutorials: [`docs/guides/`](docs/guides/)
- Forward-looking RFCs: [`docs/rfcs/`](docs/rfcs/)
- End-to-end example: [`examples/roundtrip.rs`](examples/roundtrip.rs) (`cargo run --example roundtrip --features full`)

## Benchmarks

```sh
cargo bench --features full
```

The headline numbers (Criterion, on the maintainer's laptop): HList reflection is ~free, graph reification scales linearly to thousands of nodes, and `WithContext`-based sorts are within noise of plain closure sorts. Treat these as smoke tests, not promises: re-run them on your hardware before relying on them.

## How this compares to Haskell's `reflection`

| | Haskell `reflection` | `reify-reflect` |
|---|---|---|
| Scoping mechanism | Rank-2 `forall s` | `for<'brand>` lifetimes / `NatCallback` trait |
| Internal safety | `unsafeCoerce` | No `unsafe` anywhere |
| Range | Unbounded | `0..=255` per dispatch (extensible by composition) |
| Closures over the brand | Yes | Trait impl, or one of the `#[reifiable]` / `reify_nat_fn` shortcuts |
| Cost model | Dictionary lookup per call | Direct, fully monomorphized dispatch |

The Rust version trades unbounded range for monomorphized performance and a fully safe surface area.

## Status

See [`CHANGELOG.md`](CHANGELOG.md) for what shipped in 0.1.0 and 0.1.1. The API is consistent and tested, but this is still an early release. Expect breaking changes before 1.0, and please file issues if anything in the docs is unclear or wrong.

## License

Dual licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.
