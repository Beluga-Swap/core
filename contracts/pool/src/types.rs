// Pool Types - Using types from packages

use soroban_sdk::{contracttype, Address};

// Re-export types from packages
pub use belugaswap_tick::TickInfo;
pub use belugaswap_position::{Position, PositionInfo};

// ============================================================
// POOL CONFIGURATION
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Factory that deployed this pool (or creator if direct deployment)
    pub factory: Address,
    /// Pool creator address
    pub creator: Address,
    /// First token (original order from creation)
    pub token_a: Address,
    /// Second token (original order from creation)
    pub token_b: Address,
    /// Trading fee in basis points (e.g., 30 = 0.30%)
    pub fee_bps: u32,
    /// Creator's share of trading fees in basis points
    pub creator_fee_bps: u32,
}

// ============================================================
// POOL STATE
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolState {
    /// Current sqrt price as Q64.64 fixed point
    pub sqrt_price_x64: u128,
    /// Current tick
    pub current_tick: i32,
    /// Active liquidity in range
    pub liquidity: i128,
    /// Tick spacing for this pool
    pub tick_spacing: i32,
    /// Token0 address (sorted: token0 < token1)
    pub token0: Address,
    /// Token1 address
    pub token1: Address,
    /// Global fee growth for token0
    pub fee_growth_global_0: u128,
    /// Global fee growth for token1
    pub fee_growth_global_1: u128,
    /// Accumulated creator fees for token0
    pub creator_fees_0: u128,
    /// Accumulated creator fees for token1
    pub creator_fees_1: u128,
}

// ============================================================
// CREATOR FEES INFO
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatorFeesInfo {
    pub fees_token0: u128,
    pub fees_token1: u128,
}

impl Default for CreatorFeesInfo {
    fn default() -> Self {
        Self {
            fees_token0: 0,
            fees_token1: 0,
        }
    }
}

// ============================================================
// TWAP TYPES (Reserved for future use)
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct TWAPObservation {
    pub timestamp: u64,
    pub tick_cumulative: i128,
    pub liquidity_cumulative: u128,
}

impl Default for TWAPObservation {
    fn default() -> Self {
        Self {
            timestamp: 0,
            tick_cumulative: 0,
            liquidity_cumulative: 0,
        }
    }
}