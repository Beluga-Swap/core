//! Factory type definitions

use soroban_sdk::{contracttype, Address, BytesN};

// ============================================================
// FACTORY CONFIG
// ============================================================

/// Factory configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct FactoryConfig {
    pub admin: Address,
    pub pool_wasm_hash: BytesN<32>,
    pub router: Option<Address>,  // Set after router deployment
}

// ============================================================
// CREATE POOL PARAMS
// ============================================================

/// Parameters for creating a new pool
/// Bundled into struct to stay within 10 param limit
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatePoolParams {
    /// First token address
    pub token_a: Address,
    /// Second token address
    pub token_b: Address,
    /// Fee tier (5, 30, or 100 bps)
    pub fee_bps: u32,
    /// Creator fee share (10-1000 bps = 0.1%-10%)
    pub creator_fee_bps: u32,
    /// Initial price as sqrt(price) * 2^64
    pub initial_sqrt_price_x64: u128,
    /// Amount of token0 to deposit
    pub amount0_desired: i128,
    /// Amount of token1 to deposit
    pub amount1_desired: i128,
    /// Lower tick boundary
    pub lower_tick: i32,
    /// Upper tick boundary
    pub upper_tick: i32,
    /// Lock duration in ledgers (0 = permanent)
    pub lock_duration: u32,
}

// ============================================================
// FEE TIER
// ============================================================

/// Fee tier configuration
/// - 5 bps (0.05%) + tick spacing 10 → Stablecoins
/// - 30 bps (0.30%) + tick spacing 60 → Volatile
/// - 100 bps (1.00%) + tick spacing 200 → Meme/Exotic
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeTier {
    pub fee_bps: u32,
    pub tick_spacing: i32,
    pub enabled: bool,
}

// ============================================================
// CREATOR LOCK
// ============================================================

/// Creator's locked liquidity position
/// 
/// Creator fee rules:
/// - Must lock initial LP to earn creator fees
/// - Unlock/remove LP → fee_revoked = true (PERMANENT)
/// - Out of range → temporarily no fees
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatorLock {
    pub pool: Address,
    pub creator: Address,
    pub liquidity: i128,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub lock_start: u32,
    pub lock_end: u32,          // u32::MAX = permanent
    pub is_permanent: bool,
    pub is_unlocked: bool,      // true = unlocked
    pub fee_revoked: bool,      // true = PERMANENT revocation
}

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum DataKey {
    /// Factory config
    Config,
    /// Initialization flag
    Initialized,
    /// Fee tier by fee_bps
    FeeTier(u32),
    /// Pool address by (token0, token1, fee_bps)
    Pool(Address, Address, u32),
    /// All pool addresses
    PoolList,
    /// Total pool count
    PoolCount,
    /// Creator lock by (pool, creator)
    CreatorLock(Address, Address),
}