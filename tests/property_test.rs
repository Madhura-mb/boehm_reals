use boehm_reals::evaluation::bounded_rational::BoundedRational;
use num_bigint::BigInt;
use num_integer::Integer;
use proptest::prelude::*;
use std::ops::Add;

fn rational(numerator: i64, denominator: i64) -> BoundedRational {
    BoundedRational::from_longs(numerator, denominator).expect("generated denominator is non-zero")
}

fn arb_rational() -> impl Strategy<Value = BoundedRational> {
    (
        any::<i64>(),
        any::<i64>().prop_filter("denominator must be non-zero", |d| *d != 0),
    )
        .prop_map(|(n, d)| rational(n, d))
}

proptest! {
    #[test]
    fn reduction_preserves_value_and_is_coprime(r in arb_rational()) {
        let reduced = r.reduce();

        prop_assert_eq!(r, reduced.clone());
        prop_assert_eq!(reduced.numerator().gcd(reduced.denominator()), BigInt::from(1));
    }

    #[test]
    fn denominator_is_positive(r in arb_rational()) {
        prop_assert!(r.denominator() > &BigInt::from(0));
    }

    #[test]
    fn zero_is_stored_as_zero_over_one(d in any::<i64>().prop_filter("non-zero", |d| *d != 0)) {
        let r = rational(0, d);
        prop_assert_eq!(r.numerator(), &BigInt::from(0));
        prop_assert_eq!(r.denominator(), &BigInt::from(1));
    }

    #[test]
    fn add_then_subtract_round_trips(left in arb_rational(), right in arb_rational()) {
        let sum = BoundedRational::add(left.clone(), right.clone());
        let round_trip = sum - right;

        prop_assert_eq!(round_trip, left);
    }

    #[test]
    fn compare_to_agrees_with_cross_multiplication(left in arb_rational(), right in arb_rational()) {
        let lhs = left.numerator() * right.denominator();
        let rhs = right.numerator() * left.denominator();

        prop_assert_eq!(left.compare_to(&right), lhs.cmp(&rhs));
    }

    #[test]
    fn finite_f64_bits_convert_without_panicking(bits in any::<u64>()) {
        let value = f64::from_bits(bits);
        let converted = BoundedRational::value_of_double(value);

        prop_assert_eq!(converted.is_ok(), value.is_finite());
        if let Ok(rational) = converted {
            prop_assert!(rational.denominator() > &BigInt::from(0));
        }
    }

    #[test]
    fn add_keeps_denominator_positive(a in arb_rational(), b in arb_rational()) {
        prop_assert!((&a).add(&b).denominator() > &BigInt::from(0));
    }

    #[test]
    fn mul_keeps_denominator_positive(a in arb_rational(), b in arb_rational()) {
        prop_assert!((&a * &b).denominator() > &BigInt::from(0));
    }

    #[test]
    fn sub_keeps_denominator_positive(a in arb_rational(), b in arb_rational()) {
        prop_assert!((&a - &b).denominator() > &BigInt::from(0));
    }

    #[test]
    fn div_keeps_denominator_positive(
        a in arb_rational(),
        b in arb_rational().prop_filter("divisor must be non-zero", |b| b.numerator() != &BigInt::from(0)),
    ) {
        prop_assert!((&a / &b).denominator() > &BigInt::from(0));
    }

    #[test]
    fn neg_keeps_denominator_positive(a in arb_rational()) {
        prop_assert!((-&a).denominator() > &BigInt::from(0));
    }
}
