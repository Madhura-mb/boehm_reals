//! Boehm-style arbitrary-precision real arithmetic.
//!
//! This crate implements exact real arithmetic using two complementory
//! representations:
//!
//! - Bounded Rational
//! - Constructive reals

#![deny(missing_docs)]

/// Provides evaluation utilities for computing with real numbers
/// using the Boehm representation.
pub mod evaluation;

#[cfg(target_pointer_width = "32")]
type UsizePromotion = u32;
#[cfg(target_pointer_width = "64")]
type UsizePromotion = u64;

#[cfg(target_pointer_width = "32")]
type IsizePromotion = i32;
#[cfg(target_pointer_width = "64")]
type IsizePromotion = i64;
