//! Factory storage operations

use soroban_sdk::{Address, Env, Vec};

use crate::types::{CreatorLock, DataKey, FactoryConfig, FeeTier};

// ============================================================
// TTL CONFIG
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

fn extend_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, TTL_THRESHOLD, TTL_BUMP);
}

// ============================================================
// CONFIG
// ============================================================

pub fn write_config(env: &Env, config: &FactoryConfig) {
    env.storage().persistent().set(&DataKey::Config, config);
    extend_ttl(env, &DataKey::Config);
}

pub fn read_config(env: &Env) -> FactoryConfig {
    let key = DataKey::Config;
    let config = env.storage().persistent().get(&key).unwrap();
    extend_ttl(env, &key);
    config
}

// ============================================================
// FEE TIERS
// ============================================================

pub fn write_fee_tier(env: &Env, fee_bps: u32, tier: &FeeTier) {
    let key = DataKey::FeeTier(fee_bps);
    env.storage().persistent().set(&key, tier);
    extend_ttl(env, &key);
}

pub fn read_fee_tier(env: &Env, fee_bps: u32) -> Option<FeeTier> {
    let key = DataKey::FeeTier(fee_bps);
    let result: Option<FeeTier> = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}

// ============================================================
// POOL REGISTRY
// ============================================================

/// Get canonical token order (sorted)
pub fn sort_tokens(token_a: &Address, token_b: &Address) -> (Address, Address) {
    if token_a < token_b {
        (token_a.clone(), token_b.clone())
    } else {
        (token_b.clone(), token_a.clone())
    }
}

pub fn pool_exists(env: &Env, token0: &Address, token1: &Address, fee_bps: u32) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Pool(token0.clone(), token1.clone(), fee_bps))
}

pub fn write_pool(env: &Env, token0: &Address, token1: &Address, fee_bps: u32, pool: &Address) {
    let key = DataKey::Pool(token0.clone(), token1.clone(), fee_bps);
    env.storage().persistent().set(&key, pool);
    extend_ttl(env, &key);
}

pub fn read_pool(env: &Env, token0: &Address, token1: &Address, fee_bps: u32) -> Option<Address> {
    let key = DataKey::Pool(token0.clone(), token1.clone(), fee_bps);
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}

pub fn read_pool_count(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::PoolCount)
        .unwrap_or(0)
}

/// Largest page returned in one call, well within the transaction footprint.
pub const MAX_POOL_PAGE: u32 = 100;

pub fn read_pool_page(env: &Env, start: u32, limit: u32) -> Vec<Address> {
    let end = read_pool_count(env).min(start.saturating_add(limit.min(MAX_POOL_PAGE)));
    let mut page = Vec::new(env);
    for index in start..end {
        if let Some(pool) = env.storage().persistent().get(&DataKey::PoolAt(index)) {
            page.push_back(pool);
        }
    }
    page
}

/// Append a pool to the registry and bump the count.
pub fn add_to_pool_list(env: &Env, pool: &Address) {
    let count = read_pool_count(env);
    let key = DataKey::PoolAt(count);
    env.storage().persistent().set(&key, pool);
    extend_ttl(env, &key);
    write_pool_count(env, count + 1);
}

fn write_pool_count(env: &Env, count: u32) {
    env.storage().persistent().set(&DataKey::PoolCount, &count);
    extend_ttl(env, &DataKey::PoolCount);
}

pub fn init_pool_list(env: &Env) {
    write_pool_count(env, 0);
}

// ============================================================
// CREATOR LOCK
// ============================================================

pub fn write_creator_lock(env: &Env, pool: &Address, creator: &Address, lock: &CreatorLock) {
    let key = DataKey::CreatorLock(pool.clone(), creator.clone());
    env.storage().persistent().set(&key, lock);
    extend_ttl(env, &key);
}

pub fn read_creator_lock(env: &Env, pool: &Address, creator: &Address) -> Option<CreatorLock> {
    let key = DataKey::CreatorLock(pool.clone(), creator.clone());
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}