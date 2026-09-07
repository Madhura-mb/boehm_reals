#![cfg(test)]
use crate::evaluation::bounded_rational::BoundedRational;

// Helper Functions
pub(crate) fn br(n: i64, d: i64) -> BoundedRational {
    BoundedRational::from_longs(n, d).unwrap()
}

pub(crate) fn assert_value(r: &BoundedRational, num: i64, den: i64) {
    let reduced = r.reduce().positive_den();
    let expected = BoundedRational::from_longs(num, den)
        .unwrap()
        .reduce()
        .positive_den();
    assert_eq!(*reduced.numerator(), *expected.numerator());
    assert_eq!(*reduced.denominator(), *expected.denominator());
}
