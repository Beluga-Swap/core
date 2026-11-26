mod common;

use soroban_sdk::Env;

#[test]
fn test_initialization_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    // Check initial state
    assert_eq!(client.get_total_pools(), 0);
    assert_eq!(client.get_all_pool_addresses().len(), 0);
}
