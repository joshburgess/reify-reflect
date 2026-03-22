# Phase 5 — Const-Generic Bridge

## Design Decisions

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

`Modular<N>` is a ZST — it carries no data, only the const generic value.
The `HasModulus` trait exposes this value at runtime. Users interact through
`&dyn HasModulus`, enabling generic code that works across all 256 values
without knowing `N` at compile time.

### deny(unsafe_code)

Despite the original plan designating this as the "unsafe core" crate,
the match-table approach is entirely safe. We set `#![deny(unsafe_code)]`
to enforce this, bringing the crate in line with the rest of the workspace.
