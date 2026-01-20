use belugaswap_math::liquidity::*;
use belugaswap_math::q64::ONE_X64;
use soroban_sdk::Env;

// ============================================================
// AMOUNT DELTA TESTS
// ============================================================

#[test]
fn test_get_amount_0_delta_basic() {
    let sqrt_price_lower = ONE_X64 / 2; // 0.5
    let sqrt_price_upper = ONE_X64;     // 1.0
    let liquidity = 1_000_000u128;
    
    let amount = get_amount_0_delta(sqrt_price_lower, sqrt_price_upper, liquidity, false);
    assert!(amount > 0, "Should calculate positive amount0");
}

#[test]
fn test_get_amount_0_delta_reversed_prices() {
    let sqrt_price_a = ONE_X64;
    let sqrt_price_b = ONE_X64 / 2;
    let liquidity = 1_000_000u128;
    
    // Should handle reversed prices (internally sorts them)
    let amount1 = get_amount_0_delta(sqrt_price_a, sqrt_price_b, liquidity, false);
    let amount2 = get_amount_0_delta(sqrt_price_b, sqrt_price_a, liquidity, false);
    
    assert_eq!(amount1, amount2, "Should give same result regardless of price order");
}

#[test]
fn test_get_amount_0_delta_zero_liquidity() {
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let amount = get_amount_0_delta(sqrt_price_lower, sqrt_price_upper, 0, false);
    assert_eq!(amount, 0, "Zero liquidity should give zero amount");
}

#[test]
fn test_get_amount_0_delta_equal_prices() {
    let sqrt_price = ONE_X64;
    let liquidity = 1_000_000u128;
    
    let amount = get_amount_0_delta(sqrt_price, sqrt_price, liquidity, false);
    assert_eq!(amount, 0, "Equal prices should give zero amount");
}

#[test]
fn test_get_amount_0_delta_round_up() {
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let liquidity = 1_000_001u128; // Odd number to test rounding
    
    let amount_down = get_amount_0_delta(sqrt_price_lower, sqrt_price_upper, liquidity, false);
    let amount_up = get_amount_0_delta(sqrt_price_lower, sqrt_price_upper, liquidity, true);
    
    assert!(amount_up >= amount_down, "Rounding up should give >= amount");
}

#[test]
fn test_get_amount_1_delta_basic() {
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let liquidity = 1_000_000u128;
    
    let amount = get_amount_1_delta(sqrt_price_lower, sqrt_price_upper, liquidity, false);
    assert!(amount > 0, "Should calculate positive amount1");
}

#[test]
fn test_get_amount_1_delta_zero_liquidity() {
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let amount = get_amount_1_delta(sqrt_price_lower, sqrt_price_upper, 0, false);
    assert_eq!(amount, 0, "Zero liquidity should give zero amount");
}

#[test]
fn test_get_amount_1_delta_round_up() {
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let liquidity = 1_000_001u128;
    
    let amount_down = get_amount_1_delta(sqrt_price_lower, sqrt_price_upper, liquidity, false);
    let amount_up = get_amount_1_delta(sqrt_price_lower, sqrt_price_upper, liquidity, true);
    
    assert!(amount_up >= amount_down, "Rounding up should give >= amount");
}

// ============================================================
// LIQUIDITY FROM AMOUNT TESTS
// ============================================================

#[test]
fn test_get_liquidity_for_amount0_basic() {
    let env = Env::default();
    let amount0 = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let liquidity = get_liquidity_for_amount0(&env, amount0, sqrt_price_lower, sqrt_price_upper);
    assert!(liquidity > 0, "Should calculate positive liquidity");
}

#[test]
fn test_get_liquidity_for_amount0_zero_amount() {
    let env = Env::default();
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let liquidity = get_liquidity_for_amount0(&env, 0, sqrt_price_lower, sqrt_price_upper);
    assert_eq!(liquidity, 0, "Zero amount should give zero liquidity");
}

#[test]
fn test_get_liquidity_for_amount0_negative_amount() {
    let env = Env::default();
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let liquidity = get_liquidity_for_amount0(&env, -1000, sqrt_price_lower, sqrt_price_upper);
    assert_eq!(liquidity, 0, "Negative amount should give zero liquidity");
}

#[test]
fn test_get_liquidity_for_amount0_invalid_prices() {
    let env = Env::default();
    let amount0 = 1_000_000i128;
    
    // Lower >= Upper (invalid)
    let liquidity = get_liquidity_for_amount0(&env, amount0, ONE_X64, ONE_X64 / 2);
    assert_eq!(liquidity, 0, "Invalid price range should give zero liquidity");
}

#[test]
fn test_get_liquidity_for_amount1_basic() {
    let env = Env::default();
    let amount1 = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let liquidity = get_liquidity_for_amount1(&env, amount1, sqrt_price_lower, sqrt_price_upper);
    assert!(liquidity > 0, "Should calculate positive liquidity");
}

#[test]
fn test_get_liquidity_for_amount1_proportional() {
    let env = Env::default();
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let small_amount = 1_000i128;
    let large_amount = 10_000i128;
    
    let small_liq = get_liquidity_for_amount1(&env, small_amount, sqrt_price_lower, sqrt_price_upper);
    let large_liq = get_liquidity_for_amount1(&env, large_amount, sqrt_price_lower, sqrt_price_upper);
    
    assert!(large_liq > small_liq, "Larger amount should give more liquidity");
}

// ============================================================
// LIQUIDITY FROM AMOUNTS (BOTH) TESTS
// ============================================================

#[test]
fn test_get_liquidity_for_amounts_price_below_range() {
    let env = Env::default();
    let amount0 = 1_000_000i128;
    let amount1 = 1_000_000i128;
    let sqrt_price_lower = ONE_X64;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_price = ONE_X64 / 2; // Below range
    
    let liquidity = get_liquidity_for_amounts(
        &env, amount0, amount1, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    // Should only use amount0 when price is below range
    let liq_from_0 = get_liquidity_for_amount0(&env, amount0, sqrt_price_lower, sqrt_price_upper);
    assert_eq!(liquidity, liq_from_0, "Should use only amount0 when price below range");
}

#[test]
fn test_get_liquidity_for_amounts_price_above_range() {
    let env = Env::default();
    let amount0 = 1_000_000i128;
    let amount1 = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let current_price = ONE_X64 * 2; // Above range
    
    let liquidity = get_liquidity_for_amounts(
        &env, amount0, amount1, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    // Should only use amount1 when price is above range
    let liq_from_1 = get_liquidity_for_amount1(&env, amount1, sqrt_price_lower, sqrt_price_upper);
    assert_eq!(liquidity, liq_from_1, "Should use only amount1 when price above range");
}

#[test]
fn test_get_liquidity_for_amounts_price_in_range() {
    let env = Env::default();
    let amount0 = 1_000_000i128;
    let amount1 = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_price = ONE_X64; // In range
    
    let liquidity = get_liquidity_for_amounts(
        &env, amount0, amount1, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    // Should use minimum of both when price is in range
    assert!(liquidity > 0, "Should calculate positive liquidity when in range");
}

#[test]
fn test_get_liquidity_for_amounts_invalid_range() {
    let env = Env::default();
    let amount0 = 1_000_000i128;
    let amount1 = 1_000_000i128;
    
    // Invalid range (lower >= upper)
    let liquidity = get_liquidity_for_amounts(
        &env, amount0, amount1, ONE_X64, ONE_X64 / 2, ONE_X64
    );
    
    assert_eq!(liquidity, 0, "Invalid range should give zero liquidity");
}

// ============================================================
// AMOUNTS FROM LIQUIDITY TESTS
// ============================================================

#[test]
fn test_get_amounts_for_liquidity_basic() {
    let env = Env::default();
    let liquidity = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_price = ONE_X64;
    
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, liquidity, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert!(amount0 > 0, "Should calculate positive amount0");
    assert!(amount1 > 0, "Should calculate positive amount1");
}

#[test]
fn test_get_amounts_for_liquidity_zero() {
    let env = Env::default();
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_price = ONE_X64;
    
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, 0, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert_eq!(amount0, 0, "Zero liquidity should give zero amount0");
    assert_eq!(amount1, 0, "Zero liquidity should give zero amount1");
}

#[test]
fn test_get_amounts_for_liquidity_price_below() {
    let env = Env::default();
    let liquidity = 1_000_000i128;
    let sqrt_price_lower = ONE_X64;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_price = ONE_X64 / 2; // Below range
    
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, liquidity, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert!(amount0 > 0, "Should have amount0 when price below range");
    assert_eq!(amount1, 0, "Should have zero amount1 when price below range");
}

#[test]
fn test_get_amounts_for_liquidity_price_above() {
    let env = Env::default();
    let liquidity = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let current_price = ONE_X64 * 2; // Above range
    
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, liquidity, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert_eq!(amount0, 0, "Should have zero amount0 when price above range");
    assert!(amount1 > 0, "Should have amount1 when price above range");
}

#[test]
fn test_get_amounts_for_liquidity_proportional() {
    let env = Env::default();
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_price = ONE_X64;
    
    let small_liq = 1_000i128;
    let large_liq = 10_000i128;
    
    let (small_a0, small_a1) = get_amounts_for_liquidity(
        &env, small_liq, sqrt_price_lower, sqrt_price_upper, current_price
    );
    let (large_a0, large_a1) = get_amounts_for_liquidity(
        &env, large_liq, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert!(large_a0 > small_a0, "Larger liquidity should give more amount0");
    assert!(large_a1 > small_a1, "Larger liquidity should give more amount1");
}

// ============================================================
// ROUNDTRIP TESTS (INVARIANTS)
// ============================================================

#[test]
fn test_liquidity_amount_roundtrip() {
    let env = Env::default();
    let original_amount0 = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let current_price = ONE_X64 / 2; // At lower bound, only uses amount0
    
    // amount0 -> liquidity
    let liquidity = get_liquidity_for_amount0(&env, original_amount0, sqrt_price_lower, sqrt_price_upper);
    
    // liquidity -> amount0
    let (recovered_amount0, _) = get_amounts_for_liquidity(
        &env, liquidity, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    // Should be approximately equal (allowing for rounding)
    let tolerance = original_amount0 / 1000; // 0.1% tolerance
    assert!(
        recovered_amount0 >= original_amount0 - tolerance &&
        recovered_amount0 <= original_amount0 + tolerance,
        "Roundtrip should preserve amount approximately"
    );
}

#[test]
fn test_amount_symmetry() {
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let liquidity = 1_000_000u128;
    
    // Amount0 delta for price range
    let amount0 = get_amount_0_delta(sqrt_price_lower, sqrt_price_upper, liquidity, false);
    
    // Amount1 delta for same range
    let amount1 = get_amount_1_delta(sqrt_price_lower, sqrt_price_upper, liquidity, false);
    
    // Both should be positive
    assert!(amount0 > 0, "Amount0 should be positive");
    assert!(amount1 > 0, "Amount1 should be positive");
}

// ============================================================
// MONOTONICITY TESTS
// ============================================================

#[test]
fn test_amount_increases_with_liquidity() {
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    
    let liquidities = vec![1000u128, 10000, 100000, 1000000];
    let mut prev_amount0 = 0;
    let mut prev_amount1 = 0;
    
    for liq in liquidities {
        let amount0 = get_amount_0_delta(sqrt_price_lower, sqrt_price_upper, liq, false);
        let amount1 = get_amount_1_delta(sqrt_price_lower, sqrt_price_upper, liq, false);
        
        assert!(amount0 >= prev_amount0, "Amount0 should increase with liquidity");
        assert!(amount1 >= prev_amount1, "Amount1 should increase with liquidity");
        
        prev_amount0 = amount0;
        prev_amount1 = amount1;
    }
}

#[test]
fn test_amount_increases_with_price_range() {
    let liquidity = 1_000_000u128;
    let base_price = ONE_X64;
    
    // Increasing price ranges
    let ranges = vec![
        (base_price, base_price + base_price / 10),  // 10% range
        (base_price, base_price + base_price / 5),   // 20% range
        (base_price, base_price + base_price / 2),   // 50% range
    ];
    
    let mut prev_amount = 0;
    
    for (lower, upper) in ranges {
        let amount = get_amount_1_delta(lower, upper, liquidity, false);
        
        assert!(
            amount >= prev_amount,
            "Amount should increase with wider price range"
        );
        prev_amount = amount;
    }
}

// ============================================================
// EDGE CASE TESTS
// ============================================================

#[test]
fn test_very_large_liquidity() {
    let env = Env::default();
    let large_liq = i128::MAX / 2;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64;
    let current_price = ONE_X64 / 2;
    
    // Should not panic with large liquidity
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, large_liq, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert!(amount0 >= 0, "Should handle large liquidity");
    assert!(amount1 >= 0, "Should handle large liquidity");
}

#[test]
fn test_very_narrow_price_range() {
    let env = Env::default();
    let liquidity = 1_000_000i128;
    let base_price = ONE_X64;
    
    // Very narrow range (0.01% difference)
    let sqrt_price_lower = base_price;
    let sqrt_price_upper = base_price + base_price / 10000;
    let current_price = base_price;
    
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, liquidity, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    // Should still calculate some amounts
    assert!(amount0 >= 0 || amount1 >= 0, "Should handle narrow ranges");
}

#[test]
fn test_wide_price_range() {
    let env = Env::default();
    let liquidity = 1_000_000i128;
    
    // Very wide range (100x price difference)
    let sqrt_price_lower = ONE_X64 / 10;
    let sqrt_price_upper = ONE_X64 * 10;
    let current_price = ONE_X64;
    
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, liquidity, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert!(amount0 > 0, "Should handle wide ranges");
    assert!(amount1 > 0, "Should handle wide ranges");
}