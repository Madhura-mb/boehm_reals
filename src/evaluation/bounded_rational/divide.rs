use super::br::BoundedRational;
use super::multiply::boundedrational_mul;
use crate::evaluation::constants::{MAX_SIZE, MINUS_ONE, ONE, ZERO};
use crate::evaluation::errors::ZeroDivisionError;
use crate::{IsizePromotion, UsizePromotion};
use num_bigint::BigInt;
use std::mem;
use std::ops::{Div, DivAssign};

/// Computes `r1 / r2` as `r1 * inverse(r2)`.
///
/// # Panics
/// Panics with `"attempt to divide by zero"` if `r2` is zero.
macro_rules! boundedrational_div {
    ($r1:expr, $r2:expr) => {{
        let r2: BoundedRational = $r2;
        boundedrational_mul!($r1, BoundedRational::inverse(r2))
    }};
}

impl BoundedRational {
    /// Divides `self` by `other`, returning an error instead of panicking
    /// when `other` is zero.
    ///
    /// This is the non-panicking counterpart to the `boundedrational_div!`
    /// macro (which go through [`self: inverse`] and panic on a zero divisor).
    ///
    /// # Errors
    /// Returns `Err(ZeroDivisionError)` if `other` is zero. Otherwise
    /// returns `Ok` with the exact quotient.
    pub fn checked_div(
        &self,
        other: &BoundedRational,
    ) -> Result<BoundedRational, ZeroDivisionError> {
        if *other.numerator() == *ZERO {
            return Err(ZeroDivisionError);
        }
        Ok(self / other)
    }
}

// -----------------------------------------------------------------------------
// BoundedRational Division Implementation
// -----------------------------------------------------------------------------

// &BoundedRational / &BoundedRational
impl Div<&BoundedRational> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: &BoundedRational) -> BoundedRational {
        boundedrational_div!(self.clone(), other.clone())
    }
}

// BoundedRational / BoundedRational
// BoundedRational / &BoundedRational
// &BoundedRational / BoundedRational
forward_all_binop_to_ref_ref!(impl Div for BoundedRational, div);

// ============================================================================
// BoundedRational Division Assignment Implementation
// ============================================================================

// BoundedRational /= &BoundedRational
impl DivAssign<&BoundedRational> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &BoundedRational) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= BoundedRational
forward_val_assign!(
    impl DivAssign for BoundedRational,
    div_assign
);

// ============================================================================
// Scalar Division Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// T / &BoundedRational
// &T / &BoundedRational
// BoundedRational / &T
// &BoundedRational / &T
// &T / BoundedRational
// T / BoundedRational
// &BoundedRational / T
// BoundedRational / T
promote_all_scalars!(impl Div for BoundedRational, div);

// u32 / &BoundedRational
// &u32 / &BoundedRational
// BoundedRational / &u32
// &BoundedRational / &u32
// &u32 / BoundedRational
// &BoundedRational / u32
forward_all_scalar_binop_to_val_val!(
    impl Div<u32> for BoundedRational,
    div
);

// u64 / &BoundedRational
// &u64 / &BoundedRational
// BoundedRational / &u64
// &BoundedRational / &u64
// &u64 / BoundedRational
// &BoundedRational / u64
forward_all_scalar_binop_to_val_val!(
    impl Div<u64> for BoundedRational,
    div
);

// u128 / &BoundedRational
// &u128 / &BoundedRational
// BoundedRational / &u128
// &BoundedRational / &u128
// &u128 / BoundedRational
// &BoundedRational / u128
forward_all_scalar_binop_to_val_val!(
    impl Div<u128> for BoundedRational,
    div
);

// u32 / BoundedRational
impl Div<BoundedRational> for u32 {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) / other
    }
}

// u64 / BoundedRational
impl Div<BoundedRational> for u64 {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) / other
    }
}

// u128 / BoundedRational
impl Div<BoundedRational> for u128 {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) / other
    }
}

// BoundedRational / u32
impl Div<u32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: u32) -> BoundedRational {
        self / BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / u64
impl Div<u64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: u64) -> BoundedRational {
        self / BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / u128
impl Div<u128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: u128) -> BoundedRational {
        self / BoundedRational::from_bigint(BigInt::from(other))
    }
}

// i32 / &BoundedRational
// &i32 / &BoundedRational
// BoundedRational / &i32
// &BoundedRational / &i32
// &i32 / BoundedRational
// &BoundedRational / i32
forward_all_scalar_binop_to_val_val!(
    impl Div<i32> for BoundedRational,
    div
);

// i64 / &BoundedRational
// &i64 / &BoundedRational
// BoundedRational / &i64
// &BoundedRational / &i64
// &i64 / BoundedRational
// &BoundedRational / i64
forward_all_scalar_binop_to_val_val!(
    impl Div<i64> for BoundedRational,
    div
);

// i128 / &BoundedRational
// &i128 / &BoundedRational
// BoundedRational / &i128
// &BoundedRational / &i128
// &i128 / BoundedRational
// &BoundedRational / i128
forward_all_scalar_binop_to_val_val!(
    impl Div<i128> for BoundedRational,
    div
);

// i32 / BoundedRational
impl Div<BoundedRational> for i32 {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) / other
    }
}

// i64 / BoundedRational
impl Div<BoundedRational> for i64 {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) / other
    }
}

// i128 / BoundedRational
impl Div<BoundedRational> for i128 {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(BigInt::from(self)) / other
    }
}

// BoundedRational / i32
impl Div<i32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: i32) -> BoundedRational {
        self / BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / i64
impl Div<i64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: i64) -> BoundedRational {
        self / BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / i128
impl Div<i128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: i128) -> BoundedRational {
        self / BoundedRational::from_bigint(BigInt::from(other))
    }
}

// ============================================================================
// Scalar Division Assignment Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// BoundedRational /= T
// BoundedRational /= &T
promote_all_scalars_assign!(impl DivAssign for BoundedRational, div_assign);

// BoundedRational /= u32
impl DivAssign<u32> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= u64
impl DivAssign<u64> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= u128
impl DivAssign<u128> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= i32
impl DivAssign<i32> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= i64
impl DivAssign<i64> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= i128
impl DivAssign<i128> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= &u32
impl DivAssign<&u32> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / *other;
    }
}

// BoundedRational /= &u64
impl DivAssign<&u64> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / *other;
    }
}

// BoundedRational /= &u128
impl DivAssign<&u128> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / *other;
    }
}

// BoundedRational /= &i32
impl DivAssign<&i32> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / *other;
    }
}

// BoundedRational /= &i64
impl DivAssign<&i64> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / *other;
    }
}

// BoundedRational /= &i128
impl DivAssign<&i128> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / *other;
    }
}

// ============================================================================
// BigInt Division Implementation
// ============================================================================

// BoundedRational / BigInt
impl Div<BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BigInt) -> BoundedRational {
        self / BoundedRational::from_bigint(other)
    }
}

// BoundedRational / &BigInt
impl Div<&BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: &BigInt) -> BoundedRational {
        self / BoundedRational::from_bigint(other.clone())
    }
}

// &BoundedRational / BigInt
impl Div<BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BigInt) -> BoundedRational {
        self.clone() / BoundedRational::from_bigint(other)
    }
}

// &BoundedRational / &BigInt
impl Div<&BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: &BigInt) -> BoundedRational {
        self.clone() / BoundedRational::from_bigint(other.clone())
    }
}

// BigInt / BoundedRational
impl Div<BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self) / other
    }
}

// &BigInt / BoundedRational
impl Div<BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self.clone()) / other
    }
}

// BigInt / &BoundedRational
impl Div<&BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: &BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self) / other
    }
}

// &BigInt / &BoundedRational
impl Div<&BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn div(self, other: &BoundedRational) -> BoundedRational {
        BoundedRational::from_bigint(self.clone()) / other
    }
}

// ============================================================================
// BigInt Division Assignment Implementation
// ============================================================================

// BoundedRational /= BigInt
impl DivAssign<BigInt> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

// BoundedRational /= &BigInt
impl DivAssign<&BigInt> for BoundedRational {
    #[inline]
    fn div_assign(&mut self, other: &BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n / other;
    }
}

#[cfg(test)]
mod div_tests {
    use super::*;
    use crate::evaluation::bounded_rational::test_helpers::{assert_value, br};
    use num_bigint::BigInt;

    // ── checked_div ─────────────────────────────────────────────────────────

    #[test]
    fn checked_div_by_zero_returns_err() {
        let r1 = BoundedRational::from_long(5); // 5/1
        let r2 = BoundedRational::from_long(0); // 0/1
        assert!(r1.checked_div(&r2).is_err());
    }

    #[test]
    fn checked_div_by_zero_over_nonzero_denominator_returns_err() {
        let r1 = BoundedRational::from_longs(3, 4).unwrap(); // 3/4
        let r2 = BoundedRational::from_longs(0, 7).unwrap(); // 0/7, normalizes numerator to 0
        assert!(r1.checked_div(&r2).is_err());
    }

    // -------------------------------------------------------------------
    // Basic value/reference combinations
    // -------------------------------------------------------------------

    #[test]
    fn div_ref_ref_basic() {
        let a = br(3, 4);
        let b = br(1, 2);
        let quot = &a / &b;
        assert_value(&quot, 3, 2);
        assert_value(&a, 3, 4); // a still usable
        assert_value(&b, 1, 2); // b still usable
    }

    #[test]
    fn div_ref_val_basic() {
        let a = br(1, 2);
        let quot = &a / br(1, 3);
        assert_value(&quot, 3, 2);
        assert_value(&a, 1, 2); // a still usable
    }

    #[test]
    fn div_val_ref_basic() {
        let a = br(1, 2);
        let b = br(1, 3);
        let quot = a / &b;
        assert_value(&quot, 3, 2);
        assert_value(&b, 1, 3); // b still usable
    }

    #[test]
    fn div_val_val_basic() {
        let a = br(1, 2);
        let b = br(1, 3);
        let quot = a / b;
        assert_value(&quot, 3, 2);
    }

    // -------------------------------------------------------------------
    // Edge cases: identity, negatives, self-division, reciprocal
    // -------------------------------------------------------------------

    #[test]
    fn div_by_one_is_identity() {
        let a = br(5, 7);
        let one = br(1, 1);
        let quot = &a / &one;
        assert_value(&quot, 5, 7);
    }

    #[test]
    fn div_zero_by_value_is_zero() {
        let zero = br(0, 1);
        let a = br(5, 7);
        let quot = &zero / &a;
        assert_value(&quot, 0, 1);
    }

    #[test]
    fn div_equal_values_yields_one() {
        let a = br(7, 9);
        let b = br(7, 9);
        let quot = &a / &b;
        assert_value(&quot, 1, 1);
    }

    #[test]
    fn div_self_yields_one() {
        let a = br(11, 13);
        let quot = &a / &a;
        assert_value(&quot, 1, 1);
    }

    #[test]
    fn div_result_is_fraction_less_than_one() {
        let a = br(1, 4);
        let b = br(1, 2);
        let quot = a / b;
        assert_value(&quot, 1, 2);
    }

    #[test]
    fn div_negative_by_negative() {
        let a = br(-1, 3);
        let b = br(-1, 6);
        let quot = a / b;
        assert_value(&quot, 2, 1);
    }

    #[test]
    fn div_negative_by_positive() {
        let a = br(-1, 2);
        let b = br(1, 2);
        let quot = a / b;
        assert_value(&quot, -1, 1);
    }

    #[test]
    fn div_positive_by_negative() {
        let a = br(1, 2);
        let b = br(-1, 2);
        let quot = a / b;
        assert_value(&quot, -1, 1);
    }

    #[test]
    fn div_different_denominators() {
        let a = br(3, 4);
        let b = br(1, 12);
        let quot = a / b;
        assert_value(&quot, 9, 1);
    }

    #[test]
    fn div_whole_numbers() {
        let a = br(10, 1);
        let b = br(3, 1);
        let quot = a / b;
        assert_value(&quot, 10, 3);
    }

    #[test]
    fn div_large_numerators() {
        let a = br(1_000_000_000, 1);
        let b = br(1_000, 1);
        let quot = a / b;
        assert_value(&quot, 1_000_000, 1);
    }

    #[test]
    fn div_by_reciprocal_of_self() {
        let a = br(3, 5);
        let recip = br(5, 3);
        let quot = &a / &recip;
        assert_value(&quot, 9, 25);
    }

    #[test]
    fn div_negative_denominator_both_sides() {
        let quot = br(1, -2) / br(1, -3);
        // -1/2 / -1/3 = (1/2)/(1/3) = 3/2
        assert_value(&quot, 3, 2);
    }

    #[test]
    fn div_zero_with_negative_denominator_operand() {
        let quot = br(0, -5) / br(1, -2);
        // 0 / anything nonzero = 0
        assert_value(&quot, 0, 1);
    }

    // -------------------------------------------------------------------
    // Division by zero panics
    // -------------------------------------------------------------------

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_by_zero_ref_ref_panics() {
        let a = br(5, 7);
        let zero = br(0, 1);
        let _ = &a / &zero;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_by_zero_val_val_panics() {
        let a = br(5, 7);
        let zero = br(0, 1);
        let _ = a / zero;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_zero_by_zero_panics() {
        let a = br(0, 1);
        let zero = br(0, 1);
        let _ = a / zero;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_by_zero_with_negative_denominator_panics() {
        // numerator 0 regardless of the sign carried by denominator
        let a = br(5, 7);
        let zero = br(0, -1);
        let _ = a / zero;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_assign_by_zero_panics() {
        let mut a = br(5, 7);
        a /= br(0, 1);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_by_zero_scalar_u32_panics() {
        let a = br(5, 7);
        let _ = a / 0u32;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_by_zero_scalar_i32_panics() {
        let a = br(5, 7);
        let _ = a / 0i32;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_by_zero_bigint_panics() {
        let a = br(5, 7);
        let big = BigInt::from(0);
        let _ = a / big;
    }

    // -------------------------------------------------------------------
    // DivAssign (BoundedRational /= BoundedRational)
    // -------------------------------------------------------------------

    #[test]
    fn div_assign_ref() {
        let mut a = br(3, 4);
        let b = br(1, 2);
        a /= &b;
        assert_value(&a, 3, 2);
    }

    #[test]
    fn div_assign_val() {
        let mut a = br(3, 4);
        let b = br(1, 2);
        a /= b;
        assert_value(&a, 3, 2);
    }

    #[test]
    fn div_assign_chained() {
        let mut a = br(100, 1);
        a /= br(2, 1);
        a /= br(5, 1);
        a /= &br(2, 1);
        assert_value(&a, 5, 1);
    }

    #[test]
    fn div_assign_to_one() {
        let mut a = br(5, 6);
        a /= br(5, 6);
        assert_value(&a, 1, 1);
    }

    #[test]
    fn div_large_but_reducible_triggers_pre_reduction_heuristic() {
        let factor = BigInt::from(1u32) << (MAX_SIZE / 2);
        let r1 = BoundedRational::new(&factor * 5, factor.clone()).unwrap();
        let r2 = BoundedRational::new(&factor * 3, factor).unwrap();
        let quot = &r1 / &r2;
        // (5*factor)/factor / ((3*factor)/factor) = 5 / 3
        assert_value(&quot, 5, 3);
    }

    // -------------------------------------------------------------------
    // Scalar division: u32
    // -------------------------------------------------------------------

    #[test]
    fn div_boundedrational_by_u32() {
        let a = br(9, 2);
        let quot = a / 3u32;
        assert_value(&quot, 3, 2);
    }

    #[test]
    fn div_u32_by_boundedrational() {
        let a = br(1, 2);
        let quot = 3u32 / a;
        assert_value(&quot, 6, 1);
    }

    #[test]
    fn div_ref_boundedrational_by_ref_u32() {
        let a = br(9, 2);
        let n: u32 = 3;
        let quot = &a / &n;
        assert_value(&quot, 3, 2);
        assert_value(&a, 9, 2);
    }

    #[test]
    fn div_assign_u32() {
        let mut a = br(9, 2);
        a /= 3u32;
        assert_value(&a, 3, 2);
    }

    #[test]
    fn div_assign_ref_u32() {
        let mut a = br(9, 2);
        a /= &3u32;
        assert_value(&a, 3, 2);
    }

    // -------------------------------------------------------------------
    // Scalar division: i32 (including negative scalar)
    // -------------------------------------------------------------------

    #[test]
    fn div_boundedrational_by_i32() {
        let a = br(1, 2);
        let quot = a / (-4i32);
        assert_value(&quot, -1, 8);
    }

    #[test]
    fn div_i32_by_boundedrational() {
        let a = br(1, 2);
        let quot = -3i32 / a;
        assert_value(&quot, -6, 1);
    }

    #[test]
    fn div_assign_i32_negative() {
        let mut a = br(1, 2);
        a /= -4i32;
        assert_value(&a, -1, 8);
    }

    // -------------------------------------------------------------------
    // Scalar division: u64 / i64
    // -------------------------------------------------------------------

    #[test]
    fn div_boundedrational_by_u64() {
        let a = br(9, 1);
        let quot = a / 4u64;
        assert_value(&quot, 9, 4);
    }

    #[test]
    fn div_u64_by_boundedrational() {
        let a = br(4, 1);
        let quot = 9u64 / a;
        assert_value(&quot, 9, 4);
    }

    #[test]
    fn div_assign_u64() {
        let mut a = br(9, 1);
        a /= 4u64;
        assert_value(&a, 9, 4);
    }

    #[test]
    fn div_boundedrational_by_i64() {
        let a = br(1, 3);
        let quot = a / (-2i64);
        assert_value(&quot, -1, 6);
    }

    #[test]
    fn div_assign_ref_i64() {
        let mut a = br(1, 3);
        a /= &(-2i64);
        assert_value(&a, -1, 6);
    }

    // -------------------------------------------------------------------
    // Scalar division: u128 / i128 (large scalar edge case)
    // -------------------------------------------------------------------

    #[test]
    fn div_boundedrational_by_u128_large() {
        let a = br(1, 1);
        let big: u128 = u128::MAX;
        let quot = a.clone() / big;

        let expected = BoundedRational::new(BigInt::from(1), BigInt::from(u128::MAX)).unwrap();
        assert_eq!(quot, expected);
    }

    #[test]
    fn div_u128_by_boundedrational() {
        let a = br(5, 1);
        let quot = 10u128 / a;
        assert_value(&quot, 2, 1);
    }

    #[test]
    fn div_assign_u128() {
        let mut a = br(10, 1);
        a /= 5u128;
        assert_value(&a, 2, 1);
    }

    #[test]
    fn div_boundedrational_by_i128_negative() {
        let a = br(1, 1);
        let quot = a / (-1i128);
        assert_value(&quot, -1, 1);
    }

    #[test]
    fn div_assign_ref_i128() {
        let mut a = br(10, 1);
        a /= &5i128;
        assert_value(&a, 2, 1);
    }

    // -------------------------------------------------------------------
    // Promoted scalars: u8, u16, usize, i8, i16, isize
    // -------------------------------------------------------------------

    #[test]
    fn div_boundedrational_by_u8() {
        let a = br(20, 1);
        let quot = a / 5u8;
        assert_value(&quot, 4, 1);
    }

    #[test]
    fn div_boundedrational_by_u16() {
        let a = br(1000, 1);
        let quot = a / 4u16;
        assert_value(&quot, 250, 1);
    }

    #[test]
    fn div_boundedrational_by_usize() {
        let a = br(50, 1);
        let quot = a / 5usize;
        assert_value(&quot, 10, 1);
    }

    #[test]
    fn div_boundedrational_by_i8() {
        let a = br(10, 1);
        let quot = a / (-5i8);
        assert_value(&quot, -2, 1);
    }

    #[test]
    fn div_boundedrational_by_i16() {
        let a = br(100, 1);
        let quot = a / (-100i16);
        assert_value(&quot, -1, 1);
    }

    #[test]
    fn div_boundedrational_by_isize() {
        let a = br(0, 1);
        let quot = a / (-1isize);
        assert_value(&quot, 0, 1);
    }

    #[test]
    fn div_assign_promoted_u8() {
        let mut a = br(20, 1);
        a /= 5u8;
        assert_value(&a, 4, 1);
    }

    #[test]
    fn div_assign_promoted_ref_i16() {
        let mut a = br(100, 1);
        a /= &(-100i16);
        assert_value(&a, -1, 1);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_by_zero_promoted_u8_panics() {
        let a = br(5, 7);
        let _ = a / 0u8;
    }

    // -------------------------------------------------------------------
    // BigInt division
    // -------------------------------------------------------------------

    #[test]
    fn div_boundedrational_by_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let quot = a / big;
        assert_value(&quot, 5, 2);
    }

    #[test]
    fn div_boundedrational_by_ref_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let quot = a / &big;
        assert_value(&quot, 5, 2);
    }

    #[test]
    fn div_ref_boundedrational_by_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let quot = &a / big;
        assert_value(&quot, 5, 2);
        assert_value(&a, 10, 1); // a still usable
    }

    #[test]
    fn div_ref_boundedrational_by_ref_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(4);
        let quot = &a / &big;
        assert_value(&quot, 5, 2);
    }

    #[test]
    fn div_bigint_by_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let quot = big / a;
        assert_value(&quot, 5, 2);
    }

    #[test]
    fn div_ref_bigint_by_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let quot = &big / a;
        assert_value(&quot, 5, 2);
    }

    #[test]
    fn div_bigint_by_ref_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let quot = big / &a;
        assert_value(&quot, 5, 2);
        assert_value(&a, 4, 1);
    }

    #[test]
    fn div_ref_bigint_by_ref_boundedrational() {
        let a = br(4, 1);
        let big = BigInt::from(10);
        let quot = &big / &a;
        assert_value(&quot, 5, 2);
    }

    #[test]
    fn div_boundedrational_by_negative_bigint() {
        let a = br(10, 1);
        let big = BigInt::from(-5);
        let quot = a / big;
        assert_value(&quot, -2, 1);
    }

    #[test]
    fn div_bigint_larger_than_i128() {
        // Exercise BigInt path with a value outside i128 range.
        let a = br(1, 1);
        let huge = BigInt::parse_bytes(b"123456789012345678901234567890123456789", 10).unwrap();
        let quot = &huge / &a;

        // Expected result is just `huge` as an integer BoundedRational (denominator 1),
        // since a = 1.
        let expected = BoundedRational::from_bigint(huge.clone());
        assert_eq!(quot, expected);
    }

    #[test]
    fn div_bigint_zero_by_boundedrational() {
        let a = br(5, 7);
        let big = BigInt::from(0);
        let quot = big / a;
        assert_value(&quot, 0, 1);
    }

    // -------------------------------------------------------------------
    // BigInt DivAssign
    // -------------------------------------------------------------------

    #[test]
    fn div_assign_bigint() {
        let mut a = br(10, 1);
        a /= BigInt::from(4);
        assert_value(&a, 5, 2);
    }

    #[test]
    fn div_assign_ref_bigint() {
        let mut a = br(10, 1);
        let big = BigInt::from(4);
        a /= &big;
        assert_value(&a, 5, 2);
    }

    #[test]
    fn div_assign_bigint_negative() {
        let mut a = br(3, 1);
        a /= BigInt::from(-10);
        assert_value(&a, -3, 10);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_assign_bigint_zero_panics() {
        let mut a = br(5, 7);
        a /= BigInt::from(0);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_assign_ref_bigint_zero_panics() {
        let mut a = br(5, 7);
        let zero = BigInt::from(0);
        a /= &zero;
    }

    // -------------------------------------------------------------------
    // Mixed fraction edge cases
    // -------------------------------------------------------------------

    #[test]
    fn div_fractions_with_common_denominator() {
        let a = br(5, 6);
        let b = br(1, 6);
        let quot = a / b;
        assert_value(&quot, 5, 1);
    }

    #[test]
    fn div_fractions_coprime_denominators() {
        let a = br(1, 3);
        let b = br(1, 5);
        let quot = a / b;
        assert_value(&quot, 5, 3);
    }

    #[test]
    fn div_negative_fraction_by_negative_scalar() {
        let a = br(-1, 2);
        let quot = a / (-1i32);
        assert_value(&quot, 1, 2);
    }

    #[test]
    fn div_then_multiply_back_recovers_original() {
        let a = br(7, 3);
        let b = br(2, 5);
        let quot = &a / &b;
        let back = &quot * &b;
        assert_value(&back, 7, 3);
    }

    // -------------------------------------------------------------------
    // Reverse-direction scalar division: scalar / BoundedRational
    // -------------------------------------------------------------------

    #[test]
    fn div_u8_by_boundedrational() {
        let a = br(5, 1);
        let quot = 20u8 / a;
        assert_value(&quot, 4, 1);
    }

    #[test]
    fn div_u16_by_boundedrational() {
        let a = br(4, 1);
        let quot = 1000u16 / a;
        assert_value(&quot, 250, 1);
    }

    #[test]
    fn div_usize_by_boundedrational() {
        let a = br(2, 1);
        let quot = 50usize / a;
        assert_value(&quot, 25, 1);
    }

    #[test]
    fn div_i8_by_boundedrational() {
        let a = br(-5, 1);
        let quot = 5i8 / a;
        assert_value(&quot, -1, 1);
    }

    #[test]
    fn div_i16_by_boundedrational() {
        let a = br(-100, 1);
        let quot = 100i16 / a;
        assert_value(&quot, -1, 1);
    }

    #[test]
    fn div_isize_by_boundedrational() {
        let a = br(-1, 1);
        let quot = 1isize / a;
        assert_value(&quot, -1, 1);
    }

    #[test]
    fn div_i64_by_boundedrational() {
        let a = br(1, 3);
        let quot = -2i64 / a;
        // -2 / (1/3) = -6
        assert_value(&quot, -6, 1);
    }

    #[test]
    fn div_i128_by_boundedrational() {
        let a = br(1, 1);
        let quot = -1i128 / a;
        assert_value(&quot, -1, 1);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_u32_by_zero_boundedrational_panics() {
        let zero = br(0, 1);
        let _ = 5u32 / zero;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_bigint_by_zero_boundedrational_panics() {
        let zero = br(0, 1);
        let big = BigInt::from(5);
        let _ = big / zero;
    }
}
