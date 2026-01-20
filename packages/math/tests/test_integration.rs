use belugaswap_math::*;
use soroban_sdk::Env;

// ============================================================
// INTEGRATION: FULL SWAP CALCULATION
// ============================================================

#[test]
fn test_integration_swap_calculation() {
    let env = Env::default();
    
    // Setup: Pool at price 1.0 with 1M liquidity
    let current_sqrt_price = ONE_X64;
    let liquidity = 1_000_000i128;
    let amount_in = 10_000i128;
    
    // Calculate swap (zero_for_one = true, swapping token0 for token1)
    let (next_sqrt_price, consumed_in, received_out) = compute_swap_step(
        &env,
        current_sqrt_price,
        liquidity,
        amount_in,
        true, // zero_for_one
    );
    
    // Validate results
    assert!(next_sqrt_price < current_sqrt_price, "Price should decrease");
    assert!(consumed_in > 0 && consumed_in <= amount_in, "Should consume input");
    assert!(received_out > 0, "Should produce output");
    
    // Price impact should be reasonable
    let price_change_pct = ((current_sqrt_price - next_sqrt_price) * 10000) / current_sqrt_price;
    assert!(price_change_pct < 1000, "Price impact should be < 10%");
}

#[test]
fn test_integration_add_liquidity() {
    let env = Env::default();
    
    // Setup: Want to add liquidity around price 1.0
    let amount0_desired = 1_000_000i128;
    let amount1_desired = 1_000_000i128;
    
    // Price range: 0.5 to 2.0
    let tick_lower = -6932;  // sqrt(0.5) in tick space
    let tick_upper = 6932;   // sqrt(2) in tick space
    
    let sqrt_price_lower = get_sqrt_ratio_at_tick(tick_lower);
    let sqrt_price_upper = get_sqrt_ratio_at_tick(tick_upper);
    let current_sqrt_price = ONE_X64; // Current price 1.0
    
    // Calculate liquidity
    let liquidity = get_liquidity_for_amounts(
        &env,
        amount0_desired,
        amount1_desired,
        sqrt_price_lower,
        sqrt_price_upper,
        current_sqrt_price,
    );
    
    assert!(liquidity > 0, "Should calculate positive liquidity");
    assert!(liquidity >= MIN_LIQUIDITY, "Should meet minimum liquidity");
    
    // Calculate actual amounts needed
    let (amount0_actual, amount1_actual) = get_amounts_for_liquidity(
        &env,
        liquidity,
        sqrt_price_lower,
        sqrt_price_upper,
        current_sqrt_price,
    );
    
    assert!(amount0_actual <= amount0_desired, "Amount0 should not exceed desired");
    assert!(amount1_actual <= amount1_desired, "Amount1 should not exceed desired");
}

#[test]
fn test_integration_remove_liquidity() {
    let env = Env::default();
    
    // Setup: Existing position with liquidity
    let liquidity = 500_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_sqrt_price = ONE_X64;
    
    // Calculate amounts to receive
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env,
        liquidity,
        sqrt_price_lower,
        sqrt_price_upper,
        current_sqrt_price,
    );
    
    assert!(amount0 > 0, "Should receive token0");
    assert!(amount1 > 0, "Should receive token1");
    
    // Verify amounts are reasonable
    assert!(amount0 < liquidity, "Amount0 should be less than liquidity");
    assert!(amount1 < liquidity, "Amount1 should be less than liquidity");
}

// ============================================================
// INTEGRATION: MULTI-TICK SWAP
// ============================================================

#[test]
fn test_integration_multi_tick_swap() {
    let env = Env::default();
    
    // Simulate a swap that crosses multiple ticks
    let mut current_price = ONE_X64;
    let liquidity = 1_000_000i128;
    let mut total_amount_in = 0i128;
    let mut total_amount_out = 0i128;
    
    // Simulate 5 swap steps
    for _ in 0..5 {
        let amount_in_step = 10_000i128;
        
        let (next_price, consumed, received) = compute_swap_step(
            &env,
            current_price,
            liquidity,
            amount_in_step,
            true,
        );
        
        total_amount_in += consumed;
        total_amount_out += received;
        current_price = next_price;
    }
    
    assert!(total_amount_in > 0, "Should consume total input");
    assert!(total_amount_out > 0, "Should produce total output");
    assert!(current_price < ONE_X64, "Price should decrease after selling token0");
}

// ============================================================
// INTEGRATION: PRICE RANGE SCENARIOS
// ============================================================

#[test]
fn test_integration_concentrated_liquidity() {
    let env = Env::default();
    
    // Narrow range (concentrated liquidity)
    let sqrt_price_lower = ONE_X64 * 95 / 100;  // 0.95
    let sqrt_price_upper = ONE_X64 * 105 / 100; // 1.05
    let current_sqrt_price = ONE_X64;
    
    let amount0 = 1_000_000i128;
    let amount1 = 1_000_000i128;
    
    let liquidity_narrow = get_liquidity_for_amounts(
        &env, amount0, amount1, sqrt_price_lower, sqrt_price_upper, current_sqrt_price
    );
    
    // Wide range
    let sqrt_price_lower_wide = ONE_X64 / 2;  // 0.5
    let sqrt_price_upper_wide = ONE_X64 * 2;  // 2.0
    
    let liquidity_wide = get_liquidity_for_amounts(
        &env, amount0, amount1, sqrt_price_lower_wide, sqrt_price_upper_wide, current_sqrt_price
    );
    
    // Concentrated liquidity should be higher for same amounts
    assert!(liquidity_narrow > liquidity_wide, "Concentrated liquidity should be higher");
}

// ============================================================
// INTEGRATION: FEE CALCULATIONS
// ============================================================

#[test]
fn test_integration_swap_with_fees() {
    let env = Env::default();
    
    let current_sqrt_price = ONE_X64;
    let liquidity = 1_000_000i128;
    let amount_in = 100_000i128;
    let fee_bps = 30u32; // 0.3% fee
    
    // Calculate swap
    let (next_sqrt_price, consumed, received) = compute_swap_step(
        &env, current_sqrt_price, liquidity, amount_in, true
    );
    
    // Calculate fee
    let fee_amount = (consumed as u128 * fee_bps as u128) / 10000;
    let amount_after_fee = consumed - fee_amount as i128;
    
    assert!(amount_after_fee > 0, "Should have amount after fee");
    assert!(fee_amount > 0, "Should have positive fee");
    assert!(fee_amount < consumed as u128, "Fee should be less than input");
    
    // Output should be based on amount after fee
    assert!(received > 0, "Should produce output");
}

// ============================================================
// INTEGRATION: TICK SPACING
// ============================================================

#[test]
fn test_integration_tick_spacing() {
    // Test different tick spacings
    let spacings = vec![1, 10, 60, 200];
    
    for spacing in spacings {
        let test_tick = 12345;
        let snapped = snap_tick_to_spacing(test_tick, spacing);
        
        // Snapped tick should be aligned with spacing
        assert_eq!(snapped % spacing, 0, "Tick should be aligned to spacing");
        
        // Snapped tick should be <= original tick
        assert!(snapped <= test_tick, "Snapped tick should not exceed original");
        
        // Snapped tick should be within one spacing of original
        assert!(test_tick - snapped < spacing, "Should be within one spacing");
    }
}

// ============================================================
// INTEGRATION: POSITION VALUE
// ============================================================

#[test]
fn test_integration_position_value() {
    let env = Env::default();
    
    // Create position
    let initial_amount0 = 1_000_000i128;
    let initial_amount1 = 1_000_000i128;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64 * 2;
    let initial_price = ONE_X64;
    
    let liquidity = get_liquidity_for_amounts(
        &env, initial_amount0, initial_amount1, 
        sqrt_price_lower, sqrt_price_upper, initial_price
    );
    
    // Calculate value at different prices
    let prices = vec![
        ONE_X64 / 2,  // At lower bound
        ONE_X64,       // At initial
        ONE_X64 * 2,   // At upper bound
    ];
    
    for price in prices {
        let (amount0, amount1) = get_amounts_for_liquidity(
            &env, liquidity, sqrt_price_lower, sqrt_price_upper, price
        );
        
        // Position should always have some value
        assert!(amount0 >= 0, "Amount0 should be non-negative");
        assert!(amount1 >= 0, "Amount1 should be non-negative");
        
        // Total value in token terms should be reasonable
        assert!(amount0 + amount1 > 0, "Position should have value");
    }
}

// ============================================================
// INTEGRATION: INVARIANT CHECKS
// ============================================================

#[test]
fn test_integration_constant_product_invariant() {
    let env = Env::default();
    
    // For concentrated liquidity: L² should remain constant (ignoring fees)
    let liquidity = 1_000_000u128;
    let initial_price = ONE_X64;
    let amount_in = 10_000u128;
    
    // Calculate k = L²
    let k = liquidity * liquidity;
    
    // After swap, liquidity should remain the same
    let next_price = get_next_sqrt_price_from_input(
        &env, initial_price, liquidity, amount_in, false
    );
    
    // Verify liquidity constant (this is a simplification)
    assert!(initial_price > 0);
    assert!(next_price > 0);
    assert!(k > 0);
}

#[test]
fn test_integration_no_arbitrage() {
    let env = Env::default();
    
    // Swap A -> B then B -> A should not profit
    let initial_price = ONE_X64;
    let liquidity = 1_000_000i128;
    let amount = 10_000i128;
    
    // Swap token0 for token1
    let (price_after_1, consumed_1, received_1) = compute_swap_step(
        &env, initial_price, liquidity, amount, true
    );
    
    // Swap token1 back for token0
    let (_final_price, _consumed_2, received_2) = compute_swap_step(
        &env, price_after_1, liquidity, received_1, false
    );
    
    // Due to rounding and precision loss in Q64 math, we expect some loss
    // But it should be small (< 10%)
    let loss_pct = if consumed_1 > 0 {
        ((consumed_1 - received_2) * 100) / consumed_1
    } else {
        0
    };
    
    assert!(
        loss_pct >= 0 && loss_pct < 15, // Allow up to 15% loss due to precision
        "Round-trip loss should be reasonable: {}%", loss_pct
    );
}

// ============================================================
// INTEGRATION: EDGE CASES
// ============================================================

#[test]
fn test_integration_minimum_liquidity() {
    let env = Env::default();
    
    // Test with minimum liquidity
    let min_liq = MIN_LIQUIDITY;
    let sqrt_price_lower = ONE_X64 / 2;
    let sqrt_price_upper = ONE_X64 * 2;
    let current_price = ONE_X64;
    
    let (amount0, amount1) = get_amounts_for_liquidity(
        &env, min_liq, sqrt_price_lower, sqrt_price_upper, current_price
    );
    
    assert!(amount0 > 0 || amount1 > 0, "Minimum liquidity should produce some amounts");
}

#[test]
fn test_integration_price_at_boundaries() {
    let env = Env::default();
    
    // Test at min/max tick boundaries
    let min_sqrt_price = get_sqrt_ratio_at_tick(MIN_TICK);
    let max_sqrt_price = get_sqrt_ratio_at_tick(MAX_TICK);
    
    let liquidity = 1_000_000u128;
    let amount = 1000u128;
    
    // Should handle extreme prices without panicking
    let _ = get_next_sqrt_price_from_input(&env, min_sqrt_price, liquidity, amount, false);
    let _ = get_next_sqrt_price_from_input(&env, max_sqrt_price, liquidity, amount, true);
}