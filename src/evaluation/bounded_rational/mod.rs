//! Arbitrary-precision exact rational arithmetic used for exact calculation
//! before falling back to constructive-real approximation.

#[macro_use]
mod common_arithmetic_macros;

mod add;
mod br;

pub use br::{BoundedRational, NonFiniteError, ZeroDenominatorError};
