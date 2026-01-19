use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use beluga_factory::{BelugaFactory, BelugaFactoryClient, CreatePoolParams};

pub fn setup_factory(env: &Env) -> (BelugaFactoryClient<'_>, Address) {
    let admin = Address::generate(env);
    let factory_id = env.register_contract(None, BelugaFactory);
    let client = BelugaFactoryClient::new(env, &factory_id);
    let pool_wasm_hash = BytesN::from_array(env, &[0u8; 32]);
    client.initialize(&admin, &pool_wasm_hash);
    (client, admin)
}

pub fn create_token(env: &Env) -> Address {
    let admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    token_id.address()
}

pub fn default_pool_params(_env: &Env, token_a: &Address, token_b: &Address) -> CreatePoolParams {
    CreatePoolParams {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        fee_bps: 30,
        creator_fee_bps: 100,
        initial_sqrt_price_x64: 1u128 << 64,
        amount0_desired: 10_000_000,
        amount1_desired: 10_000_000,
        lower_tick: -600,
        upper_tick: 600,
        lock_duration: 120_960,
    }
}