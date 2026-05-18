/// # Modules
///
/// - [`string_float_rep`] — Provides [`StringFloatRep`], a scientific notation
///   snapshot of a constructive real at a given precision, storing the sign,
///   mantissa, radix, and exponent as discrete fields.
///
/// [`StringFloatRep`]: string_float_rep::StringFloatRep
mod string_float_rep;
pub use string_float_rep::StringFloatRep;
