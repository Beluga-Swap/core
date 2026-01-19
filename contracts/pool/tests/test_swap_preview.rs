mod common;

use soroban_sdk::{Env, Symbol};

#[test]
fn test_preview_swap_no_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    let state = client.get_pool_state();
    
    // Try to preview swap with no liquidity
    let result = client.preview_swap(
        &state.token0,
        &1_000_000,
        &0,
        &0, // No price limit
    );
    
    // Should fail due to no liquidity
    assert!(!result.is_valid);
    assert_eq!(result.error_message, Some(Symbol::new(&env, "NO_LIQ")));
}

#[test]
fn test_preview_swap_invalid_token() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _, _, _) = common::setup_pool(&env);
    
    // Create invalid token
    let invalid_token = common::create_token(&env, &creator);
    
    let result = client.preview_swap(
        &invalid_token,
        &1_000_000,
        &0,
        &0,
    );
    
    assert!(!result.is_valid);
    assert_eq!(result.error_message, Some(Symbol::new(&env, "BAD_TOKEN")));
}

#[test]
fn test_preview_swap_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    let state = client.get_pool_state();
    
    let result = client.preview_swap(
        &state.token0,
        &0, // Zero amount
        &0,
        &0,
    );
    
    // Should fail with amount too low
    assert!(!result.is_valid);
    assert_eq!(result.error_message, Some(Symbol::new(&env, "AMT_LOW")));
}

#[test]
fn test_preview_swap_amount_too_small() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _, _, _, _) = common::setup_pool(&env);
    let state = client.get_pool_state();
    
    let result = client.preview_swap(
        &state.token0,
        &1, // Very small amount (likely dust)
        &0,
        &0,
    );
    
    assert!(!result.is_valid);
    // Error could be AMT_LOW or NO_LIQ depending on pool state
}

// Note: Tests with actual liquidity require add_liquidity to be implemented
// These tests validate the preview logic without liquidity