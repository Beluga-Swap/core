// SPDX-License-Identifier: MIT
// Pool Storage

use soroban_sdk::{contracttype, panic_with_error, Address, Env};
use crate::error::PoolError;
use belugaswap_tick::{TickInfo, TickStore};
use belugaswap_position::Position;
use crate::types::{PoolConfig, PoolState};

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum DataKey {
    PoolState,
    PoolConfig,
    Tick(i32),
    Position(Address, i32, i32),
    TickBitmap(i32),
    TickBitmapSummary(i32),
}

// ============================================================
// STORAGE CONFIGURATION
// ============================================================

const DAY_IN_LEDGERS: u32 = 17_280;

/// Entries are extended to `TTL_BUMP` once fewer than `TTL_THRESHOLD` ledgers
/// remain, so an active entry is re-extended about once a day instead of on
/// every access.
const TTL_THRESHOLD: u32 = 119 * DAY_IN_LEDGERS;
const TTL_BUMP: u32 = 120 * DAY_IN_LEDGERS;

/// Keeps the contract instance and its code alive.
pub fn extend_instance(env: &Env) {
    env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_BUMP);
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
        .unwrap_or_else(|| panic_with_error!(env, PoolError::NotInitialized))
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
        .unwrap_or_else(|| panic_with_error!(env, PoolError::NotInitialized))
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
// TICK BITMAP STORAGE
// ============================================================

fn read_bits(env: &Env, key: DataKey) -> u128 {
    env.storage().persistent().get(&key).unwrap_or(0)
}

fn write_bits(env: &Env, key: DataKey, bits: u128) {
    env.storage().persistent().set(&key, &bits);
    extend_persistent_ttl(env, &key);
}

pub fn read_bitmap_word(env: &Env, pos: i32) -> u128 {
    read_bits(env, DataKey::TickBitmap(pos))
}

pub fn write_bitmap_word(env: &Env, pos: i32, bits: u128) {
    write_bits(env, DataKey::TickBitmap(pos), bits)
}

pub fn read_bitmap_summary(env: &Env, pos: i32) -> u128 {
    read_bits(env, DataKey::TickBitmapSummary(pos))
}

pub fn write_bitmap_summary(env: &Env, pos: i32, bits: u128) {
    write_bits(env, DataKey::TickBitmapSummary(pos), bits)
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
// TTL MANAGEMENT
// ============================================================

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(
        key,
        TTL_THRESHOLD,
        TTL_BUMP,
    );
}

// ============================================================
// TICK STORE
// ============================================================

/// The pool's ticks and initialized-tick bitmap, for the tick and swap packages.
pub struct PoolTicks {
    pub tick_spacing: i32,
}

impl TickStore for PoolTicks {
    fn read_tick(&self, env: &Env, tick: i32) -> TickInfo {
        read_tick_info(env, tick)
    }

    fn write_tick(&self, env: &Env, tick: i32, info: &TickInfo) {
        write_tick_info(env, tick, info)
    }

    fn flip_tick(&self, env: &Env, tick: i32) {
        belugaswap_tick::flip_tick(
            env,
            read_bitmap_word,
            write_bitmap_word,
            read_bitmap_summary,
            write_bitmap_summary,
            tick,
            self.tick_spacing,
        )
    }

    fn next_initialized_tick(&self, env: &Env, tick: i32, zero_for_one: bool) -> Option<i32> {
        belugaswap_tick::next_initialized_tick(
            env,
            read_bitmap_word,
            read_bitmap_summary,
            tick,
            self.tick_spacing,
            zero_for_one,
        )
    }
}
