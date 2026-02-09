use soroban_sdk::{testutils::Address as _, Address, Env};
use belugaswap_pool::{BelugaPool, BelugaPoolClient};

// Test constants
pub const DEFAULT_FEE_BPS: u32 = 30; // 0.30%
pub const DEFAULT_CREATOR_FEE_BPS: u32 = 100; // 1%
pub const DEFAULT_TICK_SPACING: i32 = 60;
pub const DEFAULT_SQRT_PRICE_X64: u128 = 1u128 << 64; // Price = 1.0
pub const DEFAULT_TICK: i32 = 0;

/// Setup pool with default parameters
pub fn setup_pool(env: &Env) -> (BelugaPoolClient<'_>, Address, Address, Address, Address, Address) {
    let creator = Address::generate(env);
    let factory = Address::generate(env);
    let router = Address::generate(env);
    let token_a = create_token(env, &creator);
    let token_b = create_token(env, &creator);
    
    let pool_id = env.register(BelugaPool, ());
    let client = BelugaPoolClient::new(env, &pool_id);
    
    client.initialize(
        &factory,
        &router,
        &creator,
        &token_a,
        &token_b,
        &DEFAULT_FEE_BPS,
        &DEFAULT_CREATOR_FEE_BPS,
        &DEFAULT_SQRT_PRICE_X64,
        &DEFAULT_TICK,
        &DEFAULT_TICK_SPACING,
    );
    
    (client, creator, factory, router, token_a, token_b)
}

/// Setup pool with custom parameters
pub fn setup_custom_pool(
    env: &Env,
    fee_bps: u32,
    creator_fee_bps: u32,
    sqrt_price_x64: u128,
    current_tick: i32,
    tick_spacing: i32,
) -> (BelugaPoolClient<'_>, Address, Address, Address, Address, Address) {
    let creator = Address::generate(env);
    let factory = Address::generate(env);
    let router = Address::generate(env);
    let token_a = create_token(env, &creator);
    let token_b = create_token(env, &creator);
    
    let pool_id = env.register(BelugaPool, ());
    let client = BelugaPoolClient::new(env, &pool_id);
    
    client.initialize(
        &factory,
        &router,
        &creator,
        &token_a,
        &token_b,
        &fee_bps,
        &creator_fee_bps,
        &sqrt_price_x64,
        &current_tick,
        &tick_spacing,
    );
    
    (client, creator, factory, router, token_a, token_b)
}

/// Create a test token
pub fn create_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    token_id.address()
}

/// Mint tokens to an address
pub fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    use soroban_sdk::token::StellarAssetClient;
    let client = StellarAssetClient::new(env, token);
    client.mint(to, &amount);
}