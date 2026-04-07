use num_bigint::BigInt;
use once_cell::sync::Lazy;

/// Maximum combined bit length of numerator and denominator.
/// If `numerator.bits() + denominator.bits()` exceeds this value,
/// the rational is considered too large to be useful and `None` is returned
/// by arithmetic operations instead of a reduced result.
pub const MAX_SIZE: usize = 10_000;

/// Additive identity. Returned directly when a result is exactly zero.
pub static ZERO: Lazy<BigInt> = Lazy::new(|| BigInt::from(0i32));

/// Multiplicative identity. Used as a fast-return shortcut in multiplication.
pub static ONE: Lazy<BigInt> = Lazy::new(|| BigInt::from(1i32));

/// Negative one. Used in negation and sign-check shortcuts.
pub static MINUS_ONE: Lazy<BigInt> = Lazy::new(|| BigInt::from(-1i32));

/// Two. Used in halving, doubling, and base-2 termination checks.
pub static TWO: Lazy<BigInt> = Lazy::new(|| BigInt::from(2i32));

/// Negative two. Used in sign-aware doubling shortcuts.
pub static MINUS_TWO: Lazy<BigInt> = Lazy::new(|| BigInt::from(-2i32));

/// Ten. Used in base-10 scaling and decimal conversion.
pub static TEN: Lazy<BigInt> = Lazy::new(|| BigInt::from(10i32));

/// Used in base-10 termination checks - a decimal terminates
/// if and only if the reduced denominator has no prime factors other than 2 and 5.
pub static FIVE: Lazy<BigInt> = Lazy::new(|| BigInt::from(5i32));

/// A ratio of two arbitrary-precision integers, `numerator/denominator`
///
/// Arithmetic operations return `None` when the result would exceed
/// [`MAX_SIZE`] combined bits, signalling the caller to fall back to
/// a constructive-real approximation. All values are treated as exact
/// until that point.
///
/// # Invariants
/// - The denominator is never zero.
/// - Fractions are not always fully reduced; simplification happens
///   occasionally at random to avoid paying the cost of GCD on every
///   operation.

#[derive(Clone, Debug)]
pub struct BoundedRational {
    /// The top half of the fraction.
    pub numerator: BigInt,
    /// The bottom half of the fraction. Must never be zero.
    pub denominator: BigInt,
}
