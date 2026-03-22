use const_reify_derive::reifiable;

// --- Basic: single const-generic method, &self, no extra params ---

#[reifiable(range = 0..=255)]
trait Squarer {
    fn square<const N: u64>(&self) -> u64;
}

struct MySquarer;

impl Squarer for MySquarer {
    fn square<const N: u64>(&self) -> u64 {
        N * N
    }
}

#[test]
fn basic_dispatch() {
    assert_eq!(reify_square(0u64, &MySquarer), 0);
    assert_eq!(reify_square(5u64, &MySquarer), 25);
    assert_eq!(reify_square(12u64, &MySquarer), 144);
    assert_eq!(reify_square(255u64, &MySquarer), 255 * 255);
}

// --- Method with extra parameters ---

#[reifiable(range = 0..=255)]
trait ModArith {
    fn mul_mod<const N: u64>(&self, a: u64, b: u64) -> u64;
}

struct ModArithImpl;

impl ModArith for ModArithImpl {
    fn mul_mod<const N: u64>(&self, a: u64, b: u64) -> u64 {
        if N == 0 {
            0
        } else {
            (a % N) * (b % N) % N
        }
    }
}

#[test]
fn dispatch_with_params() {
    // 10 * 20 mod 7 = 200 mod 7 = 4
    assert_eq!(reify_mul_mod(7u64, &ModArithImpl, 10, 20), 4);
    assert_eq!(
        reify_mul_mod(13u64, &ModArithImpl, 100, 200),
        (100 * 200) % 13
    );
}

// --- Multiple methods, only const-generic ones get dispatch ---

#[reifiable(range = 0..=15)]
trait Mixed {
    fn with_const<const N: u64>(&self) -> u64;
    fn without_const(&self) -> &str;
}

struct MixedImpl;

impl Mixed for MixedImpl {
    fn with_const<const N: u64>(&self) -> u64 {
        N + 1
    }

    fn without_const(&self) -> &str {
        "hello"
    }
}

#[test]
fn mixed_methods() {
    // Dispatch function generated for with_const
    assert_eq!(reify_with_const(0u64, &MixedImpl), 1);
    assert_eq!(reify_with_const(15u64, &MixedImpl), 16);

    // without_const is a normal method, no dispatch function
    assert_eq!(MixedImpl.without_const(), "hello");
}

// --- &mut self methods ---

#[reifiable(range = 0..=31)]
trait Accumulator {
    fn accumulate<const N: u64>(&mut self);
}

struct Counter {
    total: u64,
}

impl Accumulator for Counter {
    fn accumulate<const N: u64>(&mut self) {
        self.total += N;
    }
}

#[test]
fn mut_self_dispatch() {
    let mut counter = Counter { total: 0 };
    reify_accumulate(5u64, &mut counter);
    reify_accumulate(10u64, &mut counter);
    assert_eq!(counter.total, 15);
}

// --- Predicate (returns bool) ---

#[reifiable(range = 0..=255)]
trait Predicate {
    fn is_even<const N: u64>(&self) -> bool;
}

struct EvenChecker;

impl Predicate for EvenChecker {
    fn is_even<const N: u64>(&self) -> bool {
        N % 2 == 0
    }
}

#[test]
fn bool_return_type() {
    assert!(reify_is_even(0u64, &EvenChecker));
    assert!(!reify_is_even(1u64, &EvenChecker));
    assert!(reify_is_even(42u64, &EvenChecker));
    assert!(!reify_is_even(255u64, &EvenChecker));
}

// --- Panic on out-of-range ---

#[test]
#[should_panic(expected = "out of range")]
fn out_of_range_panics() {
    reify_square(256u64, &MySquarer);
}

// --- Small range ---

#[reifiable(range = 0..=3)]
trait Tiny {
    fn val<const N: u64>(&self) -> u64;
}

struct TinyImpl;

impl Tiny for TinyImpl {
    fn val<const N: u64>(&self) -> u64 {
        N
    }
}

#[test]
fn small_range() {
    assert_eq!(reify_val(0u64, &TinyImpl), 0);
    assert_eq!(reify_val(3u64, &TinyImpl), 3);
}

#[test]
#[should_panic(expected = "out of range")]
fn small_range_out_of_bounds() {
    reify_val(4u64, &TinyImpl);
}
