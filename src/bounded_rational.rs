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

impl BoundedRational {
    /// Creates a new `BoundedRational` with the given numerator and denominator.
    pub fn new(n: BigInt, d: BigInt) -> Self {
        BoundedRational {
            numerator: n,
            denominator: d,
        }
    }

    /// Creates a `BoundedRational` equal to the integer `n` (denominator = 1).
    pub fn from_bigint(n: BigInt) -> Self {
        BoundedRational {
            numerator: n,
            denominator: ONE.clone(),
        }
    }

    /// Creates a `BoundedRational` from two `i64` values.
    pub fn from_longs(n: i64, d: i64) -> Self {
        Self::new(BigInt::from(n), BigInt::from(d))
    }

    /// Creates a `BoundedRational` equal to the integer `n` (denominator = 1).
    pub fn from_long(n: i64) -> Self {
        Self::from_bigint(BigInt::from(n))
    }

    /// Returns `true` if rational is too large to be useful.
    ///
    /// Specifically, returns `true` when `numerator.bits() + denominator.bits() > MAX_SIZE`.
    /// Pure integers (denominator == 1) are always considered representable and skip the
    /// bit-count check entirely.
    pub fn too_big(&self) -> bool {
        if self.denominator == *ONE {
            return false;
        }
        self.numerator.bits() + self.denominator.bits() > MAX_SIZE as u64
    }

    /// Returns a clone of this rational with a positive denominator.
    ///
    /// If the denominator is negative, both numerator and denominator are negated,
    /// preserving the value while ensuring `denominator > 0`. If the denominator is
    /// already positive (or zero, which is invalid). the value is returned unchanged.
    pub fn positive_den(&self) -> BoundedRational {
        if self.denominator.sign() == num_bigint::Sign::Minus {
            BoundedRational::new(-&self.numerator, -&self.denominator)
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::i64;

    use super::*;

    #[test]
    fn from_longs_stores_numerator_and_denominator() {
        let r = BoundedRational::from_longs(3, 4);
        assert_eq!(r.numerator, BigInt::from(3i64));
        assert_eq!(r.denominator, BigInt::from(4i64));
    }

    #[test]
    fn too_big_pure_integer_is_never_too_big() {
        let r = BoundedRational::from_long(i64::MAX);
        assert!(!r.too_big());
    }

    #[test]
    fn small_fraction_is_not_too_big() {
        let r = BoundedRational::from_longs(3, 4);
        assert!(!r.too_big());
    }

    #[test]
    fn positive_den_positive_denominator_unchanged() {
        let r = BoundedRational::from_longs(3, 4);
        let p = r.positive_den();
        assert_eq!(p.numerator, BigInt::from(3i64));
        assert_eq!(p.denominator, BigInt::from(4i64));
    }

    #[test]
    fn positive_den_negative_denominator_flips_both_signs() {
        let r = BoundedRational::from_longs(3, -4);
        let p = r.positive_den();
        assert_eq!(p.numerator, BigInt::from(-3i64));
        assert_eq!(p.denominator, BigInt::from(4i64));
    }

    #[test]
    fn positive_den_negative_num_and_den_flips_both() {
        let r = BoundedRational::from_longs(-3, -4);
        let p = r.positive_den();
        assert_eq!(p.numerator, BigInt::from(3i64));
        assert_eq!(p.denominator, BigInt::from(4i64));
    }
}
