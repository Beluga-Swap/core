use belugaswap_math::q64::*;
use soroban_sdk::Env;

// ============================================================
// BASIC ARITHMETIC TESTS
// ============================================================

#[test]
fn test_mul_q64_basic() {
    // 1.0 * 1.0 = 1.0
    let one = ONE_X64;
    assert_eq!(mul_q64(one, one), one);
    
    // 2.0 * 3.0 = 6.0
    let two = one * 2;
    let three = one * 3;
    let six = one * 6;
    assert_eq!(mul_q64(two, three), six);
    
    // 0.5 * 2.0 = 1.0
    let half = one / 2;
    assert_eq!(mul_q64(half, two), one);
}

#[test]
fn test_mul_q64_zero() {
    let one = ONE_X64;
    assert_eq!(mul_q64(0, one), 0);
    assert_eq!(mul_q64(one, 0), 0);
    assert_eq!(mul_q64(0, 0), 0);
}

#[test]
fn test_mul_q64_identity() {
    let values = vec![
        ONE_X64,
        ONE_X64 / 2,
        ONE_X64 * 2,
        ONE_X64 * 100,
        ONE_X64 / 100,
    ];
    
    for val in values {
        assert_eq!(mul_q64(val, ONE_X64), val, "a * 1.0 should equal a");
    }
}

#[test]
fn test_div_q64_basic() {
    // div_q64 expects RAW values and outputs Q64 result
    // div_q64(a, b) = (a << 64) / b
    
    // 1 / 1 = 1.0 in Q64
    assert_eq!(div_q64(1, 1), ONE_X64);
    
    // 2 / 1 = 2.0 in Q64
    assert_eq!(div_q64(2, 1), ONE_X64 * 2);
    
    // 6 / 2 = 3.0 in Q64
    assert_eq!(div_q64(6, 2), ONE_X64 * 3);
    
    // 1 / 2 = 0.5 in Q64
    assert_eq!(div_q64(1, 2), ONE_X64 / 2);
    
    // 10 / 5 = 2.0 in Q64
    assert_eq!(div_q64(10, 5), ONE_X64 * 2);
}

#[test]
fn test_div_q64_zero_denominator() {
    let result = div_q64(ONE_X64, 0);
    assert_eq!(result, u128::MAX, "Division by zero should return MAX");
}

#[test]
fn test_div_round_up() {
    // 10 / 3 = 3 remainder 1, should round up to 4
    assert_eq!(div_round_up(10, 3), 4);
    
    // 10 / 5 = 2 remainder 0, should stay 2
    assert_eq!(div_round_up(10, 5), 2);
    
    // 1 / 2 = 0 remainder 1, should round up to 1
    assert_eq!(div_round_up(1, 2), 1);
    
    // 0 / anything = 0
    assert_eq!(div_round_up(0, 100), 0);
}

#[test]
fn test_div_round_up_zero_denominator() {
    assert_eq!(div_round_up(100, 0), 0);
}

// ============================================================
// TYPE CONVERSION TESTS
// ============================================================

#[test]
fn test_i128_to_u128_safe() {
    assert_eq!(i128_to_u128_safe(100), 100);
    assert_eq!(i128_to_u128_safe(0), 0);
    assert_eq!(i128_to_u128_safe(-100), 0);
    assert_eq!(i128_to_u128_safe(i128::MAX), i128::MAX as u128);
    assert_eq!(i128_to_u128_safe(i128::MIN), 0);
}

#[test]
fn test_u128_to_i128_saturating() {
    assert_eq!(u128_to_i128_saturating(100), 100);
    assert_eq!(u128_to_i128_saturating(0), 0);
    assert_eq!(u128_to_i128_saturating(i128::MAX as u128), i128::MAX);
    assert_eq!(u128_to_i128_saturating(u128::MAX), i128::MAX);
}

// ============================================================
// MUL_DIV TESTS
// ============================================================

#[test]
fn test_mul_div_basic() {
    let env = Env::default();
    
    // (10 * 5) / 2 = 25
    assert_eq!(mul_div(&env, 10, 5, 2), 25);
    
    // (100 * 100) / 100 = 100
    assert_eq!(mul_div(&env, 100, 100, 100), 100);
    
    // (1000 * 2000) / 1000 = 2000
    assert_eq!(mul_div(&env, 1000, 2000, 1000), 2000);
}

#[test]
#[should_panic(expected = "divide by zero")]
fn test_mul_div_zero_denominator() {
    let env = Env::default();
    mul_div(&env, 100, 200, 0);
}

#[test]
fn test_mul_div_large_numbers() {
    let env = Env::default();
    
    // Test with large numbers that would overflow normal multiplication
    let large = 1u128 << 100;
    let result = mul_div(&env, large, large, large);
    assert_eq!(result, large);
}

#[test]
fn test_mul_div_overflow_prevention() {
    let env = Env::default();
    
    // These would overflow in regular u128 multiplication
    let a = u128::MAX / 2;
    let b = u128::MAX / 2;
    let denominator = u128::MAX / 4;
    
    // Should not panic and should return a reasonable result
    let result = mul_div(&env, a, b, denominator);
    assert!(result > 0);
}

// ============================================================
// PRECISION TESTS
// ============================================================

#[test]
fn test_mul_q64_precision() {
    let one = ONE_X64;
    
    // Test small fractions
    let tenth = one / 10;
    let hundredth = one / 100;
    
    // 0.1 * 0.1 ≈ 0.01
    let result = mul_q64(tenth, tenth);
    let expected = hundredth;
    let tolerance = hundredth / 100; // 0.01% tolerance
    
    assert!(
        result >= expected.saturating_sub(tolerance) && result <= expected + tolerance,
        "0.1 * 0.1 should be approximately 0.01"
    );
}

#[test]
fn test_div_q64_precision() {
    // div_q64 expects raw values
    
    // 1 / 3 ≈ 0.333... in Q64
    let result = div_q64(1, 3);
    let expected = ONE_X64 / 3;
    
    // Should be close
    let tolerance = expected / 100; // 1% tolerance
    assert!(
        result >= expected.saturating_sub(tolerance) && result <= expected + tolerance,
        "Result: {}, Expected: {}", result, expected
    );
    
    // 10 / 3 ≈ 3.333... in Q64
    let result = div_q64(10, 3);
    let expected = (ONE_X64 * 10) / 3;
    let tolerance = expected / 100;
    assert!(
        result >= expected.saturating_sub(tolerance) && result <= expected + tolerance,
        "Result: {}, Expected: {}", result, expected
    );
}

// ============================================================
// EDGE CASE TESTS
// ============================================================

#[test]
fn test_mul_q64_max_values() {
    let max_safe = u128::MAX / 2;
    
    // Should not panic with large values
    let result = mul_q64(max_safe, 1);
    assert_eq!(result, max_safe >> 64);
}

#[test]
fn test_div_q64_max_values() {
    let large = u128::MAX / 2;
    
    // Should handle large numerators
    let result = div_q64(large, ONE_X64);
    assert!(result > 0);
}

#[test]
fn test_mul_q64_commutative() {
    let values = vec![
        (ONE_X64, ONE_X64 * 2),
        (ONE_X64 / 2, ONE_X64 * 3),
        (ONE_X64 * 5, ONE_X64 / 5),
    ];
    
    for (a, b) in values {
        assert_eq!(
            mul_q64(a, b),
            mul_q64(b, a),
            "Multiplication should be commutative"
        );
    }
}

#[test]
fn test_mul_q64_associative() {
    let a = ONE_X64 * 2;
    let b = ONE_X64 * 3;
    let c = ONE_X64 / 2;
    
    let left = mul_q64(mul_q64(a, b), c);
    let right = mul_q64(a, mul_q64(b, c));
    
    // Allow small tolerance due to accumulated rounding
    let tolerance = 1000;
    assert!(
        left >= right.saturating_sub(tolerance) && left <= right + tolerance,
        "Multiplication should be approximately associative"
    );
}

// ============================================================
// PROPERTY TESTS
// ============================================================

#[test]
fn test_mul_div_identity() {
    let env = Env::default();
    
    let values = vec![1, 100, 1000, 10000, ONE_X64];
    
    for val in values {
        // (a * b) / b = a
        let b = 123456;
        let result = mul_div(&env, val, b, b);
        assert_eq!(result, val, "(a * b) / b should equal a");
    }
}

#[test]
fn test_div_mul_roundtrip() {
    // Test: (a / b) * b ≈ a (in Q64 space)
    
    // Start with Q64 value
    let a_q64 = ONE_X64;
    let b_raw = 2;
    
    // Divide Q64 by raw value (this requires converting back first)
    // Actually div_q64 expects raw inputs, so we need to rethink this test
    
    // Better approach: test that mul_q64 and div_q64 are inverses
    // Start with raw value
    let raw = 100u128;
    
    // Convert to Q64 using div_q64(raw * ONE_X64, 1) - but that's not how it works
    // div_q64 takes raw values and produces Q64 output
    
    // Actually let's test: (a * 2^64) / b * b ≈ a * 2^64
    let a = 100u128;
    let b = 7u128;
    
    let q64_result = div_q64(a, b); // produces Q64 format
    // To get back, we need to multiply by b and shift right 64
    let back = (q64_result * b) >> 64;
    
    // Should be close to original
    assert!(
        back >= a.saturating_sub(1) && back <= a + 1,
        "Roundtrip: {}, Original: {}", back, a
    );
}

// ============================================================
// MONOTONICITY TESTS
// ============================================================

#[test]
fn test_mul_q64_monotonic() {
    let base = ONE_X64 * 2;
    let multipliers = vec![
        ONE_X64 / 4,
        ONE_X64 / 2,
        ONE_X64,
        ONE_X64 * 2,
        ONE_X64 * 4,
    ];
    
    let mut prev_result = 0;
    for multiplier in multipliers {
        let result = mul_q64(base, multiplier);
        assert!(
            result >= prev_result,
            "Multiplication should be monotonically increasing"
        );
        prev_result = result;
    }
}

#[test]
fn test_div_q64_monotonic() {
    let numerator = ONE_X64 * 100;
    let denominators = vec![
        ONE_X64 * 4,
        ONE_X64 * 2,
        ONE_X64,
        ONE_X64 / 2,
        ONE_X64 / 4,
    ];
    
    let mut prev_result = 0;
    for denominator in denominators {
        if denominator == 0 { continue; }
        let result = div_q64(numerator, denominator);
        assert!(
            result >= prev_result,
            "Division should be monotonically increasing as denominator decreases"
        );
        prev_result = result;
    }
}