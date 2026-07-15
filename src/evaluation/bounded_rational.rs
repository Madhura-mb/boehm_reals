use crate::evaluation::constants::{MAX_SIZE, MINUS_ONE, MINUS_TWO, ONE, TEN, TWO, ZERO};
use num_bigint::BigInt;
use num_integer::Integer;
use rand::Rng;

/// Error returned when a `BoundedRational` is constructed with a zero denominator.
#[derive(Clone, Debug)]
pub struct ZeroDenominatorError;

impl std::fmt::Display for ZeroDenominatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "denominator must not be zero")
    }
}

impl std::error::Error for ZeroDenominatorError {}
/// Error returned when constructing a `BoundedRational` from a non-finite
/// `f64` (`NaN` or infinite), neither of which has a finite rational value.
#[derive(Clone, Debug, PartialEq)]
pub struct NonFiniteError;

impl std::fmt::Display for NonFiniteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "value is NaN or infinite; has no rational representation"
        )
    }
}

impl std::error::Error for NonFiniteError {}

/// A ratio of two arbitrary-precision integers, `numerator/denominator`
///
/// Arithmetic operations return `None` when the result would exceed
/// [`MAX_SIZE`] combined bits, signalling the caller to fall back to
/// a constructive-real approximation. All values are treated as exact
/// until that point.
///
/// # Invariants
/// - The denominator is never zero.
/// - Fractions are not always fully reduced; simplification happens
///   occasionally at random to avoid paying the cost of GCD on every
///   operation.

#[derive(Clone, Debug, PartialEq)]
pub struct BoundedRational {
    /// The top half of the fraction.
    numerator: BigInt,
    /// The bottom half of the fraction. Must never be zero.
    denominator: BigInt,
}

impl BoundedRational {
    /// Read-only access to the numerator.
    pub fn numerator(&self) -> &BigInt {
        &self.numerator
    }

    /// Read-only access to the denominator.
    pub fn denominator(&self) -> &BigInt {
        &self.denominator
    }

    /// Creates a new `BoundedRational` with the given numerator and denominator.
    ///
    /// Error: Returns `Err(ZeroDenominatorError)` if `d` is zero.
    pub fn new(n: BigInt, d: BigInt) -> Result<Self, ZeroDenominatorError> {
        if d == *ZERO {
            return Err(ZeroDenominatorError);
        }
        if n == *ZERO {
            return Ok(BoundedRational {
                numerator: ZERO.clone(),
                denominator: ONE.clone(),
            });
        }
        Ok(BoundedRational {
            numerator: n,
            denominator: d,
        })
    }

    /// Creates a `BoundedRational` equal to the integer `n` (denominator = 1).
    pub fn from_bigint(n: BigInt) -> Self {
        BoundedRational {
            numerator: n,
            denominator: ONE.clone(),
        }
    }

    /// Creates a `BoundedRational` from two `i64` values.
    ///
    /// Error: Returns `Err(ZeroDenominatorError)` if `d` is zero.
    pub fn from_longs(n: i64, d: i64) -> Result<Self, ZeroDenominatorError> {
        Self::new(BigInt::from(n), BigInt::from(d))
    }

    /// Creates a `BoundedRational` equal to the integer `n` (denominator = 1).
    pub fn from_long(n: i64) -> Self {
        Self::from_bigint(BigInt::from(n))
    }

    /// Returns `true` if rational is too large to be useful.
    ///
    /// Specifically, returns `true` when `numerator.bits() + denominator.bits() > MAX_SIZE`.
    ///
    /// # Preferential treatment for integers (denominator == 1)
    /// Pure integers skip the bit-count check entirely and always return `false`,
    /// even if the numerator alone exceeds MAX_SIZE bits. This is intentional:
    /// integers do not exhibit the runaway numerator+denominator co-growth that
    /// MAX_SIZE is designed to catch, and returning `None` for an exact integer
    /// would be strictly worse than keeping it.
    ///
    /// # Sign bit
    /// `BigInt::bits()` counts bits in the absolute value only — the sign bit is
    /// not included. `-n` and `n` return the same bit count. This means the check
    /// is magnitude-only, which is fine for our purposes but worth knowing when
    /// debugging: a large negative numerator counts the same as its positive counterpart.
    ///
    /// # Note on i64::MAX
    /// `BigInt::from(i64::MAX).bits()` returns 63, not 64, because
    /// i64::MAX = 2^63 - 1 fits in 63 bits. `i64::MIN` (-2^63) returns 64
    pub fn too_big(&self) -> bool {
        if self.denominator == *ONE {
            return false;
        }
        self.numerator.bits() + self.denominator.bits() > MAX_SIZE as u64
    }

    /// Returns a clone of this rational with a positive denominator.
    ///
    /// If the denominator is negative, both numerator and denominator are negated,
    /// preserving the value while ensuring `denominator > 0`. If the denominator is
    /// already positive. the value is returned unchanged.
    ///
    /// Note: a zero denominator cannot arise here because `new` rejects it at
    /// construction time.
    pub fn positive_den(&self) -> BoundedRational {
        if self.denominator < *ZERO {
            BoundedRational {
                numerator: -&self.numerator,
                denominator: -&self.denominator,
            }
        } else {
            self.clone()
        }
    }

    /// Return an equivalent fractions in lowest terms.
    ///
    /// Divides both numerator and denominator by their GCD.
    /// Denominator sign is **not** normalized here - call [`positive_den`]
    /// afterwards is a canonical positive denominator is required.
    ///
    /// An early return fires when the denominator is already `1`, because an
    /// integer needs no reduction.
    ///
    /// [`positive_den`]: BoundedRational::positive_den
    pub fn reduce(&self) -> BoundedRational {
        // already an integer - nothing to cancel.
        if self.denominator == *ONE {
            return self.clone();
        }

        let divisor = self.numerator.gcd(&self.denominator);

        BoundedRational {
            numerator: &self.numerator / &divisor,
            denominator: &self.denominator / &divisor,
        }
    }

    /// Return a possibly-reduced version of `r`, or `None` if `r` is `None`.
    ///
    /// # Reduction policy
    /// Reduction (via [`reduce`] + [`positive_den`]) is performed when either:
    /// - the value is already [`too_big`], **or**
    /// - a 1-in-16 random chance fires (to reduce GCD cost across many ops).
    ///
    /// If neither condition applies, `r` is returned unchanged.
    ///
    /// The caller is responsible for checking whether the returned value is
    /// still [`too_big`] and acting accordingly (e.g falling back to
    /// constructive-real arithmetic).
    ///
    /// # None propagation
    /// `None` input -> `None` output immediately, with no reduction attempted.
    ///
    /// [`reduce`]: BoundedRational::reduce
    /// [`positive_den`]: BoundedRational::positive_den
    /// [`too_big`]: BoundedRational::too_big
    pub fn maybe_reduce(r: Option<BoundedRational>) -> Option<BoundedRational> {
        let r = r?; // propagate None immediately

        let should_reduce = r.too_big() || (rand::rng().next_u32() & 0xf) == 0;

        if !should_reduce {
            return Some(r);
        }

        Some(r.positive_den().reduce())
    }

    ///Converts an `i64` into a `BoundedRational`.
    ///
    /// For the six most commonly encountered small integers (`-2`, `-1`, `0`, `1`, `2`, `10`),
    /// this function constructs a fresh `BoundedRational` backed by the corresponding
    /// pre-allocated [`BigInt`] constants (`MINUS_TWO`, `MINUS_ONE`, `ZERO`, `ONE`, `TWO`, `TEN`).
    /// This avoids an extra [`BigInt::from`] allocation for those hot values.
    ///
    /// All other values are forwarded to [`BoundedRational::from_long`], which heap-allocates
    /// a new [`BigInt`] for the numerator and uses `ONE` as the denominator.
    ///
    /// # Return value
    /// Always returns a valid `BoundedRational` with denominator `1`. The result is
    /// never reduced (no GCD is computed), because there is nothing to cancel against
    /// a denominator of `1`.
    ///
    /// # Note on cloning
    /// The cached [`BigInt`] statics are wrapped in [`once_cell::sync::Lazy`], so each
    /// returned `BoundedRational` clones the inner [`BigInt`] value out of the static.
    /// There is no way to return a reference to a static `BoundedRational` from a
    /// function that returns `BoundedRational` by value without lifetime complications,
    /// so cloning is the right trade-off here.
    pub fn value_of_long(x: i64) -> BoundedRational {
        match x {
            -2 => BoundedRational::from_bigint(MINUS_TWO.clone()),
            -1 => BoundedRational::from_bigint(MINUS_ONE.clone()),
            0 => BoundedRational::from_bigint(ZERO.clone()),
            1 => BoundedRational::from_bigint(ONE.clone()),
            2 => BoundedRational::from_bigint(TWO.clone()),
            10 => BoundedRational::from_bigint(TEN.clone()),
            _ => BoundedRational::from_long(x),
        }
    }

    /// Converts a given `f64` into `BoundedRational`.
    ///
    /// # Fast path
    /// If `x` is a finite whole number that fits within the `i64` range,
    /// it is converted using `value_of_long()`. This avoids the overhead
    /// of decomposing the IEEE 754 representation.
    ///
    /// # Slow path
    /// All other finite `f64` values, including non-integer values and
    /// whole numbers outside the `i64` range, are converted by extracting
    /// the IEEE 754 binary representation. The sign, exponent, and
    /// mantissa are used to construct the exact numerator and denominator
    /// of the fraction.
    ///
    /// The returned fraction exactly represents the binary value stored
    /// in the `f64`. It may not exactly match the decimal number that was
    /// originally written (because many decimal numbers cannot be
    /// represented exactly in binary). The function does not reduce the
    /// fraction to its simplest form.
    ///
    /// # Errors
    /// Returns `Err(NonFiniteError)` if `x` is `NaN` or infinite.
    pub fn value_of_double(x: f64) -> Result<BoundedRational, NonFiniteError> {
        // Reject NaN and infinity.
        if !x.is_finite() {
            return Err(NonFiniteError);
        }

        // --- Fast path: whole numbers reuse the integer constructor. ---
        let rounded = x.round();
        if rounded == x && rounded >= i64::MIN as f64 && rounded <= i64::MAX as f64 {
            return Ok(BoundedRational::value_of_long(rounded as i64));
        }

        // --- Slow path: exact IEEE 754 decomposition. ---
        let bits = x.abs().to_bits();

        let mantissa_mask: u64 = (1u64 << 52) - 1;
        let mut mantissa = bits & mantissa_mask;
        let biased_exp = (bits >> 52) & 0x7ff;

        let sign: i64 = if x < 0.0 { -1 } else { 1 };

        // 1075 = 1023 (exponent bias) + 52 (mantissa fraction bits).
        let mut exp = biased_exp as i64 - 1075;

        if biased_exp == 0 {
            // Subnormal: no hidden leading bit; exponent shifted by one.
            exp += 1;
        } else {
            // Normalized: restore the hidden leading 1 bit.
            mantissa |= 1u64 << 52;
        }

        // Mantissa is at most ~2^53, so sign * mantissa fits safely in i64.
        let signed_mantissa = sign * mantissa as i64;

        let (numerator, denominator) = if exp >= 0 {
            (BigInt::from(signed_mantissa) << exp as usize, ONE.clone())
        } else {
            (
                BigInt::from(signed_mantissa),
                ONE.clone() << (-exp) as usize,
            )
        };

        // denominator is always a positive power of two (non-zero) here, so this
        // can never trigger the zero-denominator error from `new`.
        Ok(BoundedRational::new(numerator, denominator)
            .expect("denominator is a nonzero power of two by construction"))
    }

    /// Returns the argument, but with the opposite sign.
    /// Returns `None` only for a `None` argument.
    pub fn negate(r: Option<BoundedRational>) -> Option<BoundedRational> {
        let r = r?; // propagate None immediately
        Some(BoundedRational {
            numerator: -r.numerator,
            denominator: r.denominator,
        })
    }

    /// Returns the sum of `r1` and `r2`, possibly reduces.
    /// Returns `None` if either argument is `None`.
    ///
    /// If either operand is exactly zero, the other operand is returned
    /// unchanged (aside from being passed through [`maybe_reduce`]), avoiding
    /// unnecessary cross multiplication work entirely.
    ///
    /// Before performing the addition, this function may reduce both operands
    /// when their combined bit size is large. This heuristic helps avoid
    /// creating unnecessarily large intermediate numerators and denominators
    /// during cross multiplication while preserving the final value.
    ///
    /// The resulting fraction is passed through [`maybe_reduce`] to keep its
    /// size within the configured bounds when possible.
    pub fn add(
        r1: Option<BoundedRational>,
        r2: Option<BoundedRational>,
    ) -> Option<BoundedRational> {
        let r1 = r1?; // propagate None immediately
        let r2 = r2?; // propagate None immediately

        // Zero check: adding zero is a no-op, so just return the other operand.
        if r1.numerator == *ZERO {
            return BoundedRational::maybe_reduce(Some(r2));
        }
        if r2.numerator == *ZERO {
            return BoundedRational::maybe_reduce(Some(r1));
        }

        // Heuristic: if sum of input bit sizes is already close to MAX_SIZE,
        // reduce inputs first to avoid huge intermediates
        let input_bits = r1.numerator.bits()
            + r1.denominator.bits()
            + r2.numerator.bits()
            + r2.denominator.bits();

        let (r1, r2) = if input_bits > (MAX_SIZE as u64 * 3 / 4) {
            // Reduce both inputs before multiplying
            (r1.reduce().positive_den(), r2.reduce().positive_den())
        } else {
            (r1, r2)
        };

        let den = &r1.denominator * &r2.denominator;
        let num = &r1.numerator * &r2.denominator + &r1.denominator * &r2.numerator;

        BoundedRational::maybe_reduce(Some(BoundedRational {
            numerator: num,
            denominator: den,
        }))
    }

    /// Returns `r1 - r2`. Returns `None` if either argument is `None`.
    pub fn subtract(
        r1: Option<BoundedRational>,
        r2: Option<BoundedRational>,
    ) -> Option<BoundedRational> {
        BoundedRational::add(r1, BoundedRational::negate(r2))
    }

    /// Returns `true` if this rational is equal to the integer `n`.
    ///
    /// Checks whether `numerator == n * denominator`, which handles all sign
    /// representations without needing to reduce first. For example, both
    /// `1/1` and `-1/-1` satisfy `numerator == 1 * denominator`, and both
    /// `-1/1` and `1/-1` satisfy `numerator == -1 * denominator`.
    pub fn equals(&self, n: i64) -> bool {
        self.numerator == BigInt::from(n) * &self.denominator
    }

    /// Returns the product of `r1` and `r2` , possibly reduced.
    ///
    /// # Shortcuts
    /// - If either argument equals `1` (checked via [`equals`]), the other
    ///   argument is returned immediately, skipping multiplication entirely.
    /// - If either argument equals `-1` (checked via [`equals`]), the other
    ///   argument is returned with its numerator negated, skipping
    ///   multiplication entirely.
    ///
    /// [`equals`]: BoundedRational::equals
    ///
    /// # Reduction heuristic
    /// Before multiplying, the combined bit sizes of all four components are
    /// checked against a threshold of `MAX_SIZE * 3/4`. The result numerator
    /// and denominator bit sizes are also checked independently, since either
    /// can overflow even when the total input size looks acceptable:
    /// - `input_bits  = r1.num.bits + r1.den.bits + r2.num.bits + r2.den.bits`
    /// - `result_num_bits = r1.num.bits + r2.num.bits`
    /// - `result_den_bits = r1.den.bits + r2.den.bits`
    ///
    /// If any of these exceed the threshold, both `r1` and `r2` are reduced and
    /// sign-normalised before the multiplication, keeping intermediate values
    /// small. When this pre-reduction fires, the `maybe_reduce` step afterwards
    /// is skipped, since a second reduction pass would be redundant. When the
    /// threshold is not exceeded, `maybe_reduce` is called on the raw product
    /// as usual.
    ///
    /// # None propagation
    /// `None` input on either side -> `None` output immediately. A `None`
    /// here may represent a value that was too large to keep as an exact
    /// rational, so it is never silently treated as zero or one.
    pub fn multiply(
        r1: Option<BoundedRational>,
        r2: Option<BoundedRational>,
    ) -> Option<BoundedRational> {
        let r1 = r1?; // propagate None immediately
        let r2 = r2?; // propagate None immediately

        if r1.equals(1) {
            return Some(r2);
        }
        if r2.equals(1) {
            return Some(r1);
        }
        if r1.equals(-1) {
            return Some(BoundedRational {
                numerator: -r2.numerator,
                denominator: r2.denominator,
            });
        }
        if r2.equals(-1) {
            return Some(BoundedRational {
                numerator: -r1.numerator,
                denominator: r1.denominator,
            });
        }

        let threshold = MAX_SIZE as u64 * 3 / 4;

        let input_bits = r1.numerator.bits()
            + r1.denominator.bits()
            + r2.numerator.bits()
            + r2.denominator.bits();

        let result_num_bits = r1.numerator.bits() + r2.numerator.bits();
        let result_den_bits = r1.denominator.bits() + r2.denominator.bits();

        let (r1, r2, already_reduced) =
            if input_bits > threshold || result_num_bits > threshold || result_den_bits > threshold
            {
                (r1.reduce().positive_den(), r2.reduce().positive_den(), true)
            } else {
                (r1, r2, false)
            };

        let result = Some(BoundedRational {
            numerator: &r1.numerator * &r2.numerator,
            denominator: &r1.denominator * &r2.denominator,
        });

        if already_reduced {
            result
        } else {
            BoundedRational::maybe_reduce(result)
        }
    }

    /// Returns the reciprocal of `r`, formed by swapping numerator and
    /// denominator.
    ///
    /// # None propagation
    /// `None` input -> `Ok(None)` output immediately; no zero check is
    /// performed in that case since there is no value to inspect.
    ///
    /// # Errors
    /// Returns `Err(ZeroDenominatorError)` if `r`'s numerator is zero, since
    /// the resulting denominator would be zero.
    pub fn inverse(
        r: Option<BoundedRational>,
    ) -> Result<Option<BoundedRational>, ZeroDenominatorError> {
        let r = match r {
            Some(r) => r,
            None => return Ok(None),
        };
        if r.numerator == *ZERO {
            return Err(ZeroDenominatorError);
        }
        Ok(Some(BoundedRational {
            numerator: r.denominator,
            denominator: r.numerator,
        }))
    }

    /// Returns `r1 / r2`, computed as `r1 * inverse(r2)`.
    ///
    /// # Errors
    /// Returns `Err(ZeroDenominatorError)` if `r2` is zero.
    pub fn divide(
        r1: Option<BoundedRational>,
        r2: Option<BoundedRational>,
    ) -> Result<Option<BoundedRational>, ZeroDenominatorError> {
        Ok(BoundedRational::multiply(r1, BoundedRational::inverse(r2)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use std::i64;

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Constructs a BigInt with exactly `n` bits set (value = 2^n - 1).
    fn big_with_bits(n: u64) -> BigInt {
        (BigInt::from(1) << n as usize) - BigInt::from(1)
    }

    // ── NonFiniteError display ─────────────────────────────────────────

    #[test]
    fn non_finite_error_display() {
        assert_eq!(
            NonFiniteError.to_string(),
            "value is NaN or infinite; has no rational representation"
        );
    }

    #[test]
    fn non_finite_error_is_clone_and_eq() {
        let e1 = NonFiniteError;
        let e2 = e1.clone();
        assert_eq!(e1, e2);
    }

    // ── ZeroDenominatorError display ─────────────────────────────────────────

    #[test]
    fn zero_denominator_error_display() {
        assert_eq!(
            ZeroDenominatorError.to_string(),
            "denominator must not be zero"
        );
    }

    #[test]
    fn zero_denominator_error_is_clone_and_eq() {
        let e1 = ZeroDenominatorError;
        let e2 = e1.clone();
        // Verify clone succeeds by checking both are the same unit struct
        let _ = e2;
    }

    // ── new ──────────────────────────────────────────────────────────────────

    #[test]
    fn new_valid_positive_fraction() {
        let r = BoundedRational::new(BigInt::from(3), BigInt::from(4)).unwrap();
        assert_eq!(*r.numerator(), BigInt::from(3));
        assert_eq!(*r.denominator(), BigInt::from(4));
    }

    #[test]
    fn new_valid_negative_numerator() {
        let r = BoundedRational::new(BigInt::from(-3), BigInt::from(4)).unwrap();
        assert_eq!(*r.numerator(), BigInt::from(-3));
        assert_eq!(*r.denominator(), BigInt::from(4));
    }

    #[test]
    fn new_valid_negative_denominator() {
        let r = BoundedRational::new(BigInt::from(3), BigInt::from(-4)).unwrap();
        assert_eq!(*r.numerator(), BigInt::from(3));
        assert_eq!(*r.denominator(), BigInt::from(-4));
    }

    #[test]
    fn new_valid_both_negative() {
        let r = BoundedRational::new(BigInt::from(-3), BigInt::from(-4)).unwrap();
        assert_eq!(*r.numerator(), BigInt::from(-3));
        assert_eq!(*r.denominator(), BigInt::from(-4));
    }

    #[test]
    fn new_zero_numerator_normalizes_denominator_to_one() {
        let r = BoundedRational::new(BigInt::from(0), BigInt::from(5)).unwrap();
        assert_eq!(*r.numerator(), *ZERO);
        assert_eq!(*r.denominator(), *ONE);
    }

    #[test]
    fn new_rejects_zero_denominator() {
        assert!(BoundedRational::new(BigInt::from(1), BigInt::from(0)).is_err());
    }

    #[test]
    fn new_rejects_zero_denominator_with_zero_numerator() {
        // 0/0 is indeterminate — must be rejected
        assert!(BoundedRational::new(BigInt::from(0), BigInt::from(0)).is_err());
    }

    #[test]
    fn new_denominator_of_one_is_valid() {
        let r = BoundedRational::new(BigInt::from(7), BigInt::from(1)).unwrap();
        assert_eq!(*r.denominator(), *ONE);
    }

    #[test]
    fn new_denominator_of_minus_one_is_valid() {
        let r = BoundedRational::new(BigInt::from(7), BigInt::from(-1)).unwrap();
        assert_eq!(*r.denominator(), *MINUS_ONE);
    }

    // ── from_bigint ──────────────────────────────────────────────────────────

    #[test]
    fn from_bigint_sets_denominator_to_one() {
        let r = BoundedRational::from_bigint(BigInt::from(42));
        assert_eq!(*r.denominator(), *ONE);
    }

    #[test]
    fn from_bigint_zero_numerator() {
        let r = BoundedRational::from_bigint(BigInt::from(0));
        assert_eq!(*r.numerator(), *ZERO);
        assert_eq!(*r.denominator(), *ONE);
    }

    #[test]
    fn from_bigint_negative_value() {
        let r = BoundedRational::from_bigint(BigInt::from(-99));
        assert_eq!(*r.numerator(), BigInt::from(-99));
        assert_eq!(*r.denominator(), *ONE);
    }

    // ── from_long ────────────────────────────────────────────────────────────

    #[test]
    fn from_long_zero() {
        let r = BoundedRational::from_long(0);
        assert_eq!(*r.numerator(), *ZERO);
        assert_eq!(*r.denominator(), *ONE);
    }

    #[test]
    fn from_long_i64_max() {
        let r = BoundedRational::from_long(i64::MAX);
        assert_eq!(*r.numerator(), BigInt::from(i64::MAX));
        // i64::MAX = 2^63 - 1 fits in 63 bits, not 64
        assert_eq!(r.numerator().bits(), 63);
    }

    #[test]
    fn from_long_i64_min() {
        let r = BoundedRational::from_long(i64::MIN);
        assert_eq!(*r.numerator(), BigInt::from(i64::MIN));
        // i64::MIN = -2^63, absolute value needs 64 bits
        assert_eq!(r.numerator().bits(), 64);
    }

    #[test]
    fn from_long_negative_one() {
        let r = BoundedRational::from_long(-1);
        assert_eq!(*r.numerator(), *MINUS_ONE);
        assert_eq!(*r.denominator(), *ONE);
    }

    // ── from_longs ───────────────────────────────────────────────────────────

    #[test]
    fn from_longs_stores_numerator_and_denominator() {
        let r = BoundedRational::from_longs(3, 4).unwrap();
        assert_eq!(*r.numerator(), BigInt::from(3i64));
        assert_eq!(*r.denominator(), BigInt::from(4i64));
    }

    #[test]
    fn from_longs_rejects_zero_denominator() {
        assert!(BoundedRational::from_longs(3, 0).is_err());
    }

    #[test]
    fn from_longs_zero_numerator_valid() {
        let r = BoundedRational::from_longs(0, 5).unwrap();
        assert_eq!(*r.numerator(), *ZERO);
        assert_eq!(*r.denominator(), *ONE);
    }

    #[test]
    fn from_longs_both_negative() {
        let r = BoundedRational::from_longs(-3, -4).unwrap();
        assert_eq!(*r.numerator(), BigInt::from(-3));
        assert_eq!(*r.denominator(), BigInt::from(-4));
    }

    #[test]
    fn from_longs_denominator_one() {
        let r = BoundedRational::from_longs(5, 1).unwrap();
        assert_eq!(*r.denominator(), *ONE);
    }

    // ── too_big ──────────────────────────────────────────────────────────────

    #[test]
    fn too_big_small_fraction_is_false() {
        let r = BoundedRational::from_longs(3, 4).unwrap();
        assert!(!r.too_big());
    }

    #[test]
    fn too_big_pure_integer_always_false_even_if_huge() {
        // Numerator alone far exceeds MAX_SIZE bits — but denominator is 1
        // so too_big must still return false (integers are never too big)
        let huge = BigInt::from(1) << (MAX_SIZE * 2);
        let r = BoundedRational::from_bigint(huge);
        assert!(!r.too_big());
    }

    #[test]
    fn too_big_exactly_at_limit_is_false() {
        // bits(num) + bits(den) == MAX_SIZE  →  not strictly greater, so false
        let half = MAX_SIZE as u64 / 2;
        let r = BoundedRational::new(big_with_bits(half), big_with_bits(half)).unwrap();
        assert!(!r.too_big());
    }

    #[test]
    fn too_big_one_over_limit_is_true() {
        // bits(num) + bits(den) == MAX_SIZE + 1  →  strictly greater, so true
        let half = MAX_SIZE as u64 / 2;
        let r = BoundedRational::new(big_with_bits(half + 1), big_with_bits(half)).unwrap();
        assert!(r.too_big());
    }

    #[test]
    fn too_big_sign_does_not_affect_result() {
        // bits() ignores sign — negative values with same magnitude behave identically
        let half = MAX_SIZE as u64 / 2;
        let pos = BoundedRational::new(big_with_bits(half + 1), big_with_bits(half)).unwrap();
        let neg = BoundedRational::new(-big_with_bits(half + 1), big_with_bits(half)).unwrap();
        assert_eq!(pos.too_big(), neg.too_big());
    }

    // ── positive_den ─────────────────────────────────────────────────────────

    #[test]
    fn positive_den_already_positive_unchanged() {
        let r = BoundedRational::from_longs(3, 4).unwrap();
        let p = r.positive_den();
        assert_eq!(*p.numerator(), BigInt::from(3));
        assert_eq!(*p.denominator(), BigInt::from(4));
    }

    #[test]
    fn positive_den_negative_denominator_flips_both() {
        let r = BoundedRational::from_longs(3, -4).unwrap();
        let p = r.positive_den();
        assert_eq!(*p.numerator(), BigInt::from(-3));
        assert_eq!(*p.denominator(), BigInt::from(4));
    }

    #[test]
    fn positive_den_both_negative_flips_both() {
        let r = BoundedRational::from_longs(-3, -4).unwrap();
        let p = r.positive_den();
        assert_eq!(*p.numerator(), BigInt::from(3));
        assert_eq!(*p.denominator(), BigInt::from(4));
    }

    #[test]
    fn positive_den_negative_num_positive_den_unchanged() {
        let r = BoundedRational::from_longs(-3, 4).unwrap();
        let p = r.positive_den();
        assert_eq!(*p.numerator(), BigInt::from(-3));
        assert_eq!(*p.denominator(), BigInt::from(4));
    }

    #[test]
    fn positive_den_denominator_minus_one() {
        let r = BoundedRational::from_longs(5, -1).unwrap();
        let p = r.positive_den();
        assert_eq!(*p.numerator(), BigInt::from(-5));
        assert_eq!(*p.denominator(), *ONE);
    }

    #[test]
    fn positive_den_zero_numerator_negative_den() {
        let r = BoundedRational::from_longs(0, -7).unwrap();
        let p = r.positive_den();
        assert_eq!(*p.numerator(), *ZERO);
        assert_eq!(*p.denominator(), *ONE);
    }

    #[test]
    fn positive_den_is_idempotent() {
        // Calling positive_den twice should give the same result as calling it once
        let r = BoundedRational::from_longs(3, -4).unwrap();
        let once = r.positive_den();
        let twice = once.positive_den();
        assert_eq!(twice.numerator(), once.numerator());
        assert_eq!(twice.denominator(), once.denominator());
    }

    #[test]
    fn positive_den_preserves_value() {
        // 3/-4 and -3/4 represent the same rational — cross-multiply to verify
        let r = BoundedRational::from_longs(3, -4).unwrap();
        let p = r.positive_den();
        // r.num * p.den == p.num * r.den  →  3 * 4 == -3 * -4  →  12 == 12
        assert_eq!(
            r.numerator() * p.denominator(),
            p.numerator() * r.denominator()
        );
    }

    // ── reduce ───────────────────────────────────────────────────────────────

    /// Acceptance criterion: reduce() on 6/4 returns 3/2
    #[test]
    fn reduce_six_fourths_gives_three_halves() {
        let r = BoundedRational::from_longs(6, 4).unwrap();
        let reduced = r.reduce();
        assert_eq!(*reduced.numerator(), BigInt::from(3));
        assert_eq!(*reduced.denominator(), BigInt::from(2));
    }

    #[test]
    fn reduce_integer_early_return() {
        // denominator == 1 triggers the early-return path; value unchanged
        let r = BoundedRational::from_long(7);
        let reduced = r.reduce();
        assert_eq!(*reduced.numerator(), BigInt::from(7));
        assert_eq!(*reduced.denominator(), *ONE);
    }

    #[test]
    fn reduce_already_lowest_terms_unchanged() {
        let r = BoundedRational::from_longs(3, 7).unwrap();
        let reduced = r.reduce();
        assert_eq!(*reduced.numerator(), BigInt::from(3));
        assert_eq!(*reduced.denominator(), BigInt::from(7));
    }

    #[test]
    fn reduce_negative_numerator() {
        let r = BoundedRational::from_longs(-6, 4).unwrap();
        let reduced = r.reduce();
        assert_eq!(*reduced.numerator(), BigInt::from(-3));
        assert_eq!(*reduced.denominator(), BigInt::from(2));
    }

    #[test]
    fn reduce_large_common_factor() {
        // 100/200 => 1/2
        let r = BoundedRational::from_longs(100, 200).unwrap();
        let reduced = r.reduce();
        assert_eq!(*reduced.numerator(), BigInt::from(1));
        assert_eq!(*reduced.denominator(), BigInt::from(2));
    }

    #[test]
    fn reduce_is_idempotent() {
        let r = BoundedRational::from_longs(6, 4).unwrap();
        let once = r.reduce();
        let twice = once.reduce();
        assert_eq!(once.numerator(), twice.numerator());
        assert_eq!(once.denominator(), twice.denominator());
    }

    #[test]
    fn reduce_preserves_value() {
        // cross-multiply: (6/4).num * reduced.den == (6/4).den * reduced.num
        let r = BoundedRational::from_longs(6, 4).unwrap();
        let reduced = r.reduce();
        assert_eq!(
            r.numerator() * reduced.denominator(),
            r.denominator() * reduced.numerator()
        );
    }

    // ── maybe_reduce ─────────────────────────────────────────────────────────

    /// Acceptance criterion: maybe_reduce(None) returns None
    #[test]
    fn maybe_reduce_none_returns_none() {
        assert!(BoundedRational::maybe_reduce(None).is_none());
    }

    #[test]
    fn maybe_reduce_small_fraction_preserves_value() {
        // Run many times to hit both the "skip" and "reduce" random paths.
        for _ in 0..100 {
            let r = BoundedRational::from_longs(6, 4).unwrap();
            if let Some(result) = BoundedRational::maybe_reduce(Some(r)) {
                // Value must still equal 1.5 regardless of which path was taken.
                let num: f64 = result.numerator().to_string().parse().unwrap();
                let den: f64 = result.denominator().to_string().parse().unwrap();
                assert!(
                    (num / den - 1.5).abs() < 1e-10,
                    "Value changed: {}/{}",
                    result.numerator(),
                    result.denominator()
                );
            }
        }
    }

    // ── value_of_long ─────────────────────────────────────────────────────────

    // Category 1: Cached constants (-2, -1, 0, 1, 2, 10)
    #[test]
    fn value_of_long_cached_constant_negative_two() {
        let r = BoundedRational::value_of_long(-2);
        assert_eq!(r.numerator(), &BigInt::from(-2));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn value_of_long_zcached_constant_zero() {
        let r = BoundedRational::value_of_long(0);
        assert_eq!(r.numerator(), &BigInt::from(0));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn value_of_long_cached_constant_ten() {
        let r = BoundedRational::value_of_long(10);
        assert_eq!(r.numerator(), &BigInt::from(10));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    // Category 2: In-range but not cached, the gap (3..=9)
    #[test]
    fn value_of_long_uncached_in_range_value_three() {
        let r = BoundedRational::value_of_long(3);
        assert_eq!(r.numerator(), &BigInt::from(3));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn value_of_long_uncached_in_range_value_nine() {
        let r = BoundedRational::value_of_long(9);
        assert_eq!(r.numerator(), &BigInt::from(9));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    // Category 3: Just outside the cached range (-3, 11)
    #[test]
    fn value_of_long_negative_three_returns_negative_three() {
        let r = BoundedRational::value_of_long(-3);
        assert_eq!(r.numerator(), &BigInt::from(-3));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn value_of_long_eleven_returns_eleven() {
        let r = BoundedRational::value_of_long(11);
        assert_eq!(r.numerator(), &BigInt::from(11));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    // Category 4: Extremes of i64
    #[test]
    fn value_of_long_i64_max_returns_i64() {
        let r = BoundedRational::value_of_long(i64::MAX);
        assert_eq!(r.numerator(), &BigInt::from(i64::MAX));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn value_of_long_i64_min_returns_i64() {
        let r = BoundedRational::value_of_long(i64::MIN);
        assert_eq!(r.numerator(), &BigInt::from(i64::MIN));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    // Category 5: Ordinary value far from any boundary
    #[test]
    fn value_of_long_negative_500_returns_negative_500() {
        let r = BoundedRational::value_of_long(-500);
        assert_eq!(r.numerator(), &BigInt::from(-500));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    // ── value_of_double ─────────────────────────────────────────────────────────

    #[test]
    fn value_of_double_small_integer() {
        let r = BoundedRational::value_of_double(42.0).unwrap();

        assert_eq!(r.numerator(), &BigInt::from(42));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn value_of_double_large_integer() {
        let r = BoundedRational::value_of_double(1001.0).unwrap();

        assert_eq!(r.numerator(), &BigInt::from(1001));
        assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn value_of_double_fraction() {
        let r = BoundedRational::value_of_double(0.5).unwrap().reduce();

        assert_eq!(r.numerator(), &BigInt::from(1));
        assert_eq!(r.denominator(), &BigInt::from(2));
    }

    #[test]
    fn value_of_double_negative_fraction() {
        let r = BoundedRational::value_of_double(-0.5).unwrap().reduce();

        assert_eq!(r.numerator(), &BigInt::from(-1));
        assert_eq!(r.denominator(), &BigInt::from(2));
    }

    #[test]
    fn value_of_double_positive_infinity() {
        assert!(BoundedRational::value_of_double(f64::INFINITY).is_err());
    }

    #[test]
    fn value_of_double_negative_infinity() {
        assert!(BoundedRational::value_of_double(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn value_of_double_nan() {
        assert!(BoundedRational::value_of_double(f64::NAN).is_err());
    }

    #[test]
    fn value_of_double_subnormal() {
        let r = BoundedRational::value_of_double(f64::from_bits(1)).unwrap();

        assert!(r.numerator() > &BigInt::from(0));
        assert!(r.denominator() > &BigInt::from(0));
    }

    #[test]
    fn value_of_double_smallest_normal() {
        let r = BoundedRational::value_of_double(f64::MIN_POSITIVE).unwrap();

        assert!(r.numerator() > &BigInt::from(0));
        assert!(r.denominator() > &BigInt::from(0));
    }

    #[test]
    fn value_of_double_large_power_of_two() {
        let r = BoundedRational::value_of_double(2f64.powi(60)).unwrap();

        assert_eq!(r.denominator(), &BigInt::from(1));
        assert_eq!(r.numerator(), &(BigInt::from(1) << 60));
    }

    #[test]
    fn value_of_double_large_fraction() {
        let r = BoundedRational::value_of_double(1024.5).unwrap().reduce();

        assert_eq!(r.numerator(), &BigInt::from(2049));
        assert_eq!(r.denominator(), &BigInt::from(2));
    }

    #[test]
    fn value_of_double_i64_max() {
        let r = BoundedRational::value_of_double(i64::MAX as f64).unwrap();

        assert_eq!(r.denominator(), &BigInt::from(1));
        assert_eq!(r.numerator(), &BigInt::from(i64::MAX));
    }

    #[test]
    fn value_of_double_below_i64_min_uses_slow_path() {
        let r = BoundedRational::value_of_double(-1e100).unwrap();

        assert_eq!(r.denominator(), &BigInt::from(1));
        assert_ne!(r.numerator(), &BigInt::from(i64::MIN));
    }

    // ── negate ─────────────────────────────────────────────────────────────────

    #[test]
    fn negate_none() {
        assert!(BoundedRational::negate(None).is_none());
    }

    #[test]
    fn negate_positive_fraction() {
        let r = BoundedRational::from_longs(3, 4).unwrap();
        let neg = BoundedRational::negate(Some(r)).unwrap();

        assert_eq!(neg.numerator(), &BigInt::from(-3));
        assert_eq!(neg.denominator(), &BigInt::from(4));
    }

    #[test]
    fn negate_negative_fraction() {
        let r = BoundedRational::from_longs(-5, 10).unwrap();
        let neg = BoundedRational::negate(Some(r)).unwrap();

        assert_eq!(neg.numerator(), &BigInt::from(5));
        assert_eq!(neg.denominator(), &BigInt::from(10));
    }

    // ── add ──────────────────────────────────────────────────────────────────

    #[test]
    fn add_none_both() {
        assert!(BoundedRational::add(None, None).is_none());
    }

    #[test]
    fn add_basic_fractions() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_longs(1, 3).unwrap();
        let sum = BoundedRational::add(Some(r1), Some(r2)).unwrap().reduce();

        assert_eq!(sum.numerator(), &BigInt::from(5));
        assert_eq!(sum.denominator(), &BigInt::from(6));
    }

    #[test]
    fn add_same_denominator_reduces() {
        let r1 = BoundedRational::from_longs(1, 4).unwrap();
        let r2 = BoundedRational::from_longs(1, 4).unwrap();
        let sum = BoundedRational::add(Some(r1), Some(r2)).unwrap().reduce();

        assert_eq!(sum.numerator(), &BigInt::from(1));
        assert_eq!(sum.denominator(), &BigInt::from(2));
    }

    #[test]
    fn add_different_denominator_reduces() {
        let r1 = BoundedRational::from_longs(1, 3).unwrap();
        let r2 = BoundedRational::from_longs(1, 6).unwrap();
        let sum = BoundedRational::add(Some(r1), Some(r2)).unwrap().reduce();

        assert_eq!(sum.numerator(), &BigInt::from(1));
        assert_eq!(sum.denominator(), &BigInt::from(2));
    }

    #[test]
    fn add_with_negative_denominator() {
        let r1 = BoundedRational::from_longs(1, -2).unwrap();
        let r2 = BoundedRational::from_longs(1, 3).unwrap();
        let sum = BoundedRational::add(Some(r1), Some(r2))
            .unwrap()
            .positive_den()
            .reduce();

        assert_eq!(sum.numerator(), &BigInt::from(-1));
        assert_eq!(sum.denominator(), &BigInt::from(6));
    }

    #[test]
    fn add_large_reducible_inputs_uses_reduction_heuristic() {
        // Construct two very large but easily reducible fractions.
        let factor = BigInt::from(1u32) << (MAX_SIZE / 2);

        let r1 = BoundedRational::new(factor.clone() * 2u32, factor.clone()).unwrap();
        let r2 = BoundedRational::new(factor.clone() * 3u32, factor).unwrap();
        let sum = BoundedRational::add(Some(r1), Some(r2)).unwrap().reduce();

        assert_eq!(sum.numerator(), &BigInt::from(5));
        assert_eq!(sum.denominator(), &BigInt::from(1));
    }

    #[test]
    fn add_zero_right_returns_left() {
        let r1 = BoundedRational::from_longs(3, 7).unwrap();
        let r2 = BoundedRational::from_longs(0, 1).unwrap();
        let sum = BoundedRational::add(Some(r1), Some(r2)).unwrap().reduce();

        assert_eq!(sum.numerator(), &BigInt::from(3));
        assert_eq!(sum.denominator(), &BigInt::from(7));
    }

    #[test]
    fn add_zero_with_negative_denominator_is_unaffected() {
        // Zero-numerator operand with a negative denominator should still
        // short-circuit correctly and not corrupt the sign of the result.
        let r1 = BoundedRational::from_longs(0, -1).unwrap();
        let r2 = BoundedRational::from_longs(1, -2).unwrap();
        let sum = BoundedRational::add(Some(r1), Some(r2))
            .unwrap()
            .positive_den()
            .reduce();

        assert_eq!(sum.numerator(), &BigInt::from(-1));
        assert_eq!(sum.denominator(), &BigInt::from(2));
    }

    // ── subtract ────────────────────────────────────────────────────────────────

    #[test]
    fn subtract_none_r1() {
        let r2 = BoundedRational::from_long(1);
        assert!(BoundedRational::subtract(None, Some(r2)).is_none());
    }

    #[test]
    fn subtract_from_zero() {
        let r1 = BoundedRational::from_long(0);
        let r2 = BoundedRational::from_longs(1, 2).unwrap();
        let diff = BoundedRational::subtract(Some(r1), Some(r2)).unwrap();

        assert_eq!(diff.numerator(), &BigInt::from(-1));
        assert_eq!(diff.denominator(), &BigInt::from(2));
    }

    #[test]
    fn subtract_matches_add_of_negation() {
        // Cross-check: subtract(r1, r2) should equal add(r1, negate(r2)) exactly,
        // since that's how subtract is implemented.
        let r1 = BoundedRational::from_longs(7, 9).unwrap();
        let r2 = BoundedRational::from_longs(2, 5).unwrap();

        let via_subtract = BoundedRational::subtract(Some(r1.clone()), Some(r2.clone()))
            .unwrap()
            .reduce();
        let via_add_negate = BoundedRational::add(Some(r1), BoundedRational::negate(Some(r2)))
            .unwrap()
            .reduce();

        assert_eq!(via_subtract.numerator(), via_add_negate.numerator());
        assert_eq!(via_subtract.denominator(), via_add_negate.denominator());
    }

    // ── multiply ─────────────────────────────────────────────────────────────

    #[test]
    fn multiply_basic() {
        let r1 = BoundedRational::from_longs(2, 3).unwrap();
        let r2 = BoundedRational::from_longs(3, 4).unwrap();
        let prod = BoundedRational::multiply(Some(r1), Some(r2))
            .unwrap()
            .reduce()
            .positive_den();
        // unreduced: (2*3)/(3*4) = 6/12 = 1/2
        assert_eq!(prod.numerator(), &BigInt::from(1));
        assert_eq!(prod.denominator(), &BigInt::from(2));
    }

    #[test]
    fn multiply_one_shortcut_right() {
        let r1 = BoundedRational::from_longs(5, 7).unwrap();
        let one = BoundedRational::value_of_long(1);
        let prod = BoundedRational::multiply(Some(r1), Some(one)).unwrap();
        assert_eq!(prod.numerator(), &BigInt::from(5));
        assert_eq!(prod.denominator(), &BigInt::from(7));
    }

    #[test]
    fn multiply_none_propagates_left() {
        let r2 = BoundedRational::from_longs(3, 4).unwrap();
        let prod = BoundedRational::multiply(None, Some(r2));
        assert!(prod.is_none());
    }

    #[test]
    fn multiply_by_zero() {
        let r1 = BoundedRational::from_long(0);
        let r2 = BoundedRational::from_longs(5, 9).unwrap();
        let prod = BoundedRational::multiply(Some(r1), Some(r2)).unwrap();
        assert_eq!(prod.numerator(), &BigInt::from(0));
    }

    #[test]
    fn multiply_reduces_to_lowest_terms() {
        let r1 = BoundedRational::from_longs(2, 3).unwrap();
        let r2 = BoundedRational::from_longs(3, 4).unwrap();
        let prod = BoundedRational::multiply(Some(r1), Some(r2))
            .unwrap()
            .reduce()
            .positive_den();
        assert_eq!(prod.numerator(), &BigInt::from(1));
        assert_eq!(prod.denominator(), &BigInt::from(2));
    }

    #[test]
    fn multiply_negative_values() {
        let r1 = BoundedRational::from_longs(-2, 3).unwrap();
        let r2 = BoundedRational::from_longs(3, 4).unwrap();
        let prod = BoundedRational::multiply(Some(r1), Some(r2))
            .unwrap()
            .reduce()
            .positive_den();
        // (-2/3) * (3/4) = -6/12 = -1/2
        assert_eq!(prod.numerator(), &BigInt::from(-1));
        assert_eq!(prod.denominator(), &BigInt::from(2));
    }

    // ── inverse ────────────────────────────────────────────────────────────────

    #[test]
    fn inverse_negative_numerator() {
        let r = BoundedRational::from_longs(-2, 3).unwrap();
        let inv = BoundedRational::inverse(Some(r)).unwrap().unwrap();
        assert_eq!(inv.numerator(), &BigInt::from(3));
        assert_eq!(inv.denominator(), &BigInt::from(-2));
    }

    #[test]
    fn inverse_zero_numerator_errors() {
        let r = BoundedRational::from_long(0);
        let result = BoundedRational::inverse(Some(r));
        assert!(result.is_err());
    }

    #[test]
    fn inverse_none_returns_ok_none() {
        let result = BoundedRational::inverse(None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ── divide ────────────────────────────────────────────────────────────────

    #[test]
    fn divide_basic() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_longs(1, 4).unwrap();
        let quot = BoundedRational::divide(Some(r1), Some(r2))
            .unwrap()
            .unwrap()
            .reduce()
            .positive_den();
        assert_eq!(quot.numerator(), &BigInt::from(2));
        assert_eq!(quot.denominator(), &BigInt::from(1));
    }

    #[test]
    fn divide_by_zero_errors() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_long(0);
        let result = BoundedRational::divide(Some(r1), Some(r2));
        assert!(result.is_err());
    }
}
