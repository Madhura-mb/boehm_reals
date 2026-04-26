/// Arbitrary-precision rational arithmetic with a bounded size budget.
///
/// [`bounded_rational::BoundedRational`] represents exact rational numbers
/// as `numerator/denominator` pairs of [`num_bigint::BigInt`]s. Operations
/// return `None` once the combined bit length exceeds
/// [`bounded_rational::MAX_SIZE`], at which point the caller is expected
/// to fall back to constructive-real (`CR`) approximation.
pub mod bounded_rational;
