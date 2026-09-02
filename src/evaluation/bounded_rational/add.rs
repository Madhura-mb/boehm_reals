use super::br::BoundedRational;
use crate::evaluation::constants::{MAX_SIZE, ZERO};
use crate::{IsizePromotion, UsizePromotion};
use num_bigint::BigInt;
use std::iter::Sum;
use std::mem;
use std::ops::{Add, AddAssign};

/// Returns the sum of `r1` and `r2`, possibly reduces.
///
/// If either operand is exactly zero, the other operand is returned
/// unchanged (aside from being passed through [`maybe_reduce`]), avoiding
/// unnecessary cross multiplication work entirely.
///
/// Before performing the addition, this function may reduce both operands
/// when their combined bit size is large. This heuristic helps avoid
/// creating unnecessarily large intermediate numerators and denominators
/// during cross multiplication while preserving the final value.
///
/// The resulting fraction is passed through [`maybe_reduce`] to keep its
/// size within the configured bounds when possible.
macro_rules! boundedrational_add {
    ($a:expr, $a_owned:expr, $b:expr, $b_owned:expr) => {{
        // Zero check: adding zero is a no-op, so just return the other
        // operand (still passed through `maybe_reduce`, matching the
        // original semantics).
        if *$a.numerator() == *ZERO {
            BoundedRational::maybe_reduce($b_owned)
        } else if *$b.numerator() == *ZERO {
            BoundedRational::maybe_reduce($a_owned)
        } else {
            // Heuristic: if the sum of input bit sizes is already close to
            // MAX_SIZE, reduce both inputs first to avoid huge
            // intermediates during cross multiplication.
            let input_bits = $a.numerator().bits()
                + $a.denominator().bits()
                + $b.numerator().bits()
                + $b.denominator().bits();

            if input_bits > (MAX_SIZE as u64 * 3 / 4) {
                let ra = $a_owned.reduce().positive_den();
                let rb = $b_owned.reduce().positive_den();

                let den = ra.denominator() * rb.denominator();
                let num = ra.numerator() * rb.denominator() + ra.denominator() * rb.numerator();

                BoundedRational::maybe_reduce(
                    BoundedRational::new(num, den).expect("denominator is nonzero"),
                )
            } else {
                let den = $a.denominator() * $b.denominator();
                let num = $a.numerator() * $b.denominator() + $a.denominator() * $b.numerator();
                BoundedRational::maybe_reduce(
                    BoundedRational::new(num, den).expect("denominator is nonzero"),
                )
            }
        }
    }};
}

// -----------------------------------------------------------------------------
// BoundedRational Addition Implementation
// -----------------------------------------------------------------------------

// &BoundedRational + &BoundedRational
impl Add<&BoundedRational> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: &BoundedRational) -> BoundedRational {
        boundedrational_add!(self, self.clone(), other, other.clone())
    }
}

// &BoundedRational + BoundedRational
impl Add<BoundedRational> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: BoundedRational) -> BoundedRational {
        boundedrational_add!(self, self.clone(), other, other)
    }
}

// BoundedRational + &BoundedRational
impl Add<&BoundedRational> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: &BoundedRational) -> BoundedRational {
        boundedrational_add!(self, self, other, other.clone())
    }
}

// BoundedRational + BoundedRational
impl Add<BoundedRational> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: BoundedRational) -> BoundedRational {
        boundedrational_add!(self, self, other, other)
    }
}

// ============================================================================
// BoundedRational Addition Assignment Implementation
// ============================================================================

// BoundedRational += &BoundedRational
impl AddAssign<&BoundedRational> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &BoundedRational) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += BoundedRational
forward_val_assign!(
    impl AddAssign for BoundedRational,
    add_assign
);

// ============================================================================
// Scalar Addition Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// T + &BoundedRational
// &T + &BoundedRational
// BoundedRational + &T
// &BoundedRational + &T
// &T + BoundedRational
// T + BoundedRational
// &BoundedRational + T
// BoundedRational + T
promote_all_scalars!(impl Add for BoundedRational, add);

// u32 + &BoundedRational
// &u32 + &BoundedRational
// BoundedRational + &u32
// &BoundedRational + &u32
// u32 + BoundedRational
// &u32 + BoundedRational
// &BoundedRational + u32
forward_all_scalar_binop_to_val_val_commutative!(
    impl Add<u32> for BoundedRational,
    add
);

// u64 + &BoundedRational
// &u64 + &BoundedRational
// BoundedRational + &u64
// &BoundedRational + &u64
// u64 + BoundedRational
// &u64 + BoundedRational
// &BoundedRational + u64
forward_all_scalar_binop_to_val_val_commutative!(
    impl Add<u64> for BoundedRational,
    add
);

// u128 + &BoundedRational
// &u128 + &BoundedRational
// BoundedRational + &u128
// &BoundedRational + &u128
// u128 + BoundedRational
// &u128 + BoundedRational
// &BoundedRational + u128
forward_all_scalar_binop_to_val_val_commutative!(
    impl Add<u128> for BoundedRational,
    add
);

// BoundedRational + u32
impl Add<u32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: u32) -> BoundedRational {
        self + BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational + u64
impl Add<u64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: u64) -> BoundedRational {
        self + BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational + u128
impl Add<u128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: u128) -> BoundedRational {
        self + BoundedRational::from_bigint(BigInt::from(other))
    }
}

// i32 + &BoundedRational
// &i32 + &BoundedRational
// BoundedRational + &i32
// &BoundedRational + &i32
// i32 + BoundedRational
// &i32 + BoundedRational
// &BoundedRational + i32
forward_all_scalar_binop_to_val_val_commutative!(
    impl Add<i32> for BoundedRational,
    add
);

// i64 + &BoundedRational
// &i64 + &BoundedRational
// BoundedRational + &i64
// &BoundedRational + &i64
// i64 + BoundedRational
// &i64 + BoundedRational
// &BoundedRational + i64
forward_all_scalar_binop_to_val_val_commutative!(
    impl Add<i64> for BoundedRational,
    add
);

// i128 + &BoundedRational
// &i128 + &BoundedRational
// BoundedRational + &i128
// &BoundedRational + &i128
// i128 + BoundedRational
// &i128 + BoundedRational
// &BoundedRational + i128
forward_all_scalar_binop_to_val_val_commutative!(
    impl Add<i128> for BoundedRational,
    add
);

// BoundedRational + i32
impl Add<i32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: i32) -> BoundedRational {
        self + BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational + i64
impl Add<i64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: i64) -> BoundedRational {
        self + BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational + i128
impl Add<i128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: i128) -> BoundedRational {
        self + BoundedRational::from_bigint(BigInt::from(other))
    }
}

// ============================================================================
// Scalar Addition Assignment Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// BoundedRational += T
// BoundedRational += &T
promote_all_scalars_assign!(impl AddAssign for BoundedRational, add_assign);

// BoundedRational += u32
impl AddAssign<u32> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += u64
impl AddAssign<u64> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += u128
impl AddAssign<u128> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += i32
impl AddAssign<i32> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += i64
impl AddAssign<i64> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += i128
impl AddAssign<i128> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += &u32
impl AddAssign<&u32> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + *other;
    }
}

// BoundedRational += &u64
impl AddAssign<&u64> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + *other;
    }
}

// BoundedRational += &u128
impl AddAssign<&u128> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + *other;
    }
}

// BoundedRational += &i32
impl AddAssign<&i32> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + *other;
    }
}

// BoundedRational += &i64
impl AddAssign<&i64> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + *other;
    }
}

// BoundedRational += &i128
impl AddAssign<&i128> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + *other;
    }
}

// ============================================================================
// BigInt Addition Implementation
// ============================================================================

// BoundedRational + BigInt
impl Add<BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: BigInt) -> BoundedRational {
        self + BoundedRational::from_bigint(other)
    }
}

// BoundedRational + &BigInt
impl Add<&BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: &BigInt) -> BoundedRational {
        self + BoundedRational::from_bigint(other.clone())
    }
}

// &BoundedRational + BigInt
impl Add<BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: BigInt) -> BoundedRational {
        self.clone() + BoundedRational::from_bigint(other)
    }
}

// &BoundedRational + &BigInt
impl Add<&BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: &BigInt) -> BoundedRational {
        self.clone() + BoundedRational::from_bigint(other.clone())
    }
}

// BigInt + BoundedRational
impl Add<BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: BoundedRational) -> BoundedRational {
        other + self
    }
}

// &BigInt + BoundedRational
impl Add<BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: BoundedRational) -> BoundedRational {
        other + self.clone()
    }
}

// BigInt + &BoundedRational
impl Add<&BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: &BoundedRational) -> BoundedRational {
        other + self
    }
}

// &BigInt + &BoundedRational
impl Add<&BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn add(self, other: &BoundedRational) -> BoundedRational {
        other + self.clone()
    }
}

// BoundedRational += BigInt
impl AddAssign<BigInt> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// BoundedRational += &BigInt
impl AddAssign<&BigInt> for BoundedRational {
    #[inline]
    fn add_assign(&mut self, other: &BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n + other;
    }
}

// ============================================================================
// Sum (iterator)
// ============================================================================

impl_sum_iter_type!(BoundedRational);

#[cfg(test)]
mod add_tests {
    use super::*;
    use num_bigint::BigInt;

    // Helper Functions
    fn br(n: i64, d: i64) -> BoundedRational {
        BoundedRational::from_longs(n, d).unwrap()
    }

    fn assert_value(r: &BoundedRational, num: i64, den: i64) {
        let reduced = r.reduce().positive_den();
        let expected = BoundedRational::from_longs(num, den)
            .unwrap()
            .reduce()
            .positive_den();
        assert_eq!(*reduced.numerator(), *expected.numerator());
        assert_eq!(*reduced.denominator(), *expected.denominator());
    }

    // =========================================================================
    // BoundedRational + BoundedRational — all 4 ownership combinations
    // =========================================================================

    #[test]
    fn add_val_val_basic() {
        let sum = br(1, 2) + br(1, 3);
        assert_value(&sum, 5, 6);
    }

    #[test]
    fn add_ref_ref_basic() {
        let (a, b) = (br(1, 2), br(1, 3));
        let sum = &a + &b;
        assert_value(&sum, 5, 6);
        // originals still usable
        assert_value(&a, 1, 2);
        assert_value(&b, 1, 3);
    }

    #[test]
    fn add_ref_val_basic() {
        let a = br(1, 2);
        let sum = &a + br(1, 3);
        assert_value(&sum, 5, 6);
        assert_value(&a, 1, 2); // a still usable
    }

    #[test]
    fn add_val_ref_basic() {
        let b = br(1, 3);
        let sum = br(1, 2) + &b;
        assert_value(&sum, 5, 6);
        assert_value(&b, 1, 3); // b still usable
    }

    // ── Zero edge cases ──────────────────────────────────────────────────

    #[test]
    fn add_left_zero_returns_right() {
        let sum = br(0, 1) + br(3, 7);
        assert_value(&sum, 3, 7);
    }

    #[test]
    fn add_right_zero_returns_left() {
        let sum = br(3, 7) + br(0, 1);
        assert_value(&sum, 3, 7);
    }

    #[test]
    fn add_both_zero() {
        let sum = br(0, 1) + br(0, 5);
        assert_value(&sum, 0, 1);
    }

    #[test]
    fn add_zero_with_negative_denominator_operand() {
        // zero-numerator operand with negative denominator, still short-circuits correctly
        let sum = br(0, -5) + br(1, -2);
        assert_value(&sum, -1, 2);
    }

    // ── Sign edge cases ──────────────────────────────────────────────────

    #[test]
    fn add_result_is_zero() {
        let sum = br(1, 2) + br(-1, 2);
        assert_value(&sum, 0, 1);
    }

    #[test]
    fn add_negative_plus_negative() {
        let sum = br(-1, 3) + br(-1, 6);
        assert_value(&sum, -1, 2);
    }

    #[test]
    fn add_negative_denominator_both_sides() {
        let sum = br(1, -2) + br(1, -3);
        // -1/2 + -1/3 = -5/6
        assert_value(&sum, -5, 6);
    }

    // ── Large-magnitude / reduction heuristic ──────────────────────────────

    #[test]
    fn add_large_but_reducible_triggers_pre_reduction_heuristic() {
        let factor = BigInt::from(1u32) << (MAX_SIZE / 2);
        let r1 = BoundedRational::new(&factor * 2, factor.clone()).unwrap();
        let r2 = BoundedRational::new(&factor * 3, factor).unwrap();
        let sum = &r1 + &r2;
        assert_value(&sum, 5, 1);
    }

    #[test]
    fn add_is_commutative() {
        let a = br(3, 7);
        let b = br(5, 11);
        let sum1 = (&a + &b).reduce().positive_den();
        let sum2 = (&b + &a).reduce().positive_den();
        assert_eq!(sum1.numerator(), sum2.numerator());
        assert_eq!(sum1.denominator(), sum2.denominator());
    }

    // =========================================================================
    // AddAssign: BoundedRational
    // =========================================================================

    #[test]
    fn add_assign_val() {
        let mut a = br(1, 2);
        a += br(1, 3);
        assert_value(&a, 5, 6);
    }

    #[test]
    fn add_assign_ref() {
        let mut a = br(1, 2);
        let b = br(1, 3);
        a += &b;
        assert_value(&a, 5, 6);
        assert_value(&b, 1, 3); // b still usable
    }

    #[test]
    fn add_assign_zero_is_noop() {
        let mut a = br(3, 7);
        a += br(0, 1);
        assert_value(&a, 3, 7);
    }

    #[test]
    fn add_assign_chained() {
        let mut a = br(0, 1);
        for _ in 0..5 {
            a += br(1, 1);
        }
        assert_value(&a, 5, 1);
    }

    // =========================================================================
    // Scalar addition: unsigned (u32, u64, u128) and promoted (u8, u16, usize)
    // =========================================================================

    #[test]
    fn add_u32_zero() {
        let sum = br(3, 4) + 0u32;
        assert_value(&sum, 3, 4);
    }

    #[test]
    fn add_u32_max() {
        let sum = br(0, 1) + u32::MAX;
        assert_value(&sum, u32::MAX as i64, 1);
    }

    #[test]
    fn add_u64_max() {
        let sum = BoundedRational::from_bigint(BigInt::from(0)) + u64::MAX;
        assert_eq!(*sum.numerator(), BigInt::from(u64::MAX));
    }

    #[test]
    fn add_u128_large() {
        let big = u128::MAX;
        let sum = BoundedRational::from_bigint(BigInt::from(0)) + big;
        assert_eq!(*sum.numerator(), BigInt::from(big));
    }

    #[test]
    fn add_u8_promotes_correctly() {
        let sum = br(1, 2) + 5u8;
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn add_u16_promotes_correctly() {
        let sum = br(1, 2) + 5u16;
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn add_usize_promotes_correctly() {
        let sum = br(1, 2) + 5usize;
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn add_u32_scalar_commutative() {
        let sum1 = br(1, 2) + 5u32;
        let sum2 = 5u32 + br(1, 2);
        assert_value(&sum1, 11, 2);
        assert_value(&sum2, 11, 2);
    }

    #[test]
    fn add_u32_ref_ref_combo() {
        let a = br(1, 2);
        let sum = &a + &5u32;
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn add_assign_u32() {
        let mut a = br(1, 2);
        a += 3u32;
        assert_value(&a, 7, 2);
    }

    #[test]
    fn add_assign_u8_promotes() {
        let mut a = br(1, 2);
        a += 3u8;
        assert_value(&a, 7, 2);
    }

    // =========================================================================
    // Scalar addition: signed (i32, i64, i128) and promoted (i8, i16, isize)
    // =========================================================================

    #[test]
    fn add_i32_negative() {
        let sum = br(1, 2) + (-3i32);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn add_i32_min() {
        let sum = BoundedRational::from_bigint(BigInt::from(0)) + i32::MIN;
        assert_eq!(*sum.numerator(), BigInt::from(i32::MIN));
    }

    #[test]
    fn add_i64_min() {
        let sum = BoundedRational::from_bigint(BigInt::from(0)) + i64::MIN;
        assert_eq!(*sum.numerator(), BigInt::from(i64::MIN));
    }

    #[test]
    fn add_i128_min() {
        let sum = BoundedRational::from_bigint(BigInt::from(0)) + i128::MIN;
        assert_eq!(*sum.numerator(), BigInt::from(i128::MIN));
    }

    #[test]
    fn add_i8_negative_promotes() {
        let sum = br(1, 2) + (-3i8);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn add_i16_negative_promotes() {
        let sum = br(1, 2) + (-3i16);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn add_isize_negative_promotes() {
        let sum = br(1, 2) + (-3isize);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn add_i32_scalar_commutative() {
        let sum1 = br(1, 2) + (-3i32);
        let sum2 = (-3i32) + br(1, 2);
        assert_value(&sum1, -5, 2);
        assert_value(&sum2, -5, 2);
    }

    #[test]
    fn add_assign_i32_negative() {
        let mut a = br(1, 2);
        a += -3i32;
        assert_value(&a, -5, 2);
    }

    #[test]
    fn add_assign_i8_promotes() {
        let mut a = br(1, 2);
        a += -3i8;
        assert_value(&a, -5, 2);
    }

    #[test]
    fn add_signed_zero_result() {
        let sum = br(5, 1) + (-5i32);
        assert_value(&sum, 0, 1);
    }

    // =========================================================================
    // Sum (iterator)
    // =========================================================================

    #[test]
    fn sum_empty_iterator_is_zero() {
        let items: Vec<BoundedRational> = vec![];
        let total: BoundedRational = items.into_iter().sum();
        assert_value(&total, 0, 1);
    }

    #[test]
    fn sum_single_item() {
        let items = vec![br(3, 4)];
        let total: BoundedRational = items.into_iter().sum();
        assert_value(&total, 3, 4);
    }

    #[test]
    fn sum_multiple_bounded_rationals() {
        let items = vec![br(1, 2), br(1, 3), br(1, 6)];
        let total: BoundedRational = items.into_iter().sum();
        assert_value(&total, 1, 1);
    }

    #[test]
    fn sum_with_negative_values_cancel_out() {
        let items = vec![br(1, 2), br(-1, 2)];
        let total: BoundedRational = items.into_iter().sum();
        assert_value(&total, 0, 1);
    }

    #[test]
    fn sum_of_i32_scalars() {
        let items: Vec<i32> = vec![1, 2, 3, 4, 5];
        let total: BoundedRational = items.into_iter().sum();
        assert_value(&total, 15, 1);
    }

    #[test]
    fn sum_of_negative_i32_scalars() {
        let items: Vec<i32> = vec![-1, -2, -3];
        let total: BoundedRational = items.into_iter().sum();
        assert_value(&total, -6, 1);
    }

    // =========================================================================
    // BigInt: Add — all 4 ownership combinations, both directions
    // =========================================================================

    #[test]
    fn add_bigint_val_val() {
        let sum = br(1, 2) + BigInt::from(3);
        assert_value(&sum, 7, 2);
    }

    #[test]
    fn add_bigint_val_ref() {
        let b = BigInt::from(3);
        let sum = br(1, 2) + &b;
        assert_value(&sum, 7, 2);
        assert_eq!(b, BigInt::from(3)); // b still usable
    }

    #[test]
    fn add_bigint_ref_val() {
        let a = br(1, 2);
        let sum = &a + BigInt::from(3);
        assert_value(&sum, 7, 2);
        assert_value(&a, 1, 2); // a still usable
    }

    #[test]
    fn add_bigint_ref_ref() {
        let a = br(1, 2);
        let b = BigInt::from(3);
        let sum = &a + &b;
        assert_value(&sum, 7, 2);
        assert_value(&a, 1, 2);
        assert_eq!(b, BigInt::from(3));
    }

    #[test]
    fn add_bigint_negative() {
        let sum = br(1, 2) + BigInt::from(-3);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn add_bigint_zero() {
        let sum = br(3, 4) + BigInt::from(0);
        assert_value(&sum, 3, 4);
    }

    #[test]
    fn add_bigint_huge_value() {
        let huge = BigInt::from(1u32) << 200u32;
        let sum = br(1, 2) + huge.clone();
        // sum = huge + 1/2 => numerator should be 2*huge + 1 over den 2
        let expected_num = &huge * 2 + 1;
        let reduced = sum.reduce();
        assert_eq!(*reduced.numerator(), expected_num);
        assert_eq!(*reduced.denominator(), BigInt::from(2));
    }

    // ── BigInt + BoundedRational (reverse direction) ────────────────────

    #[test]
    fn bigint_plus_boundedrational_val_val() {
        let sum = BigInt::from(3) + br(1, 2);
        assert_value(&sum, 7, 2);
    }

    #[test]
    fn bigint_plus_boundedrational_ref_val() {
        let b = BigInt::from(3);
        let sum = &b + br(1, 2);
        assert_value(&sum, 7, 2);
    }

    #[test]
    fn bigint_plus_boundedrational_val_ref() {
        let a = br(1, 2);
        let sum = BigInt::from(3) + &a;
        assert_value(&sum, 7, 2);
    }

    #[test]
    fn bigint_plus_boundedrational_ref_ref() {
        let a = br(1, 2);
        let b = BigInt::from(3);
        let sum = &b + &a;
        assert_value(&sum, 7, 2);
    }

    #[test]
    fn bigint_plus_boundedrational_matches_reverse_order() {
        let a = br(2, 3);
        let b = BigInt::from(5);
        let sum1 = (a.clone() + b.clone()).reduce().positive_den();
        let sum2 = (b + a).reduce().positive_den();
        assert_eq!(sum1.numerator(), sum2.numerator());
        assert_eq!(sum1.denominator(), sum2.denominator());
    }

    // =========================================================================
    // T + BoundedRational (commutative scalar direction) — u32/u64/u128/i32/i64/i128 only
    // These 6 types get all 4 ownership combos via
    // forward_all_scalar_binop_to_val_val_commutative!.
    // The promoted types (u8/u16/usize/i8/i16/isize) do NOT get this direction —
    // there is no `scalar + BR` impl for them, only `BR + scalar`.
    // =========================================================================

    // ---- u32 ----
    #[test]
    fn u32_plus_br_val_val() {
        let sum = 5u32 + br(1, 2);
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn u32_plus_br_val_ref() {
        let a = br(1, 2);
        let sum = 5u32 + &a;
        assert_value(&sum, 11, 2);
        assert_value(&a, 1, 2); // a still usable
    }

    #[test]
    fn u32_ref_plus_br_val() {
        let sum = &5u32 + br(1, 2);
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn u32_ref_plus_br_ref() {
        let a = br(1, 2);
        let sum = &5u32 + &a;
        assert_value(&sum, 11, 2);
        assert_value(&a, 1, 2);
    }

    #[test]
    fn u32_zero_plus_br() {
        let sum = 0u32 + br(3, 7);
        assert_value(&sum, 3, 7);
    }

    // ---- u64 ----
    #[test]
    fn u64_plus_br_val_val() {
        let sum = 5u64 + br(1, 2);
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn u64_max_plus_br() {
        let sum = u64::MAX + br(0, 1);
        assert_eq!(*sum.numerator(), BigInt::from(u64::MAX));
    }

    #[test]
    fn u64_ref_plus_br_ref() {
        let a = br(1, 3);
        let sum = &7u64 + &a;
        assert_value(&sum, 22, 3);
    }

    // ---- u128 ----
    #[test]
    fn u128_plus_br_val_val() {
        let sum = 5u128 + br(1, 2);
        assert_value(&sum, 11, 2);
    }

    #[test]
    fn u128_max_plus_br() {
        let sum = u128::MAX + br(0, 1);
        assert_eq!(*sum.numerator(), BigInt::from(u128::MAX));
    }

    #[test]
    fn u128_val_plus_br_ref() {
        let a = br(1, 4);
        let sum = 3u128 + &a;
        assert_value(&sum, 13, 4);
        assert_value(&a, 1, 4);
    }

    // ---- i32 ----
    #[test]
    fn i32_plus_br_val_val() {
        let sum = (-3i32) + br(1, 2);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn i32_min_plus_br() {
        let sum = i32::MIN + br(0, 1);
        assert_eq!(*sum.numerator(), BigInt::from(i32::MIN));
    }

    #[test]
    fn i32_ref_plus_br_ref() {
        let a = br(1, 2);
        let sum = &(-3i32) + &a;
        assert_value(&sum, -5, 2);
        assert_value(&a, 1, 2);
    }

    // ---- i64 ----
    #[test]
    fn i64_plus_br_val_val() {
        let sum = (-3i64) + br(1, 2);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn i64_min_plus_br() {
        let sum = i64::MIN + br(0, 1);
        assert_eq!(*sum.numerator(), BigInt::from(i64::MIN));
    }

    #[test]
    fn i64_val_plus_br_ref() {
        let a = br(1, 2);
        let sum = (-3i64) + &a;
        assert_value(&sum, -5, 2);
        assert_value(&a, 1, 2);
    }

    // ---- i128 ----
    #[test]
    fn i128_plus_br_val_val() {
        let sum = (-3i128) + br(1, 2);
        assert_value(&sum, -5, 2);
    }

    #[test]
    fn i128_min_plus_br() {
        let sum = i128::MIN + br(0, 1);
        assert_eq!(*sum.numerator(), BigInt::from(i128::MIN));
    }

    #[test]
    fn i128_ref_plus_br_val() {
        let sum = &(-3i128) + br(1, 2);
        assert_value(&sum, -5, 2);
    }

    // ---- Consistency: T + BR must equal BR + T for all six commutative types ----
    #[test]
    fn scalar_plus_br_matches_br_plus_scalar_all_types() {
        let a = br(3, 7);

        let s1 = (5u32 + a.clone()).reduce().positive_den();
        let s2 = (a.clone() + 5u32).reduce().positive_den();
        assert_eq!(s1.numerator(), s2.numerator());
        assert_eq!(s1.denominator(), s2.denominator());

        let s3 = (5u64 + a.clone()).reduce().positive_den();
        let s4 = (a.clone() + 5u64).reduce().positive_den();
        assert_eq!(s3.numerator(), s4.numerator());

        let s5 = (5u128 + a.clone()).reduce().positive_den();
        let s6 = (a.clone() + 5u128).reduce().positive_den();
        assert_eq!(s5.numerator(), s6.numerator());

        let s7 = ((-5i32) + a.clone()).reduce().positive_den();
        let s8 = (a.clone() + (-5i32)).reduce().positive_den();
        assert_eq!(s7.numerator(), s8.numerator());

        let s9 = ((-5i64) + a.clone()).reduce().positive_den();
        let s10 = (a.clone() + (-5i64)).reduce().positive_den();
        assert_eq!(s9.numerator(), s10.numerator());

        let s11 = ((-5i128) + a.clone()).reduce().positive_den();
        let s12 = (a + (-5i128)).reduce().positive_den();
        assert_eq!(s11.numerator(), s12.numerator());
    }
}
