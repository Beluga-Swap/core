// Position Management Logic

use crate::types::Position;

/// Update a position's fee checkpoints and calculate owed tokens
pub fn update_position(
    pos: &mut Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    if pos.liquidity > 0 {
        let liquidity_u = pos.liquidity as u128;

        let delta_0 = fee_growth_inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
        let delta_1 = fee_growth_inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

        let fee_0 = liquidity_u
            .checked_mul(delta_0)
            .map(|product| product >> 64)
            .unwrap_or(0);

        let fee_1 = liquidity_u
            .checked_mul(delta_1)
            .map(|product| product >> 64)
            .unwrap_or(0);

        pos.tokens_owed_0 = pos.tokens_owed_0.saturating_add(fee_0);
        pos.tokens_owed_1 = pos.tokens_owed_1.saturating_add(fee_1);
    }

    pos.fee_growth_inside_last_0 = fee_growth_inside_0;
    pos.fee_growth_inside_last_1 = fee_growth_inside_1;
}

/// Modify a position's liquidity
pub fn modify_position(
    pos: &mut Position,
    liquidity_delta: i128,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    update_position(pos, fee_growth_inside_0, fee_growth_inside_1);

    if liquidity_delta > 0 {
        pos.liquidity = pos.liquidity.saturating_add(liquidity_delta);
    } else if liquidity_delta < 0 {
        pos.liquidity = pos.liquidity.saturating_sub(liquidity_delta.abs());
    }
}

/// Check if a position has any liquidity
#[inline]
pub fn has_liquidity(pos: &Position) -> bool {
    pos.liquidity > 0
}
