use num_bigint::BigInt;

#[derive(Clone, Debug)]
pub struct BoundedRational {
    pub numerator: BigInt,
    pub denominator: BigInt,
}
