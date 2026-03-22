# RFC 0002: Bounded Existential Const Types

- **Status**: Pre-RFC / Design Exploration
- **Context**: reflect-rs reification problem
- **Related**: RFC 0001 (Rank-2 Const Quantification)

## Summary

Introduce first-class bounded existential types over const generics, allowing
a value to carry a const generic parameter that is determined at runtime but
statically bounded.

```rust
type SomeNat = exists<const N: u64 where N <= 255> Modular<N>;
```

A value of type `SomeNat` is a `Modular<N>` for some specific `N` in
`0..=255`, where `N` is determined at runtime. Pattern matching recovers
the const generic.

## Motivation

### The gap in Rust's type system

Given `struct Modular<const N: u64>`, Rust provides no way to hold "a
`Modular` whose `N` is determined at runtime but bounded." You must choose:

| Approach | Tradeoff |
|---|---|
| Trait object `Box<dyn HasModulus>` | Erases `N` permanently — cannot recover the const generic |
| Manual enum `enum Mod { M0(Modular<0>), M1(Modular<1>), ... }` | Doesn't scale, no generic dispatch |
| Match-table callback `reify_nat(n, &callback)` | Scoped — result must be computed inside the callback, can't return `Modular<N>` |

None of these let you **store** a `Modular<N>` with runtime `N` and later
**recover** `N` as a const generic. This is a fundamental expressiveness gap.

### What we want

```rust
// Create a Modular with runtime-determined N
fn make_modular(n: u64) -> SomeModular {
    pack_nat(n)  // returns exists<N> Modular<N>
}

// Later, recover N and use it as a const generic
fn use_modular(m: &SomeModular) -> u64 {
    unpack m as <const N: u64> => modular: Modular<N> {
        // N is a const generic here — can construct other types using N,
        // call methods that require const N, etc.
        modular.modulus() * modular.modulus()
    }
}
```

### Use cases beyond reification

- **Serialization**: Deserialize a `Vec<N>` (fixed-capacity vector) where `N`
  is read from a header. Store it as `exists<N> Vec<N>`, unpack when processing.
- **Configuration-driven types**: A matrix library where dimensions come from
  a config file: `exists<const R: usize, const C: usize> Matrix<R, C>`.
- **Protocol negotiation**: A network protocol negotiates a buffer size; the
  resulting connection carries the size as a const-generic type parameter.
- **Plugin systems**: A plugin declares its arity; the host wraps it as
  `exists<const N: usize> Plugin<N>`.

## Detailed Design

### Syntax

#### Type declaration

```rust
// Standalone type alias
type SomeNat = exists<const N: u64 where N in 0..=255> Modular<N>;

// Inline in function signatures
fn make(n: u64) -> exists<const N: u64 where N in 0..=255> Modular<N>;

// Multiple const parameters
type SomeMatrix = exists<const R: usize where R in 1..=64,
                         const C: usize where C in 1..=64>
                  Matrix<R, C>;
```

The `where N in RANGE` clause is required — unbounded existentials would
require runtime-sized types, conflicting with Rust's fixed-layout model.

#### Pack (introduction)

Explicit pack with a witness value:

```rust
// The witness (42) must be a const expression or come from dispatch context
let x: SomeNat = exists(42, Modular::<42>);
```

From a runtime value via dispatch:

```rust
fn make_nat(n: u64) -> SomeNat {
    // Compiler generates the match table:
    // match n { 0 => exists(0, Modular::<0>), 1 => exists(1, Modular::<1>), ... }
    pack_const(n)
}
```

A `pack_const` built-in (or macro) that dispatches a runtime value into the
existential — this is the reification entry point.

#### Unpack (elimination)

```rust
let x: SomeNat = make_nat(42);

// Unpack with a match-like syntax
let result = unpack x as <const N: u64> => val: Modular<N> {
    // N is a const generic, val: Modular<N>
    N * N
};
// result: u64 = 1764
```

Alternative syntax options:

```rust
// Option A: match-like
match_const x {
    <const N: u64> val: Modular<N> => { N * N }
}

// Option B: let-like
let <const N: u64> val: Modular<N> = x;
// N and val in scope for remainder of block

// Option C: closure-like (pairs well with RFC 0001)
x.unpack(|<const N: u64>| val: Modular<N> | { N * N })
```

**Recommendation**: Option A (`match_const`) — it's visually consistent with
existing `match` and makes the scope of `N` explicit.

### Representation

A bounded existential const type has a known, fixed layout:

```
┌──────────────────┬──────────────────────────────────────┐
│ discriminant     │ payload                              │
│ (encodes N)      │ (size = max over all N of size(T<N>))│
└──────────────────┴──────────────────────────────────────┘
```

- **Discriminant**: A `u8` for ranges up to 256, `u16` for ranges up to 65536, etc.
  Identical to Rust's enum discriminant sizing.
- **Payload**: The maximum size across all possible `T<N>`. For ZSTs (like
  `Modular<N>`), this is zero. For `[u8; N]` with `N in 0..=255`, this is
  255 bytes.

This is **exactly an enum** — the language feature automates its generation.

```rust
// exists<const N: u64 where N in 0..=255> Modular<N>
// is isomorphic to:
enum __SomeNat {
    __V0(Modular<0>),
    __V1(Modular<1>),
    ...
    __V255(Modular<255>),
}
```

The key difference from a manual enum: `unpack` gives you a generic `N`
— you write one handler, not 256 match arms.

### Semantics: Pack and Unpack

**Pack** is existential introduction (∃-intro):

```
Γ ⊢ n : u64    Γ ⊢ n ∈ 0..=255    Γ ⊢ v : T<n>
────────────────────────────────────────────────────
Γ ⊢ pack(n, v) : exists<const N: u64 where N in 0..=255> T<N>
```

**Unpack** is existential elimination (∃-elim):

```
Γ ⊢ e : exists<const N: u64 where N in 0..=255> T<N>
Γ, N: u64, N ∈ 0..=255, x: T<N> ⊢ body : R
R does not mention N
────────────────────────────────────────────────────
Γ ⊢ unpack e as <N> x { body } : R
```

The critical constraint: **R must not mention N**. The const generic `N`
is scoped to the unpack body — it cannot escape. This prevents:

```rust
// ILLEGAL: N escapes the unpack scope
let arr: [u8; N] = unpack x as <const N> _ { [0u8; N] };
//                       ^ N not in scope here
```

This is the same scoping that Haskell's `forall s` provides and that our
branded lifetime `for<'brand>` approximates — but now it's real const-generic
scoping, not a lifetime trick.

### Interaction with monomorphization

The compiler generates all variants at compile time. For range `0..=B`,
that's `B + 1` monomorphizations of the payload type and the unpack body.

**Compile-time cost bounds**:
- The range must be a const expression evaluable at compile time
- The compiler should enforce a maximum range size (configurable, default 1024)
- For multi-parameter existentials, the product of ranges must be within the limit

### Interaction with the borrow checker

No special interaction. The existential type is a plain value type with known
size and alignment. It can be:

- Moved, copied (if the inner type is `Copy` for all `N`), cloned
- Borrowed: `&SomeNat`, `&mut SomeNat`
- Placed in structs, enums, `Vec`, `Box`, etc.

The `N` is not a lifetime — it does not affect borrowing.

### Interaction with traits

An existential const type can implement traits if the inner type does for
all `N` in the range:

```rust
// If Modular<N>: Display for all N in 0..=255, then:
impl Display for SomeNat { ... }

// The compiler generates:
// match self.discriminant {
//     0 => fmt the Modular::<0>,
//     1 => fmt the Modular::<1>,
//     ...
// }
```

This is analogous to how enums derive traits — each variant must satisfy
the bound.

### Nesting and composition

Existentials can be nested:

```rust
type SomePair = exists<const A: u64 where A in 0..=15,
                       const B: u64 where B in 0..=15>
                (Modular<A>, Modular<B>);
```

Unpacking gives both const generics:

```rust
unpack pair as <const A, const B> => (ma, mb): (Modular<A>, Modular<B>) {
    A + B
}
```

## Comparison with Other Languages

| Language | Mechanism | Runtime cost | Recovery |
|---|---|---|---|
| **Rust (proposed)** | Bounded existential, enum-backed | Tag dispatch (O(1)) | `unpack` recovers const generic |
| **Haskell** | `SomeNat` / typeclass dictionary | Dict lookup (O(1)) | `case someNatVal of SomeNat (_ :: Proxy n) -> ...` |
| **OCaml** | First-class modules `(module M : S)` | Pointer to module record | Pattern match on module |
| **Scala 3** | Match types + union types | JVM dispatch | Pattern match |
| **C++** | `std::variant` + `std::visit` | Tag dispatch | `std::visit` with lambda |

Rust's version is distinctive in being:
- **Statically bounded**: The range is known at compile time
- **Zero-cost**: The dispatch is a match on an integer tag
- **Scope-safe**: The const generic cannot escape the unpack body

## Backwards Compatibility

Fully backwards compatible. `exists<...>` is new syntax. The keyword `exists`
is not currently reserved but is not a valid identifier in type position.

## Alternatives Considered

### 1. Manual enums (status quo)

Works but doesn't scale. 256-variant enums are unmaintainable, and you still
need 256 match arms to dispatch — defeating the purpose.

### 2. Trait objects with recovery

A hypothetical `dyn HasModulus + ConstRecover<N>` that lets you recover `N`.
This would require vtable extensions and doesn't compose well with existing
trait object semantics.

### 3. `#[derive(ConstEnum)]` proc macro

A proc macro that generates the enum and dispatch code automatically.
This is implementable TODAY and could serve as a prototype for the
language feature. See the proc macro analysis document. However, it
can't provide the scoping guarantee that a language-level `unpack` provides.

### 4. Dependent pair types (full Σ-types)

`Σ(N: u64). T<N>` — a dependent pair where the first component is a value
and the second component's type depends on it. This is the general form of
what we're proposing. Bounded existentials are Σ-types restricted to bounded
domains, which makes them compatible with Rust's compilation model.

## Open Questions

1. **Range syntax**: `where N in 0..=255` vs `where N <= 255` vs
   `where N: Bounded<255>`. The range syntax is most explicit about what
   values are valid.

2. **Non-contiguous ranges**: Should `where N in {2, 4, 8, 16, 32, 64}`
   work? This would enable power-of-two buffer sizes, for instance.

3. **Const folding through unpack**: Can the compiler constant-fold
   `unpack (pack(42, v)) as <N> x { N }` to `42`? This requires seeing
   through the pack/unpack pair.

4. **Auto-traits**: Should `SomeNat: Send` if `Modular<N>: Send` for all
   `N` in range? Almost certainly yes, but needs specification.

5. **Size optimization**: When `T<N>` is a ZST for all `N`, the existential
   should be `size_of::<u8>()` (just the tag). The compiler should detect
   this.

6. **Relationship to RFC 0001**: If `for<const N>` is adopted, `unpack`
   could desugar to a call with a `for<const N>` closure:
   ```rust
   unpack x as <const N> val { body }
   // desugars to:
   x.__unpack(|<const N>| val | { body })
   ```
   This makes RFC 0002 syntactic sugar over RFC 0001 + enum representation.

## Prior Art

- **Haskell `SomeNat`**: `data SomeNat = forall n. KnownNat n => SomeNat (Proxy n)`.
  Uses existential quantification with a typeclass constraint. No bound required
  because dictionaries are runtime. Our proposal adds the bound for
  monomorphization compatibility.

- **Haskell `Exists`**: The `exists` package provides `data Some f = forall a. Some (f a)`.
  Same pattern, generalized.

- **OCaml GADTs**: `type _ t = Int : int t | Bool : bool t` — existential types
  encoded as GADT constructors. Unpacked via pattern matching.

- **GHC Proposal #378 (Unsaturated type families)**: Related work on
  representing type-level computations as first-class values.

- **Kiselyov & Shan, "Functional Pearl: Implicit Configurations" (2004)**:
  The original paper on reification that motivates this entire design space.

- **Kmett's `reflection` library**: The Haskell implementation that `reify`
  and `reflect` in reflect-rs are based on.
