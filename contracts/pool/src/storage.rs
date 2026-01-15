// Factory storage module for BelugaSwap

use soroban_sdk::{contracttype, Address, Env};

use crate::types::{FactoryConfig, FactoryStats, FeeTier, LockedLiquidity, PoolInfo};

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum FactoryDataKey {
    /// Factory configuration
    Config,
    /// Initialization flag
    Initialized,
    /// Factory statistics
    Stats,
    /// Pool address by (token0, token1, fee_bps) - tokens must be sorted
    Pool(Address, Address, u32),
    /// Pool info by pool address
    PoolInfo(Address),
    /// Pool address by index (for enumeration)
    PoolByIndex(u32),
    /// Fee tier configuration by fee_bps
    FeeTier(u32),
    /// Locked liquidity position by (pool, owner, lower_tick, upper_tick)
    LockedLiquidity(Address, Address, i32, i32),
    /// Total locked liquidity count for creator in a pool
    CreatorPoolLockCount(Address, Address),
}

// ============================================================
// TTL CONFIGURATION
// ============================================================

/// Persistent storage lifetime in ledgers (~1 year at 5s/ledger)
const PERSISTENT_LIFETIME: u32 = 6_307_200;
/// TTL bump threshold
const PERSISTENT_BUMP: u32 = 6_307_200;

/// Extend TTL for a persistent storage key
fn extend_ttl(env: &Env, key: &FactoryDataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_LIFETIME, PERSISTENT_BUMP);
}

// ============================================================
// INITIALIZATION
// ============================================================

/// Check if factory is initialized
pub fn factory_is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .has(&FactoryDataKey::Initialized)
}

/// Set factory as initialized
pub fn factory_set_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&FactoryDataKey::Initialized, &true);
    extend_ttl(env, &FactoryDataKey::Initialized);
}

// ============================================================
// FACTORY CONFIG
// ============================================================

/// Write factory configuration
pub fn write_factory_config(env: &Env, config: &FactoryConfig) {
    env.storage()
        .persistent()
        .set(&FactoryDataKey::Config, config);
    extend_ttl(env, &FactoryDataKey::Config);
}

/// Read factory configuration
pub fn read_factory_config(env: &Env) -> FactoryConfig {
    env.storage()
        .persistent()
        .get(&FactoryDataKey::Config)
        .expect("factory not initialized")
}

// ============================================================
// FACTORY STATS
// ============================================================

/// Write factory statistics
pub fn write_factory_stats(env: &Env, stats: &FactoryStats) {
    env.storage()
        .persistent()
        .set(&FactoryDataKey::Stats, stats);
    extend_ttl(env, &FactoryDataKey::Stats);
}

/// Read factory statistics
pub fn read_factory_stats(env: &Env) -> FactoryStats {
    env.storage()
        .persistent()
        .get(&FactoryDataKey::Stats)
        .unwrap_or_default()
}

/// Increment pool count and return new total
pub fn increment_pool_count(env: &Env) -> u32 {
    let mut stats = read_factory_stats(env);
    stats.total_pools += 1;
    stats.active_pools += 1;
    write_factory_stats(env, &stats);
    stats.total_pools
}

/// Update total locked value
pub fn update_total_locked_value(env: &Env, delta: i128) {
    let mut stats = read_factory_stats(env);
    if delta > 0 {
        stats.total_locked_value = stats.total_locked_value.saturating_add(delta as u128);
    } else {
        stats.total_locked_value = stats.total_locked_value.saturating_sub(delta.unsigned_abs());
    }
    write_factory_stats(env, &stats);
}

// ============================================================
// POOL REGISTRY
// ============================================================

/// Get canonical pool key (sorted tokens)
pub fn get_pool_key(
    token_a: &Address,
    token_b: &Address,
    fee_bps: u32,
) -> (Address, Address, u32) {
    if token_a < token_b {
        (token_a.clone(), token_b.clone(), fee_bps)
    } else {
        (token_b.clone(), token_a.clone(), fee_bps)
    }
}

/// Check if pool exists for token pair and fee tier
pub fn pool_exists(env: &Env, token_a: &Address, token_b: &Address, fee_bps: u32) -> bool {
    let (t0, t1, fee) = get_pool_key(token_a, token_b, fee_bps);
    env.storage()
        .persistent()
        .has(&FactoryDataKey::Pool(t0, t1, fee))
}

/// Get pool address for token pair and fee tier
pub fn get_pool_address(
    env: &Env,
    token_a: &Address,
    token_b: &Address,
    fee_bps: u32,
) -> Option<Address> {
    let (t0, t1, fee) = get_pool_key(token_a, token_b, fee_bps);
    let key = FactoryDataKey::Pool(t0, t1, fee);
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}

/// Register a new pool
pub fn register_pool(
    env: &Env,
    token_a: &Address,
    token_b: &Address,
    fee_bps: u32,
    pool_address: &Address,
    pool_info: &PoolInfo,
) {
    let (t0, t1, fee) = get_pool_key(token_a, token_b, fee_bps);

    // Store pool address mapping
    let key = FactoryDataKey::Pool(t0, t1, fee);
    env.storage().persistent().set(&key, pool_address);
    extend_ttl(env, &key);

    // Store pool info
    let info_key = FactoryDataKey::PoolInfo(pool_address.clone());
    env.storage().persistent().set(&info_key, pool_info);
    extend_ttl(env, &info_key);

    // Store pool index mapping
    let index = increment_pool_count(env);
    let index_key = FactoryDataKey::PoolByIndex(index);
    env.storage().persistent().set(&index_key, pool_address);
    extend_ttl(env, &index_key);
}

/// Get pool info by address
pub fn get_pool_info(env: &Env, pool_address: &Address) -> Option<PoolInfo> {
    let key = FactoryDataKey::PoolInfo(pool_address.clone());
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}

/// Update pool info
pub fn update_pool_info(env: &Env, pool_address: &Address, pool_info: &PoolInfo) {
    let key = FactoryDataKey::PoolInfo(pool_address.clone());
    env.storage().persistent().set(&key, pool_info);
    extend_ttl(env, &key);
}

/// Get pool address by index
pub fn get_pool_by_index(env: &Env, index: u32) -> Option<Address> {
    let key = FactoryDataKey::PoolByIndex(index);
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}

/// Get total pool count
pub fn get_pool_count(env: &Env) -> u32 {
    read_factory_stats(env).total_pools
}

// ============================================================
// FEE TIERS
// ============================================================

/// Write fee tier configuration
pub fn write_fee_tier(env: &Env, fee_bps: u32, tier: &FeeTier) {
    let key = FactoryDataKey::FeeTier(fee_bps);
    env.storage().persistent().set(&key, tier);
    extend_ttl(env, &key);
}

/// Read fee tier configuration
pub fn read_fee_tier(env: &Env, fee_bps: u32) -> Option<FeeTier> {
    let key = FactoryDataKey::FeeTier(fee_bps);
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}

/// Check if fee tier is valid and enabled
pub fn is_valid_fee_tier(env: &Env, fee_bps: u32) -> bool {
    read_fee_tier(env, fee_bps)
        .map(|t| t.enabled)
        .unwrap_or(false)
}

/// Get tick spacing for a fee tier
pub fn get_tick_spacing_for_fee(env: &Env, fee_bps: u32) -> Option<i32> {
    read_fee_tier(env, fee_bps)
        .filter(|t| t.enabled)
        .map(|t| t.tick_spacing)
}

// ============================================================
// LOCKED LIQUIDITY
// ============================================================

/// Write locked liquidity position
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

/// Read locked liquidity position
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
    let result = env.storage().persistent().get(&key);
    if result.is_some() {
        extend_ttl(env, &key);
    }
    result
}

/// Delete locked liquidity position (after full unlock and withdrawal)
pub fn delete_locked_liquidity(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) {
    let key = FactoryDataKey::LockedLiquidity(
        pool_address.clone(),
        owner.clone(),
        lower_tick,
        upper_tick,
    );
    env.storage().persistent().remove(&key);
}

/// Check if liquidity is currently locked
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

/// Check if creator is eligible for fees (locked and in range)
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

/// Check if lock has expired
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
        if locked.is_unlocked {
            return true;
        }
        let current_ledger = env.ledger().sequence();
        current_ledger >= locked.lock_end
    } else {
        true
    }
}

/// Get remaining lock time in ledgers
pub fn get_remaining_lock_time(
    env: &Env,
    pool_address: &Address,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) -> Option<u32> {
    if let Some(locked) = read_locked_liquidity(env, pool_address, owner, lower_tick, upper_tick) {
        if locked.is_permanent {
            return Some(u32::MAX);
        }
        if locked.is_unlocked {
            return Some(0);
        }
        let current_ledger = env.ledger().sequence();
        if current_ledger >= locked.lock_end {
            Some(0)
        } else {
            Some(locked.lock_end - current_ledger)
        }
    } else {
        None
    }
}

// ============================================================
// CREATOR TRACKING
// ============================================================

/// Update creator's total locked liquidity in a pool
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
        current.saturating_sub(delta.unsigned_abs() as i128)
    };
    env.storage().persistent().set(&key, &new_total);
    extend_ttl(env, &key);
}

/// Get creator's total locked liquidity in a pool
pub fn get_creator_total_locked(env: &Env, pool_address: &Address, creator: &Address) -> i128 {
    let key = FactoryDataKey::CreatorPoolLockCount(pool_address.clone(), creator.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}