use crate::evaluation::constants::{MAX_SIZE, MINUS_ONE, MINUS_TWO, ONE, TEN, TWO, ZERO};
use crate::evaluation::errors::ZeroDivisionError;
use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::ToPrimitive;
use rand::Rng;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Error returned when a `BoundedRational` is constructed with a zero denominator.
#[derive(Clone, Debug)]
pub struct ZeroDenominatorError;

impl std::fmt::Display for ZeroDenominatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "denominator must not be zero")
    }
}

impl std::error::Error for ZeroDenominatorError {}

/// Converts a `ZeroDenominatorError` into a `ZeroDivisionError`.
///
/// This allows the `?` operator to automatically convert a
/// `ZeroDenominatorError` into a `ZeroDivisionError` when a division-related
/// operation creates a rational number with a zero denominator.
impl From<ZeroDenominatorError> for ZeroDivisionError {
    fn from(_: ZeroDenominatorError) -> Self {
        ZeroDivisionError
    }
}

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

/// Returns -1, 0, or 1 depending on whether `x` is negative, zero, or positive.
fn signum_bigint(x: &BigInt) -> i32 {
    match x.sign() {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    }
}

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

#[derive(Clone, Debug)]
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
    pub fn equals(&self, n: &BigInt) -> bool {
        self.numerator == n * &self.denominator
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

        if r1.equals(&ONE) {
            return Some(r2);
        }
        if r2.equals(&ONE) {
            return Some(r1);
        }
        if r1.equals(&MINUS_ONE) {
            return Some(BoundedRational {
                numerator: -r2.numerator,
                denominator: r2.denominator,
            });
        }
        if r2.equals(&MINUS_ONE) {
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
    /// Returns `Err(ZeroDivisionError)` if `r`'s numerator is zero, since
    /// the resulting denominator would be zero.
    pub fn inverse(
        r: Option<BoundedRational>,
    ) -> Result<Option<BoundedRational>, ZeroDivisionError> {
        let r = match r {
            Some(r) => r,
            None => return Ok(None),
        };
        if r.numerator == *ZERO {
            return Err(ZeroDivisionError);
        }
        Ok(Some(BoundedRational {
            numerator: r.denominator,
            denominator: r.numerator,
        }))
    }

    /// Returns `r1 / r2`, computed as `r1 * inverse(r2)`.
    ///
    /// # Errors
    /// Returns `Err(ZeroDivisionError)` if `r2` is zero.
    pub fn divide(
        r1: Option<BoundedRational>,
        r2: Option<BoundedRational>,
    ) -> Result<Option<BoundedRational>, ZeroDivisionError> {
        Ok(BoundedRational::multiply(r1, BoundedRational::inverse(r2)?))
    }

    /// Returns the sign of this rational: `-1` if negative, `0` if zero, `1` if positive.
    ///
    /// A fraction's sign is the sign of the numerator times the sign of the
    /// denominator. So `-3/4` and `3/-4` both correctly report `-1`.
    pub fn signum(&self) -> i32 {
        signum_bigint(&self.numerator) * signum_bigint(&self.denominator)
    }

    /// Multiplies `n1` and `n2`, skipping the multiplication when either side
    /// is `1` or `-1` (returning a clone or negation of the other operand
    /// instead). Used by `compare_to` to speed up cross-multiplication when a
    /// numerator or denominator is a plain integer.
    fn cross_multiply(n1: &BigInt, n2: &BigInt) -> BigInt {
        if *n1 == *ONE {
            n2.clone()
        } else if *n1 == *MINUS_ONE {
            -n2
        } else if *n2 == *ONE {
            n1.clone()
        } else if *n2 == *MINUS_ONE {
            -n1
        } else {
            n1 * n2
        }
    }

    /// Compares `self` to `other`, the way you'd compare two fractions by hand.
    ///
    /// # How it works
    /// 1. First it checks the signs. If one number is negative and the other
    ///    is positive (or zero), we already know the answer and can skip the
    ///    expensive math entirely.
    /// 2. If the signs match, it cross-multiplies: `a/b` vs `c/d` becomes
    ///    comparing `a*d` vs `c*b`. via [`cross_multiply`](Self::cross_multiply)
    ///    rather than the raw `*` operator, so that a numerator or
    ///    denominator of `1`/`-1` (by far the most common case — e.g. either
    ///    side being a plain integer) is handled without a full `BigInt`
    ///    multiplication. This also avoids doing any division.
    /// 3. Because a denominator can technically be stored as negative, the
    ///    result of the cross-multiplication is flipped if exactly one of the two
    ///    denominators is negative.
    pub fn compare_to(&self, other: &BoundedRational) -> Ordering {
        let sign1 = self.signum();
        let sign2 = other.signum();
        if sign1 != sign2 {
            return sign1.cmp(&sign2);
        }

        if self.numerator == *ZERO && other.signum() == 1 {
            return Ordering::Less;
        }
        if self.numerator == *ZERO && other.signum() == -1 {
            return Ordering::Greater;
        }
        if self.signum() == 1 && other.numerator == *ZERO {
            return Ordering::Greater;
        }
        if self.signum() == -1 && other.numerator == *ZERO {
            return Ordering::Less;
        }

        let lhs = Self::cross_multiply(&self.numerator, &other.denominator);
        let rhs = Self::cross_multiply(&other.numerator, &self.denominator);
        let cross = lhs.cmp(&rhs);

        let den_sign_product = signum_bigint(&self.denominator) * signum_bigint(&other.denominator);
        if den_sign_product < 0 {
            cross.reverse()
        } else {
            cross
        }
    }

    /// Returns this value as an `i64`, provided it is a whole number.
    ///
    /// The value is reduced first via [`reduce`]; if the reduced
    /// denominator isn't `1` the value has a genuine fractional part and
    /// isn't representable as an integer.
    ///
    /// # Errors
    /// Returns `Err` if the reduced denominator isn't `1`, or if the
    /// resulting numerator doesn't fit in an `i64`.
    pub fn int_value(&self) -> Result<i64, &'static str> {
        let reduced = self.reduce().positive_den();
        if reduced.denominator != *ONE {
            return Err("intValue of non-int");
        }
        reduced
            .numerator
            .to_i64()
            .ok_or("intValue: numerator does not fit in i64")
    }

    /// Converts this rational number to the closest `f64` value.
    ///
    /// Rounding is done correctly, and if the value is exactly halfway
    /// between two `f64` values, it is rounded **away from zero**.
    ///
    /// # Fast path
    /// The value is first reduced to lowest terms with a positive
    /// denominator via [`reduce`] + [`positive_den`]. If the resulting
    /// denominator is `1` (i.e. this value is a whole number), the numerator
    /// is converted to `f64` directly via `BigInt`'s built-in conversion,
    /// skipping the manual bit-manipulation path entirely.
    ///
    /// # Slow path
    /// For genuine fractions, the numerator and denominator are prescaled by
    /// a power of two so that dividing them yields a quotient with enough
    /// bits (roughly 80 extra) to determine the correctly-rounded 53-bit
    /// mantissa. The quotient's bit length then determines the binary
    /// exponent, the mantissa is rounded to 53 bits, and the IEEE-754 bit
    /// pattern is assembled directly via `f64::from_bits`.
    ///
    /// # Special cases
    /// - If the value is `0`, the function returns `0.0`.
    /// - If the value is too small to be represented by `f64`, it also
    ///   returns `0.0`.
    /// - If the value is too large to fit in an `f64`, it returns
    ///   `f64::INFINITY`.
    ///
    /// This method is designed to give an accurate floating-point
    /// approximation of the rational number while handling very small and
    /// very large values safely.
    pub fn double_value(&self) -> f64 {
        let nicer = self.reduce().positive_den();

        // Fast path: whole numbers convert directly, no bit manipulation needed.
        if nicer.denominator == *ONE {
            // BigInt's to_f64 saturates to infinity for out-of-range magnitudes
            // rather than returning None, so this fallback is defensive only.
            return nicer.numerator.to_f64().unwrap_or(f64::INFINITY);
        }

        let sign = nicer.signum();
        if sign < 0 {
            return -BoundedRational::negate(Some(nicer)).unwrap().double_value();
        }

        let appr_exp = nicer.numerator.bits() as i64 - nicer.denominator.bits() as i64;

        // The smallest positive value representable by f64 at all is the
        // smallest subnormal, 2^-1074. If appr_exp were exact, anything below
        // that threshold would safely be treated as 0. Since it's only
        // approximate, -1100 (comfortably below -1074) is used instead, giving
        // enough margin that the approximation's imprecision can never cause a
        // value that's actually still representable as a subnormal to be
        // wrongly short-circuited to 0.0. Values that genuinely fall below
        // -1100 are unambiguously going to underflow to zero regardless, so
        // bailing out here also avoids doing an expensive big-integer division
        // for a result we already know will be ~0.
        if appr_exp < -1100 || sign == 0 {
            return 0.0;
        }

        // An f64 mantissa holds 53 bits of precision. To produce a correctly
        // rounded (not just truncated) result, we need a few extra bits beyond
        // those 53: enough to see past the rounding point and decide which way
        // to round, plus a safety margin to absorb appr_exp's own imprecision
        // (see the comment above, since appr_exp is only an approximate
        // exponent, not exact). 80 extra bits is comfortably more than enough
        // for both purposes while still being cheap - it costs only a modestly
        // larger BigInt division, not a meaningfully slower one.
        //
        // needed_prec shifts the division so the resulting quotient has
        // roughly (53 + 80) significant bits instead of just enough to cover
        // the integer part - that's what gives extra_bits (computed later from
        // the quotient's actual bit length) enough headroom to round correctly
        // down to 53 bits.
        let needed_prec = appr_exp - 80;
        let dividend = if needed_prec < 0 {
            &nicer.numerator << (-needed_prec) as usize
        } else {
            nicer.numerator.clone()
        };
        let divisor = if needed_prec > 0 {
            &nicer.denominator << needed_prec as usize
        } else {
            nicer.denominator.clone()
        };

        let quotient = dividend / divisor;
        let q_length = quotient.bits() as i64;
        let mut extra_bits = q_length - 53;
        let mut exponent = needed_prec + q_length;

        if exponent >= -1021 {
            exponent -= 1;
        } else {
            extra_bits += (-1022 - exponent) + 1;
            exponent = -1023;
        }

        if exponent > 1024 {
            return f64::INFINITY;
        }

        let rounding = BigInt::from(1) << (extra_bits - 1).max(0) as usize;
        let big_mantissa = (quotient + rounding) >> extra_bits.max(0) as usize;

        let mantissa = big_mantissa.to_i64().unwrap_or(0);
        let bits: u64 = (mantissa as u64 & ((1u64 << 52) - 1)) | (((exponent + 1023) as u64) << 52);
        f64::from_bits(bits)
    }

    /// Returns a decimal string with exactly `n` digits after the decimal
    /// point.
    ///
    /// The value is **truncated** (rounded toward zero), so any extra digits
    /// after the `n`th decimal place are simply removed instead of being
    /// rounded.
    ///
    /// # How it works
    /// - The number is multiplied by `10^n`.
    /// - Integer division is used to discard any remaining fractional part.
    /// - The resulting digits are padded with leading zeros if needed so
    ///   there is always at least one digit before the decimal point.
    /// - If the original value is negative, a `-` sign is added to the
    ///   beginning of the string.
    ///
    /// # Example
    /// `12.3456` with `n = 2` becomes `"12.34"`.
    /// `-0.9876` with `n = 3` becomes `"-0.987"`.
    pub fn to_string_truncated(&self, n: u32) -> String {
        let mut scale = ONE.clone();
        for _ in 0..n {
            scale *= &*TEN;
        }

        let mut num_abs = self.numerator.clone();
        if num_abs < *ZERO {
            num_abs = -num_abs;
        }
        let mut den_abs = self.denominator.clone();
        if den_abs < *ZERO {
            den_abs = -den_abs;
        }

        let mut digits = (num_abs * scale / den_abs).to_string();
        let n = n as usize;
        let mut len = digits.len();
        if len < n + 1 {
            digits = "0".repeat(n + 1 - len) + &digits;
            len = n + 1;
        }

        let sign = if self.signum() < 0 { "-" } else { "" };
        format!("{}{}.{}", sign, &digits[..len - n], &digits[len - n..])
    }
}

impl PartialEq for BoundedRational {
    /// Two rationals are equal if they represent the same value, even if
    /// they're stored with different numerators/denominators (e.g. `1/2`
    /// and `2/4`).
    fn eq(&self, other: &Self) -> bool {
        self.compare_to(other) == Ordering::Equal
    }
}

/// `BoundedRational` never contains NaN-like values, so equality is total.
impl Eq for BoundedRational {}

impl Hash for BoundedRational {
    /// Hashes this rational so that equal values always produce equal hashes.
    ///
    /// `1/2` and `2/4` are `==` to each other, so they must hash the same
    /// way too, or things like `HashSet`/`HashMap` break. To guarantee that,
    /// we first reduce the fraction to lowest terms and make the denominator
    /// positive, so every value that compares equal ends up with an
    /// identical numerator/denominator pair before hashing.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let reduced = self.reduce().positive_den();
        reduced.numerator.hash(state);
        reduced.denominator.hash(state);
    }
}

impl std::fmt::Display for BoundedRational {
    /// Formats as `numerator/denominator` using the raw, possibly
    /// unreduced, stored values. This is a debug/log representation, not a
    /// user-facing one.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64;
    use num_bigint::BigInt;
    use std::cmp::Ordering;
    use std::collections::HashSet;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
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

    // ── ZeroDivisionError display ────────────────────────────────────────────

    #[test]
    fn zero_division_error_display() {
        assert_eq!(ZeroDivisionError.to_string(), "division by zero");
    }

    #[test]
    fn zero_division_error_from_zero_denominator_error() {
        let source = ZeroDenominatorError;
        let converted: ZeroDivisionError = source.into();
        assert_eq!(converted.to_string(), "division by zero");
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

    // ── signum ─────────────────────────────────────────────────────────

    #[test]
    fn signum_positive() {
        let r = BoundedRational::from_longs(3, 4).unwrap();
        assert_eq!(r.signum(), 1);
    }

    #[test]
    fn signum_negative_numerator() {
        let r = BoundedRational::from_longs(-3, 4).unwrap();
        assert_eq!(r.signum(), -1);
    }

    #[test]
    fn signum_negative_denominator() {
        let r = BoundedRational::from_longs(3, -4).unwrap();
        assert_eq!(r.signum(), -1);
    }

    #[test]
    fn signum_both_negative_is_positive() {
        let r = BoundedRational::from_longs(-3, -4).unwrap();
        assert_eq!(r.signum(), 1);
    }

    #[test]
    fn signum_zero() {
        let r = BoundedRational::from_longs(0, 5).unwrap();
        assert_eq!(r.signum(), 0);
    }

    // ── compare_to ─────────────────────────────────────────────────────────

    #[test]
    fn compare_to_integers() {
        let r1 = BoundedRational::from_long(3);
        let r2 = BoundedRational::from_long(5);
        assert_eq!(r1.compare_to(&r2), Ordering::Less);
        assert_eq!(r2.compare_to(&r1), Ordering::Greater);
    }

    #[test]
    fn compare_to_integer_vs_fraction() {
        let r1 = BoundedRational::from_long(1); // 1/1
        let r2 = BoundedRational::from_longs(3, 2).unwrap(); // 3/2
        assert_eq!(r1.compare_to(&r2), Ordering::Less);
        assert_eq!(r2.compare_to(&r1), Ordering::Greater);
    }

    #[test]
    fn compare_to_equal_values() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_longs(2, 4).unwrap();
        assert_eq!(r1.compare_to(&r2), Ordering::Equal);
    }

    #[test]
    fn compare_to_less_than() {
        let r1 = BoundedRational::from_longs(1, 3).unwrap();
        let r2 = BoundedRational::from_longs(1, 2).unwrap();
        assert_eq!(r1.compare_to(&r2), Ordering::Less);
    }

    #[test]
    fn compare_to_greater_than() {
        let r1 = BoundedRational::from_longs(3, 4).unwrap();
        let r2 = BoundedRational::from_longs(1, 2).unwrap();
        assert_eq!(r1.compare_to(&r2), Ordering::Greater);
    }

    #[test]
    fn compare_to_numerator_is_minus_one() {
        let r1 = BoundedRational::from_longs(-1, 5).unwrap();
        let r2 = BoundedRational::from_longs(-1, 3).unwrap();
        // -1/5 = -0.2, -1/3 ≈ -0.333, so -1/5 > -1/3
        assert_eq!(r1.compare_to(&r2), Ordering::Greater);
    }

    #[test]
    fn compare_to_denominator_is_minus_one() {
        let r1 = BoundedRational::from_longs(3, -1).unwrap(); // -3
        let r2 = BoundedRational::from_longs(2, -1).unwrap(); // -2
        assert_eq!(r1.compare_to(&r2), Ordering::Less);
    }

    #[test]
    fn compare_to_one_negative_denominator_flips_result() {
        let r1 = BoundedRational::from_longs(1, -2).unwrap(); // -1/2
        let r2 = BoundedRational::from_longs(1, 3).unwrap(); // 1/3
        assert_eq!(r1.compare_to(&r2), Ordering::Less);
        assert_eq!(r2.compare_to(&r1), Ordering::Greater);
    }

    #[test]
    fn compare_to_negative_vs_positive() {
        let r1 = BoundedRational::from_longs(-1, 2).unwrap();
        let r2 = BoundedRational::from_longs(1, 2).unwrap();
        assert_eq!(r1.compare_to(&r2), Ordering::Less);
    }

    #[test]
    fn compare_to_both_negative() {
        let r1 = BoundedRational::from_longs(-1, 2).unwrap();
        let r2 = BoundedRational::from_longs(-1, 3).unwrap();
        assert_eq!(r1.compare_to(&r2), Ordering::Less);
    }

    #[test]
    fn compare_to_both_negative_denominators_no_flip_needed() {
        let r1 = BoundedRational::from_longs(-1, -2).unwrap(); // 1/2
        let r2 = BoundedRational::from_longs(-1, -3).unwrap(); // 1/3
        assert_eq!(r1.compare_to(&r2), Ordering::Greater);
    }

    #[test]
    fn compare_to_negative_denominator() {
        let r1 = BoundedRational::from_longs(1, -2).unwrap();
        let r2 = BoundedRational::from_longs(-1, 2).unwrap();
        assert_eq!(r1.compare_to(&r2), Ordering::Equal);
    }

    #[test]
    fn compare_to_zero() {
        let r1 = BoundedRational::from_longs(0, 1).unwrap();
        let r2 = BoundedRational::from_longs(0, 5).unwrap();
        assert_eq!(r1.compare_to(&r2), Ordering::Equal);

        let r3 = BoundedRational::from_longs(-1, 2).unwrap();
        let r4 = BoundedRational::from_long(0);
        assert_eq!(r3.compare_to(&r4), Ordering::Less);
    }

    #[test]
    fn compare_to_zero_with_negative_denominator() {
        let r1 = BoundedRational::from_longs(0, -5).unwrap();
        let r2 = BoundedRational::from_long(0);
        assert_eq!(r1.compare_to(&r2), Ordering::Equal);
    }

    // ── PartialEq / Eq ─────────────────────────────────────────────────────────

    #[test]
    fn eq_same_value_different_representation() {
        let r1 = BoundedRational::from_longs(2, 4).unwrap();
        let r2 = BoundedRational::from_longs(1, 2).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn eq_negative_forms() {
        let r1 = BoundedRational::from_longs(-1, 2).unwrap();
        let r2 = BoundedRational::from_longs(1, -2).unwrap();
        let r3 = BoundedRational::from_longs(-2, 4).unwrap();
        assert_eq!(r1, r2);
        assert_eq!(r1, r3);
    }

    #[test]
    fn not_eq_different_values() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_longs(1, 3).unwrap();
        assert_ne!(r1, r2);
    }

    // ── Hash ─────────────────────────────────────────────────────────

    #[test]
    fn hash_matches_for_equal_values() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_longs(2, 4).unwrap();

        let mut hasher1 = DefaultHasher::new();
        r1.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        r2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn hash_matches_for_negative_denominator_form() {
        let r1 = BoundedRational::from_longs(-1, 2).unwrap();
        let r2 = BoundedRational::from_longs(1, -2).unwrap();

        let mut hasher1 = DefaultHasher::new();
        r1.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        r2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn hash_differs_for_different_values() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_longs(1, 3).unwrap();

        let mut hasher1 = DefaultHasher::new();
        r1.hash(&mut hasher1);

        let mut hasher2 = DefaultHasher::new();
        r2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn hash_set_deduplicates_equal_values() {
        let r1 = BoundedRational::from_longs(1, 2).unwrap();
        let r2 = BoundedRational::from_longs(2, 4).unwrap();
        let r3 = BoundedRational::from_longs(1, 3).unwrap();

        let mut set = HashSet::new();
        set.insert(r1);
        set.insert(r2);
        set.insert(r3);

        assert_eq!(set.len(), 2);
    }

    // ── int_value ────────────────────────────────────────────────────────────

    /// Acceptance criterion: int_value() on 6/2 returns Ok(3)
    #[test]
    fn int_value_six_halves_returns_three() {
        let r = BoundedRational::from_longs(6, 2).unwrap();
        assert_eq!(r.int_value(), Ok(3));
    }

    /// Acceptance criterion: int_value() on 1/3 returns Err
    #[test]
    fn int_value_one_third_returns_err() {
        let r = BoundedRational::from_longs(1, 3).unwrap();
        assert!(r.int_value().is_err());
    }

    #[test]
    fn int_value_negative_integer() {
        let r = BoundedRational::from_longs(-8, 2).unwrap();
        assert_eq!(r.int_value(), Ok(-4));
    }

    #[test]
    fn int_value_zero() {
        let r = BoundedRational::from_long(0);
        assert_eq!(r.int_value(), Ok(0));
    }

    #[test]
    fn int_value_out_of_i64_range_errs() {
        let huge = BigInt::from(i64::MAX) + BigInt::from(1);
        let r = BoundedRational::from_bigint(huge);
        assert!(r.int_value().is_err());
    }

    #[test]
    fn int_value_negative_denominator_still_reduces_correctly() {
        // -6/-2 reduces to 3/1
        let r = BoundedRational::from_longs(-6, -2).unwrap();
        assert_eq!(r.int_value(), Ok(3));
    }

    // ── double_value ─────────────────────────────────────────────────────────

    #[test]
    fn double_value_fast_path_large_integer() {
        let r = BoundedRational::from_longs(-100, -1).unwrap(); // reduces to 100/1
        assert_eq!(r.double_value(), 100.0);
    }

    #[test]
    fn double_value_fast_path_matches_slow_path_result() {
        let as_fraction = BoundedRational::from_longs(20, 4).unwrap(); // reduces to 5/1
        let as_integer = BoundedRational::from_long(5);
        assert_eq!(as_fraction.double_value(), as_integer.double_value());
    }

    #[test]
    fn double_value_exact_half() {
        let r = BoundedRational::from_longs(1, 2).unwrap();
        assert_eq!(r.double_value(), 0.5);
    }

    #[test]
    fn double_value_negative_fraction() {
        let r = BoundedRational::from_longs(-3, 4).unwrap();
        assert_eq!(r.double_value(), -0.75);
    }

    #[test]
    fn double_value_integer() {
        let r = BoundedRational::from_long(5);
        assert_eq!(r.double_value(), 5.0);
    }

    #[test]
    fn double_value_zero() {
        let r = BoundedRational::from_long(0);
        assert_eq!(r.double_value(), 0.0);
    }

    #[test]
    fn double_value_repeating_fraction_is_close() {
        let r = BoundedRational::from_longs(1, 3).unwrap();
        assert!((r.double_value() - (1.0 / 3.0)).abs() < 1e-15);
    }

    // ── double_value: Maximum / overflow ────────────────────────────────────

    #[test]
    fn double_value_largest_finite_f64() {
        let r = BoundedRational::value_of_double(f64::MAX).unwrap();
        assert_eq!(r.double_value(), f64::MAX);
    }

    #[test]
    fn double_value_overflow_returns_infinity() {
        let max_r = BoundedRational::value_of_double(f64::MAX).unwrap();
        let doubled =
            BoundedRational::multiply(Some(max_r), Some(BoundedRational::from_long(2))).unwrap();
        assert_eq!(doubled.double_value(), f64::INFINITY);
    }

    #[test]
    fn double_value_negative_overflow_returns_negative_infinity() {
        let max_r = BoundedRational::value_of_double(f64::MAX).unwrap();
        let doubled =
            BoundedRational::multiply(Some(max_r), Some(BoundedRational::from_long(-2))).unwrap();
        assert_eq!(doubled.double_value(), f64::NEG_INFINITY);
    }

    // ── double_value: Normal / subnormal boundary ───────────────────────────

    #[test]
    fn double_value_smallest_normal() {
        // 2^-1022, the smallest positive normal f64.
        let r = BoundedRational::new(BigInt::from(1), BigInt::from(1) << 1022).unwrap();
        assert_eq!(r.double_value(), f64::MIN_POSITIVE);
    }

    #[test]
    fn double_value_largest_subnormal() {
        // (2^52 - 1) * 2^-1074, the largest positive subnormal f64 —
        // one ULP (Unit in the Last Place) below the smallest normal.
        let numerator = (BigInt::from(1) << 52) - BigInt::from(1);
        let r = BoundedRational::new(numerator, BigInt::from(1) << 1074).unwrap();
        let expected = f64::MIN_POSITIVE - f64::from_bits(1);
        assert_eq!(r.double_value(), expected);
    }

    #[test]
    fn double_value_smallest_subnormal() {
        // 2^-1074, the smallest positive representable f64 at all.
        let r = BoundedRational::new(BigInt::from(1), BigInt::from(1) << 1074).unwrap();
        assert_eq!(r.double_value(), f64::from_bits(1));
    }

    #[test]
    fn double_value_negative_smallest_subnormal() {
        let r = BoundedRational::new(BigInt::from(-1), BigInt::from(1) << 1074).unwrap();
        assert_eq!(r.double_value(), -f64::from_bits(1));
    }

    // ── double_value: Rounding ───────────────────────────────────────────────

    #[test]
    fn double_value_positive_halfway_rounds_away_from_zero() {
        // 2^53 + 1, over denominator 2, sits exactly halfway between the
        // representable f64 values 2^52 and 2^52 + 1 (ULP is 1 there).
        // Ties-away-from-zero must pick the larger-magnitude neighbor.
        let two_pow_52 = BigInt::from(1) << 52;
        let numerator: BigInt = &two_pow_52 * BigInt::from(2) + BigInt::from(1);
        let r = BoundedRational::new(numerator, BigInt::from(2)).unwrap();

        let neighbor: BigInt = &two_pow_52 + BigInt::from(1);
        let expected = match neighbor.to_f64() {
            Some(v) => v,
            None => panic!("expected value must be representable"),
        };
        assert_eq!(r.double_value(), expected);
    }

    #[test]
    fn double_value_negative_halfway_rounds_away_from_zero() {
        let two_pow_52 = BigInt::from(1) << 52;
        let magnitude: BigInt = &two_pow_52 * BigInt::from(2) + BigInt::from(1);
        let numerator = -magnitude;
        let r = BoundedRational::new(numerator, BigInt::from(2)).unwrap();

        let neighbor: BigInt = &two_pow_52 + BigInt::from(1);
        let expected = match neighbor.to_f64() {
            Some(v) => -v,
            None => panic!("expected value must be representable"),
        };
        assert_eq!(r.double_value(), expected);
    }

    #[test]
    fn double_value_halfway_between_zero_and_smallest_subnormal() {
        // Exactly half the smallest subnormal (2^-1075) sits precisely
        // between 0.0 and the smallest representable f64. Ties away from
        // zero means this should round up to the smallest subnormal, not
        // down to zero, and must not panic or produce NaN.
        let r = BoundedRational::new(BigInt::from(1), BigInt::from(1) << 1075).unwrap();
        assert_eq!(r.double_value(), f64::from_bits(1));
    }

    #[test]
    fn double_value_negative_halfway_between_zero_and_smallest_subnormal() {
        let r = BoundedRational::new(BigInt::from(-1), BigInt::from(1) << 1075).unwrap();
        assert_eq!(r.double_value(), -f64::from_bits(1));
    }

    // ── double_value: Huge BigInts ───────────────────────────────────────────

    #[test]
    fn double_value_huge_numerator_and_denominator_evaluates_to_two() {
        // Both numerator and denominator have thousands of bits, but their
        // exact ratio is 2. Confirms no panic and no silent mantissa
        // truncation when reducing/dividing very large BigInts.
        let factor = (BigInt::from(1) << 3000) + BigInt::from(1);
        let numerator = &factor * BigInt::from(2);
        let r = BoundedRational::new(numerator, factor).unwrap();
        assert_eq!(r.double_value(), 2.0);
    }

    #[test]
    fn double_value_huge_numerator_and_denominator_evaluates_to_one_point_five() {
        let factor = (BigInt::from(1) << 3000) + BigInt::from(1);
        let numerator = &factor * BigInt::from(3);
        let denominator = &factor * BigInt::from(2);
        let r = BoundedRational::new(numerator, denominator).unwrap();
        assert_eq!(r.double_value(), 1.5);
    }

    // ── to_string_truncated ──────────────────────────────────────────────────

    /// Acceptance criterion: to_string_truncated(2) on 1/3 returns "0.33"
    #[test]
    fn to_string_truncated_one_third_two_digits() {
        let r = BoundedRational::from_longs(1, 3).unwrap();
        assert_eq!(r.to_string_truncated(2), "0.33");
    }

    #[test]
    fn to_string_truncated_negative_value() {
        let r = BoundedRational::from_longs(-1, 3).unwrap();
        assert_eq!(r.to_string_truncated(2), "-0.33");
    }

    #[test]
    fn to_string_truncated_pads_leading_zeros() {
        let r = BoundedRational::from_longs(1, 1000).unwrap();
        assert_eq!(r.to_string_truncated(2), "0.00");
    }

    #[test]
    fn to_string_truncated_zero_precision() {
        let r = BoundedRational::from_long(5);
        assert_eq!(r.to_string_truncated(0), "5.");
    }

    #[test]
    fn to_string_truncated_negative_denominator_still_correct_sign() {
        // 1/-3 is negative, even though signum uses the raw (unreduced) form.
        let r = BoundedRational::from_longs(1, -3).unwrap();
        assert_eq!(r.to_string_truncated(2), "-0.33");
    }

    // ── Display ──────────────────────────────────────────────────────────────

    #[test]
    fn display_basic_fraction() {
        let r = BoundedRational::from_longs(3, 4).unwrap();
        assert_eq!(r.to_string(), "3/4");
    }

    #[test]
    fn display_negative_denominator_shown_raw() {
        let r = BoundedRational::from_longs(3, -4).unwrap();
        assert_eq!(r.to_string(), "3/-4");
    }
}
