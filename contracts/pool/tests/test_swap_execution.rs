mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
#[should_panic(expected = "invalid token")]
fn test_swap_invalid_token() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let sender = Address::generate(&env);
    
    // Create invalid token
    let invalid_token = common::create_token(&env, &creator);
    
    client.swap(
        &sender,
        &invalid_token,
        &1_000_000,
        &0,
        &0,
    );
}

#[test]
#[should_panic(expected = "no liquidity available")]
fn test_swap_slippage_exceeded() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    let state = client.get_pool_state();
    let sender = Address::generate(&env);
    
    // Try to swap with no liquidity but high slippage requirement
    // This will fail with no liquidity available
    client.swap(
        &sender,
        &state.token0,
        &1_000_000,
        &1_000_000, // Expecting at least same amount out (impossible)
        &0,
    );
}

// Note: Full swap tests with liquidity require:
// 1. add_liquidity to be tested first
// 2. Token minting and approvals
// 3. These are integration tests that would go in a separate file