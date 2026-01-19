mod common;

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use beluga_factory::{BelugaFactory, BelugaFactoryClient};

#[test]
fn test_initialization_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    // Check initial state
    assert_eq!(client.get_total_pools(), 0);
    assert_eq!(client.get_all_pool_addresses().len(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let factory_id = env.register_contract(None, BelugaFactory);
    let client = BelugaFactoryClient::new(&env, &factory_id);
    let hash = BytesN::from_array(&env, &[0u8; 32]);
    
    client.initialize(&admin, &hash);
    client.initialize(&admin, &hash); // Should panic with AlreadyInitialized
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_create_pool_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Don't initialize factory
    let factory_id = env.register_contract(None, BelugaFactory);
    let client = BelugaFactoryClient::new(&env, &factory_id);
    
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env);
    let token_b = common::create_token(&env);
    let params = common::default_pool_params(&env, &token_a, &token_b);
    
    client.create_pool(&creator, &params); // Should fail: NotInitialized
}