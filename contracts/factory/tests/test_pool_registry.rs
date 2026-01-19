mod common;

use soroban_sdk::Env;

#[test]
fn test_initial_pool_state() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    assert_eq!(client.get_total_pools(), 0);
    
    let pools = client.get_all_pool_addresses();
    assert_eq!(pools.len(), 0);
}

#[test]
fn test_pool_not_deployed() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    assert!(!client.is_pool_deployed(&token_a, &token_b, &30));
    assert!(client.get_pool_address(&token_a, &token_b, &30).is_none());
}

#[test]
fn test_token_sorting() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    // Both orders should give same result
    let result_ab = client.get_pool_address(&token_a, &token_b, &30);
    let result_ba = client.get_pool_address(&token_b, &token_a, &30);
    
    assert_eq!(result_ab, result_ba);
}

#[test]
fn test_different_fee_tiers_different_pools() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    
    // Same tokens, different fees = different pools
    assert!(!client.is_pool_deployed(&token_a, &token_b, &5));
    assert!(!client.is_pool_deployed(&token_a, &token_b, &30));
    assert!(!client.is_pool_deployed(&token_a, &token_b, &100));
}