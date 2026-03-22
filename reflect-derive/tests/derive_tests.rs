use reflect_core::{Reflect, RuntimeValue};
use reflect_derive::Reflect;
use reflect_nat::{S, Z};

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

// Unit struct (no fields)
#[derive(Reflect)]
struct Empty {}

#[test]
fn derive_empty_struct() {
    assert_eq!(Empty::reflect(), RuntimeValue::List(vec![]));
}
