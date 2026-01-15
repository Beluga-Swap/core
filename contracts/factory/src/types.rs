// Factory types module for BelugaSwap

use soroban_sdk::{contracttype, Address, BytesN};

// ============================================================
// FACTORY CONFIGURATION
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct FactoryConfig {
    pub admin: Address,
    pub pool_wasm_hash: BytesN<32>,
    pub min_initial_liquidity: i128,
    pub min_lock_duration: u32,
    pub default_fee_bps: u32,
    pub default_creator_fee_bps: u32,
}

// ============================================================
// POOL REGISTRY
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolInfo {
    pub pool_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub creator: Address,
    pub fee_bps: u32,
    pub creator_fee_bps: u32,
    pub tick_spacing: i32,
    pub created_at: u32,
    pub is_active: bool,
}

// ============================================================
// LOCKED LIQUIDITY
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct LockedLiquidity {
    pub pool_address: Address,
    pub owner: Address,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub liquidity: i128,
    pub lock_start: u32,
    pub lock_end: u32,
    pub is_permanent: bool,
    pub is_unlocked: bool,
    pub initial_amount0: i128,
    pub initial_amount1: i128,
}

// ============================================================
// POOL CREATION PARAMETERS
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatePoolParams {
    pub token_a: Address,
    pub token_b: Address,
    pub initial_sqrt_price_x64: u128,
    pub fee_bps: u32,
    pub creator_fee_bps: u32,
    pub tick_spacing: i32,
    pub initial_lower_tick: i32,
    pub initial_upper_tick: i32,
    pub amount_a: i128,
    pub amount_b: i128,
    pub permanent_lock: bool,
    pub lock_duration: u32,
}

// ============================================================
// FEE TIERS
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeTier {
    pub fee_bps: u32,
    pub tick_spacing: i32,
    pub enabled: bool,
}

// ============================================================
// CREATOR STATUS
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatorStatus {
    pub pool_address: Address,
    pub creator: Address,
    pub is_in_range: bool,
    pub is_eligible: bool,
    pub is_locked: bool,
    pub locked_liquidity: i128,
    pub pending_fees_0: u128,
    pub pending_fees_1: u128,
}

// ============================================================
// FACTORY STATISTICS
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct FactoryStats {
    pub total_pools: u32,
    pub active_pools: u32,
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