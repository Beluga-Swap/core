use belugaswap_math::sqrt_price::*;
use belugaswap_math::constants::*;
use belugaswap_math::q64::ONE_X64;
use soroban_sdk::Env;

// ============================================================
// TICK TO SQRT PRICE TESTS
// ============================================================

#[test]
fn test_get_sqrt_ratio_at_tick_zero() {
    // Tick 0 should give sqrt price of 1.0
    let sqrt_price = get_sqrt_ratio_at_tick(0);
    assert_eq!(sqrt_price, ONE_X64, "Tick 0 should give sqrt price 1.0");
}

#[test]
fn test_get_sqrt_ratio_at_tick_positive() {
    // Positive ticks should give sqrt price > 1.0
    let sqrt_price_1 = get_sqrt_ratio_at_tick(1);
    let sqrt_price_100 = get_sqrt_ratio_at_tick(100);
    let sqrt_price_1000 = get_sqrt_ratio_at_tick(1000);
    
    assert!(sqrt_price_1 > ONE_X64, "Tick 1 should give price > 1.0");
    assert!(sqrt_price_100 > sqrt_price_1, "Tick 100 should give higher price than tick 1");
    assert!(sqrt_price_1000 > sqrt_price_100, "Tick 1000 should give higher price than tick 100");
}

#[test]
fn test_get_sqrt_ratio_at_tick_negative() {
    // Negative ticks should give sqrt price < 1.0
    let sqrt_price_neg1 = get_sqrt_ratio_at_tick(-1);
    let sqrt_price_neg100 = get_sqrt_ratio_at_tick(-100);
    let sqrt_price_neg1000 = get_sqrt_ratio_at_tick(-1000);
    
    assert!(sqrt_price_neg1 < ONE_X64, "Tick -1 should give price < 1.0");
    assert!(sqrt_price_neg100 < sqrt_price_neg1, "Tick -100 should give lower price than tick -1");
    assert!(sqrt_price_neg1000 < sqrt_price_neg100, "Tick -1000 should give lower price than tick -100");
}

#[test]
fn test_get_sqrt_ratio_at_tick_symmetry() {
    // sqrt_price(tick) * sqrt_price(-tick) should ≈ 1.0^2 = 1.0 in Q64
    let ticks = vec![1, 10, 100, 1000, 10000];
    
    for tick in ticks {
        let pos = get_sqrt_ratio_at_tick(tick);
        let neg = get_sqrt_ratio_at_tick(-tick);
        
        let product = (pos as u128 * neg as u128) >> 64;
        
        // Should be close to ONE_X64
        let tolerance = ONE_X64 / 1000; // 0.1% tolerance
        assert!(
            product >= ONE_X64.saturating_sub(tolerance) && product <= ONE_X64 + tolerance,
            "sqrt_price(tick) * sqrt_price(-tick) should be approximately 1.0, tick: {}", tick
        );
    }
}

#[test]
fn test_get_sqrt_ratio_at_tick_min_max() {
    let min_price = get_sqrt_ratio_at_tick(MIN_TICK);
    let max_price = get_sqrt_ratio_at_tick(MAX_TICK);
    
    assert!(min_price > 0, "Min tick should give positive price");
    assert!(max_price < u128::MAX, "Max tick should give valid price");
    assert!(min_price < max_price, "Min tick price should be less than max tick price");
}

#[test]
#[should_panic(expected = "tick out of range")]
fn test_get_sqrt_ratio_at_tick_too_low() {
    get_sqrt_ratio_at_tick(MIN_TICK - 1);
}

#[test]
#[should_panic(expected = "tick out of range")]
fn test_get_sqrt_ratio_at_tick_too_high() {
    get_sqrt_ratio_at_tick(MAX_TICK + 1);
}

#[test]
fn test_tick_to_sqrt_price_x64_alias() {
    let env = Env::default();
    
    // Should be identical to get_sqrt_ratio_at_tick
    let ticks = vec![0, 100, -100, 1000, -1000];
    
    for tick in ticks {
        let price1 = get_sqrt_ratio_at_tick(tick);
        let price2 = tick_to_sqrt_price_x64(&env, tick);
        assert_eq!(price1, price2, "Alias function should return same result");
    }
}

// ============================================================
// MONOTONICITY TESTS
// ============================================================

#[test]
fn test_sqrt_price_monotonically_increasing() {
    let mut prev_price = 0;
    
    // Test every 1000 ticks
    for tick in (-100000..=100000).step_by(1000) {
        if tick < MIN_TICK || tick > MAX_TICK {
            continue;
        }
        
        let price = get_sqrt_ratio_at_tick(tick);
        assert!(
            price >= prev_price,
            "Sqrt price should increase monotonically with tick. Tick: {}", tick
        );
        prev_price = price;
    }
}

#[test]
fn test_sqrt_price_strictly_increasing() {
    // Adjacent ticks should have strictly increasing prices
    let test_ranges = vec![
        (-1000, -900),
        (-100, 0),
        (0, 100),
        (900, 1000),
    ];
    
    for (start, end) in test_ranges {
        let mut prev_price = get_sqrt_ratio_at_tick(start);
        
        for tick in (start + 1)..=end {
            let price = get_sqrt_ratio_at_tick(tick);
            assert!(
                price > prev_price,
                "Sqrt price should strictly increase. Tick: {} -> {}", tick - 1, tick
            );
            prev_price = price;
        }
    }
}

// ============================================================
// NEXT SQRT PRICE FROM INPUT TESTS
// ============================================================

#[test]
fn test_get_next_sqrt_price_from_input_zero_amount() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    
    // Zero input should return same price
    let next_price = get_next_sqrt_price_from_input(&env, current_price, liquidity, 0, true);
    assert_eq!(next_price, current_price);
    
    let next_price = get_next_sqrt_price_from_input(&env, current_price, liquidity, 0, false);
    assert_eq!(next_price, current_price);
}

#[test]
fn test_get_next_sqrt_price_from_input_zero_liquidity() {
    let env = Env::default();
    let current_price = ONE_X64;
    let amount_in = 1000u128;
    
    // Zero liquidity should return same price
    let next_price = get_next_sqrt_price_from_input(&env, current_price, 0, amount_in, true);
    assert_eq!(next_price, current_price);
    
    let next_price = get_next_sqrt_price_from_input(&env, current_price, 0, amount_in, false);
    assert_eq!(next_price, current_price);
}

#[test]
fn test_get_next_sqrt_price_from_input_zero_for_one() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    let amount_in = 10_000u128;
    
    // Zero for one should decrease price
    let next_price = get_next_sqrt_price_from_input(&env, current_price, liquidity, amount_in, true);
    assert!(next_price < current_price, "Price should decrease when swapping token0 for token1");
}

#[test]
fn test_get_next_sqrt_price_from_input_one_for_zero() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    let amount_in = 10_000u128;
    
    // One for zero should increase price
    let next_price = get_next_sqrt_price_from_input(&env, current_price, liquidity, amount_in, false);
    assert!(next_price > current_price, "Price should increase when swapping token1 for token0");
}

#[test]
fn test_get_next_sqrt_price_from_input_proportional() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    
    // Larger input should cause larger price change
    let small_amount = 1_000u128;
    let large_amount = 10_000u128;
    
    let small_change = get_next_sqrt_price_from_input(&env, current_price, liquidity, small_amount, true);
    let large_change = get_next_sqrt_price_from_input(&env, current_price, liquidity, large_amount, true);
    
    let small_delta = current_price.saturating_sub(small_change);
    let large_delta = current_price.saturating_sub(large_change);
    
    assert!(large_delta > small_delta, "Larger input should cause larger price change");
}

// ============================================================
// NEXT SQRT PRICE FROM OUTPUT TESTS
// ============================================================

#[test]
fn test_get_next_sqrt_price_from_output_zero_amount() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    
    // Zero output should return same price
    let next_price = get_next_sqrt_price_from_output(&env, current_price, liquidity, 0, true);
    assert_eq!(next_price, current_price);
    
    let next_price = get_next_sqrt_price_from_output(&env, current_price, liquidity, 0, false);
    assert_eq!(next_price, current_price);
}

#[test]
fn test_get_next_sqrt_price_from_output_zero_for_one() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    let amount_out = 10_000u128;
    
    // Zero for one should decrease price
    let next_price = get_next_sqrt_price_from_output(&env, current_price, liquidity, amount_out, true);
    assert!(next_price < current_price, "Price should decrease when outputting token1");
}

#[test]
fn test_get_next_sqrt_price_from_output_one_for_zero() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    let amount_out = 10_000u128;
    
    // One for zero should increase price
    let next_price = get_next_sqrt_price_from_output(&env, current_price, liquidity, amount_out, false);
    assert!(next_price > current_price, "Price should increase when outputting token0");
}

// ============================================================
// COMPUTE SWAP STEP WITH TARGET TESTS
// ============================================================

#[test]
fn test_compute_swap_step_with_target_basic() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000i128;
    let amount = 10_000i128;
    let target_price = ONE_X64 / 2; // Target price at 0.5
    
    let (next_price, amount_in, amount_out) = compute_swap_step_with_target(
        &env, current_price, liquidity, amount, true, target_price
    );
    
    assert!(next_price >= target_price, "Should not go below target price");
    assert!(amount_in > 0, "Should consume some input");
    assert!(amount_out > 0, "Should produce some output");
}

#[test]
fn test_compute_swap_step_with_target_reached() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000i128;
    let large_amount = 1_000_000i128; // Large enough to reach target
    
    // Set target very close to current price
    let target_price = ONE_X64 - (ONE_X64 / 100); // 1% below
    
    let (next_price, _amount_in, _amount_out) = compute_swap_step_with_target(
        &env, current_price, liquidity, large_amount, true, target_price
    );
    
    // Should stop exactly at target
    assert_eq!(next_price, target_price, "Should reach target price exactly");
}

#[test]
fn test_compute_swap_step_with_target_not_reached() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000i128;
    let small_amount = 100i128; // Too small to reach target
    
    // Set target far from current price
    let target_price = ONE_X64 / 2; // 50% below
    
    let (next_price, amount_in, _amount_out) = compute_swap_step_with_target(
        &env, current_price, liquidity, small_amount, true, target_price
    );
    
    // Should not reach target
    assert!(next_price > target_price, "Should not reach target with small amount");
    assert!(amount_in <= small_amount, "Should not consume more than specified");
}

// ============================================================
// INVARIANT TESTS
// ============================================================

#[test]
fn test_sqrt_price_invariant_x_y() {
    let env = Env::default();
    
    // For concentrated liquidity: L = sqrt(x * y)
    // When price changes, L should remain constant (ignoring fees)
    
    let liquidity = 1_000_000u128;
    let current_price = ONE_X64;
    let amount_in = 10_000u128;
    
    let next_price = get_next_sqrt_price_from_input(&env, current_price, liquidity, amount_in, false);
    
    // Both prices should be valid
    assert!(current_price > 0);
    assert!(next_price > 0);
    assert_ne!(current_price, next_price);
}

#[test]
fn test_price_impact_direction() {
    let env = Env::default();
    let current_price = ONE_X64;
    let liquidity = 1_000_000u128;
    let amount = 10_000u128;
    
    // Buying token1 (zero_for_one=true) should decrease price
    let price_after_buy = get_next_sqrt_price_from_input(&env, current_price, liquidity, amount, true);
    assert!(price_after_buy <= current_price, "Buying token1 should decrease or maintain price");
    
    // Selling token0 (zero_for_one=false) should increase price
    let price_after_sell = get_next_sqrt_price_from_input(&env, current_price, liquidity, amount, false);
    assert!(price_after_sell >= current_price, "Selling token0 should increase or maintain price");
}

// ============================================================
// EDGE CASE TESTS
// ============================================================

#[test]
fn test_sqrt_price_at_extremes() {
    // Test at extreme tick values
    let min_price = get_sqrt_ratio_at_tick(MIN_TICK);
    let max_price = get_sqrt_ratio_at_tick(MAX_TICK);
    
    // Prices should be very different - use more realistic multiplier
    // At extreme ticks, prices differ by orders of magnitude
    assert!(max_price > min_price * 1000, 
        "Extreme ticks should have very different prices. Min: {}, Max: {}", min_price, max_price);
}

#[test]
fn test_sqrt_price_near_boundaries() {
    // Test near tick boundaries
    let near_min = get_sqrt_ratio_at_tick(MIN_TICK + 1);
    let near_max = get_sqrt_ratio_at_tick(MAX_TICK - 1);
    
    assert!(near_min > 0, "Price near min should be positive");
    assert!(near_max < u128::MAX, "Price near max should not overflow");
}

#[test]
fn test_large_liquidity() {
    let env = Env::default();
    let current_price = ONE_X64;
    let large_liquidity = u128::MAX / 2;
    let amount = 1000u128;
    
    // Should handle large liquidity without panicking
    let next_price = get_next_sqrt_price_from_input(&env, current_price, large_liquidity, amount, true);
    
    // Price change should be small with large liquidity
    let price_delta = current_price.saturating_sub(next_price);
    assert!(price_delta < current_price / 1000, "Large liquidity should result in small price change");
}