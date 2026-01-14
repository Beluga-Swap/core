// Compatible with OpenZeppelin Stellar Soroban Contracts patterns
//
// Types module following OpenZeppelin conventions:
// - Clear documentation for each type
// - Consistent Default implementations
// - Proper derive traits

use soroban_sdk::{contracttype, Address, Symbol};

// ============================================================
// POOL CONFIGURATION
// ============================================================

/// Pool configuration (immutable after initialization)
/// 
/// This struct holds the configuration parameters that are set
/// during pool initialization and cannot be changed afterwards.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Pool creator address (receives creator fees)
    pub creator: Address,
    /// First token address (as provided by user)
    pub token_a: Address,
    /// Second token address (as provided by user)
    pub token_b: Address,
    /// Swap fee in basis points (e.g., 30 = 0.30%)
    /// Range: 1-10000 bps
    pub fee_bps: u32,
    /// Creator fee in basis points (1-1000 bps = 0.01%-10% of LP fee)
    /// This fee is taken from the LP fee, not from the total swap amount
    pub creator_fee_bps: u32,
}

// ============================================================
// POOL STATE
// ============================================================

/// Pool state (mutable, updated on every swap/liquidity change)
/// 
/// This struct holds the dynamic state of the pool that changes
/// with every swap or liquidity modification.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolState {
    /// Current sqrt price in Q64.64 format
    /// Represents sqrt(token1/token0) * 2^64
    pub sqrt_price_x64: u128,
    /// Current tick (derived from sqrt_price_x64)
    /// tick = log_{1.0001}(price)
    pub current_tick: i32,
    /// Currently active liquidity (sum of in-range positions)
    /// This is the liquidity available for swaps at current price
    pub liquidity: i128,
    /// Tick spacing for this pool
    /// Determines granularity of liquidity positions
    pub tick_spacing: i32,
    /// Token0 address (sorted: lower address by bytes)
    pub token0: Address,
    /// Token1 address (sorted: higher address by bytes)
    pub token1: Address,
    /// Global fee growth for token0 (Q64.64 format)
    /// Accumulated fees per unit of liquidity for token0
    pub fee_growth_global_0: u128,
    /// Global fee growth for token1 (Q64.64 format)
    /// Accumulated fees per unit of liquidity for token1
    pub fee_growth_global_1: u128,
    /// Accumulated creator fees for token0
    pub creator_fees_0: u128,
    /// Accumulated creator fees for token1
    pub creator_fees_1: u128,
}

// ============================================================
// TICK INFO
// ============================================================

/// Information stored for each initialized tick
/// 
/// Ticks are the boundaries of liquidity positions.
/// This struct tracks liquidity and fee accumulation at each tick.
#[contracttype]
#[derive(Clone, Debug)]
pub struct TickInfo {
    /// Total liquidity referencing this tick
    /// Sum of all positions that use this tick as a boundary
    pub liquidity_gross: i128,
    /// Net liquidity change when crossing left-to-right
    /// Positive for lower ticks, negative for upper ticks
    pub liquidity_net: i128,
    /// Fee growth outside this tick for token0
    /// Used to calculate fees earned by positions
    pub fee_growth_outside_0: u128,
    /// Fee growth outside this tick for token1
    /// Used to calculate fees earned by positions
    pub fee_growth_outside_1: u128,
    /// Whether this tick is initialized (has any liquidity)
    pub initialized: bool,
}

impl Default for TickInfo {
    fn default() -> Self {
        Self {
            liquidity_gross: 0,
            liquidity_net: 0,
            fee_growth_outside_0: 0,
            fee_growth_outside_1: 0,
            initialized: false,
        }
    }
}

// ============================================================
// POSITION
// ============================================================

/// LP position data
/// 
/// Represents a liquidity provider's position in a specific tick range.
/// Each position is uniquely identified by (owner, lower_tick, upper_tick).
#[contracttype]
#[derive(Clone, Debug)]
pub struct Position {
    /// Liquidity in this position
    pub liquidity: i128,
    /// Fee growth inside at last update for token0
    /// Checkpoint for calculating earned fees
    pub fee_growth_inside_last_0: u128,
    /// Fee growth inside at last update for token1
    /// Checkpoint for calculating earned fees
    pub fee_growth_inside_last_1: u128,
    /// Uncollected fees for token0
    pub tokens_owed_0: u128,
    /// Uncollected fees for token1
    pub tokens_owed_1: u128,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            liquidity: 0,
            fee_growth_inside_last_0: 0,
            fee_growth_inside_last_1: 0,
            tokens_owed_0: 0,
            tokens_owed_1: 0,
        }
    }
}

impl Position {
    /// Check if position has any liquidity
    #[inline]
    pub fn has_liquidity(&self) -> bool {
        self.liquidity > 0
    }

    /// Check if position has uncollected fees
    #[inline]
    pub fn has_uncollected_fees(&self) -> bool {
        self.tokens_owed_0 > 0 || self.tokens_owed_1 > 0
    }

    /// Check if position is empty (no liquidity and no fees)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.liquidity == 0 && self.tokens_owed_0 == 0 && self.tokens_owed_1 == 0
    }
}

// ============================================================
// RETURN TYPES (for contract functions)
// ============================================================

/// Position information returned by get_position
/// 
/// This is a view-only struct that provides a snapshot of
/// a position's current state including pending fees.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PositionInfo {
    /// Position's liquidity
    pub liquidity: i128,
    /// Current amount of token0 in position
    pub amount0: i128,
    /// Current amount of token1 in position
    pub amount1: i128,
    /// Uncollected fees for token0 (including pending)
    pub fees_owed_0: u128,
    /// Uncollected fees for token1 (including pending)
    pub fees_owed_1: u128,
}

impl Default for PositionInfo {
    fn default() -> Self {
        Self {
            liquidity: 0,
            amount0: 0,
            amount1: 0,
            fees_owed_0: 0,
            fees_owed_1: 0,
        }
    }
}

/// Swap result returned by swap functions
#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapResult {
    /// Actual amount of input token used
    pub amount_in: i128,
    /// Actual amount of output token received
    pub amount_out: i128,
    /// Current tick after swap
    pub current_tick: i32,
    /// Current sqrt price after swap (Q64.64)
    pub sqrt_price_x64: u128,
}

impl Default for SwapResult {
    fn default() -> Self {
        Self {
            amount_in: 0,
            amount_out: 0,
            current_tick: 0,
            sqrt_price_x64: 0,
        }
    }
}

/// Preview result returned by preview_swap functions
/// 
/// Provides information about what would happen if a swap
/// were executed, without actually executing it.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PreviewResult {
    /// Amount of input token that would be used
    pub amount_in_used: i128,
    /// Expected amount of output token
    pub amount_out_expected: i128,
    /// Fee that would be paid
    pub fee_paid: i128,
    /// Price impact in basis points
    pub price_impact_bps: i128,
    /// Whether the swap would succeed
    pub is_valid: bool,
    /// Error message if invalid (None if valid)
    pub error_message: Option<Symbol>,
}

impl Default for PreviewResult {
    fn default() -> Self {
        Self {
            amount_in_used: 0,
            amount_out_expected: 0,
            fee_paid: 0,
            price_impact_bps: 0,
            is_valid: false,
            error_message: None,
        }
    }
}

impl PreviewResult {
    /// Create a valid preview result
    pub fn valid(
        amount_in_used: i128,
        amount_out_expected: i128,
        fee_paid: i128,
        price_impact_bps: i128,
    ) -> Self {
        Self {
            amount_in_used,
            amount_out_expected,
            fee_paid,
            price_impact_bps,
            is_valid: true,
            error_message: None,
        }
    }

    /// Create an invalid preview result with error
    pub fn invalid(error: Symbol) -> Self {
        Self {
            amount_in_used: 0,
            amount_out_expected: 0,
            fee_paid: 0,
            price_impact_bps: 0,
            is_valid: false,
            error_message: Some(error),
        }
    }
}

/// Creator fees info
/// 
/// Provides information about accumulated creator fees.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatorFeesInfo {
    /// Accumulated creator fees for token0
    pub fees_token0: u128,
    /// Accumulated creator fees for token1
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
// TWAP TYPES
// ============================================================

/// TWAP observation data
/// 
/// Used for time-weighted average price calculations.
/// Reserved for future oracle functionality.
#[contracttype]
#[derive(Clone, Debug)]
pub struct TWAPObservation {
    /// Timestamp of this observation
    pub timestamp: u64,
    /// Cumulative tick * time at this observation
    pub tick_cumulative: i128,
    /// Cumulative 1/liquidity * time at this observation
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