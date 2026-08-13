#![no_main]

use boehm_reals::evaluation::bounded_rational::BoundedRational;
use libfuzzer_sys::fuzz_target;

fn i64_at(data: &[u8], offset: usize) -> i64 {
    let mut bytes = [0_u8; 8];
    let available = data.len().saturating_sub(offset).min(8);
    if available > 0 {
        bytes[..available].copy_from_slice(&data[offset..offset + available]);
    }
    i64::from_le_bytes(bytes)
}

fuzz_target!(|data: &[u8]| {
    let left_numerator = i64_at(data, 0);
    let left_denominator = i64_at(data, 8);
    let right_numerator = i64_at(data, 16);
    let right_denominator = i64_at(data, 24);
    let float_bits = u64::from_le_bytes(i64_at(data, 32).to_le_bytes());

    let left = BoundedRational::from_longs(left_numerator, left_denominator).ok();
    let right = BoundedRational::from_longs(right_numerator, right_denominator).ok();

    if let Some(ref value) = left {
        let _ = value.positive_den();
        let _ = value.reduce();
        let _ = value.signum();
    }

    if let (Some(left), Some(right)) = (&left, &right) {
        let _ = left.compare_to(right);
    }

    let _ = BoundedRational::add(left.clone(), right.clone());
    let _ = BoundedRational::subtract(left.clone(), right.clone());
    let _ = BoundedRational::multiply(left.clone(), right.clone());
    let _ = BoundedRational::divide(left, right);
    let _ = BoundedRational::value_of_double(f64::from_bits(float_bits));
});
