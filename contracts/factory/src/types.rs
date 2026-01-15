// Factory types module for BelugaSwap

use soroban_sdk::{contracttype, Address, BytesN};

// ============================================================
// FACTORY CONFIGURATION
// ============================================================

/// Factory global configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct FactoryConfig {
    /// Admin address with special privileges
    pub admin: Address,
    /// WASM hash of pool contract for deployment
    pub pool_wasm_hash: BytesN<32>,
    /// Minimum initial liquidity required to create a pool (per token)
    pub min_initial_liquidity: i128,
    /// Minimum lock duration in ledgers for creator fee eligibility
    pub min_lock_duration: u32,
    /// Default trading fee in basis points (e.g., 30 = 0.30%)
    pub default_fee_bps: u32,
    /// Default creator fee in basis points (portion of trading fee)
    pub default_creator_fee_bps: u32,
}

// ============================================================
// POOL REGISTRY
// ============================================================

/// Information about a deployed pool
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolInfo {
    /// Pool contract address
    pub pool_address: Address,
    /// First token (sorted, token0 < token1)
    pub token0: Address,
    /// Second token
    pub token1: Address,
    /// Address that created the pool
    pub creator: Address,
    /// Trading fee in basis points
    pub fee_bps: u32,
    /// Creator's share of trading fees in basis points
    pub creator_fee_bps: u32,
    /// Tick spacing for this pool
    pub tick_spacing: i32,
    /// Ledger sequence when pool was created
    pub created_at: u32,
    /// Whether pool is active
    pub is_active: bool,
}

// ============================================================
// LOCKED LIQUIDITY
// ============================================================

/// Represents a locked liquidity position
#[contracttype]
#[derive(Clone, Debug)]
pub struct LockedLiquidity {
    /// Pool this position is in
    pub pool_address: Address,
    /// Owner of the position
    pub owner: Address,
    /// Lower tick boundary
    pub lower_tick: i32,
    /// Upper tick boundary
    pub upper_tick: i32,
    /// Amount of liquidity locked
    pub liquidity: i128,
    /// Ledger when lock started
    pub lock_start: u32,
    /// Ledger when lock ends (u32::MAX for permanent)
    pub lock_end: u32,
    /// If true, liquidity is permanently locked
    pub is_permanent: bool,
    /// If true, position has been unlocked (no longer eligible for creator fees)
    pub is_unlocked: bool,
    /// Initial amount of token0 deposited
    pub initial_amount0: i128,
    /// Initial amount of token1 deposited
    pub initial_amount1: i128,
}

// ============================================================
// POOL CREATION PARAMETERS
// ============================================================

/// Parameters for creating a new pool
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatePoolParams {
    /// First token address
    pub token_a: Address,
    /// Second token address
    pub token_b: Address,
    /// Initial sqrt price as Q64.64 fixed point
    /// sqrt_price = sqrt(token1/token0) * 2^64
    pub initial_sqrt_price_x64: u128,
    /// Trading fee tier in basis points
    pub fee_bps: u32,
    /// Creator fee share in basis points (0 = use default)
    pub creator_fee_bps: u32,
    /// Tick spacing (must match fee tier, or 0 to use default)
    pub tick_spacing: i32,
    /// Lower tick for initial liquidity position
    pub initial_lower_tick: i32,
    /// Upper tick for initial liquidity position
    pub initial_upper_tick: i32,
    /// Amount of token_a to provide as initial liquidity
    pub amount_a: i128,
    /// Amount of token_b to provide as initial liquidity
    pub amount_b: i128,
    /// If true, lock liquidity permanently (eternal creator fees)
    pub permanent_lock: bool,
    /// Lock duration in ledgers (ignored if permanent_lock is true)
    pub lock_duration: u32,
}

// ============================================================
// FEE TIERS
// ============================================================

/// Fee tier configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeTier {
    /// Fee in basis points (e.g., 30 = 0.30%)
    pub fee_bps: u32,
    /// Minimum tick spacing for this tier
    pub tick_spacing: i32,
    /// Whether this tier is enabled for new pool creation
    pub enabled: bool,
}

// ============================================================
// CREATOR STATUS
// ============================================================

/// Status of creator's fee eligibility and pending rewards
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatorStatus {
    /// Pool address
    pub pool_address: Address,
    /// Creator address
    pub creator: Address,
    /// Whether position is in current price range
    pub is_in_range: bool,
    /// Whether creator is eligible for fees (locked + in range)
    pub is_eligible: bool,
    /// Whether liquidity is currently locked
    pub is_locked: bool,
    /// Total locked liquidity amount
    pub locked_liquidity: i128,
    /// Pending token0 fees to claim
    pub pending_fees_0: u128,
    /// Pending token1 fees to claim
    pub pending_fees_1: u128,
}

// ============================================================
// FACTORY STATISTICS
// ============================================================

/// Factory-wide statistics
#[contracttype]
#[derive(Clone, Debug)]
pub struct FactoryStats {
    /// Total number of pools created
    pub total_pools: u32,
    /// Number of currently active pools
    pub active_pools: u32,
    /// Total value locked across all pools (in some base denomination)
    pub total_locked_value: u128,
}

impl Default for FactoryStats {
    fn default() -> Self {
        Self {
            total_pools: 0,
            active_pools: 0,
            total_locked_value: 0,
        }
    }
}

// ============================================================
// QUERY RESULTS
// ============================================================

/// Result of querying multiple pools
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolList {
    /// List of pool addresses
    pub pools: soroban_sdk::Vec<Address>,
    /// Total count (may be more than returned)
    pub total: u32,
    /// Starting index of this batch
    pub offset: u32,
}

/// Summary information for a pool
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolSummary {
    pub pool_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee_bps: u32,
    pub is_active: bool,
}