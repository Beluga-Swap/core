//! Factory events

use soroban_sdk::{Address, Env, Symbol};

/// Emitted when factory is initialized
pub fn emit_initialized(env: &Env, admin: &Address) {
    env.events().publish(
        (Symbol::new(env, "FactoryInit"),),
        (admin.clone(),),
    );
}

/// Emitted when router is set
pub fn emit_router_set(env: &Env, router: &Address) {
    env.events().publish(
        (Symbol::new(env, "RouterSet"),),
        (router.clone(),),
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