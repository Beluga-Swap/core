// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Stellar Soroban Contracts patterns
//
// Storage module following OpenZeppelin conventions:
// - Clear storage key definitions
// - Encapsulated storage access functions
// - Consistent error handling

use soroban_sdk::{contracttype, Address, Env};

use crate::types::{PoolConfig, PoolState, Position, TickInfo, TWAPObservation};

// ============================================================
// STORAGE KEYS (OpenZeppelin Style)
// ============================================================
// Following OpenZeppelin's pattern of using an enum for all storage keys
// This ensures type safety and prevents key collisions

/// All storage keys used in the contract
/// 
/// Using an enum for storage keys provides:
/// - Type safety (compiler checks)
/// - Clear documentation of all stored data
/// - Prevents key collision
#[contracttype]
pub enum DataKey {
    /// Pool state (prices, liquidity, fees)
    PoolState,
    /// Pool configuration (tokens, fee settings)
    PoolConfig,
    /// Initialization flag
    Initialized,
    /// Tick data by tick index
    Tick(i32),
    /// Position by (owner, lower_tick, upper_tick)
    Position(Address, i32, i32),
    /// TWAP observation by index
    TWAPObservation(u32),
    /// Newest TWAP observation index
    TWAPNewestIndex,
    /// TWAP initialization flag
    TWAPInitialized,
}

// ============================================================
// STORAGE CONFIGURATION
// ============================================================

/// Storage TTL constants (in ledgers)
/// Following Soroban best practices for persistent storage
pub mod storage_ttl {
    /// Default TTL for persistent storage (about 1 year at 5s per ledger)
    pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 6_307_200;
    /// Bump amount when extending TTL
    pub const PERSISTENT_BUMP_AMOUNT: u32 = 6_307_200;
}

// ============================================================
// INITIALIZATION STORAGE
// ============================================================

/// Check if pool is initialized
/// 
/// # Arguments
/// * `env` - Soroban environment
/// 
/// # Returns
/// `true` if the pool has been initialized
#[inline]
pub fn is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Initialized)
}

/// Mark pool as initialized
/// 
/// # Arguments
/// * `env` - Soroban environment
/// 
/// # Panics
/// Does not panic (idempotent operation)
pub fn set_initialized(env: &Env) {
    env.storage().persistent().set(&DataKey::Initialized, &true);
    extend_persistent_ttl(env, &DataKey::Initialized);
}

// ============================================================
// POOL CONFIG STORAGE
// ============================================================

/// Write pool configuration to storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `config` - Pool configuration to store
pub fn write_pool_config(env: &Env, config: &PoolConfig) {
    env.storage().persistent().set(&DataKey::PoolConfig, config);
    extend_persistent_ttl(env, &DataKey::PoolConfig);
}

/// Read pool configuration from storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// 
/// # Returns
/// Pool configuration
/// 
/// # Panics
/// Panics if pool is not initialized
pub fn read_pool_config(env: &Env) -> PoolConfig {
    env.storage()
        .persistent()
        .get(&DataKey::PoolConfig)
        .unwrap_or_else(|| panic!("pool not initialized"))
}

// ============================================================
// POOL STATE STORAGE
// ============================================================

/// Write pool state to storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `state` - Pool state to store
pub fn write_pool_state(env: &Env, state: &PoolState) {
    env.storage().persistent().set(&DataKey::PoolState, state);
    extend_persistent_ttl(env, &DataKey::PoolState);
}

/// Read pool state from storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// 
/// # Returns
/// Pool state
/// 
/// # Panics
/// Panics if pool is not initialized
pub fn read_pool_state(env: &Env) -> PoolState {
    env.storage()
        .persistent()
        .get(&DataKey::PoolState)
        .unwrap_or_else(|| panic!("pool not initialized"))
}

/// Initialize pool state with given parameters
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `sqrt_price_x64` - Initial sqrt price in Q64.64 format
/// * `current_tick` - Initial tick
/// * `tick_spacing` - Tick spacing for the pool
/// * `token0` - Sorted token0 address
/// * `token1` - Sorted token1 address
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

/// Write tick info to storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `tick` - Tick index
/// * `info` - Tick information to store
pub fn write_tick_info(env: &Env, tick: i32, info: &TickInfo) {
    let key = DataKey::Tick(tick);
    env.storage().persistent().set(&key, info);
    extend_persistent_ttl(env, &key);
}

/// Read tick info from storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `tick` - Tick index
/// 
/// # Returns
/// Tick information (returns default if not exists)
pub fn read_tick_info(env: &Env, tick: i32) -> TickInfo {
    env.storage()
        .persistent()
        .get(&DataKey::Tick(tick))
        .unwrap_or_default()
}

// ============================================================
// POSITION STORAGE
// ============================================================

/// Write position to storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `owner` - Position owner address
/// * `lower` - Lower tick boundary
/// * `upper` - Upper tick boundary
/// * `pos` - Position data to store
pub fn write_position(env: &Env, owner: &Address, lower: i32, upper: i32, pos: &Position) {
    let key = DataKey::Position(owner.clone(), lower, upper);
    env.storage().persistent().set(&key, pos);
    extend_persistent_ttl(env, &key);
}

/// Read position from storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `owner` - Position owner address
/// * `lower` - Lower tick boundary
/// * `upper` - Upper tick boundary
/// 
/// # Returns
/// Position data (returns default if not exists)
pub fn read_position(env: &Env, owner: &Address, lower: i32, upper: i32) -> Position {
    env.storage()
        .persistent()
        .get(&DataKey::Position(owner.clone(), lower, upper))
        .unwrap_or_default()
}

// ============================================================
// TWAP STORAGE (Reserved for future use)
// ============================================================

/// Write TWAP observation to storage
#[allow(dead_code)]
pub fn write_twap_observation(env: &Env, index: u32, obs: &TWAPObservation) {
    let key = DataKey::TWAPObservation(index);
    env.storage().persistent().set(&key, obs);
    extend_persistent_ttl(env, &key);
}

/// Read TWAP observation from storage
#[allow(dead_code)]
pub fn read_twap_observation(env: &Env, index: u32) -> TWAPObservation {
    env.storage()
        .persistent()
        .get(&DataKey::TWAPObservation(index))
        .unwrap_or_default()
}

/// Set newest TWAP index
#[allow(dead_code)]
pub fn set_twap_newest_index(env: &Env, index: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::TWAPNewestIndex, &index);
    extend_persistent_ttl(env, &DataKey::TWAPNewestIndex);
}

/// Get newest TWAP index
#[allow(dead_code)]
pub fn get_twap_newest_index(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::TWAPNewestIndex)
        .unwrap_or(0)
}

/// Check if TWAP is initialized
#[allow(dead_code)]
pub fn is_twap_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::TWAPInitialized)
}

/// Mark TWAP as initialized
#[allow(dead_code)]
pub fn set_twap_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&DataKey::TWAPInitialized, &true);
    extend_persistent_ttl(env, &DataKey::TWAPInitialized);
}

// ============================================================
// TTL MANAGEMENT (OpenZeppelin Pattern)
// ============================================================

/// Extend TTL for persistent storage key
/// 
/// This follows Soroban best practices for managing storage TTL
/// to ensure data persists appropriately.
fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(
        key,
        storage_ttl::PERSISTENT_LIFETIME_THRESHOLD,
        storage_ttl::PERSISTENT_BUMP_AMOUNT,
    );
}

/// Extend TTL for all critical storage keys
/// 
/// Call this periodically to ensure pool data persists
#[allow(dead_code)]
pub fn extend_all_ttl(env: &Env) {
    if is_initialized(env) {
        extend_persistent_ttl(env, &DataKey::Initialized);
        extend_persistent_ttl(env, &DataKey::PoolConfig);
        extend_persistent_ttl(env, &DataKey::PoolState);
    }
}