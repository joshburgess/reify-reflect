# Guide 3: Const-Generic Reification (Value → Type-Level)

## The Gap

Branded reification (Guide 2) gives you scoped access to a runtime value,
but the value stays at the value level — you can't use it as a type
parameter. This guide covers the mechanism that actually bridges runtime
values into the type system.

## The Core Idea

Rust's const generics let you parameterize types by values:

```rust
struct Modular<const N: u64>;  // N is a compile-time constant

impl<const N: u64> Modular<N> {
    fn modulus(&self) -> u64 { N }
}
```

But `N` must be known at compile time. If you have a runtime `u64`, you
can't write `Modular<runtime_value>`.

The trick: **generate all 256 possible instantiations at compile time,
then match on the runtime value to pick the right one:**

```rust
fn reify_const(val: u64, f: impl FnOnce(&dyn HasModulus)) {
    match val {
        0 => f(&Modular::<0>),
        1 => f(&Modular::<1>),
        // ...
        255 => f(&Modular::<255>),
        _ => panic!("out of range"),
    }
}
```

This is a **match-table dispatch**. The compiler generates 256
monomorphizations. At runtime, a single integer comparison selects
the right one.

## Three Levels of Ergonomics

### Level 1: Closure-based (simplest, no const generic access)

```rust
use const_reify::reify_nat_fn;

let squared = reify_nat_fn(12, |n| n * n);
assert_eq!(squared, 144);

let sum = reify_nat2_fn(5, 3, |a, b| a + b);
assert_eq!(sum, 8);
```

The closure receives `N` as a plain `u64`. You can't construct types
parameterized by `N` — but for arithmetic, comparisons, and most
computations, this is all you need.

### Level 2: Macro-defined callback (const generic available)

```rust
use const_reify::{def_nat_callback, nat_reify::reify_nat};

def_nat_callback!(Factorial -> u64 {
    let mut result = 1u64;
    let mut i = 1u64;
    while i <= N {
        result *= i;
        i += 1;
    }
    result
});

assert_eq!(reify_nat(5, &Factorial), 120);
assert_eq!(reify_nat(10, &Factorial), 3628800);
```

Inside the macro body, `N` is a real `const u64` — the compiler
knows its value for each monomorphization. But you still can't
construct `Modular<N>` because the macro expands to a simple
expression, not a full impl block.

For callbacks that need to capture data:

```rust
def_nat_callback!(AddOffset { offset: u64 } -> u64 { |s| N + s.offset });

assert_eq!(reify_nat(10, &AddOffset { offset: 5 }), 15);
```

### Level 3: Trait impl (full power)

```rust
use const_reify::nat_reify::{NatCallback, reify_nat};

struct ModPow { base: u64, exp: u64 }

impl NatCallback<u64> for ModPow {
    fn call<const M: u64>(&self) -> u64 {
        // M is a real const generic. We can construct Mod<M>:
        struct Mod<const M: u64> { value: u64 }

        impl<const M: u64> Mod<M> {
            fn new(v: u64) -> Self {
                Mod { value: if M == 0 { 0 } else { v % M } }
            }
            fn mul(self, other: Self) -> Self {
                Self::new(self.value * other.value)
            }
        }

        // Type-safe modular arithmetic — the type system
        // prevents mixing values from different moduli.
        let mut result = Mod::<M>::new(1);
        let mut base = Mod::<M>::new(self.base);
        let mut e = self.exp;
        while e > 0 {
            if e % 2 == 1 { result = result.mul(base); }
            base = base.mul(base);
            e /= 2;
        }
        result.value
    }
}

// The modulus is a RUNTIME value, but inside call<M>,
// M is a compile-time const generic.
assert_eq!(reify_nat(7, &ModPow { base: 3, exp: 6 }), 1); // Fermat's little theorem
```

This is the full power: inside `call<const M: u64>`, you have a real
type-level value. You can construct types parameterized by it, and the
compiler enforces type-level invariants.

## Two-Value Dispatch

For operations between two reified values:

```rust
use const_reify::nat_reify::{Nat2Callback, reify_nat2};

struct GCD;

impl Nat2Callback<u64> for GCD {
    fn call<const A: u64, const B: u64>(&self) -> u64 {
        // Both A and B are known const generics
        let mut a = A;
        let mut b = B;
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }
}

assert_eq!(reify_nat2(12, 8, &GCD), 4);
assert_eq!(reify_nat2(17, 13, &GCD), 1);  // coprime
```

Internally, `reify_nat2` does nested dispatch: first on `a` (256 arms),
then on `b` (256 arms per outer arm). The compiler generates up to
256 × 256 = 65,536 monomorphizations, though LLVM typically optimizes
most of them away.

## Why a Trait, Not a Closure?

You might wonder: why can't we just write `|<const N: u64>| N * N`?

Because Rust doesn't support const-generic closures. A closure's type is
anonymous and compiler-generated — there's no way to add `<const N: u64>`
to it. The `NatCallback` trait is a workaround: it's a named type with
a const-generic method, which Rust does support.

This is a language limitation, not a fundamental one. We've written an
[RFC sketch](../rfcs/0001-rank2-const-quantification.md) for
`for<const N: u64>` that would eliminate the need for the trait.

## The Tradeoff

| Approach | Const generic available? | Ergonomics | Compile cost |
|---|---|---|---|
| `reify_nat_fn(n, \|n\| ...)` | No (just u64) | Best | 256 monomorphizations |
| `def_nat_callback!(...)` | Yes (N in expressions) | Good | 256 monomorphizations |
| `impl NatCallback` | Yes (full power) | Verbose | 256 monomorphizations |
| `reify_nat2` | Yes (A and B) | Verbose | 256² monomorphizations |

## Limits

- **Range**: 0..=255 (256 values). Configurable in `#[reifiable]`, but
  larger ranges increase compile time linearly.
- **Type**: `u64` only. Other const-generic types would need separate
  dispatch tables.
- **Nesting**: Each level of nesting multiplies the monomorphization count.

Next guide: [The #[reifiable] Macro](04-reifiable-macro.md) — automating
dispatch for your own traits.
