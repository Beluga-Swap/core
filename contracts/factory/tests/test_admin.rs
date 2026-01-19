mod common;

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_admin_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _old_admin) = common::setup_factory(&env);
    let new_admin = Address::generate(&env);
    
    client.set_admin(&new_admin);
    
    // New admin should be able to modify fee tiers
    client.set_fee_tier(&15, &30, &true);
    
    let tier = client.get_fee_tier(&15);
    assert!(tier.is_some());
}

#[test]
fn test_update_pool_wasm_hash() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    let new_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    client.set_pool_wasm_hash(&new_hash);
    
    // Successfully updated (no panic)
}

#[test]
#[should_panic]
fn test_non_admin_cannot_set_fee_tier() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin) = common::setup_factory(&env);
    
    // Try to set fee tier without being admin
    // This should panic due to authorization failure
    let _non_admin = Address::generate(&env);
    env.mock_auths(&[]);
    
    client.set_fee_tier(&15, &30, &true);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_transfer_admin() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin) = common::setup_factory(&env);
    let new_admin = Address::generate(&env);
    
    // Try to transfer admin without being admin
    env.mock_auths(&[]);
    
    client.set_admin(&new_admin);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_update_wasm_hash() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin) = common::setup_factory(&env);
    let new_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // Try to update without being admin
    env.mock_auths(&[]);
    
    client.set_pool_wasm_hash(&new_hash);
}