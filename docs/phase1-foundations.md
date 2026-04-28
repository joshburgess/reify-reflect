# Phase 1: Foundations (Unified Reify/Reflect Trait Family)

## What this phase covers

The trait vocabulary that everything else in the workspace builds on:

- The `Reflect` trait, which says "this type carries a compile-time value, and here is how to extract it at runtime."
- The `RuntimeValue` enum, a small structural type that's the standard payload for `Reflect` impls.
- The `reify` function and `Reified<'brand, T>` token, which together let you take any runtime value and use it inside a callback as a scoped, type-level fact.
- A first batch of `Reflect` instances on Peano naturals, type-level booleans, and HLists (`reflect-nat`).
- `#[derive(Reflect)]`, which generates `Reflect` impls for ordinary structs and enums (`reflect-derive`).

If you're new to the project, [Guide 1: Reflect basics](guides/01-reflect-basics.md) and [Guide 2: branded reify](guides/02-branded-reify.md) walk through the same material at tutorial pace.

## Crates introduced

- [`reify-reflect-core`](https://docs.rs/reify-reflect-core)
- [`reflect-nat`](https://docs.rs/reflect-nat)
- [`reflect-derive`](https://docs.rs/reflect-derive)

## Design decisions

### Reflect trait returns associated `Value` type

Rather than hardcoding `RuntimeValue` as the return type, `Reflect` uses an
associated `Value` type. This allows type-level booleans to reflect to `bool`
and HLists to reflect to `Vec<RuntimeValue>`, keeping the API natural for
each domain while maintaining a unified trait interface.

### Peano encoding for naturals

Peano encoding (`Z`/`S<N>`) was chosen over binary encoding because:
- It maps directly to trait-level recursion patterns
- Arithmetic operations (Add, Mul, Lt) have straightforward recursive impls
- It integrates cleanly with the `Reflect` trait via `Nat::to_u64()`

The tradeoff is compile-time cost for large numbers, which is acceptable for
the type-level programming use cases targeted here.

### HList reflects to Vec, not RuntimeValue::List

`HCons`/`HNil` reflect to `Vec<RuntimeValue>` rather than `RuntimeValue::List`
because HLists are containers of heterogeneous reflected values. Wrapping in
`RuntimeValue::List` would add an unnecessary layer of indirection for the
most common use case (iterating over reflected field values).

### Derive macro encodes field names as byte lists

Field names in `#[derive(Reflect)]` are encoded as
`RuntimeValue::List(bytes.map(Nat))` rather than introducing a
`RuntimeValue::String` variant. This keeps `RuntimeValue` minimal and
demonstrates that strings can be represented within the existing type-level
vocabulary.

## Next

- [Phase 2: Graph reification](phase2-graph-reification.md)
- [Documentation index](README.md)
