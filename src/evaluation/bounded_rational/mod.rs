//! Arbitrary-precision exact rational arithmetic used for exact calculation
//! before falling back to constructive-real approximation.

#[macro_use]
mod macros;

mod add;
mod b_rational;

pub use b_rational::{BoundedRational, NonFiniteError, ZeroDenominatorError};
