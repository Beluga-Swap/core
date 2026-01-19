mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};
use belugaswap_pool::{BelugaPool, BelugaPoolClient};

#[test]
fn test_initialization_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _token_a, _token_b, _) = common::setup_pool(&env);
    
    // Check initialization
    assert!(client.is_initialized());
    
    // Check pool state
    let state = client.get_pool_state();
    assert_eq!(state.sqrt_price_x64, common::DEFAULT_SQRT_PRICE_X64);
    assert_eq!(state.current_tick, common::DEFAULT_TICK);
    assert_eq!(state.liquidity, 0);
    assert_eq!(state.tick_spacing, common::DEFAULT_TICK_SPACING);
    
    // Check pool config
    let config = client.get_pool_config();
    assert_eq!(config.creator, creator);
    assert_eq!(config.fee_bps, common::DEFAULT_FEE_BPS);
    assert_eq!(config.creator_fee_bps, common::DEFAULT_CREATOR_FEE_BPS);
    
    // Token order should be sorted
    assert!(config.token_a < config.token_b || config.token_a == config.token_b);
}

#[test]
#[should_panic(expected = "pool already initialized")]
fn test_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    // First init
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &common::DEFAULT_FEE_BPS,
        &common::DEFAULT_CREATOR_FEE_BPS,
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &common::DEFAULT_TICK_SPACING,
    );
    
    // Second init should panic
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &common::DEFAULT_FEE_BPS,
        &common::DEFAULT_CREATOR_FEE_BPS,
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &common::DEFAULT_TICK_SPACING,
    );
}

#[test]
#[should_panic(expected = "invalid fee")]
fn test_invalid_fee_zero() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &0, // Invalid: zero fee
        &common::DEFAULT_CREATOR_FEE_BPS,
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &common::DEFAULT_TICK_SPACING,
    );
}

#[test]
#[should_panic(expected = "invalid fee")]
fn test_invalid_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &10001, // Invalid: > 10000 bps (100%)
        &common::DEFAULT_CREATOR_FEE_BPS,
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &common::DEFAULT_TICK_SPACING,
    );
}

#[test]
#[should_panic(expected = "invalid creator fee")]
fn test_invalid_creator_fee_too_low() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &common::DEFAULT_FEE_BPS,
        &0, // Invalid: < 1 bps
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &common::DEFAULT_TICK_SPACING,
    );
}

#[test]
#[should_panic(expected = "invalid creator fee")]
fn test_invalid_creator_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &common::DEFAULT_FEE_BPS,
        &1001, // Invalid: > 1000 bps (10%)
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &common::DEFAULT_TICK_SPACING,
    );
}

#[test]
#[should_panic(expected = "invalid tick spacing")]
fn test_invalid_tick_spacing_zero() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &common::DEFAULT_FEE_BPS,
        &common::DEFAULT_CREATOR_FEE_BPS,
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &0, // Invalid: zero spacing
    );
}

#[test]
#[should_panic(expected = "invalid tick spacing")]
fn test_invalid_tick_spacing_negative() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &common::DEFAULT_FEE_BPS,
        &common::DEFAULT_CREATOR_FEE_BPS,
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &-10, // Invalid: negative spacing
    );
}

#[test]
fn test_token_sorting() {
    let env = Env::default();
    env.mock_all_auths();
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);
    
    let pool_id = env.register_contract(None, BelugaPool);
    let client = BelugaPoolClient::new(&env, &pool_id);
    
    // Initialize with tokens (may be in any order)
    client.initialize(
        &creator,
        &token_a,
        &token_b,
        &common::DEFAULT_FEE_BPS,
        &common::DEFAULT_CREATOR_FEE_BPS,
        &common::DEFAULT_SQRT_PRICE_X64,
        &common::DEFAULT_TICK,
        &common::DEFAULT_TICK_SPACING,
    );
    
    let state = client.get_pool_state();
    
    // token0 should always be < token1 (sorted)
    assert!(state.token0 < state.token1);
}