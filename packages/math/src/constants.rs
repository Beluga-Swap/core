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
// LIQUIDITY CONSTANTS
// ============================================================

/// Minimum liquidity for a position
/// Prevents dust positions and ensures meaningful LP participation
pub const MIN_LIQUIDITY: i128 = 1000;

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

// ============================================================
// FEE CONSTANTS
// ============================================================

/// Maximum fee in basis points (100% = 10000 bps)
pub const MAX_FEE_BPS: u32 = 10000;


// ============================================================
// MATH CONSTANTS
// ============================================================

/// Q64 multiplier (2^64) for fixed-point math
/// Used as the scaling factor for Q64.64 format
pub const Q64: u128 = 1u128 << 64;

