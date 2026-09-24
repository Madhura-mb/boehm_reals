use boehm_reals::evaluation::bounded_rational::BoundedRational;

fn main() {
    println!("--- IEEE 754 f64 vs Boehm's Real Bounded Rational Precision Demo ---");
    
    // 1. Floating-point Rounding Error
    println!("\n1. Floating-point Rounding Error");
    let x = 0.1_f64;
    let y = 0.2_f64;
    let z = x + y;
    println!("Standard f64 (IEEE 754): ");
    println!("0.1 + 0.2 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_longs(1, 10).unwrap();
    let y = BoundedRational::from_longs(2, 10).unwrap();
    let z = x + y;
    println!("0.1 + 0.2 = {}", z.to_string_truncated(17)); 

    // 2. Cumulative Fraction
    println!("\n2. Cumulative Fraction");
    let x = 0.1_f64;
    let z = x + x + x + x + x + x + x + x + x + x;
    let a = x * 10_f64;
    println!("Standard f64 (IEEE 754): ");
    println!("0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 = {}", z);
    println!("0.1 * 10 = {}", a);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_longs(1, 10).unwrap();
    let z = &x + &x + &x + &x + &x + &x + &x + &x + &x + &x;
    let a = &x * BoundedRational::from_long(10);
    println!("0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 = {}", z.to_string_truncated(16));
    println!("0.1 * 10 = {}", a.to_string_truncated(1));
    
    // 3. Micro Subtraction Gap
    println!("\n3. Micro Subtraction Gap");
    let x = 0.3_f64;
    let y = 0.2_f64;
    let w = 0.1_f64;
    let z = x - y - w;
    println!("Standard f64 (IEEE 754): ");
    println!("0.3 - 0.2 - 0.1 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_longs(3, 10).unwrap();
    let y = BoundedRational::from_longs(2, 10).unwrap();
    let w = BoundedRational::from_longs(1, 10).unwrap();
    let z = &x - &y - &w;
    println!("0.3 - 0.2 - 0.1 = {}", z.to_string_truncated(33));
    
    // 4. Multiplicative Expansion Error
    println!("\n4. Multiplicative Expansion Error");
    let x = 0.15_f64;
    let y = 3_f64;
    let z = x * y;
    println!("Standard f64 (IEEE 754): ");
    println!("0.15 * 3 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_longs(15, 100).unwrap();
    let y = BoundedRational::from_long(3);
    let z = &x * &y;
    println!("0.15 * 3 = {}", z.to_string_truncated(17));
    
    // 5. Non-Integer Accumulation
    println!("\n5. Non-Integer Accumulation");
    let x = 0.7_f64;
    let y = 0.1_f64;
    let z = x + y;
    println!("Standard f64 (IEEE 754): ");
    println!("0.7 + 0.1 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_longs(7, 10).unwrap();
    let y = BoundedRational::from_longs(1, 10).unwrap();
    let z = &x + &y;
    println!("0.7 + 0.1 = {}", z.to_string_truncated(16));
    
    // 6. Compound Interest Style Multiplication
    println!("\n6. Compound Interest Style Multiplication");
    let x = 1.005_f64;
    let y = 100_f64;
    let z = x * y - y;
    println!("Standard f64 (IEEE 754): ");
    println!("1.005 * 100 - 100 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_longs(1005, 1000).unwrap();
    let y = BoundedRational::from_long(100);
    let z = &x * &y - &y;
    println!("1.005 * 100 - 100 = {}", z.to_string_truncated(16));
    
    // 7. Distributive Discrepancy
    println!("\n7. Distributive Discrepancy");
    let x = 0.1_f64;
    let y = 0.1_f64;
    let z = x * y;
    println!("Standard f64 (IEEE 754): ");
    println!("0.1 * 0.1 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_longs(1, 10).unwrap();
    let y = BoundedRational::from_longs(1, 10).unwrap();
    let z = &x * &y;
    println!("0.1 * 0.1 = {}", z.to_string_truncated(18));
    
    // 8. Large Number Integer Truncation
    println!("\n8. Large Number Integer Truncation");
    let x = 12345678901234567_f64;
    let y = 12345678901234560_f64;
    let z = x - y;
    println!("Standard f64 (IEEE 754): ");
    println!("12345678901234567 - 12345678901234560 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_long(12345678901234567);
    let y = BoundedRational::from_long(12345678901234560);
    let z = &x - &y;
    println!("12345678901234567 - 12345678901234560 = {}", z.to_string_truncated(1));
    
    // 9. Floating-Point Precision Loss
    println!("\n9. Floating-Point Precision Loss");
    let x = 1e16_f64;
    let y = 1_f64;
    let z = x + y - x;
    println!("Standard f64 (IEEE 754): ");
    println!("10000000000000000 + 1 - 10000000000000000 = {}", z);
    println!("Bounded Rational Crate: ");
    let x = BoundedRational::from_long(10_000_000_000_000_000);
    let y = BoundedRational::from_long(1);
    let z = &x + &y - &x;
    println!("10000000000000000 + 1 - 10000000000000000 = {}", z.to_string_truncated(1));
}