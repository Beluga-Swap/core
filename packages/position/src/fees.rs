use crate::types::Position;

pub fn calculate_pending_fees(
    pos: &Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) -> (u128, u128) {
    if pos.liquidity <= 0 {
        return (0, 0);
    }

    let liquidity_u = pos.liquidity as u128;

    let delta_0 = fee_growth_inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
    let delta_1 = fee_growth_inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

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
