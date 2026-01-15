// Pool Types - Using types from packages

use soroban_sdk::{contracttype, Address, Symbol};

// Re-export types from packages
pub use belugaswap_tick::TickInfo;
pub use belugaswap_position::{Position, PositionInfo};

// ============================================================
// POOL CONFIGURATION
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolConfig {
    pub creator: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub fee_bps: u32,
    pub creator_fee_bps: u32,
}

// ============================================================
// POOL STATE
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolState {
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub liquidity: i128,
    pub tick_spacing: i32,
    pub token0: Address,
    pub token1: Address,
    pub fee_growth_global_0: u128,
    pub fee_growth_global_1: u128,
    pub creator_fees_0: u128,
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