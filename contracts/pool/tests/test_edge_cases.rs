mod common;

use soroban_sdk::Env;

// ============================================================
// EXTREME PARAMETER TESTS
// ============================================================

#[test]
fn test_minimum_fee() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        1, // Minimum fee (0.01%)
        100,
        common::DEFAULT_SQRT_PRICE_X64,
        common::DEFAULT_TICK,
        common::DEFAULT_TICK_SPACING,
    );
    
    let config = client.get_pool_config();
    assert_eq!(config.fee_bps, 1);
}

#[test]
fn test_maximum_fee() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        10000, // Maximum fee (100%)
        100,
        common::DEFAULT_SQRT_PRICE_X64,
        common::DEFAULT_TICK,
        common::DEFAULT_TICK_SPACING,
    );
    
    let config = client.get_pool_config();
    assert_eq!(config.fee_bps, 10000);
}

#[test]
fn test_minimum_creator_fee() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        30,
        1, // Minimum creator fee (0.01%)
        common::DEFAULT_SQRT_PRICE_X64,
        common::DEFAULT_TICK,
        common::DEFAULT_TICK_SPACING,
    );
    
    let config = client.get_pool_config();
    assert_eq!(config.creator_fee_bps, 1);
}

#[test]
fn test_maximum_creator_fee() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        30,
        1000, // Maximum creator fee (10%)
        common::DEFAULT_SQRT_PRICE_X64,
        common::DEFAULT_TICK,
        common::DEFAULT_TICK_SPACING,
    );
    
    let config = client.get_pool_config();
    assert_eq!(config.creator_fee_bps, 1000);
}

#[test]
fn test_various_tick_spacings() {
    let env = Env::default();
    env.mock_all_auths();
    
    let test_spacings = vec![1, 10, 60, 200, 1000];
    
    for spacing in test_spacings {
        let (client, _, _, _, _) = common::setup_custom_pool(
            &env,
            30,
            100,
            common::DEFAULT_SQRT_PRICE_X64,
            0,
            spacing,
        );
        
        let state = client.get_pool_state();
        assert_eq!(state.tick_spacing, spacing);
    }
}

// ============================================================
// PRICE RANGE TESTS
// ============================================================

#[test]
fn test_very_low_price() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Very low price (close to zero but not zero)
    let low_price = 1u128 << 32; // sqrt(price) = very small
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        30,
        100,
        low_price,
        -887272, // Very negative tick
        60,
    );
    
    let state = client.get_pool_state();
    assert_eq!(state.sqrt_price_x64, low_price);
}

#[test]
fn test_very_high_price() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Very high price
    let high_price = 1u128 << 96; // sqrt(price) = very large
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        30,
        100,
        high_price,
        887272, // Very positive tick
        60,
    );
    
    let state = client.get_pool_state();
    assert_eq!(state.sqrt_price_x64, high_price);
}

// ============================================================
// TICK BOUNDARY TESTS
// ============================================================

#[test]
fn test_negative_tick() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        30,
        100,
        1u128 << 63, // Price < 1
        -6932, // Negative tick
        60,
    );
    
    let state = client.get_pool_state();
    assert!(state.current_tick < 0);
}

#[test]
fn test_positive_tick() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        30,
        100,
        1u128 << 65, // Price > 1
        6932, // Positive tick
        60,
    );
    
    let state = client.get_pool_state();
    assert!(state.current_tick > 0);
}

#[test]
fn test_zero_tick() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        30,
        100,
        1u128 << 64, // Price = 1
        0, // Zero tick
        60,
    );
    
    let state = client.get_pool_state();
    assert_eq!(state.current_tick, 0);
}

// ============================================================
// MULTIPLE POOLS INDEPENDENCE TEST
// ============================================================

#[test]
fn test_multiple_pools_independence() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Create 3 different pools
    let (client1, _, _, _, _) = common::setup_pool(&env);
    let (client2, _, _, _, _) = common::setup_pool(&env);
    let (client3, _, _, _, _) = common::setup_pool(&env);
    
    // All should be initialized
    assert!(client1.is_initialized());
    assert!(client2.is_initialized());
    assert!(client3.is_initialized());
    
    // All should have independent states
    let state1 = client1.get_pool_state();
    let state2 = client2.get_pool_state();
    let state3 = client3.get_pool_state();
    
    // Different tokens
    assert_ne!(state1.token0, state2.token0);
    assert_ne!(state2.token0, state3.token0);
}