use num_bigint::BigInt;
use lazy_static::lazy_static;

/*------------- Primitive constants -------------- */
pub const MAX_SIZE: u32 = 10_000;

/*---------- BigInt constants ----------------- */
lazy_static! {
    // Core numeric sentinels used as fast-return shortcuts throughout the crate
    pub static ref ZERO:      BigInt = BigInt::from(0i32);
    pub static ref ONE:       BigInt = BigInt::from(1i32);
    pub static ref MINUS_ONE: BigInt = BigInt::from(-1i32);
    pub static ref TWO:       BigInt = BigInt::from(2i32);
    pub static ref MINUS_TWO: BigInt = BigInt::from(-2i32);
    pub static ref TEN:       BigInt = BigInt::from(10i32);

    // Used in reduction / GCD short-circuit paths
    pub static ref BIG_FIVE:  BigInt = BigInt::from(5i32);
}

/*----------- Struct ---------------- */
#[derive(Clone, Debug)]
pub struct BoundedRational {
    pub numerator: BigInt,
    pub denominator: BigInt,
}
