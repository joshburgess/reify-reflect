# RFC 0001: Rank-2 Const Quantification (`for<const N: u64>`)

- **Status**: Pre-RFC / Design Exploration
- **Context**: reflect-rs reification problem

## Summary

Extend Rust's `for<...>` higher-rank syntax to support const generic parameters, enabling closures and function pointers to be polymorphic over const values.

```rust
fn reify<R>(val: u64, f: impl for<const N: u64 where N <= 255> FnOnce() -> R) -> R;
```

## Motivation

Rust has `for<'a>` for rank-2 lifetime polymorphism:

```rust
fn apply(f: impl for<'a> Fn(&'a str) -> &'a str) { ... }
```

There is no analog for const generics. You cannot write:

```rust
fn reify(val: u64, f: impl for<const N: u64> FnOnce() -> R) -> R
```

### The problem this solves

Runtime-to-type-level dispatch (reification) requires a function that works
"for any const N" where the caller (the dispatch table) chooses N. Today the
only workaround is:

```rust
// Define a trait with a const-generic method
trait NatCallback<R> {
    fn call<const N: u64>(&self) -> R;
}

// Implement it on a struct
struct Square;
impl NatCallback<u64> for Square {
    fn call<const N: u64>(&self) -> u64 { N * N }
}

// Pass the struct
reify_nat(42, &Square);
```

What should be a one-line closure becomes a struct definition, a trait
implementation, and a function call. This is the same ergonomic cliff that
motivated `for<'a>` for lifetimes.

### What we want to write

```rust
let offset = 100u64;
reify_nat(42, |<const N: u64>| N + offset);
// Returns 142
```

The closure captures `offset` from the environment and is polymorphic in `N`.
The dispatch table calls it with `N = 42`.

## Detailed Design

### Syntax

`for<const IDENT: TYPE>` in any position where `for<'a>` currently works:

```rust
// Function pointers
type ConstPolyFn = for<const N: u64> fn() -> u64;

// Trait bounds
fn reify<R>(val: u64, f: impl for<const N: u64> FnOnce() -> R) -> R;

// Where clauses
fn foo<F>(f: F) where F: for<const N: u64> Fn() -> u64;

// Bounded variant (recommended for V1)
fn bar(f: impl for<const N: u64 where N <= 255> FnOnce() -> u64);

// Multiple const parameters
fn baz(f: impl for<const A: u64, const B: u64 where A <= 15, B <= 15> Fn() -> u64);
```

### Closure syntax

Const-generic closures use `|<const IDENT: TYPE>|` to introduce the parameter:

```rust
let f = |<const N: u64>| N * N;
let g = |<const N: u64>| {
    // N is a const generic in scope — can construct types parameterized by N
    let arr: [u8; N] = [0; N];
    arr.len() as u64
};
```

The `<const N: u64>` is visually distinct from regular closure parameters,
making it clear this is const-polymorphic.

### Semantics

A value of type `for<const N: u64 where N <= 255> FnOnce() -> R` is a
function that can be called at **any** const-generic instantiation within
the bound. The caller chooses `N`; the callee must work for all `N`.

This is rank-2 quantification: the type is `∀(N: u64, N ≤ 255). () → R`,
with the quantifier inside the function type.

### Monomorphization strategy

**V1 (recommended): Bounded enumeration only.**

When the bound is finite (`where N <= 255`), the compiler generates all
instantiations and emits a dispatch table — exactly what a hand-written
match table does, but automated:

```rust
// User writes:
reify(42, |<const N: u64>| N * N);

// Compiler generates (conceptually):
match 42u64 {
    0 => { const N: u64 = 0; N * N },
    1 => { const N: u64 = 1; N * N },
    ...
    255 => { const N: u64 = 255; N * N },
    _ => unreachable!(), // bound guarantees val <= 255
}
```

The bound is required in V1. Unbounded `for<const N: u64>` without a `where`
clause would be a future extension requiring a different compilation strategy
(dictionary passing or JIT specialization).

**Compile-time cost**: For a bound of `0..=B`, the compiler generates `B + 1`
monomorphizations per call site. The compiler should warn when the total
exceeds a configurable threshold (default: 1024).

### Interaction with dyn safety

`for<const N: u64>` bounds (unbounded) are **not dyn-safe**, because the
vtable cannot represent an infinite family of methods.

Bounded `for<const N: u64 where N <= 255>` is theoretically dyn-safe (the
vtable would have 256 entries), but this is deferred to a future RFC to
limit scope.

### Interaction with type inference

Inference is driven by the expected type, analogous to `for<'a>`:

```rust
fn reify<R>(val: u64, f: impl for<const N: u64 where N <= 255> FnOnce() -> R) -> R;

// The bound on N is inferred from reify's signature
reify(42, |<const N>| N * N);

// Explicit bound annotation also allowed
reify(42, |<const N: u64 where N <= 255>| N * N);
```

The `<const N>` syntax on the closure is required — without it, the compiler
cannot distinguish a const-polymorphic closure from a regular one.

### Interaction with captures

Const-generic closures capture environment exactly like regular closures:

```rust
let offset = 100u64;
let data = vec![1, 2, 3];

reify(42, |<const N: u64>| {
    // `offset` captured by copy, `data` captured by reference
    N + offset + data.len() as u64
});
```

Each monomorphization shares the same captured environment. The closure's
layout is identical across all instantiations — only the generated code
differs.

### Interaction with return-type-dependent-on-N

```rust
let f = |<const N: u64>| -> [u8; N] { [0u8; N] };
```

This is valid — each monomorphization returns a different-sized array. However,
the caller must handle this: the return type varies per instantiation, so the
dispatch site needs to erase or unify the return type. In practice this means
either:

- The return type must be independent of `N` (common case)
- The caller must immediately process the result within the same dispatch scope
- A future "bounded existential" feature (see RFC 0002) handles returning
  N-dependent types

## Backwards Compatibility

Fully backwards compatible. `for<const ...>` is new syntax in positions where
no valid syntax currently exists.

## Alternatives Considered

### 1. Trait + struct workaround (status quo)

Works but has severe ergonomic cost. Every "closure" becomes a struct + trait
impl. This is what motivated `for<'a>` for lifetimes; the same argument
applies here.

### 2. Compiler-magic `reify` built-in

A special `reify` keyword or intrinsic. Too narrow — `for<const>` is a
general-purpose feature that enables reification as one use case among many.

### 3. Full dependent types

`for<const N: u64>` is the minimal point on the design space that solves the
problem. Full dependent types (terms in types, types in terms) are a much
larger language change with deep consequences for type inference, compilation,
and the borrow checker.

### 4. Only support `for<const>` on named traits, not closures

This is what we have today (the `NatCallback` pattern). Adding `for<const>`
on closures is the specific ergonomic improvement this RFC targets.

## Open Questions

1. **Unbounded quantification**: Should `for<const N: u64>` without a `where`
   clause ever be allowed? This would require dictionary passing (Strategy B),
   which conflicts with Rust's monomorphization model.

2. **Const types beyond primitives**: Should `for<const S: &str>` work?
   `for<const T: SomeType>` where `SomeType: ConstParamTy`? Probably yes,
   but the range semantics need definition.

3. **Composition of bounds**: `for<const A: u64, const B: u64 where A + B <= 255>`
   — should the bound be an arbitrary const expression? This intersects with
   const evaluation and may be hard to enumerate.

4. **Inference of bounds**: Can the compiler infer the bound from usage context
   without explicit annotation? E.g., if `reify` is only called with values
   known to be `<= 255`, can the bound propagate?

5. **Standard library integration**: Should `core::ops::Fn` traits be
   extended, or should this use new traits? Extending `Fn` is more ergonomic
   but a larger change.

## Prior Art

- **Rust `for<'a>`**: Direct analog for lifetimes. This RFC extends the same
  mechanism to const generics.
- **Haskell `RankNTypes`**: `forall a. ...` in arbitrary positions. Haskell's
  `reflection` library uses this for `reify`.
- **OCaml first-class modules**: `(module M : S)` packages existential types.
  Not directly analogous but solves similar "choose the implementation at
  runtime" problems.
- **Scala 3 polymorphic function types**: `[N] => (x: N) => N` — polymorphic
  lambdas. Similar motivation.
