/// Represents a number in scientific notation as an approximation of a constructive real.
///
/// A constructive real is a number that can be computed to arbitrary precision.
/// This struct captures a snapshot of such a number at a given precision, expressed
/// in the form: `sign × 0.mantissa × radix^exponent`
///
/// # Example
///
/// ```
/// use boehm_reals::creals::string_float_rep::StringFloatRep;
///
/// let rep = StringFloatRep::new(1, "314159".to_string(), 10, 1);
/// assert_eq!(rep.to_string(), "314159E1");
/// ```
#[derive(Debug, Clone)]
pub struct StringFloatRep {
    /// Whether the number is positive, negetive, or zero.
    ///
    /// Only three values are valid:
    /// - `-1` — the number is negetive
    /// -  `0` — the number is zero
    /// -  `1` — the number is positive
    pub sign: i8,

    /// The significant digits of the number, without a decimal point.
    ///
    /// The decimal point is implicitly placed to the **left** of all digits,
    /// so `"314"` means `0.314`, not `314`. The actual value of the number
    /// is recovered by combining this with `exponent`:
    /// `0.mantissa × radix^exponent`.
    pub mantissa: String,

    /// The base of the numeric system used for both the mantissa and exponent.
    ///
    /// Typically `10` for decimal. Other bases (e.g. `2` for binary, `16` for hex)
    /// are supported and will be noted in the [`Display`] output.
    ///
    /// [`Display`]: std::fmt::Display
    pub radix: u32,

    /// The power to which `radix` is raised to scale the mantissa.
    ///
    /// For example, with `radix = 10` and `exponent = 2`, the mantissa is multiplied
    /// by `10^2 = 100`, so `0.5` becomes `50.0`.
    /// Negative exponents shift the value toward zero.For example, with
    /// `exponent = -2`, the mantissa is divided by `10^2 = 100`, so `0.5`
    /// becomes `0.005`.
    pub exponent: i32,
}

impl StringFloatRep {
    /// Creates a new `StringFloatRep` from its component parts.
    ///
    /// # Arguments
    ///
    /// * `sign`     — The sign of the number: `-1`, `0`, or `1`
    /// * `mantissa` — The significant digits as a string (decimal point is implicit, placed left of all digits)
    /// * `radix`    — The numeric base (e.g. `10` for decimal)
    /// * `exponent` — The power of `radix` used to scale the mantissa
    ///
    /// # Examples
    ///
    /// ```
    /// use boehm_reals::creals::string_float_rep::StringFloatRep;
    ///
    /// // sign=-1, mantissa=0.5, so value = -1 × 0.5 × 10^1 = -5.0
    /// let rep = StringFloatRep::new(-1, "5".to_string(), 10, 1);
    /// ```
    pub fn new(sign: i8, mantissa: String, radix: u32, exponent: i32) -> Self {
        Self {
            sign,
            mantissa,
            radix,
            exponent,
        }
    }
}

impl std::fmt::Display for StringFloatRep {
    /// Formats the value as a human-readable scientific notation string.
    ///
    /// The output format is `[-]mantissaE<exponent>`, where the leading `-`
    /// only appears for negative values. For non-decimal bases, the radix is
    /// appended in parentheses.
    ///
    /// # Examples
    ///
    /// ```
    /// use boehm_reals::creals::string_float_rep::StringFloatRep;
    ///
    /// // Base 10: sign=-1, mantissa=0.5, exponent=1 → prints as "-5E1"
    /// let rep = StringFloatRep::new(-1, "5".to_string(), 10, 1);
    /// assert_eq!(rep.to_string(), "-5E1");
    ///
    /// // Non-decimal: radix is appended explicitly
    /// let rep = StringFloatRep::new(1, "1011".to_string(), 2, 8);
    /// assert_eq!(rep.to_string(), "1011E8(radix 2)");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign_str = if self.sign < 0 { "-" } else { "" };

        if self.radix == 10 {
            write!(f, "{}{}E{}", sign_str, self.mantissa, self.exponent)
        } else {
            write!(
                f,
                "{}{}E{}(radix {})",
                sign_str, self.mantissa, self.exponent, self.radix
            )
        }
    }
}
