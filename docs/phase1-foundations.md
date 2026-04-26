# Phase 1: Foundations (Unified Reify/Reflect Trait Family)

## Design Decisions

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
