# boehm_reals
A Rust implementation of exact real number arithmetic, inspired by the Android Calculator and based on Hans-J. Boehm’s research.

---

## Motivation
 
**Floating-point arithmetic (IEEE 754)** is fast, but it also leads to surprising errors:
(10^100 + 1) - 10^100 = 0 
and the answer is wrong!
The correct answer is 1.
These inaccuracies occur because floating-point numbers are **approximations**, not exact values.

**Constructive real arithmetic** solves this by deferring commitment. A real number is represented as a computation that can produce arbitrarily many correct digits *on demand*. You ask for 50 digits, you get 50 correct digits. No rounding surprises. No accumulated error.
 
This library brings that guarantee to Rust.

---

## About the project
 
This project is a **Rust library** that aims to:

- Provide **exact arithmetic** instead of floating-point approximations
- Mimic the design of the Android calculator
- Implement concepts from:
  - Boehm’s *Constructive Reals*
  - PLDI 2020 paper: *Towards an API for the Real Numbers*

Unlike traditional calculators, this system:
- Represents numbers **exactly**
- Computes results **on demand**
- Avoids rounding errors

---

## Architecture and design
 
The library follows the two-layer architecture described in Boehm (2020).
 
```
┌─────────────────────────────────────────────────────┐
│                  ConstructiveReal                   │  ← planned
│  Lazy precision-on-demand representation            │
│  Transcendental functions: π, e, sin, log, sqrt …   │
└────────────────────┬────────────────────────────────┘
                     │ falls back to ↓ when exact
┌────────────────────▼────────────────────────────────┐
│                  BoundedRational                    │  ← active
│  p / q  (BigInt numerator + denominator)            │
│  Returns None when bits(p) + bits(q) > MAX_SIZE     │
└─────────────────────────────────────────────────────┘
```

The two layers work as a team, not a hierarchy.

BoundedRational is the fast, exact layer. Every number is stored as a precise fraction p/q backed by arbitrary-precision integers, so operations like 1/3 + 1/6 = 1/2 are computed with zero rounding error. The catch is cost — fractions can grow very large very fast, so each operation checks whether the combined bit-length of numerator and denominator has crossed MAX_SIZE. If it has, the operation returns None instead of a result, signalling that exact rational arithmetic is no longer practical for this value.

ConstructiveReal is the fallback layer. Rather than storing a number as a single value, it stores a recipe — a function that, given a requested precision n, returns an integer approximation correct to within 2⁻ⁿ. The computation only happens when digits are actually needed, and you can ask for as many as you like. Transcendental values like π, e, sin(x), and log(x) live here because they cannot be represented as finite fractions at all.

The handoff: during a calculation, as long as intermediate results stay within the bit budget, everything runs through BoundedRational — exact and fast. The moment a result exceeds MAX_SIZE, the caller promotes it to a ConstructiveReal and continues. The final answer is then evaluated to however many digits the user actually needs.

---

## Roadmap

See the full project roadmap here:  
[Project Roadmap](./ROADMAP.md)

NOTE: The roadmap is not complete yet. Some phases are defined, and more will be added soon as development continues.

---

## References

1. **Hans-J. Boehm. 2020.** *Towards an API for the real numbers.*
   In *Proceedings of the 41st ACM SIGPLAN Conference on Programming Language Design and Implementation (PLDI 2020).*
   DOI: [10.1145/3385412.3386037](https://doi.org/10.1145/3385412.3386037)
 
2. **Android Open Source Project — ExactCalculator.**
   Original Java implementation by Hans-J. Boehm.
   [android/ExactCalculator](https://android-review.googlesource.com/c/platform/art/+/1012109)


---

## License
 
Licensed under the [MIT License](LICENSE).
 
Copyright © 2026 Madhura-mb.
