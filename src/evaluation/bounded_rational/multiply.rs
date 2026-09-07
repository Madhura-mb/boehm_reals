use super::br::BoundedRational;
use crate::evaluation::constants::{MAX_SIZE, MINUS_ONE, ONE, ZERO};
use crate::{IsizePromotion, UsizePromotion};
use num_bigint::BigInt;
use std::iter::Product;
use std::mem;
use std::ops::{Mul, MulAssign};

/// Returns the product of `r1` and `r2` , possibly reduced.
///
/// # Shortcuts
/// - If either argument equals `1` (checked via [`equals`]), the other
///   argument is returned immediately, skipping multiplication entirely.
/// - If either argument equals `-1` (checked via [`equals`]), the other
///   argument is returned with its numerator negated, skipping
///   multiplication entirely.
///
/// [`equals`]: BoundedRational::equals
///
/// # Reduction heuristic
/// Before multiplying, the combined bit sizes of all four components are
/// checked against a threshold of `MAX_SIZE * 3/4`. The result numerator
/// and denominator bit sizes are also checked independently, since either
/// can overflow even when the total input size looks acceptable:
/// - `input_bits  = r1.num.bits + r1.den.bits + r2.num.bits + r2.den.bits`
/// - `result_num_bits = r1.num.bits + r2.num.bits`
/// - `result_den_bits = r1.den.bits + r2.den.bits`
///
/// If any of these exceed the threshold, both `r1` and `r2` are reduced and
/// sign-normalised before the multiplication, keeping intermediate values
/// small. When this pre-reduction fires, the `maybe_reduce` step afterwards
/// is skipped, since a second reduction pass would be redundant. When the
/// threshold is not exceeded, `maybe_reduce` is called on the raw product
/// as usual.
macro_rules! boundedrational_mul {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;

        if a.equals(&ONE) {
            b
        } else if b.equals(&ONE) {
            a
        } else if a.equals(&MINUS_ONE) {
            BoundedRational {
                numerator: -b.numerator,
                denominator: b.denominator,
            }
        } else if b.equals(&MINUS_ONE) {
            BoundedRational {
                numerator: -a.numerator,
                denominator: a.denominator,
            }
        } else {
            let threshold = MAX_SIZE as u64 * 3 / 4;

            let input_bits = a.numerator.bits()
                + a.denominator.bits()
                + b.numerator.bits()
                + b.denominator.bits();

            let result_num_bits = a.numerator.bits() + b.numerator.bits();
            let result_den_bits = a.denominator.bits() + b.denominator.bits();

            let (a, b, already_reduced) = if input_bits > threshold
                || result_num_bits > threshold
                || result_den_bits > threshold
            {
                (a.reduce().positive_den(), b.reduce().positive_den(), true)
            } else {
                (a, b, false)
            };

            let result = BoundedRational {
                numerator: &a.numerator * &b.numerator,
                denominator: &a.denominator * &b.denominator,
            };

            if already_reduced {
                result
            } else {
                BoundedRational::maybe_reduce(result)
            }
        }
    }};
}

// -----------------------------------------------------------------------------
// BoundedRational Multiplication Implementation
// -----------------------------------------------------------------------------

// &BoundedRational * &BoundedRational
impl Mul<&BoundedRational> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BoundedRational) -> BoundedRational {
        boundedrational_mul!(self.clone(), other.clone())
    }
}

// &BoundedRational * BoundedRational
impl Mul<BoundedRational> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BoundedRational) -> BoundedRational {
        boundedrational_mul!(self.clone(), other)
    }
}

// BoundedRational * &BoundedRational
impl Mul<&BoundedRational> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BoundedRational) -> BoundedRational {
        boundedrational_mul!(self, other.clone())
    }
}

// BoundedRational * BoundedRational
impl Mul<BoundedRational> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BoundedRational) -> BoundedRational {
        boundedrational_mul!(self, other)
    }
}

// ============================================================================
// BoundedRational Multiplication Assignment Implementation
// ============================================================================

// BoundedRational *= &BoundedRational
impl MulAssign<&BoundedRational> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &BoundedRational) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= BoundedRational
forward_val_assign!(
    impl MulAssign for BoundedRational,
    mul_assign
);

// ============================================================================
// Scalar Multiplication Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// T * &BoundedRational
// &T * &BoundedRational
// BoundedRational * &T
// &BoundedRational * &T
// &T * BoundedRational
// T * BoundedRational
// &BoundedRational * T
// BoundedRational * T
promote_all_scalars!(impl Mul for BoundedRational, mul);

// u32 * &BoundedRational
// &u32 * &BoundedRational
// BoundedRational * &u32
// &BoundedRational * &u32
// u32 * BoundedRational
// &u32 * BoundedRational
// &BoundedRational * u32
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<u32> for BoundedRational,
    mul
);

// u64 * &BoundedRational
// &u64 * &BoundedRational
// BoundedRational * &u64
// &BoundedRational * &u64
// u64 * BoundedRational
// &u64 * BoundedRational
// &BoundedRational * u64
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<u64> for BoundedRational,
    mul
);

// u128 * &BoundedRational
// &u128 * &BoundedRational
// BoundedRational * &u128
// &BoundedRational * &u128
// u128 * BoundedRational
// &u128 * BoundedRational
// &BoundedRational * u128
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<u128> for BoundedRational,
    mul
);

// BoundedRational * u32
impl Mul<u32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: u32) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational * u64
impl Mul<u64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: u64) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational * u128
impl Mul<u128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: u128) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// i32 * &BoundedRational
// &i32 * &BoundedRational
// BoundedRational * &i32
// &BoundedRational * &i32
// i32 * BoundedRational
// &i32 * BoundedRational
// &BoundedRational * i32
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<i32> for BoundedRational,
    mul
);

// i64 * &BoundedRational
// &i64 * &BoundedRational
// BoundedRational * &i64
// &BoundedRational * &i64
// i64 * BoundedRational
// &i64 * BoundedRational
// &BoundedRational * i64
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<i64> for BoundedRational,
    mul
);

// i128 * &BoundedRational
// &i128 * &BoundedRational
// BoundedRational * &i128
// &BoundedRational * &i128
// i128 * BoundedRational
// &i128 * BoundedRational
// &BoundedRational * i128
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<i128> for BoundedRational,
    mul
);

// BoundedRational * i32
impl Mul<i32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: i32) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational * i64
impl Mul<i64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: i64) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational * i128
impl Mul<i128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: i128) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// ============================================================================
// Scalar Multiplication Assignment Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// BoundedRational *= T
// BoundedRational *= &T
promote_all_scalars_assign!(impl MulAssign for BoundedRational, mul_assign);

// BoundedRational *= u32
impl MulAssign<u32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= u64
impl MulAssign<u64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= u128
impl MulAssign<u128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= i32
impl MulAssign<i32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= i64
impl MulAssign<i64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= i128
impl MulAssign<i128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= &u32
impl MulAssign<&u32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational *= &u64
impl MulAssign<&u64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational *= &u128
impl MulAssign<&u128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational *= &i32
impl MulAssign<&i32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational *= &i64
impl MulAssign<&i64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational *= &i128
impl MulAssign<&i128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// ============================================================================
// BigInt Multiplication Implementation
// ============================================================================

// BoundedRational * BigInt
impl Mul<BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BigInt) -> BoundedRational {
        self * BoundedRational::from_bigint(other)
    }
}

// BoundedRational * &BigInt
impl Mul<&BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BigInt) -> BoundedRational {
        self * BoundedRational::from_bigint(other.clone())
    }
}

// &BoundedRational * BigInt
impl Mul<BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BigInt) -> BoundedRational {
        self.clone() * BoundedRational::from_bigint(other)
    }
}

// &BoundedRational * &BigInt
impl Mul<&BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BigInt) -> BoundedRational {
        self.clone() * BoundedRational::from_bigint(other.clone())
    }
}

// BigInt * BoundedRational
impl Mul<BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BoundedRational) -> BoundedRational {
        other * self
    }
}

// &BigInt * BoundedRational
impl Mul<BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BoundedRational) -> BoundedRational {
        other * self.clone()
    }
}

// BigInt * &BoundedRational
impl Mul<&BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BoundedRational) -> BoundedRational {
        other * self
    }
}

// &BigInt * &BoundedRational
impl Mul<&BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BoundedRational) -> BoundedRational {
        other * self.clone()
    }
}

// ============================================================================
// BigInt Multiplication Assignment Implementation
// ============================================================================

// BoundedRational *= BigInt
impl MulAssign<BigInt> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational *= &BigInt
impl MulAssign<&BigInt> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// ============================================================================
// Product (iterator)
// ============================================================================

impl_product_iter_type!(BoundedRational);

#[cfg(test)]
mod mul_tests {
    use super::*;
    use crate::evaluation::bounded_rational::add::add_tests::{assert_value, br};
    use num_bigint::BigInt;

    // ============================================================================
    // Basic reference/value combination tests
    // ============================================================================

    #[test]
    fn mul_ref_ref_basic() {
        let a = br(1, 2);
        let b = br(2, 3);
        let product = &a * &b;
        assert_value(&product, 1, 3);
        assert_value(&a, 1, 2); // a still usable
        assert_value(&b, 2, 3); // b still usable
    }

    #[test]
    fn mul_ref_val_basic() {
        let a = br(1, 2);
        let product = &a * br(2, 3);
        assert_value(&product, 1, 3);
        assert_value(&a, 1, 2); // a still usable
    }

    #[test]
    fn mul_val_ref_basic() {
        let a = br(1, 2);
        let b = br(2, 3);
        let product = a * &b;
        assert_value(&product, 1, 3);
        assert_value(&b, 2, 3); // b still usable
    }

    #[test]
    fn mul_val_val_basic() {
        let a = br(1, 2);
        let b = br(2, 3);
        let product = a * b;
        assert_value(&product, 1, 3);
    }

    // ============================================================================
    // Identity / shortcut edge cases (equals ONE / MINUS_ONE)
    // ============================================================================

    #[test]
    fn mul_by_one_returns_other_unchanged() {
        let a = br(1, 1);
        let b = br(3, 7);
        let product = &a * &b;
        assert_value(&product, 3, 7);
    }

    #[test]
    fn mul_one_by_other_returns_other_unchanged() {
        let a = br(3, 7);
        let b = br(1, 1);
        let product = &a * &b;
        assert_value(&product, 3, 7);
    }

    #[test]
    fn mul_by_minus_one_negates_numerator() {
        let a = br(-1, 1);
        let b = br(3, 7);
        let product = &a * &b;
        assert_value(&product, -3, 7);
    }

    #[test]
    fn mul_minus_one_by_other_negates_numerator() {
        let a = br(3, 7);
        let b = br(-1, 1);
        let product = &a * &b;
        assert_value(&product, -3, 7);
    }

    #[test]
    fn mul_minus_one_by_minus_one() {
        let a = br(-1, 1);
        let b = br(-1, 1);
        let product = &a * &b;
        assert_value(&product, 1, 1);
    }

    // ============================================================================
    // Zero handling
    // ============================================================================

    #[test]
    fn mul_by_zero_is_zero() {
        let a = br(0, 1);
        let b = br(5, 9);
        let product = &a * &b;
        assert_value(&product, 0, 1);
    }

    #[test]
    fn mul_zero_by_zero_is_zero() {
        let a = br(0, 1);
        let b = br(0, 1);
        let product = &a * &b;
        assert_value(&product, 0, 1);
    }

    // ============================================================================
    // Sign handling
    // ============================================================================

    #[test]
    fn mul_negative_by_positive() {
        let a = br(-2, 3);
        let b = br(5, 7);
        let product = &a * &b;
        assert_value(&product, -10, 21);
    }

    #[test]
    fn mul_negative_by_negative_yields_positive() {
        let a = br(-2, 3);
        let b = br(-5, 7);
        let product = &a * &b;
        assert_value(&product, 10, 21);
    }

    #[test]
    fn mul_with_negative_denominator_normalizes_sign() {
        let a = br(3, -4);
        let b = br(2, 5);
        let product = &a * &b;
        assert_value(&product, -3, 10);
    }

    // ============================================================================
    // Reduction behavior
    // ============================================================================

    #[test]
    fn mul_result_gets_reduced() {
        let a = br(2, 4); // reduces internally to 1/2 conceptually
        let b = br(3, 9); // 1/3
        let product = &a * &b;
        assert_value(&product, 1, 6);
    }

    #[test]
    fn mul_common_factors_cancel() {
        let a = br(4, 9);
        let b = br(9, 4);
        let product = &a * &b;
        assert_value(&product, 1, 1);
    }

    #[test]
    fn mul_large_numerators_triggers_pre_reduction() {
        // Values sized to exceed MAX_SIZE * 3/4 bit threshold, forcing the
        // reduce()+positive_den() path before multiplying.
        let a = br_from_str(
            "123456789012345678901234567890123456789",
            "987654321098765432109876543210987654321",
        );
        let b = br_from_str(
            "111111111111111111111111111111111111111",
            "222222222222222222222222222222222222221",
        );
        let product = &a * &b;
        // Sanity: result should be well-formed and reproducible via commutation
        let product_swapped = &b * &a;
        assert_eq!(product, product_swapped);
    }

    // ============================================================================
    // Commutativity / associativity sanity checks
    // ============================================================================

    #[test]
    fn mul_is_commutative() {
        let a = br(3, 5);
        let b = br(7, 11);
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn mul_is_associative() {
        let a = br(1, 2);
        let b = br(2, 3);
        let c = br(3, 4);
        let left = &(&a * &b) * &c;
        let right = &a * &(&b * &c);
        assert_eq!(left, right);
    }

    // ============================================================================
    // MulAssign
    // ============================================================================

    #[test]
    fn mul_assign_ref_basic() {
        let mut a = br(1, 2);
        let b = br(2, 3);
        a *= &b;
        assert_value(&a, 1, 3);
        assert_value(&b, 2, 3); // b still usable
    }

    #[test]
    fn mul_assign_val_basic() {
        let mut a = br(1, 2);
        a *= br(2, 3);
        assert_value(&a, 1, 3);
    }

    #[test]
    fn mul_assign_by_one_no_change() {
        let mut a = br(5, 9);
        a *= br(1, 1);
        assert_value(&a, 5, 9);
    }

    #[test]
    fn mul_assign_by_minus_one_negates() {
        let mut a = br(5, 9);
        a *= br(-1, 1);
        assert_value(&a, -5, 9);
    }

    #[test]
    fn mul_assign_by_zero() {
        let mut a = br(5, 9);
        a *= br(0, 1);
        assert_value(&a, 0, 1);
    }

    #[test]
    fn mul_assign_repeated() {
        let mut a = br(1, 1);
        for _ in 0..5 {
            a *= br(2, 1);
        }
        assert_value(&a, 32, 1);
    }

    // ============================================================================
    // Scalar multiplication: signed/unsigned promoted small types
    // ============================================================================

    #[test]
    fn mul_u8_scalar() {
        let a = br(1, 3);
        let product = a * 6u8;
        assert_value(&product, 2, 1);
    }

    #[test]
    fn mul_i8_negative_scalar() {
        let a = br(1, 3);
        let product = a * (-6i8);
        assert_value(&product, -2, 1);
    }

    #[test]
    fn mul_usize_scalar() {
        let a = br(1, 4);
        let product = a * 8usize;
        assert_value(&product, 2, 1);
    }

    #[test]
    fn mul_isize_negative_scalar() {
        let a = br(1, 4);
        let product = a * (-8isize);
        assert_value(&product, -2, 1);
    }

    // ============================================================================
    // Scalar multiplication: u32/u64/u128 and i32/i64/i128
    // ============================================================================

    #[test]
    fn mul_u32_scalar_val() {
        let a = br(1, 6);
        let product = a * 3u32;
        assert_value(&product, 1, 2);
    }

    #[test]
    fn mul_u32_scalar_ref() {
        let a = br(1, 6);
        let product = &a * 3u32;
        assert_value(&product, 1, 2);
        assert_value(&a, 1, 6); // a still usable
    }

    #[test]
    fn mul_u32_commutative_scalar_first() {
        let a = br(1, 6);
        let product = 3u32 * &a;
        assert_value(&product, 1, 2);
    }

    #[test]
    fn mul_u64_scalar_zero() {
        let a = br(5, 7);
        let product = a * 0u64;
        assert_value(&product, 0, 1);
    }

    #[test]
    fn mul_u128_scalar_large() {
        let a = br(1, 2);
        let product = a * u128::MAX;
        // just verify denominator becomes 2 and numerator equals u128::MAX
        assert_value(&product, u128::MAX as i128, 2); // adjust helper as needed for big values
    }

    #[test]
    fn mul_i32_scalar_negative() {
        let a = br(2, 5);
        let product = a * (-3i32);
        assert_value(&product, -6, 5);
    }

    #[test]
    fn mul_i64_scalar_val_and_ref_equivalent() {
        let a = br(2, 5);
        let b = a.clone();
        let via_val = a * 4i64;
        let via_ref = &b * 4i64;
        assert_eq!(via_val, via_ref);
    }

    #[test]
    fn mul_i128_scalar_min_value() {
        let a = br(1, 1);
        let product = a * i128::MIN;
        assert_value(&product, i128::MIN, 1);
    }

    // ============================================================================
    // Scalar MulAssign
    // ============================================================================

    #[test]
    fn mul_assign_u32_scalar() {
        let mut a = br(1, 6);
        a *= 3u32;
        assert_value(&a, 1, 2);
    }

    #[test]
    fn mul_assign_u32_ref_scalar() {
        let mut a = br(1, 6);
        a *= &3u32;
        assert_value(&a, 1, 2);
    }

    #[test]
    fn mul_assign_i32_negative_scalar() {
        let mut a = br(1, 6);
        a *= -3i32;
        assert_value(&a, -1, 2);
    }

    #[test]
    fn mul_assign_u64_scalar() {
        let mut a = br(1, 3);
        a *= 9u64;
        assert_value(&a, 3, 1);
    }

    #[test]
    fn mul_assign_i64_ref_scalar() {
        let mut a = br(1, 3);
        a *= &(-9i64);
        assert_value(&a, -3, 1);
    }

    #[test]
    fn mul_assign_u128_scalar() {
        let mut a = br(1, 2);
        a *= 2u128;
        assert_value(&a, 1, 1);
    }

    #[test]
    fn mul_assign_i128_scalar() {
        let mut a = br(1, 2);
        a *= -2i128;
        assert_value(&a, -1, 1);
    }

    // ============================================================================
    // BigInt multiplication
    // ============================================================================

    #[test]
    fn mul_bigint_val() {
        let a = br(1, 6);
        let product = a * BigInt::from(3);
        assert_value(&product, 1, 2);
    }

    #[test]
    fn mul_bigint_ref() {
        let a = br(1, 6);
        let product = a * &BigInt::from(3);
        assert_value(&product, 1, 2);
    }

    #[test]
    fn mul_ref_bigint_val() {
        let a = br(1, 6);
        let product = &a * BigInt::from(3);
        assert_value(&product, 1, 2);
        assert_value(&a, 1, 6); // a still usable
    }

    #[test]
    fn mul_ref_bigint_ref() {
        let a = br(1, 6);
        let product = &a * &BigInt::from(3);
        assert_value(&product, 1, 2);
        assert_value(&a, 1, 6); // a still usable
    }

    #[test]
    fn mul_bigint_by_boundedrational_val() {
        let a = br(1, 6);
        let product = BigInt::from(3) * a;
        assert_value(&product, 1, 2);
    }

    #[test]
    fn mul_bigint_ref_by_boundedrational_val() {
        let a = br(1, 6);
        let product = &BigInt::from(3) * a;
        assert_value(&product, 1, 2);
    }

    #[test]
    fn mul_bigint_by_boundedrational_ref() {
        let a = br(1, 6);
        let product = BigInt::from(3) * &a;
        assert_value(&product, 1, 2);
        assert_value(&a, 1, 6); // a still usable
    }

    #[test]
    fn mul_bigint_ref_by_boundedrational_ref() {
        let a = br(1, 6);
        let product = &BigInt::from(3) * &a;
        assert_value(&product, 1, 2);
        assert_value(&a, 1, 6); // a still usable
    }

    #[test]
    fn mul_bigint_negative() {
        let a = br(1, 6);
        let product = a * BigInt::from(-3);
        assert_value(&product, -1, 2);
    }

    #[test]
    fn mul_bigint_zero() {
        let a = br(5, 7);
        let product = a * BigInt::from(0);
        assert_value(&product, 0, 1);
    }

    // ============================================================================
    // BigInt MulAssign
    // ============================================================================

    #[test]
    fn mul_assign_bigint_val() {
        let mut a = br(1, 6);
        a *= BigInt::from(3);
        assert_value(&a, 1, 2);
    }

    #[test]
    fn mul_assign_bigint_ref() {
        let mut a = br(1, 6);
        a *= &BigInt::from(3);
        assert_value(&a, 1, 2);
    }

    #[test]
    fn mul_assign_bigint_negative() {
        let mut a = br(1, 6);
        a *= BigInt::from(-3);
        assert_value(&a, -1, 2);
    }

    // ============================================================================
    // Product (iterator)
    // ============================================================================

    #[test]
    fn product_empty_iterator_is_one() {
        let values: Vec<BoundedRational> = vec![];
        let product: BoundedRational = values.into_iter().product();
        assert_value(&product, 1, 1);
    }

    #[test]
    fn product_single_element() {
        let values = vec![br(3, 5)];
        let product: BoundedRational = values.into_iter().product();
        assert_value(&product, 3, 5);
    }

    #[test]
    fn product_multiple_elements() {
        let values = vec![br(1, 2), br(2, 3), br(3, 4)];
        let product: BoundedRational = values.into_iter().product();
        assert_value(&product, 1, 4);
    }

    #[test]
    fn product_with_zero_element_is_zero() {
        let values = vec![br(1, 2), br(0, 1), br(3, 4)];
        let product: BoundedRational = values.into_iter().product();
        assert_value(&product, 0, 1);
    }

    #[test]
    fn product_with_negative_elements() {
        let values = vec![br(-1, 2), br(-2, 3)];
        let product: BoundedRational = values.into_iter().product();
        assert_value(&product, 1, 3);
    }

    #[test]
    fn product_of_refs() {
        let a = br(1, 2);
        let b = br(2, 3);
        let c = br(3, 4);
        let values = vec![&a, &b, &c];
        let product: BoundedRational = values.into_iter().product();
        assert_value(&product, 1, 4);
        // originals still usable
        assert_value(&a, 1, 2);
        assert_value(&b, 2, 3);
        assert_value(&c, 3, 4);
    }
}
