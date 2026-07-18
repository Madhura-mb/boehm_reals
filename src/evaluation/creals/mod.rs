/// # Modules
///
/// - [`string_float_rep`] — Provides [`StringFloatRep`], a scientific notation
///   snapshot of a constructive real at a given precision, storing the sign,
///   mantissa, radix, and exponent as discrete fields.
///
/// - [`cr`] — Provides [`CR`], the core constructive real type: a lazily
///   evaluated real number that can be approximated to arbitrary precision
///   on demand.
///
/// [`StringFloatRep`]: string_float_rep::StringFloatRep
/// [`CR`]: cr::CR
mod string_float_rep;
pub use string_float_rep::StringFloatRep;

mod cr;
pub use cr::CR;
