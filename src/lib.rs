//! Boehm-style arbitrary-precision real arithmetic.
//!
//! This crate implements exact real arithmetic using two complementory
//! representations:
//!
//! - Bounded Rational
//! - Constructive reals

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
    )
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
