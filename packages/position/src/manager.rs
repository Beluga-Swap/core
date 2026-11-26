// Position Management Logic

use soroban_sdk::Env;
use belugaswap_math::{mul_div, ONE_X64};
use crate::types::Position;

/// Update a position's fee checkpoints and calculate owed tokens
/// 
/// This is the core Uniswap V3 fee collection pattern:
/// 1. Calculate delta = current_inside - last_inside (using wrapping arithmetic)
/// 2. owed_tokens += liquidity * delta / 2^64
/// 3. Update last_inside = current_inside
pub fn update_position(
    env: &Env,
    pos: &mut Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    if pos.liquidity > 0 {
        let liquidity_u = pos.liquidity as u128;

        // Calculate fee deltas using wrapping subtraction
        let delta_0 = fee_growth_inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
        let delta_1 = fee_growth_inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

        // owed = (liquidity * delta) >> 64, computed in 256-bit to avoid
        // overflowing u128. The previous `checked_mul(..).unwrap_or(0)` silently
        // dropped ALL fees for large positions whose liquidity*delta exceeded
        // u128 — this makes the accounting exact instead.
        let fee_0 = mul_div(env, liquidity_u, delta_0, ONE_X64);
        let fee_1 = mul_div(env, liquidity_u, delta_1, ONE_X64);

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
    env: &Env,
    pos: &mut Position,
    liquidity_delta: i128,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    // First update fees
    update_position(env, pos, fee_growth_inside_0, fee_growth_inside_1);

    // Then adjust liquidity
    if liquidity_delta > 0 {
        pos.liquidity = pos.liquidity.checked_add(liquidity_delta).expect("position liquidity overflow");
    } else if liquidity_delta < 0 {
        pos.liquidity = pos.liquidity.saturating_sub(liquidity_delta.abs());
    }
}

