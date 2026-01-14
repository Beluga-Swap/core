// Compatible with OpenZeppelin Stellar Soroban Contracts patterns
//
// Constants module following OpenZeppelin conventions:
// - Clear documentation for each constant
// - Grouped by functionality
// - Uses appropriate types for each constant

// ============================================================
// TICK CONSTANTS
// ============================================================

/// Minimum valid tick value (corresponds to minimum price)
/// Price at MIN_TICK ≈ 2.94e-39
/// This is the lower bound for all tick values in the pool
pub const MIN_TICK: i32 = -887272;

/// Maximum valid tick value (corresponds to maximum price)
/// Price at MAX_TICK ≈ 3.40e+38
/// This is the upper bound for all tick values in the pool
pub const MAX_TICK: i32 = 887272;

// ============================================================
// SQRT PRICE CONSTANTS (Q64.64 format)
// ============================================================

/// Minimum sqrt price (at MIN_TICK)
/// Calculated as: sqrt(1.0001^-887272) * 2^64
#[allow(dead_code)]
pub const MIN_SQRT_PRICE: u128 = 4295128739;

/// Maximum sqrt price (at MAX_TICK)
/// Using u128::MAX as safe upper bound
#[allow(dead_code)]
pub const MAX_SQRT_PRICE: u128 = u128::MAX;

/// Sqrt price for 1:1 price ratio (2^64)
/// This represents price = 1.0 in Q64.64 format
#[allow(dead_code)]
pub const SQRT_PRICE_1_1: u128 = 18446744073709551616_u128;

// ============================================================
// LIQUIDITY CONSTANTS
// ============================================================

/// Minimum liquidity for a position
/// Prevents dust positions and ensures meaningful LP participation
pub const MIN_LIQUIDITY: i128 = 1000;

/// Maximum liquidity per tick to prevent overflow
/// Set to half of i128::MAX for safety margin
#[allow(dead_code)]
pub const MAX_LIQUIDITY_PER_TICK: i128 = i128::MAX / 2;

// ============================================================
// SWAP CONSTANTS
// ============================================================

/// Minimum amount for a swap
/// Prevents gas waste on negligible trades
pub const MIN_SWAP_AMOUNT: i128 = 1;

/// Minimum output amount (dust threshold)
/// Outputs below this are considered dust
pub const MIN_OUTPUT_AMOUNT: i128 = 1;

/// Maximum slippage in basis points (50% = 5000 bps)
/// Protection against extreme price impact
pub const MAX_SLIPPAGE_BPS: i128 = 5000;

/// Maximum iterations in swap loop (prevents infinite loops)
/// Safety limit for complex multi-tick swaps
pub const MAX_SWAP_ITERATIONS: u32 = 1024;

/// Maximum steps when searching for next initialized tick
/// Limits computation for tick traversal
pub const MAX_TICK_SEARCH_STEPS: i32 = 2000;

// ============================================================
// FEE CONSTANTS
// ============================================================

/// Maximum fee in basis points (100% = 10000 bps)
pub const MAX_FEE_BPS: u32 = 10000;

/// Minimum creator fee in basis points (0.01%)
pub const MIN_CREATOR_FEE_BPS: u32 = 1;

/// Maximum creator fee in basis points (10% of swap = 1000 bps)
pub const MAX_CREATOR_FEE_BPS: u32 = 1000;

/// Default LP fee (0.3% = 30 bps)
#[allow(dead_code)]
pub const DEFAULT_FEE_BPS: u32 = 30;

/// Default creator fee (1% of LP fee = 100 bps)
#[allow(dead_code)]
pub const DEFAULT_CREATOR_FEE_BPS: u32 = 100;

// ============================================================
// MATH CONSTANTS
// ============================================================

/// Q64 multiplier (2^64) for fixed-point math
/// Used as the scaling factor for Q64.64 format
pub const Q64: u128 = 1u128 << 64;

/// Q128 multiplier (2^128) for high-precision intermediate calculations
/// Used when extra precision is needed to prevent precision loss
#[allow(dead_code)]
pub const Q128: u128 = 1u128 << 64 << 64;

/// Maximum reasonable fee delta (for overflow detection)
/// Used to validate fee growth values
#[allow(dead_code)]
pub const MAX_REASONABLE_FEE_DELTA: u128 = 1u128 << 96;

// ============================================================
// TWAP CONSTANTS (Reserved for future use)
// ============================================================

/// Maximum TWAP observations stored
/// Limits storage usage for price oracle
#[allow(dead_code)]
pub const MAX_TWAP_OBSERVATIONS: u32 = 100;

/// Minimum time between TWAP observations (in seconds)
/// Prevents observation spam
#[allow(dead_code)]
pub const MIN_TWAP_OBSERVATION_INTERVAL: u64 = 1;