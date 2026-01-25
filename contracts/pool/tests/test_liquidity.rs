mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

// ============================================================
// ADD LIQUIDITY VALIDATION TESTS
// ============================================================

#[test]
#[should_panic(expected = "invalid tick range")]
fn test_add_liquidity_invalid_tick_range_equal() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Lower tick == Upper tick (invalid)
    client.add_liquidity(
        &lp,
        &-600,
        &-600, // Same as lower
        &1_000_000,
        &1_000_000,
        &0,
        &0,
    );
}

#[test]
#[should_panic(expected = "invalid tick range")]
fn test_add_liquidity_invalid_tick_range_inverted() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Lower tick > Upper tick (invalid)
    client.add_liquidity(
        &lp,
        &600,
        &-600, // Inverted
        &1_000_000,
        &1_000_000,
        &0,
        &0,
    );
}

#[test]
#[should_panic(expected = "liquidity amount too low")]
fn test_add_liquidity_zero_amounts() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Both amounts zero
    client.add_liquidity(
        &lp,
        &-600,
        &600,
        &0, // Zero
        &0, // Zero
        &0,
        &0,
    );
}

#[test]
#[should_panic(expected = "liquidity amount too low")]
fn test_add_liquidity_below_minimum() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Amounts too small (below MIN_LIQUIDITY threshold)
    client.add_liquidity(
        &lp,
        &-600,
        &600,
        &10, // Too small
        &10, // Too small
        &0,
        &0,
    );
}

// ============================================================
// REMOVE LIQUIDITY VALIDATION TESTS
// ============================================================

#[test]
#[should_panic(expected = "insufficient liquidity")]
fn test_remove_liquidity_nonexistent_position() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Try to remove liquidity from non-existent position
    client.remove_liquidity(
        &lp,
        &-600,
        &600,
        &1_000_000, // No liquidity exists
    );
}

#[test]
#[should_panic(expected = "liquidity amount must be positive")]
fn test_remove_liquidity_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Try to remove zero liquidity
    client.remove_liquidity(
        &lp,
        &-600,
        &600,
        &0, // Zero amount
    );
}

// ============================================================
// COLLECT FEES TESTS
// ============================================================

#[test]
fn test_collect_fees_no_position() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Collecting from non-existent position should return zero
    let result = client.collect(&lp, &-600, &600);
    
    assert_eq!(result.0, 0);
    assert_eq!(result.1, 0);
}

// ============================================================
// POSITION QUERY TESTS
// ============================================================

#[test]
fn test_get_position_various_ranges() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Query different tick ranges (all should be empty)
    let pos1 = client.get_position(&lp, &-600, &600);
    let pos2 = client.get_position(&lp, &-1200, &1200);
    let pos3 = client.get_position(&lp, &0, &600);
    
    assert_eq!(pos1.liquidity, 0);
    assert_eq!(pos2.liquidity, 0);
    assert_eq!(pos3.liquidity, 0);
}

// Note: Full liquidity tests with actual token transfers require:
// 1. Token minting setup
// 2. Allowance/approval handling
// 3. Integration test environment
// These tests focus on validation logic