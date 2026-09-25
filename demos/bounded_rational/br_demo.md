**Example 1:** Neither `0.1` nor `0.2` has an exact binary representation, so each is stored as the nearest representable double. Adding these two approximations produces a true binary value slightly above `0.3`, causing f64 to print a trailing error digit. `BoundedRational` stores `1/10` and `2/10` as exact fractions, so `1/10 + 2/10 = 3/10` prints cleanly as `0.3`.
```
1. Floating-point Rounding Error
Standard f64 (IEEE 754): 
0.1 + 0.2 = 0.30000000000000004
Bounded Rational Crate (Truncated to 17 digits after decimal point): 
0.1 + 0.2 = 0.30000000000000000
```


**Example 2:** Each addition of the imprecise `0.1` double compounds a tiny rounding error, so summing it ten times lands just below `1`. Interestingly, multiplying by 10 happens to round back to exactly `1`, showing f64 errors are inconsistent even between mathematically equivalent operations. The rational crate gives exactly `1` in both cases since `10 × (1/10) = 1` and `1/10 × 10` summed is `10/10`.
```
2. Cumulative Fraction
Standard f64 (IEEE 754): 
0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 = 0.9999999999999999
0.1 * 10 = 1
Bounded Rational Crate (Truncated to 1 digit after decimal point): 
0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 = 1.0000000000000000
0.1 * 10 = 1.0
```


**Example 3:** This expression should mathematically equal exactly `0`, but chained floating-point rounding across three imprecise operands leaves a tiny residual on the order of `10^-17`, a classic "epsilon" artifact. `BoundedRational` computes `3/10 − 2/10 − 1/10 = 0/10` exactly, leaving no residue at all.
```
3. Micro Subtraction Gap
Standard f64 (IEEE 754): 
0.3 - 0.2 - 0.1 = -0.000000000000000027755575615628914
Bounded Rational Crate (Truncated to 33 digits after decimal point): 
0.3 - 0.2 - 0.1 = 0.000000000000000000000000000000000
```


**Example 4:** `0.15` cannot be represented exactly in binary, so multiplying its rounded approximation by `3` produces a result just under `0.45`. `BoundedRational` treats `0.15` as the exact fraction `15/100`, so `15/100 × 3 = 45/100`, which prints as a clean `0.45`.
```
4. Multiplicative Rounding Error
Standard f64 (IEEE 754): 
0.15 * 3 = 0.44999999999999996
Bounded Rational Crate (Truncated to 17 digits after decimal point): 
0.15 * 3 = 0.45000000000000000
```


**Example 5:** Both `0.7` and `0.1` are stored as imprecise binary approximations, and their sum's true stored value rounds down just below `0.8`. `BoundedRational` adds the exact fractions `7/10 + 1/10 = 8/10`, giving a clean `0.8` with no rounding artifact.
```
5. Non-Integer Accumulation
Standard f64 (IEEE 754): 
0.7 + 0.1 = 0.7999999999999999
Bounded Rational Crate (Truncated to 16 digits after decimal point): 
0.7 + 0.1 = 0.8000000000000000
```


**Example 6:** `1.005` isn't exactly representable in binary, so the multiplication and subsequent subtraction compound that small initial error into a visibly large drift for what should be a simple interest calculation. The rational crate keeps `1005/1000` as an exact fraction, multiplies and subtracts exactly, and lands precisely on `0.5`.
```
6. Compound Interest Style Multiplication
Standard f64 (IEEE 754): 
1.005 * 100 - 100 = 0.4999999999999858
Bounded Rational Crate (Truncated to 16 digits after decimal point): 
1.005 * 100 - 100 = 0.5000000000000000
```


**Example 7:** Multiplying two already-imprecise doubles compounds their individual rounding errors, pushing the result just above the true value of `0.01`. `BoundedRational` computes the exact product `1/10 × 1/10 = 1/100`, which prints as a precise `0.01`.
```
7. Fractional Multiplication Precision Loss
Standard f64 (IEEE 754): 
0.1 * 0.1 = 0.010000000000000002
Bounded Rational Crate (Truncated to 18 digits after decimal point): 
0.1 * 0.1 = 0.010000000000000000
```


**Example 8:** Both operands exceed 2^53 (~9×10^15), the largest range where f64 can represent every integer exactly, so the literals themselves get silently rounded before the subtraction even happens — giving a wrong answer of `8` instead of the true `7`. `BoundedRational` uses arbitrary-precision big integers, storing and subtracting the exact values correctly.
```
8. Large Integer Precision Loss
Standard f64 (IEEE 754): 
12345678901234567 - 12345678901234560 = 8
Bounded Rational Crate (Truncated to 0 digits after decimal point): 
12345678901234567 - 12345678901234560 = 7
```


**Example 9:** At `1e16`, the gap between adjacent representable doubles is larger than `1`, so adding `1` to `x` doesn't change the stored bit pattern at all ("absorption") — the subsequent subtraction then loses the `+1` entirely, giving `0`. `BoundedRational` has no such magnitude-dependent precision limit, so it tracks the `+1` exactly and correctly returns `1`.
```
9. Floating-Point Precision Loss
Standard f64 (IEEE 754): 
10000000000000000 + 1 - 10000000000000000 = 0
Bounded Rational Crate (Truncated to 0 digits after decimal point): 
10000000000000000 + 1 - 10000000000000000 = 1
```


**Example 10:** `10/3` is a repeating decimal that also cannot terminate in binary, so f64 rounds to the nearest representable double, landing on a value ending in `...335` rather than the true repeating `...333`. `BoundedRational` keeps `10/3` as an exact fraction internally and only rounds when explicitly asked to print a truncated decimal, giving the more mathematically faithful `...333`.
```
10. Floating Point Division Error
Standard f64 (IEEE 754): 
10 / 3 = 3.3333333333333335
Bounded Rational Crate (Truncated to 16 digits after decimal point): 
10 / 3 = 3.3333333333333333
```


**Example 11:** The divisor is a subnormal number near the smallest positive value f64 can represent, so it already carries rounding error before the division even starts; dividing `1.0` by it lands close to, but subtly different from, `f64::MAX`. `BoundedRational` represents the divisor as the exact fraction `556268464626801/10^323` and performs exact division, so its result (shown as both a truncated decimal and an exact fraction) reflects the true mathematical quotient rather than one constrained by f64's exponent/mantissa limits.
```
11. f64 Range Boundary / Near-Overflow Error
Standard f64 (IEEE 754): 
1.0 / 5.56268464626801e-309 = 179769313486231430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
Bounded Rational Crate (Truncated to 16 digits after decimal point): 
1.0 / 5.56268464626801e-309 = 179769313486231379346208425711541347696424817860128104872135702695597119837260862030253533154766535830546706266054827019154133407007538047949214938791468371197416708967143622584877308845925009939203308575654626686640830924736442610099631741027369700203432068985288685586806996460442502833664463821476459633050.0116676479787585
1.0 / 5.56268464626801e-309 = 100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000/556268464626801
f64::MAX = 179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```


**Example 12:** `1.7976931348623157e308` and `1.7976931348623156e308` are two adjacent representable f64 values near the top of the exponent range, where doubles are spaced extremely far apart (a huge ULP), so subtracting two "adjacent" values gives a difference far larger than the true mathematical answer instead of something small. `BoundedRational` computes the exact big-integer subtraction `17976931348623157×10^292 − 17976931348623156×10^292 = 1×10^292`, showing how catastrophic cancellation in f64 can be off by roughly 2x (~1.9958×10^292), not just imprecise.
```
12. Floating Point Catastrophic Cancellation Error
Standard f64 (IEEE 754): 
1.7976931348623157e308 - 1.7976931348623156e308 = 19958403095347200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
Bounded Rational Crate (Truncated to 0 digits after decimal point): 
1.7976931348623157e308 - 1.7976931348623156e308 = 10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```