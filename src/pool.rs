use soroban_sdk::{Env, Symbol, contracttype, Address};

use crate::DataKey;
use crate::math::{tick_to_sqrt_price_x64, snap_tick_to_spacing};

// ============================================================
// POOL STATE (DYNAMIC DATA)
// ============================================================
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolState {
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub liquidity: i128,
    pub tick_spacing: i32,
    pub token0: Address,
    pub token1: Address,

    // Global fee accumulators (Q64.64 format)
    pub fee_growth_global_0: u128,
    pub fee_growth_global_1: u128,

    // Protocol fee accounting
    pub protocol_fees_0: u128,
    pub protocol_fees_1: u128,
}

// ============================================================
// POOL CONFIG (STATIC DATA)
// ============================================================
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolConfig {
    pub admin: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub fee_bps: u32,          // Total fee in basis points (e.g., 30 for 0.3%)
    pub protocol_fee_bps: u32, // Protocol's cut of the fee (e.g., 10 for 10%)
}

// ============================================================
// STORAGE HELPERS
// ============================================================

pub fn read_pool_state(env: &Env) -> PoolState {
    env.storage()
        .persistent()
        .get::<_, PoolState>(&DataKey::PoolState)
        .expect("pool not initialized")
}

pub fn write_pool_state(env: &Env, state: &PoolState) {
    env.storage()
        .persistent()
        .set::<_, PoolState>(&DataKey::PoolState, state);
}

pub fn read_pool_config(env: &Env) -> PoolConfig {
    env.storage()
        .persistent()
        .get::<_, PoolConfig>(&DataKey::PoolConfig)
        .expect("pool config not set")
}

pub fn write_pool_config(env: &Env, cfg: &PoolConfig) {
    env.storage()
        .persistent()
        .set::<_, PoolConfig>(&DataKey::PoolConfig, cfg);
}

// ============================================================
// INITIALIZATION
// ============================================================

pub fn init_pool(
    env: &Env,
    _sqrt_price_x64: u128,
    initial_tick: i32,
    tick_spacing: i32,
    token0: Address,
    token1: Address,
) {
    if tick_spacing <= 0 {
        panic!("tick_spacing must be > 0");
    }

    let snapped_tick = snap_tick_to_spacing(initial_tick, tick_spacing);
    let sqrt_price_x64 = tick_to_sqrt_price_x64(env, snapped_tick);

    let state = PoolState {
        sqrt_price_x64,
        current_tick: snapped_tick,
        liquidity: 0,
        tick_spacing,
        token0,
        token1,
        fee_growth_global_0: 0,
        fee_growth_global_1: 0,
        protocol_fees_0: 0,
        protocol_fees_1: 0,
    };

    write_pool_state(env, &state);

    env.events().publish(
        (Symbol::new(env, "pool_init"),),
        (sqrt_price_x64, snapped_tick, tick_spacing),
    );
}