#![allow(dead_code)]

use reflect_derive::Reflect;
use reflect_nat::{S, Z};
use reify_reflect_core::{Reflect, RuntimeValue};

// A simple struct with two reflectable fields
#[derive(Reflect)]
struct Point {
    x: S<S<Z>>,
    y: S<S<S<Z>>>,
}

#[test]
fn derive_simple_struct() {
    let reflected = Point::reflect();
    match &reflected {
        RuntimeValue::List(fields) => {
            assert_eq!(fields.len(), 2);

            // First field: ("x", Nat(2))
            match &fields[0] {
                RuntimeValue::List(pair) => {
                    assert_eq!(pair.len(), 2);
                    // Field name "x" encoded as bytes
                    assert_eq!(
                        pair[0],
                        RuntimeValue::List(vec![RuntimeValue::Nat(b'x' as u64)])
                    );
                    assert_eq!(pair[1], RuntimeValue::Nat(2));
                }
                other => panic!("expected List for field entry, got {:?}", other),
            }

            // Second field: ("y", Nat(3))
            match &fields[1] {
                RuntimeValue::List(pair) => {
                    assert_eq!(pair.len(), 2);
                    assert_eq!(
                        pair[0],
                        RuntimeValue::List(vec![RuntimeValue::Nat(b'y' as u64)])
                    );
                    assert_eq!(pair[1], RuntimeValue::Nat(3));
                }
                other => panic!("expected List for field entry, got {:?}", other),
            }
        }
        other => panic!("expected List at top level, got {:?}", other),
    }
}

// A struct with a skipped field
#[derive(Reflect)]
struct Labeled {
    #[reflect(skip)]
    _label: String,
    value: Z,
}

#[test]
fn derive_with_skip() {
    let reflected = Labeled::reflect();
    match &reflected {
        RuntimeValue::List(fields) => {
            // Only 'value' should be present, '_label' is skipped
            assert_eq!(fields.len(), 1);
            match &fields[0] {
                RuntimeValue::List(pair) => {
                    assert_eq!(pair[1], RuntimeValue::Nat(0));
                }
                other => panic!("expected List for field entry, got {:?}", other),
            }
        }
        other => panic!("expected List at top level, got {:?}", other),
    }
}

// A nested struct
#[derive(Reflect)]
struct Inner {
    a: S<Z>,
}

#[derive(Reflect)]
struct Outer {
    inner: Inner,
    b: S<S<S<S<Z>>>>,
}

#[test]
fn derive_nested_struct() {
    let reflected = Outer::reflect();
    match &reflected {
        RuntimeValue::List(fields) => {
            assert_eq!(fields.len(), 2);

            // First field: inner, which is itself a RuntimeValue::List
            match &fields[0] {
                RuntimeValue::List(pair) => {
                    assert_eq!(pair.len(), 2);
                    // The inner struct reflects to a List of its own fields
                    match &pair[1] {
                        RuntimeValue::List(inner_fields) => {
                            assert_eq!(inner_fields.len(), 1);
                            match &inner_fields[0] {
                                RuntimeValue::List(inner_pair) => {
                                    assert_eq!(inner_pair[1], RuntimeValue::Nat(1));
                                }
                                other => panic!("expected List, got {:?}", other),
                            }
                        }
                        other => panic!("expected List for inner struct, got {:?}", other),
                    }
                }
                other => panic!("expected List for field entry, got {:?}", other),
            }

            // Second field: b = 4
            match &fields[1] {
                RuntimeValue::List(pair) => {
                    assert_eq!(pair[1], RuntimeValue::Nat(4));
                }
                other => panic!("expected List for field entry, got {:?}", other),
            }
        }
        other => panic!("expected List at top level, got {:?}", other),
    }
}

// Empty named struct (no fields, but braced)
#[derive(Reflect)]
struct EmptyNamed {}

#[test]
fn derive_empty_named_struct() {
    assert_eq!(EmptyNamed::reflect(), RuntimeValue::List(vec![]));
}

// Unit struct (no fields, no braces) → RuntimeValue::Unit
#[derive(Reflect)]
struct Pixel;

#[test]
fn derive_unit_struct() {
    assert_eq!(Pixel::reflect(), RuntimeValue::Unit);
}

// Tuple struct → list of positional reflected values
#[derive(Reflect)]
struct Pair(S<Z>, S<S<Z>>);

#[test]
fn derive_tuple_struct() {
    let reflected = Pair::reflect();
    match &reflected {
        RuntimeValue::List(values) => {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], RuntimeValue::Nat(1));
            assert_eq!(values[1], RuntimeValue::Nat(2));
        }
        other => panic!("expected List, got {:?}", other),
    }
}

// Tuple struct with skipped field
#[derive(Reflect)]
struct Tagged(#[reflect(skip)] u32, Z);

#[test]
fn derive_tuple_struct_with_skip() {
    let reflected = Tagged::reflect();
    match &reflected {
        RuntimeValue::List(values) => {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0], RuntimeValue::Nat(0));
        }
        other => panic!("expected List, got {:?}", other),
    }
}

// Enum with mixed-shape variants
#[derive(Reflect)]
enum Shape {
    Dot,
    Line(S<S<Z>>),
    Box { w: S<Z>, h: S<S<S<Z>>> },
}

#[test]
fn derive_enum_with_mixed_variants() {
    let reflected = Shape::reflect();
    let variants = match reflected {
        RuntimeValue::List(v) => v,
        other => panic!("expected List, got {:?}", other),
    };
    assert_eq!(variants.len(), 3);

    // Dot (unit) → [name_bytes, Unit]
    let dot = match &variants[0] {
        RuntimeValue::List(v) => v,
        _ => panic!(),
    };
    assert_eq!(
        dot[0],
        RuntimeValue::List(
            b"Dot"
                .iter()
                .map(|b| RuntimeValue::Nat(*b as u64))
                .collect()
        )
    );
    assert_eq!(dot[1], RuntimeValue::Unit);

    // Line(S<S<Z>>) → [name_bytes, [Nat(2)]]
    let line = match &variants[1] {
        RuntimeValue::List(v) => v,
        _ => panic!(),
    };
    assert_eq!(
        line[0],
        RuntimeValue::List(
            b"Line"
                .iter()
                .map(|b| RuntimeValue::Nat(*b as u64))
                .collect()
        )
    );
    assert_eq!(line[1], RuntimeValue::List(vec![RuntimeValue::Nat(2)]));

    // Box { w, h } → [name_bytes, [[name_bytes, Nat(1)], [name_bytes, Nat(3)]]]
    let bx = match &variants[2] {
        RuntimeValue::List(v) => v,
        _ => panic!(),
    };
    assert_eq!(
        bx[0],
        RuntimeValue::List(
            b"Box"
                .iter()
                .map(|b| RuntimeValue::Nat(*b as u64))
                .collect()
        )
    );
    let bx_fields = match &bx[1] {
        RuntimeValue::List(v) => v,
        _ => panic!(),
    };
    assert_eq!(bx_fields.len(), 2);
}
