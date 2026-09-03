use super::add::boundedrational_add;
use super::br::BoundedRational;
use crate::evaluation::constants::{MAX_SIZE, ZERO};
use crate::{IsizePromotion, UsizePromotion};
use num_bigint::BigInt;
use std::mem;
use std::ops::{Sub, SubAssign};

/// `r1 - r2` is defined as `r1 + (-r2)`. We negate the right-hand operand
/// into a local binding, then forward straight into `boundedrational_add!`
/// so all of `add`'s zero-check / reduction-heuristic logic is reused
/// as-is — no duplication, and `add`'s own optimizations automatically
/// apply to subtraction too.
macro_rules! boundedrational_sub {
    ($a:expr, $a_owned:expr, $b:expr, $b_owned:expr) => {{
        let neg_b = BoundedRational::negate($b_owned);
        boundedrational_add!($a, $a_owned, &neg_b, neg_b)
    }};
}

// -----------------------------------------------------------------------------
// BoundedRational Subtraction Implementation
// -----------------------------------------------------------------------------

// &BoundedRational - &BoundedRational
impl Sub<&BoundedRational> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: &BoundedRational) -> BoundedRational {
        boundedrational_sub!(self, self.clone(), other, other.clone())
    }
}

// &BoundedRational - BoundedRational
impl Sub<BoundedRational> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        boundedrational_sub!(self, self.clone(), other, other)
    }
}

// BoundedRational - &BoundedRational
impl Sub<&BoundedRational> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: &BoundedRational) -> BoundedRational {
        boundedrational_sub!(self, self, other, other.clone())
    }
}

// BoundedRational - BoundedRational
impl Sub<BoundedRational> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        boundedrational_sub!(self, self, other, other)
    }
}

// ============================================================================
// BoundedRational Subtraction Assignment Implementation
// ============================================================================

// BoundedRational -= &BoundedRational
impl SubAssign<&BoundedRational> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &BoundedRational) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= BoundedRational
forward_val_assign!(
    impl SubAssign for BoundedRational,
    sub_assign
);

// ============================================================================
// Scalar Subtraction Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// T - &BoundedRational
// &T - &BoundedRational
// BoundedRational - &T
// &BoundedRational - &T
// &T - BoundedRational
// T - BoundedRational
// &BoundedRational - T
// BoundedRational - T
promote_all_scalars!(impl Sub for BoundedRational, sub);

// u32 - &BoundedRational
// &u32 - &BoundedRational
// BoundedRational - &u32
// &BoundedRational - &u32
// &u32 - BoundedRational
// &BoundedRational - u32
forward_all_scalar_binop_to_val_val!(
    impl Sub<u32> for BoundedRational,
    sub
);

// u64 - &BoundedRational
// &u64 - &BoundedRational
// BoundedRational - &u64
// &BoundedRational - &u64
// &u64 - BoundedRational
// &BoundedRational - u64
forward_all_scalar_binop_to_val_val!(
    impl Sub<u64> for BoundedRational,
    sub
);

// u128 - &BoundedRational
// &u128 - &BoundedRational
// BoundedRational - &u128
// &BoundedRational - &u128
// &u128 - BoundedRational
// &BoundedRational - u128
forward_all_scalar_binop_to_val_val!(
    impl Sub<u128> for BoundedRational,
    sub
);

// u32 - BoundedRational
impl Sub<BoundedRational> for u32 {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) - other
    }
}

// u64 - BoundedRational
impl Sub<BoundedRational> for u64 {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) - other
    }
}

// u128 - BoundedRational
impl Sub<BoundedRational> for u128 {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) - other
    }
}

// BoundedRational - u32
impl Sub<u32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: u32) -> BoundedRational {
        self - BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational - u64
impl Sub<u64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: u64) -> BoundedRational {
        self - BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational - u128
impl Sub<u128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: u128) -> BoundedRational {
        self - BoundedRational::from_bigint(BigInt::from(other))
    }
}

// i32 - &BoundedRational
// &i32 - &BoundedRational
// BoundedRational - &i32
// &BoundedRational - &i32
// &i32 - BoundedRational
// &BoundedRational - i32
forward_all_scalar_binop_to_val_val!(
    impl Sub<i32> for BoundedRational,
    sub
);

// i64 - &BoundedRational
// &i64 - &BoundedRational
// BoundedRational - &i64
// &BoundedRational - &i64
// &i64 - BoundedRational
// &BoundedRational - i64
forward_all_scalar_binop_to_val_val!(
    impl Sub<i64> for BoundedRational,
    sub
);

// i128 - &BoundedRational
// &i128 - &BoundedRational
// BoundedRational - &i128
// &BoundedRational - &i128
// &i128 - BoundedRational
// &BoundedRational - i128
forward_all_scalar_binop_to_val_val!(
    impl Sub<i128> for BoundedRational,
    sub
);

// i32 - BoundedRational
impl Sub<BoundedRational> for i32 {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) - other
    }
}

// i64 - BoundedRational
impl Sub<BoundedRational> for i64 {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) - other
    }
}

// i128 - BoundedRational
impl Sub<BoundedRational> for i128 {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) - other
    }
}

// BoundedRational - i32
impl Sub<i32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: i32) -> BoundedRational {
        self - BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational - i64
impl Sub<i64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: i64) -> BoundedRational {
        self - BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational - i128
impl Sub<i128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: i128) -> BoundedRational {
        self - BoundedRational::from_bigint(BigInt::from(other))
    }
}

// ============================================================================
// Scalar Subtraction Assignment Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// BoundedRational -= T
// BoundedRational -= &T
promote_all_scalars_assign!(impl SubAssign for BoundedRational, sub_assign);

// BoundedRational -= u32
impl SubAssign<u32> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= u64
impl SubAssign<u64> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= u128
impl SubAssign<u128> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= i32
impl SubAssign<i32> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= i64
impl SubAssign<i64> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= i128
impl SubAssign<i128> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= &u32
impl SubAssign<&u32> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - *other;
    }
}

// BoundedRational -= &u64
impl SubAssign<&u64> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - *other;
    }
}

// BoundedRational -= &u128
impl SubAssign<&u128> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - *other;
    }
}

// BoundedRational -= &i32
impl SubAssign<&i32> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - *other;
    }
}

// BoundedRational -= &i64
impl SubAssign<&i64> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - *other;
    }
}

// BoundedRational -= &i128
impl SubAssign<&i128> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - *other;
    }
}

// ============================================================================
// BigInt Subtraction Implementation
// ============================================================================

// BoundedRational - BigInt
impl Sub<BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BigInt) -> BoundedRational {
        self - BoundedRational::from_bigint(other)
    }
}

// BoundedRational - &BigInt
impl Sub<&BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: &BigInt) -> BoundedRational {
        self - BoundedRational::from_bigint(other.clone())
    }
}

// &BoundedRational - BigInt
impl Sub<BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BigInt) -> BoundedRational {
        self.clone() - BoundedRational::from_bigint(other)
    }
}

// &BoundedRational - &BigInt
impl Sub<&BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: &BigInt) -> BoundedRational {
        self.clone() - BoundedRational::from_bigint(other.clone())
    }
}

// BigInt - BoundedRational
impl Sub<BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self) - other
    }
}

// &BigInt - BoundedRational
impl Sub<BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self.clone()) - other
    }
}

// BigInt - &BoundedRational
impl Sub<&BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: &BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self) - other
    }
}

// &BigInt - &BoundedRational
impl Sub<&BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn sub(self, other: &BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self.clone()) - other
    }
}

// ============================================================================
// BigInt Subtrction Assignment Implementation
// ============================================================================

// BoundedRational -= BigInt
impl SubAssign<BigInt> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

// BoundedRational -= &BigInt
impl SubAssign<&BigInt> for BoundedRational {
    #[inline]
    fn sub_assign(&mut self, other: &BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n - other;
    }
}

#[cfg(test)]
mod sub_tests {
    use super::*;
    use crate::evaluation::bounded_rational::add::add_tests::{assert_value, br};
    use num_bigint::BigInt;

    // Assumes helper functions `br(num, den)` -> BoundedRational
    // and `assert_value(&BoundedRational, num, den)` exist in the test harness,
    // matching the style used elsewhere in this crate's test suite.

    // -------------------------------------------------------------------
    // Basic value/reference combinations
    // -------------------------------------------------------------------

    #[test]
    fn sub_ref_ref_basic() {
        let a = br(3, 4);
        let b = br(1, 4);
        let diff = &a - &b;
        assert_value(&diff, 1, 2);
        assert_value(&a, 3, 4); // a still usable
        assert_value(&b, 1, 4); // b still usable
    }

    #[test]
    fn sub_ref_val_basic() {
        let a = br(1, 2);
        let diff = &a - br(1, 3);
        assert_value(&diff, 1, 6);
        assert_value(&a, 1, 2); // a still usable
    }

    #[test]
    fn sub_val_ref_basic() {
        let a = br(1, 2);
        let b = br(1, 3);
        let diff = a - &b;
        assert_value(&diff, 1, 6);
        assert_value(&b, 1, 3); // b still usable
    }

    #[test]
    fn sub_val_val_basic() {
        let a = br(1, 2);
        let b = br(1, 3);
        let diff = a - b;
        assert_value(&diff, 1, 6);
    }

    // -------------------------------------------------------------------
    // Edge cases: zero, identity, negative results, self-subtraction
    // -------------------------------------------------------------------

    #[test]
    fn sub_subtracting_zero_is_identity() {
        let a = br(5, 7);
        let zero = br(0, 1);
        let diff = &a - &zero;
        assert_value(&diff, 5, 7);
    }

    #[test]
    fn sub_zero_minus_value_negates() {
        let zero = br(0, 1);
        let a = br(5, 7);
        let diff = &zero - &a;
        assert_value(&diff, -5, 7);
    }

    #[test]
    fn sub_equal_values_yields_zero() {
        let a = br(7, 9);
        let b = br(7, 9);
        let diff = &a - &b;
        assert_value(&diff, 0, 1);
    }

    #[test]
    fn sub_self_yields_zero() {
        let a = br(11, 13);
        let diff = &a - &a;
        assert_value(&diff, 0, 1);
    }

    #[test]
    fn sub_result_is_negative() {
        let a = br(1, 4);
        let b = br(1, 2);
        let diff = a - b;
        assert_value(&diff, -1, 4);
    }

    #[test]
    fn sub_negative_minus_negative() {
        let a = br(-1, 3);
        let b = br(-1, 6);
        let diff = a - b;
        assert_value(&diff, -1, 6);
    }

    #[test]
    fn sub_negative_minus_positive() {
        let a = br(-1, 2);
        let b = br(1, 2);
        let diff = a - b;
        assert_value(&diff, -1, 1);
    }

    #[test]
    fn sub_different_denominators_reduces() {
        // 2/4 - 1/4 = 1/4 (numerator should reduce, not stay 2/4 - 1/4 unreduced)
        let a = br(2, 4);
        let b = br(1, 4);
        let diff = a - b;
        assert_value(&diff, 1, 4);
    }

    #[test]
    fn sub_whole_numbers() {
        let a = br(10, 1);
        let b = br(3, 1);
        let diff = a - b;
        assert_value(&diff, 7, 1);
    }

    #[test]
    fn sub_large_numerators() {
        let a = br(1_000_000_007, 1);
        let b = br(999_999_999, 1);
        let diff = a - b;
        assert_value(&diff, 8, 1);
    }

    // -------------------------------------------------------------------
    // SubAssign (BoundedRational -= BoundedRational)
    // -------------------------------------------------------------------

    #[test]
    fn sub_assign_ref() {
        let mut a = br(3, 4);
        let b = br(1, 4);
        a -= &b;
        assert_value(&a, 1, 2);
    }

    #[test]
    fn sub_assign_val() {
        let mut a = br(3, 4);
        let b = br(1, 4);
        a -= b;
        assert_value(&a, 1, 2);
    }

    #[test]
    fn sub_assign_chained() {
        let mut a = br(10, 1);
        a -= br(3, 1);
        a -= br(2, 1);
        a -= &br(1, 1);
        assert_value(&a, 4, 1);
    }

    #[test]
    fn sub_assign_to_zero() {
        let mut a = br(5, 6);
        a -= br(5, 6);
        assert_value(&a, 0, 1);
    }

    // -------------------------------------------------------------------
    // Scalar subtraction: u32
    // -------------------------------------------------------------------

    #[test]
    fn sub_boundedrational_minus_u32() {
        let a = br(7, 2);
        let diff = a - 3u32;
        assert_value(&diff, 1, 2);
    }

    #[test]
    fn sub_u32_minus_boundedrational() {
        let a = br(1, 2);
        let diff = 3u32 - a;
        assert_value(&diff, 5, 2);
    }

    #[test]
    fn sub_ref_boundedrational_minus_ref_u32() {
        let a = br(7, 2);
        let n: u32 = 3;
        let diff = &a - &n;
        assert_value(&diff, 1, 2);
        assert_value(&a, 7, 2);
    }

    #[test]
    fn sub_assign_u32() {
        let mut a = br(7, 2);
        a -= 3u32;
        assert_value(&a, 1, 2);
    }

    #[test]
    fn sub_assign_ref_u32() {
        let mut a = br(7, 2);
        a -= &3u32;
        assert_value(&a, 1, 2);
    }

    // -------------------------------------------------------------------
    // Scalar subtraction: i32 (including negative scalar)
    // -------------------------------------------------------------------

    #[test]
    fn sub_boundedrational_minus_i32() {
        let a = br(1, 2);
        let diff = a - (-3i32);
        assert_value(&diff, 7, 2);
    }

    #[test]
    fn sub_i32_minus_boundedrational() {
        let a = br(1, 2);
        let diff = -3i32 - a;
        assert_value(&diff, -7, 2);
    }

    #[test]
    fn sub_assign_i32_negative() {
        let mut a = br(1, 2);
        a -= -3i32;
        assert_value(&a, 7, 2);
    }

    // -------------------------------------------------------------------
    // Scalar subtraction: u64 / i64
    // -------------------------------------------------------------------

    #[test]
    fn sub_boundedrational_minus_u64() {
        let a = br(9, 1);
        let diff = a - 4u64;
        assert_value(&diff, 5, 1);
    }

    #[test]
    fn sub_u64_minus_boundedrational() {
        let a = br(4, 1);
        let diff = 9u64 - a;
        assert_value(&diff, 5, 1);
    }

    #[test]
    fn sub_assign_u64() {
        let mut a = br(9, 1);
        a -= 4u64;
        assert_value(&a, 5, 1);
    }

    #[test]
    fn sub_boundedrational_minus_i64() {
        let a = br(1, 3);
        let diff = a - (-2i64);
        assert_value(&diff, 7, 3);
    }

    #[test]
    fn sub_assign_ref_i64() {
        let mut a = br(1, 3);
        a -= &(-2i64);
        assert_value(&a, 7, 3);
    }

    // -------------------------------------------------------------------
    // Scalar subtraction: u128 / i128 (large scalar edge case)
    // -------------------------------------------------------------------

    #[test]
    fn sub_boundedrational_minus_u128_large() {
        let a = br(1, 1);
        let big: u128 = u128::MAX;
        let diff = a.clone() - big;

        let expected_num = BigInt::from(1) - BigInt::from(u128::MAX);
        let expected = BoundedRational::from_bigint(expected_num);

        assert_eq!(diff, expected);
    }

    #[test]
    fn sub_u128_minus_boundedrational() {
        let a = br(1, 1);
        let diff = 5u128 - a;
        assert_value(&diff, 4, 1);
    }

    #[test]
    fn sub_assign_u128() {
        let mut a = br(10, 1);
        a -= 3u128;
        assert_value(&a, 7, 1);
    }

    #[test]
    fn sub_boundedrational_minus_i128_negative() {
        let a = br(0, 1);
        let diff = a - (-1i128);
        assert_value(&diff, 1, 1);
    }

    #[test]
    fn sub_assign_ref_i128() {
        let mut a = br(10, 1);
        a -= &3i128;
        assert_value(&a, 7, 1);
    }

    // -------------------------------------------------------------------
    // Promoted scalars: u8, u16, usize, i8, i16, isize
    // -------------------------------------------------------------------

    #[test]
    fn sub_boundedrational_minus_u8() {
        let a = br(20, 1);
        let diff = a - 5u8;
        assert_value(&diff, 15, 1);
    }

    #[test]
    fn sub_boundedrational_minus_u16() {
        let a = br(1000, 1);
        let diff = a - 999u16;
        assert_value(&diff, 1, 1);
    }

    #[test]
    fn sub_boundedrational_minus_usize() {
        let a = br(50, 1);
        let diff = a - 50usize;
        assert_value(&diff, 0, 1);
    }

    #[test]
    fn sub_boundedrational_minus_i8() {
        let a = br(5, 1);
        let diff = a - (-5i8);
        assert_value(&diff, 10, 1);
    }

    #[test]
    fn sub_boundedrational_minus_i16() {
        let a = br(100, 1);
        let diff = a - (-100i16);
        assert_value(&diff, 200, 1);
    }

    #[test]
    fn sub_boundedrational_minus_isize() {
        let a = br(0, 1);
        let diff = a - (-1isize);
        assert_value(&diff, 1, 1);
    }

    #[test]
    fn sub_assign_promoted_u8() {
        let mut a = br(20, 1);
        a -= 5u8;
        assert_value(&a, 15, 1);
    }

    #[test]
    fn sub_assign_promoted_ref_i16() {
        let mut a = br(100, 1);
        a -= &(-100i16);
        assert_value(&a, 200, 1);
    }

    // -------------------------------------------------------------------
    // BigInt subtraction
    // -------------------------------------------------------------------

    #[test]
    fn sub_boundedrational_minus_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let diff = a - big;
        assert_value(&diff, 6, 1);
    }

    #[test]
    fn sub_boundedrational_minus_ref_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let diff = a - &big;
        assert_value(&diff, 6, 1);
    }

    #[test]
    fn sub_ref_boundedrational_minus_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let diff = &a - big;
        assert_value(&diff, 6, 1);
        assert_value(&a, 10, 1); // a still usable
    }

    #[test]
    fn sub_ref_boundedrational_minus_ref_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let diff = &a - &big;
        assert_value(&diff, 6, 1);
    }

    #[test]
    fn sub_bigint_minus_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let diff = big - a;
        assert_value(&diff, 6, 1);
    }

    #[test]
    fn sub_ref_bigint_minus_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let diff = &big - a;
        assert_value(&diff, 6, 1);
    }

    #[test]
    fn sub_bigint_minus_ref_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let diff = big - &a;
        assert_value(&diff, 6, 1);
        assert_value(&a, 4, 1);
    }

    #[test]
    fn sub_ref_bigint_minus_ref_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let diff = &big - &a;
        assert_value(&diff, 6, 1);
    }

    #[test]
    fn sub_boundedrational_minus_negative_bigint() {
        let a = br(5, 1);
        let big = BigInt::from(-5);
        let diff = a - big;
        assert_value(&diff, 10, 1);
    }

    #[test]
    fn sub_bigint_larger_than_i128() {
        // Exercise BigInt path with a value outside i128 range.
        let a = br(0, 1);
        let huge = BigInt::parse_bytes(b"123456789012345678901234567890123456789", 10).unwrap();
        let diff = &huge - &a;

        // Expected result is just `huge` as an integer BoundedRational (denominator 1),
        // since a = 0. Compare directly instead of poking at internal parts.
        let expected = BoundedRational::from_bigint(huge.clone());
        assert_eq!(diff, expected);
    }

    // -------------------------------------------------------------------
    // BigInt SubAssign
    // -------------------------------------------------------------------

    #[test]
    fn sub_assign_bigint() {
        let mut a = br(10, 1);
        a -= BigInt::from(4);
        assert_value(&a, 6, 1);
    }

    #[test]
    fn sub_assign_ref_bigint() {
        let mut a = br(10, 1);
        let big = BigInt::from(4);
        a -= &big;
        assert_value(&a, 6, 1);
    }

    #[test]
    fn sub_assign_bigint_to_negative() {
        let mut a = br(3, 1);
        a -= BigInt::from(10);
        assert_value(&a, -7, 1);
    }

    // -------------------------------------------------------------------
    // Mixed fraction edge cases
    // -------------------------------------------------------------------

    #[test]
    fn sub_fractions_with_common_reduction() {
        // 5/6 - 1/6 = 4/6 -> reduces to 2/3
        let a = br(5, 6);
        let b = br(1, 6);
        let diff = a - b;
        assert_value(&diff, 2, 3);
    }

    #[test]
    fn sub_fractions_coprime_denominators() {
        // 1/3 - 1/5 = 2/15
        let a = br(1, 3);
        let b = br(1, 5);
        let diff = a - b;
        assert_value(&diff, 2, 15);
    }

    #[test]
    fn sub_negative_fraction_minus_negative_scalar() {
        let a = br(-1, 2);
        let diff = a - (-1i32);
        assert_value(&diff, 1, 2);
    }
}
