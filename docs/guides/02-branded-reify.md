# Guide 2: Branded Reification (Value → Scoped Context)

## The Problem

We can go from types to values with `Reflect`. But what about the reverse?

Given a runtime value, can we "lift" it into a context where it behaves
like a type-level value — safely, without unsafe code?

## The Haskell Inspiration

In Haskell, the `reflection` library provides:

```haskell
reify :: a -> (forall s. Reifies s a => Proxy s -> r) -> r
```

This takes a runtime value and passes it to a callback where it's accessible
through a "phantom type" `s`. The `forall s` ensures the phantom type can't
escape the callback — it's scoped.

## Rust's `reify` Function

Our `reify` function achieves the same scoping guarantee using Rust's
borrow checker instead of Haskell's `forall`:

```rust
use reflect_core::reify;

let result = reify(&42i32, |token| {
    // token is a Reified<'brand, i32>
    // 'brand is a unique lifetime that can't escape this closure
    let val: &i32 = token.reflect();
    *val + 1
});
assert_eq!(result, 43);
```

## How the Branding Works

The `Reified<'brand, T>` token carries two things:
1. A reference to the value (`&T`)
2. An invariant lifetime `'brand` that makes each `reify` call unique

The function signature is:

```rust
pub fn reify<T: ?Sized, F, R>(val: &T, f: F) -> R
where
    F: for<'brand> FnOnce(Reified<'brand, T>) -> R,
```

The `for<'brand>` is key — it means the closure must work for *any*
lifetime `'brand`. Since the closure can't know what `'brand` is, it
can't return anything that contains `'brand`. This prevents the token
from escaping:

```rust
// This WON'T compile:
let escaped = reify(&42, |token| {
    token  // ERROR: token contains 'brand, which can't escape
});
```

## Why "Branded"?

The `'brand` lifetime is like a unique stamp on the token. Each call to
`reify` creates a fresh stamp. Two tokens from different `reify` calls
have different stamps, so they can't be confused:

```rust
reify(&10i32, |outer| {
    reify(&20i32, |inner| {
        // outer and inner have different 'brand lifetimes
        // the compiler tracks them separately
        let sum = outer.reflect() + inner.reflect();
        assert_eq!(sum, 30);
    })
});
```

## Compared to Haskell

| | Haskell `reflection` | Rust `reify` |
|---|---|---|
| Scoping mechanism | Rank-2 `forall s` (parametricity) | `for<'brand>` (borrow checker) |
| Safety basis | Semantic (parametricity argument) | Mechanical (compiler-enforced) |
| Escape prevention | Type variable `s` can't escape | Lifetime `'brand` can't escape |
| Unsafe code needed | Yes (`unsafeCoerce` internally) | No — fully safe |
| Works with any type | Yes | Yes |

The Rust version is safer: no `unsafeCoerce`, no reliance on compiler
internals. The scoping is mechanically enforced by the borrow checker.

## What This Doesn't Do

Branded reification gives you safe scoped access to a runtime value. But
the token is just a reference — you **can't** use it as a type parameter:

```rust
reify(&5u64, |token| {
    // This is just a &u64 with a fancy lifetime.
    // You CAN'T do: let arr: [u8; ???] = ...;
    // There's no way to use 5 as a const generic here.
    let val = token.reflect();
    *val + 1
});
```

For true value→type dispatch — where a runtime value becomes a const
generic — you need the next level: [Const-Generic Reification](03-const-reify.md).

## When to Use Branded Reify

- When you need scoped access to a value with a lifetime guarantee
- When you want to prevent a reference from leaking out of a context
- When composing with other `reify` calls (nesting works naturally)
- When working with non-integer types (`String`, `Vec`, custom types)

Branded reify works with **any type**. Const-generic reify (next guide)
only works with `u64` values in `0..=255`.
