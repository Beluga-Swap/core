mod common;

use soroban_sdk::Env;

#[test]
fn test_pool_initialized_state() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    
    // Pool should be initialized
    assert!(client.is_initialized());
    
    // Get state to verify
    let state = client.get_pool_state();
    assert_eq!(state.liquidity, 0); // No liquidity yet
}