// Factory storage module for BelugaSwap

use soroban_sdk::{contracttype, Address, Env};

use crate::types::{FactoryConfig, FactoryStats, FeeTier, LockedLiquidity, PoolInfo};

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum FactoryDataKey {
    Config,
    Initialized,
    Stats,
    Pool(Address, Address, u32),
    PoolInfo(Address),
    PoolByIndex(u32),
    FeeTier(u32),
    LockedLiquidity(Address, Address, i32, i32),
    CreatorPoolLockCount(Address, Address),
}

// ============================================================
// TTL CONFIGURATION
// ============================================================

const PERSISTENT_LIFETIME: u32 = 6_307_200;
const PERSISTENT_BUMP: u32 = 6_307_200;

fn extend_ttl(env: &Env, key: &FactoryDataKey) {
    env.storage().persistent().extend_ttl(key, PERSISTENT_LIFETIME, PERSISTENT_BUMP);
}

// ============================================================
// INITIALIZATION
// ============================================================

pub fn factory_is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&FactoryDataKey::Initialized)
}

pub fn factory_set_initialized(env: &Env) {
    env.storage().persistent().set(&FactoryDataKey::Initialized, &true);
    extend_ttl(env, &FactoryDataKey::Initialized);
}

// ============================================================
// FACTORY CONFIG
// ============================================================

pub fn write_factory_config(env: &Env, config: &FactoryConfig) {
    env.storage().persistent().set(&FactoryDataKey::Config, config);
    extend_ttl(env, &FactoryDataKey::Config);
}

pub fn read_factory_config(env: &Env) -> FactoryConfig {
    env.storage()
        .persistent()
        .get(&FactoryDataKey::Config)
        .expect("factory not initialized")
}

// ============================================================
// FACTORY STATS
// ============================================================

pub fn write_factory_stats(env: &Env, stats: &FactoryStats) {
    env.storage().persistent().set(&FactoryDataKey::Stats, stats);
    extend_ttl(env, &FactoryDataKey::Stats);
}

pub fn read_factory_stats(env: &Env) -> FactoryStats {
    env.storage()
        .persistent()
        .get(&FactoryDataKey::Stats)
        .unwrap_or_default()
}

pub fn increment_pool_count(env: &Env) -> u32 {
    let mut stats = read_factory_stats(env);
    stats.total_pools += 1;
    stats.active_pools += 1;
    write_factory_stats(env, &stats);
    stats.total_pools
}

// ============================================================
// POOL REGISTRY
// ============================================================

pub fn get_pool_key(token_a: &Address, token_b: &Address, fee_bps: u32) -> (Address, Address, u32) {
    if token_a < token_b {
        (token_a.clone(), token_b.clone(), fee_bps)
    } else {
        (token_b.clone(), token_a.clone(), fee_bps)
    }
}

pub fn pool_exists(env: &Env, token_a: &Address, token_b: &Address, fee_bps: u32) -> bool {
    let (t0, t1, fee) = get_pool_key(token_a, token_b, fee_bps);
    env.storage().persistent().has(&FactoryDataKey::Pool(t0, t1, fee))
}

pub fn get_pool_address(env: &Env, token_a: &Address, token_b: &Address, fee_bps: u32) -> Option<Address> {
    let (t0, t1, fee) = get_pool_key(token_a, token_b, fee_bps);
    env.storage().persistent().get(&FactoryDataKey::Pool(t0, t1, fee))
}

pub fn register_pool(
    env: &Env,
    token_a: &Address,
    token_b: &Address,
    fee_bps: u32,
    pool_address: &Address,
    pool_info: &PoolInfo,
) {
    let (t0, t1, fee) = get_pool_key(token_a, token_b, fee_bps);
    
    let key = FactoryDataKey::Pool(t0, t1, fee);
    env.storage().persistent().set(&key, pool_address);
    extend_ttl(env, &key);
    
    let info_key = FactoryDataKey::PoolInfo(pool_address.clone());
    env.storage().persistent().set(&info_key, pool_info);
    extend_ttl(env, &info_key);
    
    let index = increment_pool_count(env);
    let index_key = FactoryDataKey::PoolByIndex(index);
    env.storage().persistent().set(&index_key, pool_address);
    extend_ttl(env, &index_key);
}

pub fn get_pool_info(env: &Env, pool_address: &Address) -> Option<PoolInfo> {
    env.storage().persistent().get(&FactoryDataKey::PoolInfo(pool_address.clone()))
}

pub fn get_pool_count(env: &Env) -> u32 {
    read_factory_stats(env).total_pools
}

// ============================================================
// FEE TIERS
// ============================================================

pub fn write_fee_tier(env: &Env, fee_bps: u32, tier: &FeeTier) {
    let key = FactoryDataKey::FeeTier(fee_bps);
    env.storage().persistent().set(&key, tier);
    extend_ttl(env, &key);
}

pub fn read_fee_tier(env: &Env, fee_bps: u32) -> Option<FeeTier> {
    env.storage().persistent().get(&FactoryDataKey::FeeTier(fee_bps))
}

pub fn is_valid_fee_tier(env: &Env, fee_bps: u32) -> bool {
    read_fee_tier(env, fee_bps)
        .map(|t| t.enabled)
        .unwrap_or(false)
}

pub fn get_tick_spacing_for_fee(env: &Env, fee_bps: u32) -> Option<i32> {
    read_fee_tier(env, fee_bps).map(|t| t.tick_spacing)
}

// ============================================================
// LOCKED LIQUIDITY
// ============================================================

pub fn write_locked_liquidity(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
    locked: &LockedLiquidity,
) {
    let key = FactoryDataKey::LockedLiquidity(
        pool_address.clone(),
        owner.clone(),
        lower_tick,
        upper_tick,
    );
    env.storage().persistent().set(&key, locked);
    extend_ttl(env, &key);
}

pub fn read_locked_liquidity(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) -> Option<LockedLiquidity> {
    let key = FactoryDataKey::LockedLiquidity(
        pool_address.clone(),
        owner.clone(),
        lower_tick,
        upper_tick,
    );
    env.storage().persistent().get(&key)
}

pub fn is_liquidity_locked(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) -> bool {
    if let Some(locked) = read_locked_liquidity(env, pool_address, owner, lower_tick, upper_tick) {
        if locked.is_unlocked {
            return false;
        }
        if locked.is_permanent {
            return true;
        }
        let current_ledger = env.ledger().sequence();
        current_ledger < locked.lock_end
    } else {
        false
    }
}

pub fn is_creator_fee_eligible(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) -> bool {
    if let Some(locked) = read_locked_liquidity(env, pool_address, owner, lower_tick, upper_tick) {
        if locked.is_unlocked {
            return false;
        }
        if locked.is_permanent {
            return true;
        }
        let current_ledger = env.ledger().sequence();
        current_ledger < locked.lock_end
    } else {
        false
    }
}

pub fn is_lock_expired(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) -> bool {
    if let Some(locked) = read_locked_liquidity(env, pool_address, owner, lower_tick, upper_tick) {
        if locked.is_permanent {
            return false;
        }
        let current_ledger = env.ledger().sequence();
        current_ledger >= locked.lock_end
    } else {
        true
    }
}

pub fn update_creator_total_locked(
    env: &Env,
    pool_address: &Address,
    creator: &Address,
    delta: i128,
) {
    let key = FactoryDataKey::CreatorPoolLockCount(pool_address.clone(), creator.clone());
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new_total = if delta > 0 {
        current.saturating_add(delta)
    } else {
        current.saturating_sub(delta.abs())
    };
    env.storage().persistent().set(&key, &new_total);
    extend_ttl(env, &key);
}