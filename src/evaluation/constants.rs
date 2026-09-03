//! Shared numeric constants used throughout the `evaluation` module.
//!
//! This module stores commonly used `BigInt` values and size limits in one
//! place. This avoids creating the same values again and again or defining
//! them in multiple files.
//!
//! # Usage
//! ```ignore
//! use crate::evaluation::constants::{ZERO, ONE, MAX_SIZE};
//! ```

use num_bigint::BigInt;
use once_cell::sync::Lazy;

/// Maximum combined bit length of numerator and denominator.
/// If `numerator.bits() + denominator.bits()` exceeds this value,
/// the rational is considered too large to be useful and  may be reduced
/// to control the growth of intermediate values.
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
