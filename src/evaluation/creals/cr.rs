use crate::evaluation::constants::{ONE, ZERO};
use num_bigint::BigInt;
use std::sync::Mutex;

/// Errors that can occur while evaluating a constructive real.
///
/// Both variants represent conditions a caller may want to detect and
/// handle (e.g. retry, or report a clearer error further up the call
/// stack), so they're modeled as an ordinary, matchable `enum` rather
/// than surfaced only as a panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CRError {
    /// A long-running approximation was cancelled from another thread.
    Aborted,
    /// The requested precision needed more than 28 bits, which usually
    /// means a computation is diverging (e.g. division by zero).
    PrecisionOverflow,
}

impl std::fmt::Display for CRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CRError::Aborted => write!(f, "constructive real computation was aborted"),
            CRError::PrecisionOverflow => {
                write!(f, "requested precision overflowed safe range")
            }
        }
    }
}

impl std::error::Error for CRError {}

/// A constructive real: a number defined not by a stored value, but by an
/// algorithm that can produce an arbitrarily accurate approximation on
/// demand.
///
/// # Contract
///
/// `approximate(precision)` must return a [`BigInt`] `result` such that
///
/// ```text
/// |true_value - result * 2^precision| <= 1
/// ```
///
/// `precision` is expressed as a (typically negative) power-of-two
/// exponent: `-20` means "accurate to within `2^-20`".
///
/// `Send + Sync` is required so that any `Arc<dyn CR>` built from this
/// trait can be safely shared and evaluated from multiple threads at
/// once, with that guarantee checked at compile time.
pub trait CR: Send + Sync {
    /// Computes an approximation of the constructive real at the
    /// requested precision.
    fn approximate(&self, precision: i32) -> BigInt;
}

/// The memoized approximation cache for a [`CachedCR`].
///
/// The three fields are bundled into one struct so a single `Mutex` can
/// guard all of them together — a reader can never observe `min_prec`
/// updated without `max_appr`, or vice versa.
#[derive(Debug, Clone)]
pub struct ApprCache {
    /// The finest (most negative) precision computed so far.
    pub min_prec: i32,
    /// The cached approximation at `min_prec`.
    pub max_appr: BigInt,
    /// Whether `min_prec` / `max_appr` currently hold a usable value.
    pub appr_valid: bool,
}

impl Default for ApprCache {
    fn default() -> Self {
        ApprCache {
            min_prec: 0,
            max_appr: ZERO.clone(),
            appr_valid: false,
        }
    }
}

/// Wraps any [`CR`] implementation with a thread-safe memoization cache.
///
/// Caching is added by composition rather than by giving every [`CR`]
/// implementor its own cache fields: wrap a type in `CachedCR<T>` and it
/// gains memoized, lock-protected lookups without having to manage a
/// `Mutex` itself.
pub struct CachedCR<T: CR> {
    inner: T,
    cache: Mutex<ApprCache>,
}

impl<T: CR> CachedCR<T> {
    /// Wraps `inner` with a fresh, empty cache.
    pub fn new(inner: T) -> Self {
        CachedCR {
            inner,
            cache: Mutex::new(ApprCache::default()),
        }
    }

    /// Returns `value / 2^precision` rounded to an integer, with error
    /// strictly less than 1 — using the cache when possible.
    ///
    /// The cache is guarded by a `Mutex` that's locked for the entire
    /// method body, so two threads calling `get_appr` concurrently can
    /// never observe, or write, a half-updated cache.
    pub fn get_appr(&self, precision: i32) -> BigInt {
        check_prec(precision).expect("precision overflow");

        let mut cache = self.cache.lock().expect("CR cache mutex poisoned");

        if cache.appr_valid && precision >= cache.min_prec {
            scale(cache.max_appr.clone(), cache.min_prec - precision)
        } else {
            let result = self.inner.approximate(precision);
            cache.min_prec = precision;
            cache.max_appr = result.clone();
            cache.appr_valid = true;
            result
        }
    }
}

/// Multiplies `k` by `2^n`. Exact when `n >= 0`; truncates toward zero
/// (no rounding) when `n < 0`. Used internally by [`scale`], and directly
/// by combinators that don't need rounding (e.g. exact rescalings).
pub(crate) fn shift(k: BigInt, n: i32) -> BigInt {
    if n >= 0 {
        k << (n as u32)
    } else {
        k >> ((-n) as u32)
    }
}

/// Multiplies `k` by `2^n`, like [`shift`], but rounds to the nearest
/// integer (ties round up) rather than truncating when `n < 0`.
///
/// Every approximation handed back to a caller must satisfy the `CR`
/// error bound of strictly less than 1; truncating instead of rounding
/// here would silently double that worst-case error.
pub(crate) fn scale(k: BigInt, n: i32) -> BigInt {
    if n >= 0 {
        k << (n as u32)
    } else {
        let adjusted = shift(k, n + 1) + ONE.clone();
        adjusted >> 1u32
    }
}

/// Checks that `n` is at least a factor of 8 (3 bits) away from
/// overflowing the `i32` used to hold a precision value.
///
/// Bits 28 and 29 of `n` must agree — i.e. `n` is properly sign-extended
/// within its low 29 bits. If they disagree, `n` is close enough to
/// `i32::MIN`/`MAX` that further arithmetic on it (padding with guard
/// bits, as combinators commonly do) risks overflowing the `i32` —
/// almost always a sign that the underlying computation is diverging
/// rather than converging to a value.
pub fn check_prec(n: i32) -> Result<(), CRError> {
    let high = n >> 28;
    let high_shifted = n >> 29;
    if high ^ high_shifted != 0 {
        Err(CRError::PrecisionOverflow)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple stand-in `CR` so we have something to test `CachedCR` with.
    // It always returns the same fixed value, and remembers how many
    // times `approximate` was called — that's how we can check whether
    // the cache actually worked.
    struct ConstApprox {
        value: BigInt,
        calls: std::sync::atomic::AtomicU32,
    }

    impl ConstApprox {
        fn new(value: BigInt) -> Self {
            ConstApprox {
                value,
                calls: std::sync::atomic::AtomicU32::new(0),
            }
        }

        fn call_count(&self) -> u32 {
            self.calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl CR for ConstApprox {
        fn approximate(&self, _precision: i32) -> BigInt {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.value.clone()
        }
    }

    #[test]
    fn zero_constant_is_zero() {
        assert_eq!(*ZERO, BigInt::from(0));
    }

    #[test]
    fn one_constant_is_one() {
        assert_eq!(*ONE, BigInt::from(1));
    }

    #[test]
    fn shift_multiplies_by_a_power_of_two() {
        // 1 * 2^10 = 1024
        assert_eq!(shift(BigInt::from(1), 10), BigInt::from(1024));
    }

    #[test]
    fn shift_divides_by_a_power_of_two() {
        // 7 / 2 = 3 (rounds down, no special rounding)
        assert_eq!(shift(BigInt::from(7), -1), BigInt::from(3));
    }

    #[test]
    fn scale_multiplies_by_a_power_of_two() {
        // 3 * 2^2 = 12
        assert_eq!(scale(BigInt::from(3), 2), BigInt::from(12));
    }

    #[test]
    fn scale_rounds_instead_of_truncating() {
        // 7 / 2 = 3.5, which rounds up to 4 (unlike `shift`, which gave 3)
        assert_eq!(scale(BigInt::from(7), -1), BigInt::from(4));
    }

    #[test]
    fn check_prec_accepts_a_normal_precision() {
        assert_eq!(check_prec(0), Ok(()));
    }

    #[test]
    fn check_prec_rejects_a_huge_precision() {
        assert_eq!(check_prec(i32::MAX), Err(CRError::PrecisionOverflow));
    }

    #[test]
    fn apprcache_default_starts_empty() {
        let cache = ApprCache::default();
        assert_eq!(cache.min_prec, 0);
        assert_eq!(cache.max_appr, *ZERO);
        assert!(!cache.appr_valid);
    }

    #[test]
    fn crerror_aborted_has_a_readable_message() {
        assert_eq!(
            CRError::Aborted.to_string(),
            "constructive real computation was aborted"
        );
    }

    #[test]
    fn crerror_precision_overflow_has_a_readable_message() {
        assert_eq!(
            CRError::PrecisionOverflow.to_string(),
            "requested precision overflowed safe range"
        );
    }

    #[test]
    fn get_appr_computes_a_value_the_first_time() {
        let node = CachedCR::new(ConstApprox::new(BigInt::from(1000)));
        assert_eq!(node.get_appr(-10), BigInt::from(1000));
        assert_eq!(node.inner.call_count(), 1);
    }

    #[test]
    fn get_appr_does_not_recompute_on_a_repeat_call() {
        let node = CachedCR::new(ConstApprox::new(BigInt::from(1000)));
        node.get_appr(-10); // first call: computes and caches
        node.get_appr(-10); // second call: should reuse the cache
        assert_eq!(node.inner.call_count(), 1);
    }

    #[test]
    #[should_panic(expected = "precision overflow")]
    fn get_appr_panics_on_too_large_a_precision() {
        let node = CachedCR::new(ConstApprox::new(BigInt::from(0)));
        node.get_appr(i32::MAX);
    }
}
