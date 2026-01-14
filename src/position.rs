// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Stellar Soroban Contracts patterns
//
// Position management module following OpenZeppelin conventions:
// - Clear function documentation
// - Consistent error handling
// - Separation of concerns

use soroban_sdk::{Address, Env};

use crate::storage::{read_position as storage_read, write_position as storage_write};
use crate::types::Position;

// ============================================================
// POSITION STORAGE HELPERS
// ============================================================

/// Read a position from storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `owner` - Position owner address
/// * `lower` - Lower tick boundary
/// * `upper` - Upper tick boundary
/// 
/// # Returns
/// Position data (returns default if not exists)
#[inline]
pub fn read_position(env: &Env, owner: &Address, lower: i32, upper: i32) -> Position {
    storage_read(env, owner, lower, upper)
}

/// Write a position to storage
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `owner` - Position owner address
/// * `lower` - Lower tick boundary
/// * `upper` - Upper tick boundary
/// * `pos` - Position data to store
#[inline]
pub fn write_position(env: &Env, owner: &Address, lower: i32, upper: i32, pos: &Position) {
    storage_write(env, owner, lower, upper, pos);
}

// ============================================================
// POSITION UPDATE (Fee Accumulation)
// ============================================================

/// Update a position's fee checkpoints and calculate owed tokens
/// 
/// This is the core Uniswap V3 fee collection pattern:
/// 1. Calculate delta = current_inside - last_inside (using wrapping arithmetic)
/// 2. owed_tokens += liquidity * delta / 2^64
/// 3. Update last_inside = current_inside
/// 
/// In Uniswap V3, wrapping arithmetic is used and it always works correctly
/// because fee_growth_inside is consistent through tick crossings.
/// 
/// # Arguments
/// * `pos` - Mutable reference to position
/// * `fee_growth_inside_0` - Current fee growth inside for token0
/// * `fee_growth_inside_1` - Current fee growth inside for token1
/// 
/// # Note
/// This function modifies the position in place. The caller is responsible
/// for persisting the changes to storage.
pub fn update_position(
    pos: &mut Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    if pos.liquidity > 0 {
        let liquidity_u = pos.liquidity as u128;

        // Calculate fee deltas using wrapping subtraction
        // Wrapping is correct here because fee_growth values can wrap around
        let delta_0 = fee_growth_inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
        let delta_1 = fee_growth_inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

        // Calculate owed fees using checked multiplication to detect overflow
        // fee = (liquidity * delta) >> 64
        let fee_0 = liquidity_u
            .checked_mul(delta_0)
            .map(|product| product >> 64)
            .unwrap_or(0); // Overflow indicates invalid delta

        let fee_1 = liquidity_u
            .checked_mul(delta_1)
            .map(|product| product >> 64)
            .unwrap_or(0);

        // Accumulate owed tokens with saturation
        pos.tokens_owed_0 = pos.tokens_owed_0.saturating_add(fee_0);
        pos.tokens_owed_1 = pos.tokens_owed_1.saturating_add(fee_1);
    }

    // Always update checkpoints to current values
    pos.fee_growth_inside_last_0 = fee_growth_inside_0;
    pos.fee_growth_inside_last_1 = fee_growth_inside_1;
}

// ============================================================
// POSITION MODIFICATION
// ============================================================

/// Modify a position's liquidity
/// 
/// This combines fee update with liquidity change:
/// 1. First update fees based on current fee_growth_inside
/// 2. Then adjust liquidity
/// 
/// # Arguments
/// * `pos` - Mutable reference to position
/// * `liquidity_delta` - Change in liquidity (positive = add, negative = remove)
/// * `fee_growth_inside_0` - Current fee growth inside for token0
/// * `fee_growth_inside_1` - Current fee growth inside for token1
/// 
/// # Note
/// This function modifies the position in place. The caller is responsible
/// for persisting the changes to storage and updating tick states.
pub fn modify_position(
    pos: &mut Position,
    liquidity_delta: i128,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    // First update fees
    update_position(pos, fee_growth_inside_0, fee_growth_inside_1);

    // Then adjust liquidity
    if liquidity_delta > 0 {
        pos.liquidity = pos.liquidity.saturating_add(liquidity_delta);
    } else if liquidity_delta < 0 {
        pos.liquidity = pos.liquidity.saturating_sub(liquidity_delta.abs());
    }
    // If liquidity_delta == 0, no change needed
}

// ============================================================
// PENDING FEE CALCULATION
// ============================================================

/// Calculate pending fees without modifying position
/// 
/// This is a read-only calculation for display purposes.
/// Uses the same formula as update_position but doesn't modify state.
/// 
/// # Arguments
/// * `pos` - Reference to position
/// * `fee_growth_inside_0` - Current fee growth inside for token0
/// * `fee_growth_inside_1` - Current fee growth inside for token1
/// 
/// # Returns
/// `(pending_fee_0, pending_fee_1)` - Tuple of pending fees for each token
pub fn calculate_pending_fees(
    pos: &Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) -> (u128, u128) {
    if pos.liquidity <= 0 {
        return (0, 0);
    }

    let liquidity_u = pos.liquidity as u128;

    // Calculate deltas using wrapping subtraction
    let delta_0 = fee_growth_inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
    let delta_1 = fee_growth_inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

    // Calculate pending fees with overflow protection
    let pending_0 = liquidity_u
        .checked_mul(delta_0)
        .map(|product| product >> 64)
        .unwrap_or(0);

    let pending_1 = liquidity_u
        .checked_mul(delta_1)
        .map(|product| product >> 64)
        .unwrap_or(0);

    (pending_0, pending_1)
}

// ============================================================
// POSITION HELPERS
// ============================================================

/// Check if a position has any liquidity
/// 
/// # Arguments
/// * `pos` - Reference to position
/// 
/// # Returns
/// `true` if position has liquidity > 0
#[inline]
pub fn has_liquidity(pos: &Position) -> bool {
    pos.liquidity > 0
}

/// Check if a position has uncollected fees
/// 
/// # Arguments
/// * `pos` - Reference to position
/// 
/// # Returns
/// `true` if position has any uncollected fees
#[inline]
#[allow(dead_code)]
pub fn has_uncollected_fees(pos: &Position) -> bool {
    pos.tokens_owed_0 > 0 || pos.tokens_owed_1 > 0
}

/// Check if a position is empty (no liquidity and no fees)
/// 
/// # Arguments
/// * `pos` - Reference to position
/// 
/// # Returns
/// `true` if position has no liquidity and no uncollected fees
#[inline]
#[allow(dead_code)]
pub fn is_empty(pos: &Position) -> bool {
    pos.liquidity == 0 && pos.tokens_owed_0 == 0 && pos.tokens_owed_1 == 0
}

/// Clear collected fees from position
/// 
/// # Arguments
/// * `pos` - Mutable reference to position
/// * `amount0` - Amount of token0 fees to clear
/// * `amount1` - Amount of token1 fees to clear
/// 
/// # Note
/// Uses saturating subtraction to prevent underflow
#[allow(dead_code)]
pub fn clear_fees(pos: &mut Position, amount0: u128, amount1: u128) {
    pos.tokens_owed_0 = pos.tokens_owed_0.saturating_sub(amount0);
    pos.tokens_owed_1 = pos.tokens_owed_1.saturating_sub(amount1);
}

// ============================================================
// POSITION VALIDATION
// ============================================================

/// Validate position parameters
/// 
/// # Arguments
/// * `lower` - Lower tick boundary
/// * `upper` - Upper tick boundary
/// * `tick_spacing` - Pool's tick spacing
/// 
/// # Returns
/// `Ok(())` if valid, `Err(&str)` if invalid
#[allow(dead_code)]
pub fn validate_position_params(
    lower: i32,
    upper: i32,
    tick_spacing: i32,
) -> Result<(), &'static str> {
    if lower >= upper {
        return Err("lower tick must be less than upper tick");
    }

    if tick_spacing <= 0 {
        return Err("tick spacing must be positive");
    }

    if lower % tick_spacing != 0 {
        return Err("lower tick must be aligned to tick spacing");
    }

    if upper % tick_spacing != 0 {
        return Err("upper tick must be aligned to tick spacing");
    }

    Ok(())
}