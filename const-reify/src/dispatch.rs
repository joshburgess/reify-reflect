//! Match-table dispatch for runtime-to-const-generic bridging.

/// A const-generic type parameterized by a `u64` modulus.
///
/// # Examples
///
/// ```
/// use const_reify::{Modular, HasModulus};
///
/// let m = Modular::<42>;
/// assert_eq!(m.modulus(), 42);
/// ```
pub struct Modular<const N: u64>;

/// Trait providing access to the const-generic modulus value.
///
/// Implemented automatically for all [`Modular<N>`] instances.
///
/// # Examples
///
/// ```
/// use const_reify::{Modular, HasModulus};
///
/// fn print_modulus(m: &dyn HasModulus) {
///     println!("modulus = {}", m.modulus());
/// }
///
/// print_modulus(&Modular::<7>);
/// ```
pub trait HasModulus {
    /// Returns the modulus value.
    fn modulus(&self) -> u64;
}

impl<const N: u64> HasModulus for Modular<N> {
    fn modulus(&self) -> u64 {
        N
    }
}

/// Dispatch a runtime `u64` value to the corresponding [`Modular<N>`]
/// monomorphization, passing it to `f` as a `&dyn HasModulus`.
///
/// Supports values in `0..=255`.
///
/// # Panics
///
/// Panics if `val > 255` with a message indicating the value is out of
/// the supported range.
///
/// # Examples
///
/// ```
/// use const_reify::{reify_const, HasModulus};
///
/// let result = reify_const(17, |m| m.modulus());
/// assert_eq!(result, 17);
///
/// // Use it for computation that depends on the const value
/// let doubled = reify_const(21, |m| m.modulus() * 2);
/// assert_eq!(doubled, 42);
/// ```
pub fn reify_const<F, R>(val: u64, f: F) -> R
where
    F: FnOnce(&dyn HasModulus) -> R,
{
    macro_rules! dispatch_arm {
        ($val:expr, $f:expr, $($n:literal),*) => {
            match $val {
                $( $n => $f(&Modular::<$n>), )*
                other => panic!(
                    "const-reify: value {} is out of supported range 0..=255",
                    other
                ),
            }
        };
    }

    dispatch_arm!(
        val, f, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
        23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68,
        69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91,
        92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111,
        112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129,
        130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147,
        148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165,
        166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183,
        184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201,
        202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219,
        220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237,
        238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255
    )
}
