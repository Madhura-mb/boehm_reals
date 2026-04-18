# Roadmap

Development is tracked through [GitHub Issues](https://github.com/Madhura-mb/boehm_reals/issues) and organised into milestones.

> Note: This roadmap is evolving. Some phases are defined, and more refinements will be added as development progresses.

---

## M1 — Core Foundation  
🔗 [View Milestone](https://github.com/Madhura-mb/boehm_reals/milestone/1)  
📅 **Target: April 2026** · ✅ **~75% Complete**

Struct definition, constants, constructors, and foundational helpers.

- [x] `BoundedRational` struct and `MAX_SIZE` constant  
- [x] Lazy static constants (`ZERO`, `ONE`, `TWO`, `TEN`, `FIVE`, …)  
- [x] Constructors: `new`, `from_bigint`, `from_long`, `from_longs`  
- [x] `too_big()` and `positive_den()` — [issue #4](https://github.com/Madhura-mb/boehm_reals/issues/4)  
- [ ] `reduce()` and `maybe_reduce()` — [issue #5](https://github.com/Madhura-mb/boehm_reals/issues/5)  

---

## M2 — Arithmetic Core  
🔗 [View Milestone](https://github.com/Madhura-mb/boehm_reals/milestone/2)  
📅 **Target: April 2026**

The operations that make `BoundedRational` useful as an exact number type.

- [ ] `value_of_long()` and `value_of_double()` — [issue #6](https://github.com/Madhura-mb/boehm_reals/issues/6)  
- [ ] `negate()`, `add()`, `subtract()` — [issue #7](https://github.com/Madhura-mb/boehm_reals/issues/7)  
- [ ] `raw_multiply()`, `multiply()`, `inverse()`, `divide()` — [issue #8](https://github.com/Madhura-mb/boehm_reals/issues/8)  
- [ ] `ZeroDivisionError` type — [issue #9](https://github.com/Madhura-mb/boehm_reals/issues/9)  

---

## M3 — Complete & Usable  
🔗 [View Milestone](https://github.com/Madhura-mb/boehm_reals/milestone/3)  
📅 **Target: April 2026**

Comparisons, conversions, utilities, and full integration support.

- [ ] Comparisons and traits — [issue #10](https://github.com/Madhura-mb/boehm_reals/issues/10)  
- [ ] Output conversions — [issue #11](https://github.com/Madhura-mb/boehm_reals/issues/11)  
- [ ] Utility functions — [issue #12](https://github.com/Madhura-mb/boehm_reals/issues/12)  
- [ ] Integration test suite — [issue #13](https://github.com/Madhura-mb/boehm_reals/issues/13)  

---

## Beyond `BoundedRational`

Once the rational layer is complete and well-tested:

- [ ] Implement `ConstructiveReal`
- [ ] Lazy evaluation of real numbers  
- [ ] Arbitrary precision computation  
- [ ] Support for irrational/transcendental numbers:
  - π, e, √2, sin, log, …

📅 **Target: April 2026 and beyond**

Based on concepts described in **Boehm (2020)**.

---

## Long-Term Vision

- Publish as a Rust crate  
- Benchmark vs floating-point arithmetic  
- Integrate with scientific computing tools  
- Explore WASM/browser-based calculator  

---

## Milestone Strategy

- **M1 → Foundation** (data structures + helpers)  
- **M2 → Computation** (core arithmetic)  
- **M3 → Usability** (interfaces + testing)  
- **Next → Research-level features**

---
