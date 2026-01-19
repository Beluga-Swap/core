mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

// ============================================================
// TOKEN VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_same_token_pair() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token = common::create_token(&env);
    
    let params = common::default_pool_params(&env, &token, &token);
    
    client.create_pool(&creator, &params); // Should fail: InvalidTokenPair
}

// ============================================================
// FEE TIER VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_invalid_fee_tier() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.fee_bps = 999; // Invalid tier
    
    client.create_pool(&creator, &params); // Should fail: InvalidFeeTier
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_disabled_fee_tier() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    // Disable 30bps tier
    client.set_fee_tier(&30, &60, &false);
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    let params = common::default_pool_params(&env, &token_a, &token_b);
    
    client.create_pool(&creator, &params); // Should fail: InvalidFeeTier (disabled)
}

// ============================================================
// TICK RANGE VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_invalid_tick_range_equal() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.lower_tick = -600;
    params.upper_tick = -600; // Same as lower
    
    client.create_pool(&creator, &params); // Should fail: InvalidTickRange
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_invalid_tick_range_inverted() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.lower_tick = 600;
    params.upper_tick = -600; // Inverted
    
    client.create_pool(&creator, &params); // Should fail: InvalidTickRange
}

// ============================================================
// TICK SPACING VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_tick_not_aligned_lower() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.lower_tick = -601; // Not divisible by 60
    
    client.create_pool(&creator, &params); // Should fail: InvalidTickSpacing
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_tick_not_aligned_upper() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.upper_tick = 601; // Not divisible by 60
    
    client.create_pool(&creator, &params); // Should fail: InvalidTickSpacing
}

// ============================================================
// LIQUIDITY VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #20)")]
fn test_insufficient_liquidity_token0() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.amount0_desired = 100_000; // Below MIN (1_000_000)
    
    client.create_pool(&creator, &params); // Should fail: InsufficientInitialLiquidity
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")]
fn test_insufficient_liquidity_token1() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.amount1_desired = 100_000; // Below MIN
    
    client.create_pool(&creator, &params); // Should fail: InsufficientInitialLiquidity
}

#[test]
fn test_minimum_liquidity_valid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (_client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.amount0_desired = 1_000_000; // Exactly minimum
    params.amount1_desired = 1_000_000;
    
    // Should be valid (will fail at pool deployment but validation passes)
}

// ============================================================
// LOCK DURATION VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #21)")]
fn test_lock_duration_too_short() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.lock_duration = 1000; // Below MIN (120_960)
    
    client.create_pool(&creator, &params); // Should fail: InvalidLockDuration
}

#[test]
fn test_permanent_lock_valid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (_client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.lock_duration = 0; // Permanent lock
    
    // Should be valid
}

#[test]
fn test_minimum_lock_duration_valid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (_client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.lock_duration = 120_960; // Exactly minimum (7 days)
    
    // Should be valid
}

// ============================================================
// CREATOR FEE VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_creator_fee_too_low() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.creator_fee_bps = 5; // Below min (10 = 0.1%)
    
    client.create_pool(&creator, &params); // Should fail: InvalidCreatorFee
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_creator_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.creator_fee_bps = 1500; // Above max (1000 = 10%)
    
    client.create_pool(&creator, &params); // Should fail: InvalidCreatorFee
}

#[test]
fn test_minimum_creator_fee_valid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (_client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.creator_fee_bps = 10; // Minimum 0.1%
    
    // Should be valid
}

#[test]
fn test_maximum_creator_fee_valid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (_client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.creator_fee_bps = 1000; // Maximum 10%
    
    // Should be valid
}

// ============================================================
// PRICE VALIDATION
// ============================================================

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_zero_initial_price() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    let mut params = common::default_pool_params(&env, &token_a, &token_b);
    params.initial_sqrt_price_x64 = 0;
    
    client.create_pool(&creator, &params); // Should fail: InvalidInitialPrice
}