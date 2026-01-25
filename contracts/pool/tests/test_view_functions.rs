mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_get_pool_state() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    
    let state = client.get_pool_state();
    
    // Verify initial state
    assert_eq!(state.sqrt_price_x64, common::DEFAULT_SQRT_PRICE_X64);
    assert_eq!(state.current_tick, common::DEFAULT_TICK);
    assert_eq!(state.liquidity, 0);
    assert_eq!(state.tick_spacing, common::DEFAULT_TICK_SPACING);
}

#[test]
fn test_get_pool_config() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    
    let config = client.get_pool_config();
    
    // Verify configuration
    assert_eq!(config.creator, creator);
    assert_eq!(config.fee_bps, common::DEFAULT_FEE_BPS);
    assert_eq!(config.creator_fee_bps, common::DEFAULT_CREATOR_FEE_BPS);
}

#[test]
fn test_get_position_empty() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let user = Address::generate(&env);
    
    let position = client.get_position(&user, &-600, &600);
    
    // Position should be empty
    assert_eq!(position.liquidity, 0);
    assert_eq!(position.amount0, 0);
    assert_eq!(position.amount1, 0);
}

#[test]
fn test_is_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    
    assert!(client.is_initialized());
}