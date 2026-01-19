mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_nonexistent_creator_lock() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let fake_pool = Address::generate(&env);
    let fake_creator = Address::generate(&env);
    
    let lock = client.get_creator_lock(&fake_pool, &fake_creator);
    assert!(lock.is_none());
}

#[test]
#[should_panic(expected = "Error(Contract, #32)")] // ← Changed from #22 to #32
fn test_unlock_nonexistent_lock() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let fake_pool = Address::generate(&env);
    let creator = Address::generate(&env);
    
    client.unlock_creator_liquidity(&fake_pool, &creator); // Should fail
}