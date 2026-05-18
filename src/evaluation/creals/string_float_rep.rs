/// Represents a number in scientific notation as an approximation of a constructive real.
///
/// A constructive real is a number that can be computed to arbitrary precision.
/// This struct captures a snapshot of such a number at a given precision, expressed
/// in the form: `sign × 0.mantissa × radix^exponent`
///
/// Both `mantissa` and `exponent` are strings whose characters must be valid digits
/// for the given `radix`. For example, with `radix = 16`, valid digits are `0..9` and `a..f`.
/// The `exponent` may additionally carry a leading `-` for negative values.
///
/// When displayed, the value is normalized to standard scientific notation: the decimal
/// point is shifted one place to the right of the first digit, and the exponent is
/// adjusted accordingly. For example, `0.314159 × 10^1` is displayed as `3.14159e0`.
///
/// # Example
///
/// ```
/// use boehm_reals::evaluation::creals::StringFloatRep;
///
/// // 0.314159 × 10^1 = 3.14159, displayed in normalized form as "3.14159e0"
/// let rep = StringFloatRep::new(1, "314159".to_string(), 10, "1".to_string()).unwrap();
/// assert_eq!(rep.to_string(), "3.14159e0");
/// ```
#[derive(Debug, Clone)]
pub struct StringFloatRep {
    /// Whether the number is positive, negative, or zero.
    ///
    /// Only three values are valid:
    /// - `-1` — the number is negative
    /// -  `0` — the number is zero
    /// -  `1` — the number is positive
    pub sign: i8,

    /// The significant digits of the number, without a decimal point.
    ///
    /// The decimal point is implicitly placed to the **left** of all digits,
    /// so `"314"` means `0.314`, not `314`. The actual value of the number
    /// is recovered by combining this with `exponent`:
    /// `0.mantissa × radix^exponent`.
    ///
    /// All characters must be valid digits for the given `radix`. For example,
    /// with `radix = 16`, valid digits are `0..9` and `a..f`.
    pub mantissa: String,

    /// The base of the numeric system used for both the mantissa and exponent.
    ///
    /// Valid range is `2..=16`. For bases above 10, digits `a..f` (case-insensitive)
    /// represent values 10 through 15, following standard hexadecimal convention.
    /// Bases above 16 are not supported as there is no established symbol convention
    /// beyond `f` for base-16. The radix is appended in the [`Display`] output for
    /// non-decimal bases.
    ///
    /// [`Display`]: std::fmt::Display
    pub radix: u8,

    /// The power to which `radix` is raised to scale the mantissa, expressed as a
    /// string in the same base as `radix`.
    ///
    /// A leading `-` is permitted to represent negative exponents. All remaining
    /// characters must be valid digits for the given `radix`.
    ///
    /// For example, with `radix = 10` and `exponent = "2"`, the mantissa is multiplied
    /// by `10^2 = 100`, so `0.5` becomes `50.0`.
    /// With `exponent = "-2"`, the mantissa is divided by `10^2 = 100`, so `0.5`
    /// becomes `0.005`.
    pub exponent: String,
}

impl StringFloatRep {
    /// Creates a new `StringFloatRep` from its component parts, validating all inputs.
    ///
    /// # Arguments
    ///
    /// * `sign`     — The sign of the number: `-1`, `0`, or `1`
    /// * `mantissa` — The significant digits as a string; all characters must be valid digits for the given `radix`
    /// * `radix`    — The numeric base, must be in the range `2..=16`
    /// * `exponent` — The power of `radix` used to scale the mantissa, expressed as a string in the same base as `radix`; a leading `-` is permitted
    ///
    /// # Errors
    ///
    /// Returns an `Err` if:
    /// - `radix` is outside the range `2..=16`
    /// - `mantissa` contains any character that is not a valid digit for `radix`
    /// - `exponent` contains any character that is not a valid digit for `radix`
    ///   (excluding an optional leading `-`)
    ///
    /// # Examples
    ///
    /// ```
    /// use boehm_reals::evaluation::creals::StringFloatRep;
    ///
    /// // sign=-1, mantissa=0.5, exponent=1, so value = -1 × 0.5 × 10^1 = -5.0
    /// let rep = StringFloatRep::new(-1, "5".to_string(), 10, "1".to_string()).unwrap();
    ///
    /// // Negative exponent: value = 1 × 0.5 × 10^-2 = 0.005
    /// let rep = StringFloatRep::new(1, "5".to_string(), 10, "-2".to_string()).unwrap();
    ///
    /// // Invalid radix: must be between 2 and 16
    /// assert!(StringFloatRep::new(1, "5".to_string(), 17, "1".to_string()).is_err());
    ///
    /// // Invalid mantissa: 'g' is not a valid hex digit
    /// assert!(StringFloatRep::new(1, "g1".to_string(), 16, "1".to_string()).is_err());
    ///
    /// // Invalid exponent: 'z' is not a valid digit for radix 10
    /// assert!(StringFloatRep::new(1, "5".to_string(), 10, "1z".to_string()).is_err());
    ///
    /// // Invalid exponent for radix 2: '8' is not a valid binary digit
    /// assert!(StringFloatRep::new(1, "1011".to_string(), 2, "8".to_string()).is_err());
    /// ```
    pub fn new(sign: i8, mantissa: String, radix: u8, exponent: String) -> Result<Self, String> {
        if !(2..=16).contains(&radix) {
            return Err(format!("radix must be between 2 and 16, got {}", radix));
        }

        let valid_digits: Vec<char> = "0123456789abcdef".chars().take(radix as usize).collect();

        let validate = |s: &str, field: &str| -> Result<(), String> {
            let chars = s.strip_prefix('-').unwrap_or(s);
            for ch in chars.chars() {
                if !valid_digits.contains(&ch.to_ascii_lowercase()) {
                    return Err(format!(
                        "{} contains invalid symbol '{}' for radix {}",
                        field, ch, radix
                    ));
                }
            }
            Ok(())
        };

        validate(&mantissa, "mantissa")?;
        validate(&exponent, "exponent")?;

        Ok(Self {
            sign,
            mantissa,
            radix,
            exponent,
        })
    }
}

impl std::fmt::Display for StringFloatRep {
    /// Formats the value as a normalized scientific notation string.
    ///
    /// The internal representation `0.mantissa × radix^exponent` is normalized
    /// so that exactly one significant digit appears before the decimal point:
    /// `d.dddde<adjusted_exponent>`. This is achieved by shifting the decimal
    /// point one place right (taking the first digit of the mantissa as the
    /// integer part) and decrementing the exponent by 1 to compensate.
    ///
    /// The separator between the mantissa and exponent is the lowercase `e` to
    /// avoid ambiguity with the hexadecimal digit `E`.
    ///
    /// The output format is `[-]d.dddde<exponent>[(radix N)]`, where:
    /// - the leading `-` only appears for negative values
    /// - the `(radix N)` suffix is omitted for base-10
    ///
    /// # Examples
    ///
    /// ```
    /// use boehm_reals::evaluation::creals::StringFloatRep;
    ///
    /// // 0.5 × 10^1 = 5.0, displayed as "5.e0" (no fractional digits after the first)
    /// let rep = StringFloatRep::new(-1, "5".to_string(), 10, "1".to_string()).unwrap();
    /// assert_eq!(rep.to_string(), "-5.e0");
    ///
    /// // 0.1011 × 2^1000 normalized: first digit before point, exponent decremented by 1
    /// // exponent "1000" in base 2 = 8 in decimal, decremented to 7 = "111" in base 2
    /// let rep = StringFloatRep::new(1, "1011".to_string(), 2, "1000".to_string()).unwrap();
    /// assert_eq!(rep.to_string(), "1.011e111(radix 2)");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign_str = if self.sign < 0 { "-" } else { "" };

        // Normalize: shift decimal point one place right by splitting the mantissa
        // into its first digit (integer part) and the remainder (fractional part),
        // then decrement the exponent by 1 to compensate for the shift.
        let first = &self.mantissa[..1];
        let rest = &self.mantissa[1..];
        let adjusted_exponent = self.compute_adjusted_exponent();

        if self.radix == 10 {
            write!(f, "{}{}.{}e{}", sign_str, first, rest, adjusted_exponent)
        } else {
            write!(
                f,
                "{}{}.{}e{}(radix {})",
                sign_str, first, rest, adjusted_exponent, self.radix
            )
        }
    }
}

impl StringFloatRep {
    /// Decrements the exponent by 1 in the given radix to compensate for the
    /// one-place rightward shift of the decimal point during display normalization.
    ///
    /// The exponent is stored as a string in `self.radix`. To adjust it, we parse
    /// it into a `i64` using the radix, subtract 1, then reformat it back into a
    /// string in the same radix.
    ///
    /// # Panics
    ///
    /// Panics if `self.exponent` cannot be parsed as a valid integer in `self.radix`.
    /// This should never occur given that the constructor validates all digit symbols.
    fn compute_adjusted_exponent(&self) -> String {
        let value = i64::from_str_radix(&self.exponent, self.radix as u32)
            .expect("exponent was validated at construction and must be parseable");
        let adjusted = value - 1;
        // Reformat back into the original radix.
        radix_fmt::radix(adjusted, self.radix).to_string()
    }
}
