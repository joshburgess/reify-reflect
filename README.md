# reify-reflect

[![CI](https://github.com/joshburgess/reify-reflect/actions/workflows/ci.yml/badge.svg)](https://github.com/joshburgess/reify-reflect/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/reify-reflect.svg)](https://crates.io/crates/reify-reflect)
[![Docs.rs](https://docs.rs/reify-reflect/badge.svg)](https://docs.rs/reify-reflect)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.75.0-orange.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

Unified reification and reflection ecosystem for Rust.

## Crates

| Crate | Description |
|---|---|
| `reify-reflect-core` | `Reflect` trait, `reify` function, `Reified` token, `RuntimeValue` enum |
| `reflect-nat` | Type-level naturals (`Z`/`S<N>`), booleans, HLists, optional `frunk`/`typenum` bridges |
| `reflect-derive` | `#[derive(Reflect)]` proc macro |
| `reify-graph` | `Rc<RefCell<T>>` and `Arc<Mutex<T>>` graph reification and reconstruction |
| `context-trait` | Runtime-synthesized trait instances (Ord, Hash, Display) |
| `async-reify` | Async computation step graph extraction |
| `async-reify-macros` | `#[trace_async]` attribute proc macro for `async-reify` |
| `const-reify` | Runtime-to-const-generic dispatch via match-table |
| `const-reify-derive` | `#[reifiable]` proc macro that generates const-generic dispatch tables |

## Quick Start

```rust
use reify_reflect::core::{Reflect, RuntimeValue};
use reify_reflect::nat::{S, Z};

type Three = S<S<S<Z>>>;
assert_eq!(Three::reflect(), RuntimeValue::Nat(3));
```

### Derive Reflect for structs

```rust
use reflect_derive::Reflect;
use reify_reflect_core::{Reflect, RuntimeValue};

#[derive(Reflect)]
struct Config {
    x: S<S<Z>>,        // reflects to Nat(2)
    #[reflect(skip)]
    _internal: String,  // skipped
}
```

### Reify pointer graphs

```rust
use reify_graph::{reify_graph, reflect_graph};

fn round_trip(root: Rc<RefCell<Node>>) -> Result<Rc<RefCell<Node>>, serde_json::Error> {
    let graph = reify_graph(root, |n| n.children.clone());
    let json = serde_json::to_string(&graph)?;
    let restored = serde_json::from_str(&json)?;
    Ok(reflect_graph(restored, |n, kids| n.children = kids))
}
```

### Custom trait instances

```rust
use context_trait::{with_ord, WithContext, OrdContext};

with_ord!(items, |a: &Item, b: &Item| a.score.cmp(&b.score),
    |wrapped: &[WithContext<Item, OrdContext<Item>>]| {
        let mut sorted = wrapped.to_vec();
        sorted.sort();
    }
);
```

### Trace async workflows

```rust
use async_reify::{TracedFuture, reify_execution, to_dot};

let (result, trace) = TracedFuture::run(my_async_fn()).await;
let graph = reify_execution(trace.events);
println!("{}", to_dot(&graph));
```

### Runtime-to-const-generic dispatch

```rust
use const_reify::{reify_const, HasModulus};

let result = reify_const(42, |m| m.modulus() * 2);
assert_eq!(result, 84);
```

## Features

| Feature | Default | Description |
|---|---|---|
| `serde` | yes | Serialization for `reify-graph` and `async-reify` |
| `const-reify` | no | Runtime-to-const-generic dispatch (adds compile time) |
| `full` | no | All features |

## Documentation

- `cargo doc --features full --no-deps --open`
- Design docs in [`docs/`](docs/), one per phase

## Benchmarks

```sh
cargo bench --features full
```

## License

Dual licensed under [MIT](LICENSE-MIT) or [APACHE 2.0](LICENSE-APACHE).
