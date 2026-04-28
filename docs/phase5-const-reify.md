# Phase 5: Const-Generic Bridge

## What this phase covers

The headline feature of the project: using a runtime `u64` as if it were a `const N: u64`, safely.

Const generics in Rust have to be known at compile time. That's frustrating when the value you actually want to parameterize on (a modulus, a buffer size, a feature flag) only becomes known at runtime. The orthodox workarounds are to drop the const generic and lose the type safety, or to write a giant `match` by hand.

This phase ships the giant `match`, generated for you, with three progressively more powerful APIs on top:

1. `reify_const` / `reify!`: smallest surface area. You get a `&dyn HasModulus` whose `modulus()` returns your runtime value.
2. `reify_nat_fn` / `reify_nat2_fn`: closure-based, when you only need the runtime value as a plain `u64` inside the callback.
3. `NatCallback` / `reify_nat`: the full power form. Inside `call::<const N: u64>()`, `N` is a real const generic.

The `#[reifiable]` proc macro on a trait declaration generates the `NatCallback` and dispatch wiring automatically for every const-generic method.

Importantly, this is all safe. The original design called for unsafe vtable fabrication; we replaced it with a flat 256-arm match generated at compile time. See [`const-reify/DESIGN.md`](../const-reify/DESIGN.md) for the full safety analysis and [Guide 3](guides/03-const-reify.md) / [Guide 4](guides/04-reifiable-macro.md) for tutorials.

## Crates introduced

- [`const-reify`](https://docs.rs/const-reify) (with [`DESIGN.md`](../const-reify/DESIGN.md))
- [`const-reify-derive`](https://docs.rs/const-reify-derive)

## Design decisions

### Match-table dispatch over vtable fabrication

The original plan called for unsafe vtable inspection and fabrication. We
replaced this with a safe match-table approach because:

1. **Vtable layout is unstable.** Rustc provides no guarantees about vtable
   structure, entry order, or representation across versions or optimization
   levels.

2. **Fabricating vtables is undefined behavior.** Creating trait objects from
   hand-crafted pointers violates Rust's provenance and aliasing rules.

3. **No mitigation is reliable.** Compile-time layout assertions can detect
   *some* changes but cannot prevent UB from future compiler changes.

The match-table approach achieves the same user-facing API (`reify_const`
dispatches a runtime value to a const-generic monomorphization) with zero
unsafe code.

See `const-reify/DESIGN.md` for the full analysis.

### 0..=255 range limit

256 monomorphizations is a practical balance between utility and compile
time. The dispatch macro generates a flat match with 256 arms, which
compiles quickly and optimizes well. Future extensions could use two-level
dispatch (high byte + low byte) for larger ranges.

### Modular<const N: u64> as the bridge type

`Modular<N>` is a ZST: it carries no data, only the const generic value.
The `HasModulus` trait exposes this value at runtime. Users interact through
`&dyn HasModulus`, enabling generic code that works across all 256 values
without knowing `N` at compile time.

### deny(unsafe_code)

Despite the original plan designating this as the "unsafe core" crate,
the match-table approach is entirely safe. We set `#![deny(unsafe_code)]`
to enforce this, bringing the crate in line with the rest of the workspace.

## Next

- [Phase 6: Integration](phase6-integration.md)
- [Documentation index](README.md)
