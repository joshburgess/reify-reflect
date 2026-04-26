# Reification and Reflection in Rust: Bridging Runtime and Type-Level

Haskell programmers have long enjoyed `Data.Reflection`: the ability
to take a runtime value and "lift" it into the type system, scoped to a
callback. Inside the callback, the value is available as a type parameter,
enabling type-safe abstractions parameterized by runtime data.

Rust doesn't have this. Until now.

`reify-reflect` is a collection of crates that implements the full
reification/reflection pattern in safe Rust, with no `unsafe` code,
no compiler-internal assumptions, and ergonomic macros that make it
practical.

## What Problem Does This Solve?

Imagine you're writing a cryptography library. You have modular
arithmetic parameterized by a modulus:

```rust
struct Mod<const M: u64> {
    value: u64,
}

impl<const M: u64> Mod<M> {
    fn new(v: u64) -> Self { Mod { value: v % M } }
    fn mul(self, other: Self) -> Self { Self::new(self.value * other.value) }
}
```

The type system enforces that you can't accidentally multiply values
from different moduli (`Mod<7>` and `Mod<13>` are different types).

But const generics must be known at compile time. If the modulus comes
from user input, a config file, or a protocol negotiation, you're stuck.
You'd have to erase the type safety by dropping down to plain `u64`.

**Reification solves this.** It takes the runtime modulus and enters a
context where `M` is a const generic, preserving the type safety.

## The Three Layers

### Layer 1: Reflect (Type → Value)

The simplest direction. Type-level values become runtime values:

```rust
use reify_reflect_core::{Reflect, RuntimeValue};
use reflect_nat::{S, Z};

type Three = S<S<S<Z>>>;
assert_eq!(Three::reflect(), RuntimeValue::Nat(3));
```

This works for naturals, booleans, heterogeneous lists, and any struct
with `#[derive(Reflect)]`.

### Layer 2: Branded Reify (Value → Scoped Reference)

Based on Kiselyov & Shan's "Implicit Configurations", adapted to Rust
using branded lifetimes instead of Haskell's `unsafeCoerce`:

```rust
use reify_reflect_core::reify;

let result = reify(&42i32, |token| {
    let val = token.reflect();
    *val + 1
});
assert_eq!(result, 43);
```

The `token` is branded with a unique lifetime that can't escape the
closure. The borrow checker enforces this mechanically, unlike Haskell
where it relies on parametricity.

This works with any type, but the value stays at the value level. You
can't use it as a const generic.

### Layer 3: Const-Generic Reify (Value → Type)

This is the main event. A runtime `u64` becomes a const generic `N`
inside a callback:

```rust
use const_reify::nat_reify::{NatCallback, reify_nat};

struct ModPow { base: u64, exp: u64 }

impl NatCallback<u64> for ModPow {
    fn call<const M: u64>(&self) -> u64 {
        // M is a REAL const generic. We can construct Mod<M>.
        let b = Mod::<M>::new(self.base);
        b.pow(self.exp).value
    }
}

// The modulus comes from runtime, but inside call<M>, it's type-level.
let modulus: u64 = 7;
let result = reify_nat(modulus, &ModPow { base: 3, exp: 6 });
assert_eq!(result, 1);  // Fermat's little theorem: 3^6 ≡ 1 (mod 7)
```

How? A 256-arm match table generated at compile time. Each arm calls
`callback.call::<0>()`, `callback.call::<1>()`, ..., `callback.call::<255>()`.
At runtime, the match selects the right monomorphization in O(1).

## The `#[reifiable]` Macro

Writing `NatCallback` impls is verbose. The `#[reifiable]` proc macro
eliminates the boilerplate entirely:

```rust
use const_reify_derive::reifiable;

#[reifiable(range = 0..=255)]
trait ModArith {
    fn pow_mod<const N: u64>(&self, base: u64, exp: u64) -> u64;
    fn mul_mod<const N: u64>(&self, a: u64, b: u64) -> u64;
    fn name(&self) -> &str;
}
```

The macro generates `reify_pow_mod` and `reify_mul_mod`: dispatch
functions that take a runtime `u64` and forward to the correct
const-generic instantiation. Non-const-generic methods (`name`) are
left alone.

Now you implement the trait normally:

```rust
struct FastMod;

impl ModArith for FastMod {
    fn pow_mod<const N: u64>(&self, base: u64, exp: u64) -> u64 {
        if N == 0 { return 0; }
        let mut result = 1u64;
        let mut b = base % N;
        let mut e = exp;
        while e > 0 {
            if e % 2 == 1 { result = result * b % N; }
            b = b * b % N;
            e /= 2;
        }
        result
    }

    fn mul_mod<const N: u64>(&self, a: u64, b: u64) -> u64 {
        if N == 0 { return 0; }
        (a % N) * (b % N) % N
    }

    fn name(&self) -> &str { "fast" }
}

// Use it with a runtime modulus:
let modulus = 13u64;
let result = reify_pow_mod(modulus, &FastMod, 2, 12);
assert_eq!(result, 1);  // Fermat's little theorem
```

## Quick One-Liners

For cases where you don't need the full const generic:

```rust
use const_reify::reify_nat_fn;

// Closure receives N as a plain u64
let squared = reify_nat_fn(12, |n| n * n);
assert_eq!(squared, 144);
```

For two values:

```rust
use const_reify::reify_nat2_fn;

let sum = reify_nat2_fn(100, 55, |a, b| a + b);
assert_eq!(sum, 155);
```

## Full Roundtrip

The whole ecosystem composes. Start at the type level, go to values,
go back to the type level, compute, return:

```rust
use reify_reflect_core::{Reflect, RuntimeValue};
use reflect_nat::N3;

// Step 1: Type-level → value
let base = match N3::reflect() {
    RuntimeValue::Nat(n) => n,
    _ => unreachable!(),
};

// Step 2: Get a runtime modulus (from user input, config, etc.)
let modulus = 7u64;

// Step 3: Value → type-level → compute → value
let result = reify_pow_mod(modulus, &FastMod, base, 6);
assert_eq!(result, 1);  // 3^6 mod 7 = 1
```

`S<S<S<Z>>>` → `3` → `reify(7, ...)` → `Mod<7>::pow(3, 6)` → `1`.

Type-level to value to type-level to value. Safe all the way.

## How It Compares to Haskell

| | Haskell `reflection` | Rust `reify-reflect` |
|---|---|---|
| API | `reify val $ \proxy -> ...` | `reify_nat(val, &callback)` |
| Scoping | Rank-2 `forall s` | `for<'brand>` / `NatCallback` trait |
| Safety | `unsafeCoerce` internally | No `unsafe` code anywhere |
| Range | Unbounded (dictionary passing) | 0..=255 (match-table dispatch) |
| Closures | Yes (Haskell has rank-2 closures) | Need trait impl or macro |
| Ergonomics | Excellent | Good with `#[reifiable]` |
| Performance | Dictionary lookup per operation | Direct dispatch, fully monomorphized |

The Rust version trades range (256 values vs. unbounded) for
performance (monomorphized code vs. dictionary lookup) and safety
(zero unsafe vs. `unsafeCoerce`).

## The Crates

| Crate | What it does |
|---|---|
| `reify-reflect-core` | `Reflect` trait, `reify` function, `RuntimeValue` |
| `reflect-nat` | Peano naturals, booleans, HLists |
| `reflect-derive` | `#[derive(Reflect)]` for structs |
| `const-reify` | `NatCallback`, `reify_nat`, `reify_nat_fn`, `def_nat_callback!` |
| `const-reify-derive` | `#[reifiable]` proc macro for automatic dispatch |
| `reify-graph` | Rc/Arc graph reification/reconstruction |
| `context-trait` | Runtime-synthesized trait instances |
| `async-reify` | Async step graph extraction |

## What's Next

The `#[reifiable]` macro covers V1: single const parameter, range
0..=255. On the roadmap:

- Multiple const parameters per method (nested dispatch)
- Async method support
- A `#[derive(ConstEnum)]` for bounded existential types: store a
  `Modular<N>` with runtime `N` and later recover `N`
- Exploring whether Rust could support `for<const N: u64>` natively
  ([RFC sketch](rfcs/0001-rank2-const-quantification.md))

## Try It

```toml
[dependencies]
reify-reflect-core = "0.1"
const-reify = "0.1"
const-reify-derive = "0.1"
```

Source: [github.com/joshburgess/reify-reflect](https://github.com/joshburgess/reify-reflect)
