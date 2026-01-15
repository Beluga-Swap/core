// Position Management Logic

use crate::types::Position;

/// Update a position's fee checkpoints and calculate owed tokens
/// 
/// This is the core Uniswap V3 fee collection pattern:
/// 1. Calculate delta = current_inside - last_inside (using wrapping arithmetic)
/// 2. owed_tokens += liquidity * delta / 2^64
/// 3. Update last_inside = current_inside
pub fn update_position(
    pos: &mut Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    if pos.liquidity > 0 {
        let liquidity_u = pos.liquidity as u128;

        // Calculate fee deltas using wrapping subtraction
        let delta_0 = fee_growth_inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
        let delta_1 = fee_growth_inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

        // Calculate owed fees with overflow protection
        let fee_0 = liquidity_u
            .checked_mul(delta_0)
            .map(|product| product >> 64)
            .unwrap_or(0);

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

/// Modify a position's liquidity
/// 
/// This combines fee update with liquidity change:
/// 1. First update fees based on current fee_growth_inside
/// 2. Then adjust liquidity
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
}

// ============================================================
// POSITION HELPERS (were missing from refactored package)
// ============================================================

/// Check if a position has any liquidity
#[inline]
pub fn has_liquidity(pos: &Position) -> bool {
    pos.liquidity > 0
}

/// Check if a position has uncollected fees
#[inline]
pub fn has_uncollected_fees(pos: &Position) -> bool {
    pos.tokens_owed_0 > 0 || pos.tokens_owed_1 > 0
}

/// Check if a position is empty (no liquidity and no fees)
#[inline]
pub fn is_empty(pos: &Position) -> bool {
    pos.liquidity == 0 && pos.tokens_owed_0 == 0 && pos.tokens_owed_1 == 0
}

/// Clear collected fees from position
/// Uses saturating subtraction to prevent underflow
pub fn clear_fees(pos: &mut Position, amount0: u128, amount1: u128) {
    pos.tokens_owed_0 = pos.tokens_owed_0.saturating_sub(amount0);
    pos.tokens_owed_1 = pos.tokens_owed_1.saturating_sub(amount1);
}

// ============================================================
// POSITION VALIDATION (was missing from refactored package)
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