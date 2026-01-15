// Factory events module for BelugaSwap

use soroban_sdk::{Address, Env, Symbol};

pub fn emit_factory_initialized(
    env: &Env,
    admin: &Address,
    pool_wasm_hash: &soroban_sdk::BytesN<32>,
    min_initial_liquidity: i128,
) {
    env.events().publish(
        (Symbol::new(env, "FactoryInit"),),
        (admin.clone(), pool_wasm_hash.clone(), min_initial_liquidity),
    );
}

pub fn emit_pool_created_simple(
    env: &Env,
    pool_address: &Address,
    token0: &Address,
    token1: &Address,
    creator: &Address,
    fee_bps: u32,
) {
    env.events().publish(
        (Symbol::new(env, "PoolCreated"),),
        (pool_address.clone(), token0.clone(), token1.clone(), creator.clone(), fee_bps),
    );
}

pub fn emit_liquidity_locked_simple(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    liquidity: i128,
    lower_tick: i32,
    upper_tick: i32,
    is_permanent: bool,
) {
    env.events().publish(
        (Symbol::new(env, "LiqLocked"),),
        (pool_address.clone(), owner.clone(), liquidity, lower_tick, upper_tick, is_permanent),
    );
}

pub fn emit_liquidity_unlocked(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    liquidity: i128,
    lower_tick: i32,
    upper_tick: i32,
) {
    env.events().publish(
        (Symbol::new(env, "LiqUnlocked"),),
        (pool_address.clone(), owner.clone(), liquidity, lower_tick, upper_tick),
    );
}

pub fn emit_creator_fees_claimed_simple(
    env: &Env,
    pool_address: &Address,
    creator: &Address,
    amount0: u128,
    amount1: u128,
    was_in_range: bool,
) {
    env.events().publish(
        (Symbol::new(env, "CreatorClaim"),),
        (pool_address.clone(), creator.clone(), amount0, amount1, was_in_range),
    );
}

pub fn emit_creator_rights_revoked(
    env: &Env,
    pool_address: &Address,
    creator: &Address,
    reason: &str,
) {
    env.events().publish(
        (Symbol::new(env, "CreatorRevoked"),),
        (pool_address.clone(), creator.clone(), Symbol::new(env, reason)),
    );
}

pub fn emit_fee_tier_updated(
    env: &Env,
    fee_bps: u32,
    tick_spacing: i32,
    enabled: bool,
) {
    env.events().publish(
        (Symbol::new(env, "TierUpdated"),),
        (fee_bps, tick_spacing, enabled),
    );
}

pub fn emit_admin_updated(
    env: &Env,
    old_admin: &Address,
    new_admin: &Address,
) {
    env.events().publish(
        (Symbol::new(env, "AdminUpd"),),
        (old_admin.clone(), new_admin.clone()),
    );
}