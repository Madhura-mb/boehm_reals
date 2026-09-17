use super::multiply::boundedrational_mul;
use super::br::BoundedRational;
// use crate::evaluation::constants::{MAX_SIZE, ZERO};
use crate::evaluation::errors::ZeroDivisionError;
use crate::{IsizePromotion, UsizePromotion};
use num_bigint::BigInt;
use std::ops::{Div, DivAssign};

/// Computes the division of two `BoundedRational` values as
/// `a * inverse(b)`.
///
/// # Errors
/// Returns `Err(ZeroDivisionError)` if the divisor is zero.
macro_rules! boundedrational_div {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;

        let inverse_b = BoundedRational::inverse(b)?;
        Ok(boundedrational_mul!(a, inverse_b))
    }};
}

// -----------------------------------------------------------------------------
// BoundedRational Division Implementation
// -----------------------------------------------------------------------------

// &BoundedRational / &BoundedRational
impl Div<&BoundedRational> for &BoundedRational {
    type Output = Result<BoundedRational, ZeroDivisionError>;

    #[inline]
    fn div(self, other: &BoundedRational) -> Self::Output {
        boundedrational_div!(self.clone(), other.clone())
    }
}

// BoundedRational / BoundedRational
// BoundedRational / &BoundedRational
// &BoundedRational / BoundedRational
forward_all_div_to_ref_ref!(impl Div for BoundedRational, div);

// ============================================================================
// BoundedRational Division Assignment Implementation
// ============================================================================

// BoundedRational /= &BoundedRational
impl MulAssign<&BoundedRational> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &BoundedRational) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
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
// u32 / BoundedRational
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
// u64 / BoundedRational
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
// u128 / BoundedRational
// &u128 / BoundedRational
// &BoundedRational / u128
forward_all_scalar_binop_to_val_val!(
    impl Div<u128> for BoundedRational,
    div
);

// BoundedRational / u32
impl Mul<u32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: u32) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / u64
impl Mul<u64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: u64) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / u128
impl Mul<u128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: u128) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// i32 / &BoundedRational
// &i32 / &BoundedRational
// BoundedRational / &i32
// &BoundedRational / &i32
// i32 / BoundedRational
// &i32 / BoundedRational
// &BoundedRational / i32
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<i32> for BoundedRational,
    mul
);

// i64 / &BoundedRational
// &i64 / &BoundedRational
// BoundedRational / &i64
// &BoundedRational / &i64
// i64 / BoundedRational
// &i64 / BoundedRational
// &BoundedRational / i64
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<i64> for BoundedRational,
    mul
);

// i128 / &BoundedRational
// &i128 / &BoundedRational
// BoundedRational / &i128
// &BoundedRational / &i128
// i128 / BoundedRational
// &i128 / BoundedRational
// &BoundedRational / i128
forward_all_scalar_binop_to_val_val_commutative!(
    impl Mul<i128> for BoundedRational,
    mul
);

// BoundedRational / i32
impl Mul<i32> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: i32) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / i64
impl Mul<i64> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: i64) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// BoundedRational / i128
impl Mul<i128> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: i128) -> BoundedRational {
        self * BoundedRational::from_bigint(BigInt::from(other))
    }
}

// ============================================================================
// Scalar Division Assignment Implementation
// ============================================================================

// T = {u8, u16, usize, i8, i16, isize}
// BoundedRational /= T
// BoundedRational /= &T
promote_all_scalars_assign!(impl MulAssign for BoundedRational, mul_assign);

// BoundedRational /= u32
impl MulAssign<u32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational /= u64
impl MulAssign<u64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational /= u128
impl MulAssign<u128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational /= i32
impl MulAssign<i32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational /= i64
impl MulAssign<i64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational /= i128
impl MulAssign<i128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational /= &u32
impl MulAssign<&u32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &u32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational /= &u64
impl MulAssign<&u64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &u64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational /= &u128
impl MulAssign<&u128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &u128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational /= &i32
impl MulAssign<&i32> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &i32) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational /= &i64
impl MulAssign<&i64> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &i64) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// BoundedRational /= &i128
impl MulAssign<&i128> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &i128) {
        let n = mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * *other;
    }
}

// ============================================================================
// BigInt DIvision Implementation
// ============================================================================

// BoundedRational / BigInt
impl Mul<BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BigInt) -> BoundedRational {
        self * BoundedRational::from_bigint(other)
    }
}

// BoundedRational / &BigInt
impl Mul<&BigInt> for BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BigInt) -> BoundedRational {
        self * BoundedRational::from_bigint(other.clone())
    }
}

// &BoundedRational / BigInt
impl Mul<BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BigInt) -> BoundedRational {
        self.clone() * BoundedRational::from_bigint(other)
    }
}

// &BoundedRational / &BigInt
impl Mul<&BigInt> for &BoundedRational {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BigInt) -> BoundedRational {
        self.clone() * BoundedRational::from_bigint(other.clone())
    }
}

// BigInt / BoundedRational
impl Mul<BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BoundedRational) -> BoundedRational {
        other * self
    }
}

// &BigInt / BoundedRational
impl Mul<BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: BoundedRational) -> BoundedRational {
        other * self.clone()
    }
}

// BigInt / &BoundedRational
impl Mul<&BoundedRational> for BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BoundedRational) -> BoundedRational {
        other * self
    }
}

// &BigInt / &BoundedRational
impl Mul<&BoundedRational> for &BigInt {
    type Output = BoundedRational;

    #[inline]
    fn mul(self, other: &BoundedRational) -> BoundedRational {
        other * self.clone()
    }
}

// ============================================================================
// BigInt Division Assignment Implementation
// ============================================================================

// BoundedRational /= BigInt
impl MulAssign<BigInt> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}

// BoundedRational /= &BigInt
impl MulAssign<&BigInt> for BoundedRational {
    #[inline]
    fn mul_assign(&mut self, other: &BigInt) {
        let n = core::mem::replace(self, BoundedRational::from_bigint(ZERO.clone()));
        *self = n * other;
    }
}