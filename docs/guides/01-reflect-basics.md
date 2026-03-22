# Guide 1: Type-Level Values and Reflection

## The Problem

Rust's type system can encode values at the type level — a type like
`S<S<S<Z>>>` represents the number 3, known at compile time. But there's
no built-in way to get that 3 back as a runtime `u64`.

Going the other direction is even harder: given a `u64` whose value is
only known at runtime, how do you "enter" the type system with it?

This guide covers the first direction: **type → value** (reflection).

## Peano Naturals

The `reflect-nat` crate encodes natural numbers using Peano arithmetic:

```rust
use reflect_nat::{Z, S};

type Zero  = Z;           // 0
type One   = S<Z>;        // 1
type Two   = S<S<Z>>;     // 2
type Three = S<S<S<Z>>>;  // 3
```

`Z` is zero. `S<N>` is "successor of N" — i.e., N + 1. These are
zero-sized types. They exist only in the type system; they take up
no memory at runtime.

## The Reflect Trait

The `Reflect` trait converts type-level values to runtime values:

```rust
use reflect_core::{Reflect, RuntimeValue};
use reflect_nat::{S, Z, N5};

// Type-level 3 → runtime Nat(3)
assert_eq!(<S<S<S<Z>>>>::reflect(), RuntimeValue::Nat(3));

// Convenience aliases exist for N0 through N8
assert_eq!(N5::reflect(), RuntimeValue::Nat(5));
```

`Reflect` is a trait with a static method — no instance needed:

```rust
pub trait Reflect {
    type Value;
    fn reflect() -> Self::Value;
}
```

## Type-Level Booleans

```rust
use reflect_nat::{True, False};
use reflect_core::Reflect;

assert_eq!(True::reflect(), true);
assert_eq!(False::reflect(), false);
```

## Type-Level Arithmetic

Arithmetic happens entirely at compile time via trait resolution:

```rust
use reflect_nat::{Z, S, N2, N3, N5, Add, Mul, Lt, Nat};

// 2 + 3 = 5 (computed by the compiler)
type Sum = <N2 as Add<N3>>::Result;
assert_eq!(Sum::reflect(), RuntimeValue::Nat(5));

// 2 * 3 = 6
type Product = <N2 as Mul<N3>>::Result;
assert_eq!(Product::reflect(), RuntimeValue::Nat(6));

// 2 < 5 is true
assert!(<N2 as Lt<N5>>::VALUE);
```

The compiler resolves `<N2 as Add<N3>>::Result` to `S<S<S<S<S<Z>>>>>`
(which is 5) entirely during type checking. No runtime cost.

## Heterogeneous Lists

Type-level lists can hold different types of type-level values:

```rust
use reflect_nat::{HNil, HCons, Z, S, N3};
use reflect_core::{Reflect, RuntimeValue};

type MyList = HCons<N3, HCons<S<Z>, HCons<Z, HNil>>>;

assert_eq!(
    MyList::reflect(),
    vec![RuntimeValue::Nat(3), RuntimeValue::Nat(1), RuntimeValue::Nat(0)]
);
```

## Derive Reflect for Structs

The `reflect-derive` crate provides `#[derive(Reflect)]` for structs:

```rust
use reflect_derive::Reflect;
use reflect_core::{Reflect, RuntimeValue};
use reflect_nat::{N3, N5};

#[derive(Reflect)]
struct Config {
    width: N3,
    height: N5,
    #[reflect(skip)]  // excluded from reflection
    _internal: String,
}

let reflected = Config::reflect();
// Returns a RuntimeValue::List of (field_name, field_value) pairs
```

Fields must implement `Reflect`. Use `#[reflect(skip)]` to exclude fields
that don't.

## Summary

| What | How | Direction |
|---|---|---|
| `S<S<S<Z>>>` → `3` | `Reflect::reflect()` | Type → Value |
| `True` → `true` | `Reflect::reflect()` | Type → Value |
| `HCons<N3, HNil>` → `vec![Nat(3)]` | `Reflect::reflect()` | Type → Value |
| `#[derive(Reflect)]` | Proc macro | Struct → RuntimeValue |

Next guide: [Branded Reification](02-branded-reify.md) — going the other direction.
