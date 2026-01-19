use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use beluga_factory::{BelugaFactory, BelugaFactoryClient, CreatePoolParams};

fn setup_factory(env: &Env) -> (BelugaFactoryClient, Address) {
    let admin = Address::generate(env);
    let factory_id = env.register_contract(None, BelugaFactory);
    let client = BelugaFactoryClient::new(env, &factory_id);
    let pool_wasm_hash = BytesN::from_array(env, &[0u8; 32]);
    client.initialize(&admin, &pool_wasm_hash);
    (client, admin)
}

#[test]
fn test_init() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_factory(&env);
    assert_eq!(client.get_total_pools(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // ← Ganti ini!
fn test_double_init() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let factory_id = env.register_contract(None, BelugaFactory);
    let client = BelugaFactoryClient::new(&env, &factory_id);
    let hash = BytesN::from_array(&env, &[0u8; 32]);
    client.initialize(&admin, &hash);
    client.initialize(&admin, &hash);
}

#[test]
fn test_fee_tiers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_factory(&env);
    
    // Check default fee tiers
    assert!(client.get_fee_tier(&5).is_some());
    assert!(client.get_fee_tier(&30).is_some());
    assert!(client.get_fee_tier(&100).is_some());
}

#[test]
fn test_pool_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_factory(&env);
    
    assert_eq!(client.get_total_pools(), 0);
    let pools = client.get_all_pool_addresses();
    assert_eq!(pools.len(), 0);
}
