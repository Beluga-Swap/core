//! Factory storage operations

use soroban_sdk::{Address, Env, Vec};

use crate::types::{CreatorLock, DataKey, FactoryConfig, FeeTier};

// ============================================================
// TTL CONFIG
// ============================================================

const PERSISTENT_TTL: u32 = 6_307_200; // ~1 year

fn extend_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, PERSISTENT_TTL, PERSISTENT_TTL);
}

// ============================================================
// INITIALIZATION
// ============================================================

pub fn is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage().persistent().set(&DataKey::Initialized, &true);
    extend_ttl(env, &DataKey::Initialized);
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

pub fn increment_pool_count(env: &Env) {
    let count = read_pool_count(env) + 1;
    env.storage().persistent().set(&DataKey::PoolCount, &count);
}

pub fn read_pool_list(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::PoolList)
        .unwrap_or(Vec::new(env))
}

pub fn add_to_pool_list(env: &Env, pool: &Address) {
    let mut list = read_pool_list(env);
    list.push_back(pool.clone());
    env.storage().persistent().set(&DataKey::PoolList, &list);
}

pub fn init_pool_list(env: &Env) {
    env.storage()
        .persistent()
        .set(&DataKey::PoolList, &Vec::<Address>::new(env));
    env.storage().persistent().set(&DataKey::PoolCount, &0u32);
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