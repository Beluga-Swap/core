mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_creator_fee_configuration() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _factory, _token_a, _token_b) = common::setup_custom_pool(
        &env,
        30,  // fee_bps
        1,   // creator_fee_bps (minimum: 1 bps = 0.01%)
        common::DEFAULT_SQRT_PRICE_X64,
        common::DEFAULT_TICK,
        common::DEFAULT_TICK_SPACING,
    );
    
    let config = client.get_pool_config();
    assert_eq!(config.creator, creator);
    assert_eq!(config.creator_fee_bps, 1);
}

#[test]
fn test_get_creator_fees_initial_state() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    
    // Initially, no fees should be accumulated
    let fees = client.get_creator_fees();
    assert_eq!(fees.fees_token0, 0);
    assert_eq!(fees.fees_token1, 0);
}

#[test]
fn test_claim_creator_fees_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    
    // Claiming when no fees exist should return zero
    let result = client.claim_creator_fees(&creator);
    assert_eq!(result.0, 0);
    assert_eq!(result.1, 0);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_claim_creator_fees_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let non_creator = Address::generate(&env);
    
    // Non-creator should not be able to claim fees
    client.claim_creator_fees(&non_creator);
}