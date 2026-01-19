mod common;

use soroban_sdk::Env;

#[test]
fn test_is_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    
    assert!(client.is_initialized());
}

#[test]
fn test_get_pool_state() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    
    let state = client.get_pool_state();
    
    // Verify state fields
    assert_eq!(state.sqrt_price_x64, common::DEFAULT_SQRT_PRICE_X64);
    assert_eq!(state.current_tick, common::DEFAULT_TICK);
    assert_eq!(state.liquidity, 0); // No liquidity initially
    assert_eq!(state.tick_spacing, common::DEFAULT_TICK_SPACING);
    assert!(state.token0 < state.token1); // Tokens should be sorted
    assert_eq!(state.fee_growth_global_0, 0);
    assert_eq!(state.fee_growth_global_1, 0);
    assert_eq!(state.creator_fees_0, 0);
    assert_eq!(state.creator_fees_1, 0);
}

#[test]
fn test_get_pool_config() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, token_a, token_b, _) = common::setup_pool(&env);
    
    let config = client.get_pool_config();
    
    // Verify config fields
    assert_eq!(config.creator, creator);
    assert_eq!(config.fee_bps, common::DEFAULT_FEE_BPS);
    assert_eq!(config.creator_fee_bps, common::DEFAULT_CREATOR_FEE_BPS);
    
    // Original tokens (token_a, token_b) should match
    assert!((config.token_a == token_a && config.token_b == token_b) ||
            (config.token_a == token_b && config.token_b == token_a));
}

#[test]
fn test_get_position_empty() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _, _, _) = common::setup_pool(&env);
    
    // Query non-existent position
    let position = client.get_position(&creator, &-600, &600);
    
    assert_eq!(position.liquidity, 0);
    assert_eq!(position.amount0, 0);
    assert_eq!(position.amount1, 0);
    assert_eq!(position.fees_owed_0, 0);
    assert_eq!(position.fees_owed_1, 0);
}

#[test]
fn test_get_creator_fees_initial() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    
    let fees = client.get_creator_fees();
    
    assert_eq!(fees.fees_token0, 0);
    assert_eq!(fees.fees_token1, 0);
}

#[test]
fn test_get_swap_direction_token0() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    let state = client.get_pool_state();
    
    // Swap token0 for token1
    let zero_for_one = client.get_swap_direction(&state.token0);
    assert!(zero_for_one);
}

#[test]
fn test_get_swap_direction_token1() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    let state = client.get_pool_state();
    
    // Swap token1 for token0
    let zero_for_one = client.get_swap_direction(&state.token1);
    assert!(!zero_for_one);
}

#[test]
#[should_panic(expected = "invalid token")]
fn test_get_swap_direction_invalid_token() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _, _, _) = common::setup_pool(&env);
    
    // Create a random token not in the pool
    let invalid_token = common::create_token(&env, &creator);
    
    client.get_swap_direction(&invalid_token); // Should panic
}

#[test]
fn test_custom_initialization_params() {
    let env = Env::default();
    env.mock_all_auths();
    
    let custom_fee = 100; // 1%
    let custom_creator_fee = 50; // 0.5%
    let custom_price = 1u128 << 65; // Price = 2.0
    let custom_tick = 6932; // log(2) in ticks
    let custom_spacing = 200;
    
    let (client, _, _, _, _) = common::setup_custom_pool(
        &env,
        custom_fee,
        custom_creator_fee,
        custom_price,
        custom_tick,
        custom_spacing,
    );
    
    let config = client.get_pool_config();
    let state = client.get_pool_state();
    
    assert_eq!(config.fee_bps, custom_fee);
    assert_eq!(config.creator_fee_bps, custom_creator_fee);
    assert_eq!(state.sqrt_price_x64, custom_price);
    assert_eq!(state.current_tick, custom_tick);
    assert_eq!(state.tick_spacing, custom_spacing);
}