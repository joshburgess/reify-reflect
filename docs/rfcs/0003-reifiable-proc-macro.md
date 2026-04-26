# Design: `#[reifiable]` Proc Macro for Automatic Const-Generic Dispatch

- **Status**: Design / Feasibility Analysis
- **Context**: reify-reflect reification, implementable today
- **Related**: RFC 0001, RFC 0002 (this is the "do it now" version)

## Summary

A proc macro attribute `#[reifiable]` that generates match-table dispatch
functions for traits with const-generic methods. This automates the
`NatCallback` pattern for arbitrary user-defined traits, without waiting
for language-level `for<const N>` support.

```rust
#[reifiable(range = 0..=255)]
trait MyTrait {
    fn compute<const N: u64>(&self, x: f64) -> f64;
}

// Generated: reify_compute(val: u64, obj: &impl MyTrait, x: f64) -> f64
```

## Motivation

The `NatCallback` pattern works but requires per-use-case boilerplate:

```rust
trait NatCallback<R> {
    fn call<const N: u64>(&self) -> R;
}

fn reify_nat<C: NatCallback<R>, R>(val: u64, callback: &C) -> R {
    match val {
        0 => callback.call::<0>(),
        1 => callback.call::<1>(),
        // ... 256 arms
        _ => panic!()
    }
}
```

For each new trait with const-generic methods, the user must manually write
the dispatch table. A proc macro can generate this automatically.

## Proposed Interface

### Basic usage

```rust
#[reifiable(range = 0..=255)]
trait Hasher {
    fn hash<const N: u64>(&self, data: &[u8]) -> [u8; 32];
    fn name(&self) -> &str;  // non-const-generic, left alone
}
```

The macro:
1. Leaves the trait definition unchanged
2. Generates a dispatch function for each const-generic method
3. Skips non-const-generic methods

### Generated code

For the trait above, the macro generates:

```rust
// Dispatch function for hash
fn reify_hash<T: Hasher>(val: u64, obj: &T, data: &[u8]) -> [u8; 32] {
    match val {
        0 => obj.hash::<0>(data),
        1 => obj.hash::<1>(data),
        ...
        255 => obj.hash::<255>(data),
        _ => panic!("#[reifiable]: value {} out of range 0..=255", val),
    }
}

// NatCallback wrapper for hash (for integration with reify_nat)
struct HashCallback<'a, T: Hasher> {
    obj: &'a T,
    data: &'a [u8],
}

impl<T: Hasher> const_reify::NatCallback<[u8; 32]> for HashCallback<'_, T> {
    fn call<const N: u64>(&self) -> [u8; 32] {
        self.obj.hash::<N>(self.data)
    }
}
```

### Per-parameter range configuration

```rust
#[reifiable(N: u64 in 0..=255, M: u32 in 0..=7)]
trait Mixed {
    fn thing<const N: u64, const M: u32>(&self) -> u64;
}
```

Generates nested dispatch with different ranges per parameter.

### Visibility control

```rust
#[reifiable(range = 0..=255, visibility = pub)]
trait MyTrait { ... }

#[reifiable(range = 0..=255, visibility = pub(crate))]
trait InternalTrait { ... }
```

Default visibility matches the trait's visibility.

## Implementation Details

### What the proc macro receives

A `TokenStream` containing:
```rust
#[reifiable(range = 0..=255)]
trait MyTrait {
    fn compute<const N: u64>(&self, x: f64) -> f64;
}
```

### Parsing steps

Using `syn`:

1. Parse the attribute arguments to extract range configuration
2. Parse the trait definition (`syn::ItemTrait`)
3. For each `syn::TraitItemFn`, check `sig.generics.params` for
   `GenericParam::Const` entries
4. Extract: method name, const param name/type, `&self`/`&mut self`,
   regular parameters (names and types), return type
5. Collect non-const-generic methods to skip

### Code generation steps

Using `quote`:

For each const-generic method:

1. **Generate the range literals**: Expand `0..=255` into a token stream
   of `0, 1, 2, ..., 255`
2. **Generate the dispatch function**: A standalone `fn` that takes the
   runtime value, the trait implementor, and all non-const parameters
3. **Generate match arms**: `N => obj.method::<N>(args...)`
4. **Generate the NatCallback wrapper struct**: A struct capturing the
   trait object and parameters, implementing `NatCallback<R>`

### Multi-parameter dispatch

For two const parameters, generate nested matches:

```rust
fn reify_thing<T: Mixed>(n: u64, m: u32, obj: &T) -> u64 {
    match n {
        0 => match m {
            0 => obj.thing::<0, 0>(),
            1 => obj.thing::<0, 1>(),
            ...
            7 => obj.thing::<0, 7>(),
            _ => panic!("..."),
        },
        1 => match m {
            0 => obj.thing::<1, 0>(),
            ...
        },
        ...
    }
}
```

Total monomorphizations: `|range_N| × |range_M|`. For `256 × 8 = 2048`,
this is manageable. The macro should emit a compile-time warning when the
product exceeds a configurable threshold.

## Handling Complex Signatures

### `&mut self` methods

```rust
#[reifiable(range = 0..=255)]
trait Stateful {
    fn update<const N: u64>(&mut self, x: u64) -> u64;
}

// Generated:
fn reify_update<T: Stateful>(val: u64, obj: &mut T, x: u64) -> u64 {
    match val {
        0 => obj.update::<0>(x),
        ...
    }
}
```

The dispatch function takes `&mut T` instead of `&T`. Determined by
inspecting `sig.receiver()`.

### Methods with lifetime parameters

```rust
#[reifiable(range = 0..=255)]
trait Parser {
    fn parse<'a, const N: u64>(&self, input: &'a str) -> &'a str;
}

// Generated:
fn reify_parse<'a, T: Parser>(val: u64, obj: &T, input: &'a str) -> &'a str {
    match val {
        0 => obj.parse::<0>(input),
        ...
    }
}
```

Lifetime parameters on the method are propagated to the dispatch function.
The const generic is the only parameter consumed by the dispatch.

### Methods with generic type parameters alongside const

```rust
#[reifiable(range = 0..=255)]
trait Processor {
    fn process<const N: u64, T: Debug>(&self, items: Vec<T>) -> Vec<T>;
}

// Generated:
fn reify_process<T: Debug, P: Processor>(val: u64, obj: &P, items: Vec<T>) -> Vec<T> {
    match val {
        0 => obj.process::<0, T>(items),
        ...
    }
}
```

Non-const generic parameters are propagated to the dispatch function
signature and forwarded to each match arm.

### Traits with generic parameters

```rust
#[reifiable(range = 0..=255)]
trait Parametric<T: Clone> {
    fn compute<const N: u64>(&self, x: T) -> T;
}

// Generated:
fn reify_compute<T: Clone, P: Parametric<T>>(val: u64, obj: &P, x: T) -> T {
    match val {
        0 => obj.compute::<0>(x),
        ...
    }
}
```

The trait's own generic parameters are propagated to the dispatch function
and used in the trait bound.

## Limitations and Edge Cases

### Return types that depend on N

```rust
#[reifiable(range = 0..=255)]
trait ArrayMaker {
    fn make<const N: u64>(&self) -> [u8; N];  // return type depends on N!
}
```

**This is a hard problem.** The dispatch function would need a single return
type, but `[u8; N]` varies per arm.

Options:
1. **Error**: The macro rejects methods where the return type mentions the
   const parameter. This is the conservative V1 approach.
2. **Erase to Vec**: Generate a dispatch function returning `Vec<u8>` with
   an internal `to_vec()` call. Loses zero-copy but works.
3. **User-annotated erased type**: `#[reifiable(range = 0..=255, erase_return = Vec<u8>)]`,
   where the user specifies how to erase the return type.
4. **Defer to bounded existentials** (RFC 0002): Return
   `exists<const N> [u8; N]`. Not available today.

**Recommendation for V1**: Option 1 (error). Document the limitation and
point users to the manual `NatCallback` pattern for these cases.

### Associated types that depend on N

Similar to return-type-depends-on-N. If the trait has `type Output<const N: u64>`,
the dispatch can't unify different associated types.

**Recommendation**: Error in V1.

### Async methods

```rust
#[reifiable(range = 0..=255)]
trait AsyncProcessor {
    async fn process<const N: u64>(&self, x: u64) -> u64;
}
```

The dispatch function would need to return a `Pin<Box<dyn Future<Output = u64>>>`,
or use AFIT (async fn in traits). This is feasible but adds complexity.

**Recommendation**: Support in V2, not V1.

### Default method implementations

```rust
#[reifiable(range = 0..=255)]
trait WithDefault {
    fn required<const N: u64>(&self) -> u64;
    fn optional<const N: u64>(&self) -> u64 { N + 1 }  // default impl
}
```

The dispatch function calls the method regardless of whether it's default
or overridden. Default impls are handled transparently, with no special casing
needed.

### Multiple const parameters with different types

```rust
#[reifiable(N: u64 in 0..=255, B: bool in [false, true])]
trait Configurable {
    fn run<const N: u64, const B: bool>(&self) -> u64;
}
```

The range for `bool` is `[false, true]` (2 values). The range for non-integer
const types would need to be an explicit list. The macro generates the
cartesian product of ranges.

## Compile-Time Cost

### Per-method cost

Each dispatch function generates `R` match arms, each instantiating the
method with a different const value.

| Range size | Arms | Typical compile time impact |
|---|---|---|
| 0..=15 (16) | 16 | Negligible |
| 0..=255 (256) | 256 | ~1–3 seconds per method |
| 0..=1023 (1024) | 1024 | ~5–15 seconds per method |

### Multi-parameter cost

For `k` parameters with ranges of size `R₁, R₂, ..., Rₖ`, the total
monomorphizations are `R₁ × R₂ × ... × Rₖ`.

| Parameters | Ranges | Total | Feasible? |
|---|---|---|---|
| 1 | 256 | 256 | Yes |
| 2 | 16 × 16 | 256 | Yes |
| 2 | 256 × 256 | 65,536 | Slow but works |
| 2 | 256 × 8 | 2,048 | Yes |
| 3 | 16 × 16 × 16 | 4,096 | Slow |

### Mitigation strategies

1. **Configurable threshold**: The macro warns when total monomorphizations
   exceed a threshold (default: 1024). Configurable:
   `#[reifiable(range = 0..=255, max_monomorphizations = 4096)]`

2. **Outline mode**: `#[reifiable(range = 0..=255, outline)]` generates the
   dispatch in a `#[inline(never)]` function, reducing code duplication at
   call sites.

3. **Lazy instantiation**: Only generate monomorphizations for const values
   that are actually used. This requires whole-program analysis and is
   not feasible for a proc macro: it's a compiler optimization.

## Implementation Plan

### V1 scope

- Single `const N: PrimitiveType` parameter per method
- Configurable range via attribute
- Generates dispatch functions and `NatCallback` wrappers
- Errors on return types that depend on N
- Warns when total monomorphizations exceed 1024
- Supports `&self`, `&mut self`, and regular parameters
- Propagates lifetime and type generic parameters

### V2 scope

- Multiple const parameters with per-parameter ranges
- Async method support
- User-annotated return type erasure for N-dependent returns
- `#[reifiable(outline)]` mode for reduced code size

### Crate placement

New crate: `const-reify-derive` (proc-macro crate), added to the workspace.
Depends on `syn`, `quote`, `proc-macro2`. Optionally generates code that
references `const-reify::NatCallback` for the callback wrapper.

## Prototype

A working prototype of the core dispatch generation (without the full proc
macro attribute parsing) can be demonstrated with a declarative macro:

```rust
macro_rules! generate_dispatch {
    ($trait:ident, $method:ident, $ret:ty, ($($param:ident : $pty:ty),*)) => {
        fn paste::paste! { [<reify_ $method>] }<T: $trait>(
            val: u64, obj: &T, $($param: $pty),*
        ) -> $ret {
            macro_rules! arms {
                ($($n:literal),*) => {
                    match val {
                        $( $n => obj.$method::<$n>($($param),*), )*
                        other => panic!("out of range: {}", other),
                    }
                };
            }
            arms!(0, 1, 2, ... , 255)
        }
    };
}
```

A full proc macro replaces the manual signature parsing with `syn`-based
analysis.

## Relationship to RFCs 0001 and 0002

This proc macro is the **"do it now" bridge** while the language-level
features are designed and stabilized:

| Need | Today (proc macro) | Future (RFC 0001) | Future (RFC 0002) |
|---|---|---|---|
| Reify a value into const-generic context | `#[reifiable]` + dispatch fn | `for<const N>` closure | `pack_const` |
| Recover const generic from existential | N/A (scoped callback only) | N/A | `unpack` / `match_const` |
| Store existential const type | N/A | N/A | `exists<const N> T<N>` as value type |

The proc macro covers the reification use case completely. Bounded existentials
(RFC 0002) cover the "store and later recover" use case that the proc macro
cannot address.

If RFC 0001 is adopted, the proc macro becomes unnecessary: closures with
`for<const N>` would replace the generated dispatch functions entirely.
