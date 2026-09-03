/// Arbitrary-precision rational arithmetic with a bounded size budget.
///
/// This module represents exact rational numbers as `numerator/denominator`
/// pairs of [`num_bigint::BigInt`]s. The combined bit length of the numerator
/// and denominator is bounded by the limit defined by [`constants::MAX_SIZE`].
/// When a rational exceeds this limit, it may be reduced to control the growth
/// of intermediate values. If the value remains too large, the caller can fall
/// back to constructive-real (`CR`) approximation.
pub mod bounded_rational;

/// This module defines commonly used `BigInt` values and size limits that
/// are shared across the project. Keeping them here avoids duplicate
/// definitions and repeated object creation.
pub mod constants;

/// This module exposes the building blocks for representing and working with
/// constructive reals — numbers that can be computed to arbitrary precision
/// on demand, rather than being stored as fixed-size floating point values.
pub mod creals;

/// This module includes the errors commonly used accrosed the crate.
pub mod errors;
