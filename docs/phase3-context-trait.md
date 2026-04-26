# Phase 3: Local Trait Contexts

## Design Decisions

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
