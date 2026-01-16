//! Factory events

use soroban_sdk::{Address, Env, Symbol};

/// Emitted when factory is initialized
pub fn emit_initialized(env: &Env, admin: &Address) {
    env.events().publish(
        (Symbol::new(env, "FactoryInit"),),
        (admin.clone(),),
    );
}

/// Emitted when a new pool is created
pub fn emit_pool_created(
    env: &Env,
    pool: &Address,
    token0: &Address,
    token1: &Address,
    creator: &Address,
    fee_bps: u32,
) {
    env.events().publish(
        (Symbol::new(env, "PoolCreated"),),
        (pool.clone(), token0.clone(), token1.clone(), creator.clone(), fee_bps),
    );
}

/// Emitted when creator locks liquidity
pub fn emit_creator_locked(
    env: &Env,
    pool: &Address,
    creator: &Address,
    liquidity: i128,
    lock_end: u32,
    is_permanent: bool,
) {
    env.events().publish(
        (Symbol::new(env, "CreatorLocked"),),
        (pool.clone(), creator.clone(), liquidity, lock_end, is_permanent),
    );
}

/// Emitted when creator unlocks liquidity
pub fn emit_creator_unlocked(env: &Env, pool: &Address, creator: &Address, liquidity: i128) {
    env.events().publish(
        (Symbol::new(env, "CreatorUnlocked"),),
        (pool.clone(), creator.clone(), liquidity),
    );
}

/// Emitted when creator fee is revoked (PERMANENT)
pub fn emit_creator_fee_revoked(env: &Env, pool: &Address, creator: &Address) {
    env.events().publish(
        (Symbol::new(env, "CreatorFeeRevoked"),),
        (pool.clone(), creator.clone()),
    );
}

/// Emitted when fee tier is updated
pub fn emit_fee_tier_updated(env: &Env, fee_bps: u32, tick_spacing: i32, enabled: bool) {
    env.events().publish(
        (Symbol::new(env, "FeeTierUpdated"),),
        (fee_bps, tick_spacing, enabled),
    );
}

/// Emitted when admin is updated
pub fn emit_admin_updated(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (Symbol::new(env, "AdminUpdated"),),
        (old_admin.clone(), new_admin.clone()),
    );
}