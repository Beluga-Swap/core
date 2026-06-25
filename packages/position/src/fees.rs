use soroban_sdk::Env;
use belugaswap_math::{mul_div, ONE_X64};
use crate::types::Position;

pub fn calculate_pending_fees(
    env: &Env,
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

    // (liquidity * delta) >> 64 in 256-bit to avoid the u128 overflow that the
    // old checked_mul().unwrap_or(0) turned into silently-dropped fees.
    let pending_0 = mul_div(env, liquidity_u, delta_0, ONE_X64);
    let pending_1 = mul_div(env, liquidity_u, delta_1, ONE_X64);

    (pending_0, pending_1)
}
