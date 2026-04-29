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

/// This module exposes the building blocks for representing and working with
/// constructive reals — numbers that can be computed to arbitrary precision
/// on demand, rather than being stored as fixed-size floating point values.
pub mod creals;
