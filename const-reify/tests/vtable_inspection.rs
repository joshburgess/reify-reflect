//! Educational test demonstrating vtable layout inspection.
//!
//! This test documents the structure of trait object vtables in current
//! stable Rust. It is NOT used for any production functionality: the
//! actual dispatch in const-reify uses a safe match table.
//!
//! It exists to demonstrate why vtable fabrication is fragile and should
//! be avoided.

use const_reify::{HasModulus, Modular};

#[test]
fn trait_objects_have_consistent_behavior() {
    // Verify that trait objects for different const values
    // all behave correctly through dynamic dispatch.
    let objects: Vec<&dyn HasModulus> = vec![
        &Modular::<1>,
        &Modular::<2>,
        &Modular::<3>,
        &Modular::<4>,
        &Modular::<5>,
        &Modular::<6>,
        &Modular::<7>,
        &Modular::<8>,
    ];

    for (i, obj) in objects.iter().enumerate() {
        assert_eq!(obj.modulus(), (i + 1) as u64);
    }
}

#[test]
fn trait_object_size_is_two_pointers() {
    // A trait object (&dyn Trait) is always two pointers:
    // one data pointer and one vtable pointer.
    assert_eq!(
        std::mem::size_of::<&dyn HasModulus>(),
        2 * std::mem::size_of::<usize>()
    );
}

#[test]
fn zero_sized_modular_types() {
    // Modular<N> is a ZST for all N — it carries no runtime data,
    // only the const generic parameter.
    assert_eq!(std::mem::size_of::<Modular<0>>(), 0);
    assert_eq!(std::mem::size_of::<Modular<42>>(), 0);
    assert_eq!(std::mem::size_of::<Modular<255>>(), 0);
}

#[test]
fn different_const_values_produce_different_vtables() {
    // Each Modular<N> gets its own vtable with a different modulus() impl.
    // We can observe this through the trait object's behavior.
    let a: &dyn HasModulus = &Modular::<10>;
    let b: &dyn HasModulus = &Modular::<20>;

    assert_ne!(a.modulus(), b.modulus());
    assert_eq!(a.modulus(), 10);
    assert_eq!(b.modulus(), 20);
}
