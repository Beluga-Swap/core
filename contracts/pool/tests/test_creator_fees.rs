mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_get_creator_fees_initial_state() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    
    let fees = client.get_creator_fees();
    
    assert_eq!(fees.fees_token0, 0);
    assert_eq!(fees.fees_token1, 0);
}

#[test]
fn test_claim_creator_fees_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _, _, _) = common::setup_pool(&env);
    
    // Claim with no fees accumulated
    let result = client.claim_creator_fees(&creator);
    
    assert_eq!(result.0, 0);
    assert_eq!(result.1, 0);
    
    // Fees should still be zero
    let fees = client.get_creator_fees();
    assert_eq!(fees.fees_token0, 0);
    assert_eq!(fees.fees_token1, 0);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_claim_creator_fees_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    let non_creator = Address::generate(&env);
    
    // Try to claim as non-creator
    client.claim_creator_fees(&non_creator);
}

#[test]
fn test_creator_fee_configuration() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Test different creator fee levels
    let test_cases = vec![
        1,    // Minimum (0.01%)
        50,   // 0.5%
        100,  // 1%
        500,  // 5%
        1000, // Maximum (10%)
    ];
    
    for creator_fee_bps in test_cases {
        let (client, _, _, _, _) = common::setup_custom_pool(
            &env,
            common::DEFAULT_FEE_BPS,
            creator_fee_bps,
            common::DEFAULT_SQRT_PRICE_X64,
            common::DEFAULT_TICK,
            common::DEFAULT_TICK_SPACING,
        );
        
        let config = client.get_pool_config();
        assert_eq!(config.creator_fee_bps, creator_fee_bps);
    }
}

// Note: Tests for actual fee accumulation require:
// 1. Swaps to be executed (which generate fees)
// 2. Token transfers and balances
// 3. Integration test setup
// These tests focus on the claim mechanism and authorization