// Factory events module for BelugaSwap
// All events use compact names to reduce storage/gas costs

use soroban_sdk::{Address, Env, Symbol};

/// Emitted when the factory is initialized
/// Topics: ("FactoryInit",)
/// Data: (admin, pool_wasm_hash, min_initial_liquidity)
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

/// Emitted when a new pool is created
/// Topics: ("PoolCreated",)
/// Data: (pool_address, token0, token1, creator, fee_bps)
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
        (
            pool_address.clone(),
            token0.clone(),
            token1.clone(),
            creator.clone(),
            fee_bps,
        ),
    );
}

/// Emitted with full pool creation details
/// Topics: ("PoolCreatedFull",)
/// Data: (pool_address, token0, token1, creator, fee_bps, creator_fee_bps, tick_spacing, sqrt_price)
pub fn emit_pool_created_full(
    env: &Env,
    pool_address: &Address,
    token0: &Address,
    token1: &Address,
    creator: &Address,
    fee_bps: u32,
    creator_fee_bps: u32,
    tick_spacing: i32,
    initial_sqrt_price: u128,
) {
    env.events().publish(
        (Symbol::new(env, "PoolCreatedFull"),),
        (
            pool_address.clone(),
            token0.clone(),
            token1.clone(),
            creator.clone(),
            fee_bps,
            creator_fee_bps,
            tick_spacing,
            initial_sqrt_price,
        ),
    );
}

/// Emitted when liquidity is locked
/// Topics: ("LiqLocked",)
/// Data: (pool_address, owner, liquidity, lower_tick, upper_tick, is_permanent)
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
        (
            pool_address.clone(),
            owner.clone(),
            liquidity,
            lower_tick,
            upper_tick,
            is_permanent,
        ),
    );
}

/// Emitted with full lock details
/// Topics: ("LiqLockedFull",)
/// Data: (pool_address, owner, liquidity, lower_tick, upper_tick, lock_start, lock_end, is_permanent, amount0, amount1)
pub fn emit_liquidity_locked_full(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    liquidity: i128,
    lower_tick: i32,
    upper_tick: i32,
    lock_start: u32,
    lock_end: u32,
    is_permanent: bool,
    amount0: i128,
    amount1: i128,
) {
    env.events().publish(
        (Symbol::new(env, "LiqLockedFull"),),
        (
            pool_address.clone(),
            owner.clone(),
            liquidity,
            lower_tick,
            upper_tick,
            lock_start,
            lock_end,
            is_permanent,
            amount0,
            amount1,
        ),
    );
}

/// Emitted when liquidity is unlocked
/// Topics: ("LiqUnlocked",)
/// Data: (pool_address, owner, liquidity, lower_tick, upper_tick)
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
        (
            pool_address.clone(),
            owner.clone(),
            liquidity,
            lower_tick,
            upper_tick,
        ),
    );
}

/// Emitted when lock duration is extended
/// Topics: ("LockExtended",)
/// Data: (pool_address, owner, lower_tick, upper_tick, new_lock_end)
pub fn emit_lock_extended(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
    new_lock_end: u32,
) {
    env.events().publish(
        (Symbol::new(env, "LockExtended"),),
        (
            pool_address.clone(),
            owner.clone(),
            lower_tick,
            upper_tick,
            new_lock_end,
        ),
    );
}

/// Emitted when a timed lock is converted to permanent
/// Topics: ("LockPermanent",)
/// Data: (pool_address, owner, lower_tick, upper_tick)
pub fn emit_lock_made_permanent(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) {
    env.events().publish(
        (Symbol::new(env, "LockPermanent"),),
        (pool_address.clone(), owner.clone(), lower_tick, upper_tick),
    );
}

/// Emitted when creator claims fees
/// Topics: ("CreatorClaim",)
/// Data: (pool_address, creator, amount0, amount1, was_in_range)
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
        (
            pool_address.clone(),
            creator.clone(),
            amount0,
            amount1,
            was_in_range,
        ),
    );
}

/// Emitted when creator fee rights are revoked
/// Topics: ("CreatorRevoked",)
/// Data: (pool_address, creator, reason)
pub fn emit_creator_rights_revoked(
    env: &Env,
    pool_address: &Address,
    creator: &Address,
    reason: &str,
) {
    env.events().publish(
        (Symbol::new(env, "CreatorRevoked"),),
        (
            pool_address.clone(),
            creator.clone(),
            Symbol::new(env, reason),
        ),
    );
}

/// Emitted when a fee tier is added or updated
/// Topics: ("TierUpdated",)
/// Data: (fee_bps, tick_spacing, enabled)
pub fn emit_fee_tier_updated(env: &Env, fee_bps: u32, tick_spacing: i32, enabled: bool) {
    env.events().publish(
        (Symbol::new(env, "TierUpdated"),),
        (fee_bps, tick_spacing, enabled),
    );
}

/// Emitted when admin is updated
/// Topics: ("AdminUpd",)
/// Data: (old_admin, new_admin)
pub fn emit_admin_updated(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (Symbol::new(env, "AdminUpd"),),
        (old_admin.clone(), new_admin.clone()),
    );
}

/// Emitted when factory config is updated
/// Topics: ("ConfigUpd",)
/// Data: (param_name, new_value)
pub fn emit_config_updated(env: &Env, param_name: &str, new_value: i128) {
    env.events().publish(
        (Symbol::new(env, "ConfigUpd"),),
        (Symbol::new(env, param_name), new_value),
    );
}

/// Emitted when pool WASM hash is updated
/// Topics: ("WasmUpdated",)
/// Data: (new_hash)
pub fn emit_wasm_hash_updated(env: &Env, new_hash: &soroban_sdk::BytesN<32>) {
    env.events().publish(
        (Symbol::new(env, "WasmUpdated"),),
        (new_hash.clone(),),
    );
}