// SPDX-License-Identifier: MIT
// Pool Storage

use soroban_sdk::{contracttype, Address, Env};
use belugaswap_tick::TickInfo;
use belugaswap_position::Position;
use crate::types::{PoolConfig, PoolState, TWAPObservation};

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum DataKey {
    PoolState,
    PoolConfig,
    Initialized,
    Tick(i32),
    Position(Address, i32, i32),
    TWAPObservation(u32),
    TWAPNewestIndex,
    TWAPInitialized,
}

// ============================================================
// STORAGE CONFIGURATION
// ============================================================

pub mod storage_ttl {
    pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 6_307_200;
    pub const PERSISTENT_BUMP_AMOUNT: u32 = 6_307_200;
}

// ============================================================
// INITIALIZATION STORAGE
// ============================================================

#[inline]
pub fn is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage().persistent().set(&DataKey::Initialized, &true);
    extend_persistent_ttl(env, &DataKey::Initialized);
}

// ============================================================
// POOL CONFIG STORAGE
// ============================================================

pub fn write_pool_config(env: &Env, config: &PoolConfig) {
    env.storage().persistent().set(&DataKey::PoolConfig, config);
    extend_persistent_ttl(env, &DataKey::PoolConfig);
}

pub fn read_pool_config(env: &Env) -> PoolConfig {
    env.storage()
        .persistent()
        .get(&DataKey::PoolConfig)
        .unwrap_or_else(|| panic!("pool not initialized"))
}

// ============================================================
// POOL STATE STORAGE
// ============================================================

pub fn write_pool_state(env: &Env, state: &PoolState) {
    env.storage().persistent().set(&DataKey::PoolState, state);
    extend_persistent_ttl(env, &DataKey::PoolState);
}

pub fn read_pool_state(env: &Env) -> PoolState {
    env.storage()
        .persistent()
        .get(&DataKey::PoolState)
        .unwrap_or_else(|| panic!("pool not initialized"))
}

pub fn init_pool_state(
    env: &Env,
    sqrt_price_x64: u128,
    current_tick: i32,
    tick_spacing: i32,
    token0: Address,
    token1: Address,
) {
    let state = PoolState {
        sqrt_price_x64,
        current_tick,
        liquidity: 0,
        tick_spacing,
        token0,
        token1,
        fee_growth_global_0: 0,
        fee_growth_global_1: 0,
        creator_fees_0: 0,
        creator_fees_1: 0,
    };

    write_pool_state(env, &state);
}

// ============================================================
// TICK STORAGE
// ============================================================

pub fn write_tick_info(env: &Env, tick: i32, info: &TickInfo) {
    let key = DataKey::Tick(tick);
    env.storage().persistent().set(&key, info);
    extend_persistent_ttl(env, &key);
}

pub fn read_tick_info(env: &Env, tick: i32) -> TickInfo {
    env.storage()
        .persistent()
        .get(&DataKey::Tick(tick))
        .unwrap_or_default()
}

// ============================================================
// POSITION STORAGE
// ============================================================

pub fn write_position(env: &Env, owner: &Address, lower: i32, upper: i32, pos: &Position) {
    let key = DataKey::Position(owner.clone(), lower, upper);
    env.storage().persistent().set(&key, pos);
    extend_persistent_ttl(env, &key);
}

pub fn read_position(env: &Env, owner: &Address, lower: i32, upper: i32) -> Position {
    env.storage()
        .persistent()
        .get(&DataKey::Position(owner.clone(), lower, upper))
        .unwrap_or_default()
}

// ============================================================
// TWAP STORAGE (Reserved for future use)
// ============================================================

#[allow(dead_code)]
pub fn write_twap_observation(env: &Env, index: u32, obs: &TWAPObservation) {
    let key = DataKey::TWAPObservation(index);
    env.storage().persistent().set(&key, obs);
    extend_persistent_ttl(env, &key);
}

#[allow(dead_code)]
pub fn read_twap_observation(env: &Env, index: u32) -> TWAPObservation {
    env.storage()
        .persistent()
        .get(&DataKey::TWAPObservation(index))
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn set_twap_newest_index(env: &Env, index: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::TWAPNewestIndex, &index);
    extend_persistent_ttl(env, &DataKey::TWAPNewestIndex);
}

#[allow(dead_code)]
pub fn get_twap_newest_index(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::TWAPNewestIndex)
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn is_twap_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::TWAPInitialized)
}

#[allow(dead_code)]
pub fn set_twap_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&DataKey::TWAPInitialized, &true);
    extend_persistent_ttl(env, &DataKey::TWAPInitialized);
}

// ============================================================
// TTL MANAGEMENT
// ============================================================

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(
        key,
        storage_ttl::PERSISTENT_LIFETIME_THRESHOLD,
        storage_ttl::PERSISTENT_BUMP_AMOUNT,
    );
}

#[allow(dead_code)]
pub fn extend_all_ttl(env: &Env) {
    if is_initialized(env) {
        extend_persistent_ttl(env, &DataKey::Initialized);
        extend_persistent_ttl(env, &DataKey::PoolConfig);
        extend_persistent_ttl(env, &DataKey::PoolState);
    }
}