# Phase 3: Local Trait Contexts

## What this phase covers

A way to swap a type's `Ord`, `Hash`, or `Display` implementation for one block of code, without writing a newtype.

The orthodox Rust answer to "I need a different `Ord` for this `Vec<i32>` just for this one sort" is to wrap the value in a newtype with a custom impl. That works, but it's heavyweight: a new struct, manual trait impls, and you lose the existing impls on the inner type.

This phase introduces `WithContext<T, Ctx>`, a thin wrapper that pairs a value with a *context*: a small `Copy` struct of function pointers that supplies the trait implementation. Three built-in contexts (`OrdContext`, `HashContext`, `DisplayContext`) cover the common cases. For arbitrary user traits, `impl_context_trait!` generates new context types.

## Crate introduced

- [`context-trait`](https://docs.rs/context-trait)

## Design decisions

### Function pointer tables, not closures

Contexts store `fn` pointers rather than `Fn` trait objects. This makes
context types `Copy` regardless of `T`, which is essential for ergonomic
use with `BTreeSet`, `sort()`, and other stdlib APIs that clone or copy
wrapper values. The tradeoff is no captured state, but this matches the
"scoped instance" use case where the comparison logic is stateless.

### Manual Copy/Clone/Debug impls

Context types implement `Copy`, `Clone`, and `Debug` manually rather than
via `derive`. This is because `derive(Copy)` adds a `T: Copy` bound, but
our context structs only contain `fn` pointers (always `Copy`), and the
`T` parameter is purely phantom. Manual impls avoid this spurious bound.

### WithContext is generic over Ctx

`WithContext<T, Ctx>` is parameterized over the context type rather than
using a fixed context. This enables the `impl_context_trait!` macro to
generate new context types for arbitrary user traits, making the library
extensible beyond the built-in Ord/Hash/Display contexts.

### Macros require explicit type annotations on closures

The `with_ord!`/`with_hash!`/`with_display!` macros pass the callback a
slice reference, which sometimes requires the user to annotate the closure
parameter type. This is a Rust type inference limitation with closures
passed through macros. We chose clarity over magic: the type annotation
makes the API self-documenting.

## Next

- [Phase 4: Async computation inspector](phase4-async-reify.md)
- [Documentation index](README.md)
