// SPDX-License-Identifier: MIT
// Liquidity Calculations

use soroban_sdk::Env;
use crate::q64::{i128_to_u128_safe, u128_to_i128_saturating, ONE_X64};
use crate::wide::mul_div_wide;

/// Calculate token0 amount for a liquidity and price range
///
/// `L * (upper - lower) * 2^64 / (upper * lower)`, exact in the rounding
/// direction requested.
pub fn get_amount_0_delta(
    sqrt_price_a: u128,
    sqrt_price_b: u128,
    liquidity: u128,
    round_up: bool,
) -> u128 {
    let (sqrt_lower, sqrt_upper) = if sqrt_price_a < sqrt_price_b {
        (sqrt_price_a, sqrt_price_b)
    } else {
        (sqrt_price_b, sqrt_price_a)
    };

    if sqrt_lower == 0 { return 0; }

    mul_div_wide(liquidity, sqrt_upper - sqrt_lower, 64, sqrt_upper, sqrt_lower, round_up)
}

/// Calculate token1 amount for a liquidity and price range
///
/// `L * (upper - lower) / 2^64`
pub fn get_amount_1_delta(
    sqrt_price_a: u128,
    sqrt_price_b: u128,
    liquidity: u128,
    round_up: bool,
) -> u128 {
    let delta = sqrt_price_a.abs_diff(sqrt_price_b);
    mul_div_wide(liquidity, delta, 0, ONE_X64, 1, round_up)
}

/// Calculate liquidity from token0 amount
pub fn get_liquidity_for_amount0(
    _env: &Env,
    amount0: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128
) -> i128 {
    if amount0 <= 0 || sqrt_price_lower >= sqrt_price_upper { return 0; }

    // amount0 * upper * lower / ((upper - lower) * 2^64), rounding down at both
    // steps so the position never needs more than was offered.
    let delta = sqrt_price_upper - sqrt_price_lower;
    let scaled = mul_div_wide(i128_to_u128_safe(amount0), sqrt_price_upper, 0, delta, 1, false);
    u128_to_i128_saturating(mul_div_wide(scaled, sqrt_price_lower, 0, ONE_X64, 1, false))
}

/// Calculate liquidity from token1 amount
pub fn get_liquidity_for_amount1(
    _env: &Env,
    amount1: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128
) -> i128 {
    if amount1 <= 0 || sqrt_price_lower >= sqrt_price_upper { return 0; }

    let delta = sqrt_price_upper - sqrt_price_lower;
    u128_to_i128_saturating(mul_div_wide(i128_to_u128_safe(amount1), 1, 64, delta, 1, false))
}

/// Calculate liquidity from both token amounts
pub fn get_liquidity_for_amounts(
    env: &Env,
    amount0_desired: i128,
    amount1_desired: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    current_sqrt_price: u128,
) -> i128 {
    if sqrt_price_lower >= sqrt_price_upper { return 0; }

    if current_sqrt_price <= sqrt_price_lower {
        get_liquidity_for_amount0(env, amount0_desired, sqrt_price_lower, sqrt_price_upper)
    } else if current_sqrt_price >= sqrt_price_upper {
        get_liquidity_for_amount1(env, amount1_desired, sqrt_price_lower, sqrt_price_upper)
    } else {
        let liq0 = get_liquidity_for_amount0(env, amount0_desired, current_sqrt_price, sqrt_price_upper);
        let liq1 = get_liquidity_for_amount1(env, amount1_desired, sqrt_price_lower, current_sqrt_price);
        liq0.min(liq1)
    }
}

/// Token amounts a position is worth, rounded down. Use for withdrawals.
pub fn get_amounts_for_liquidity(
    _env: &Env,
    liquidity: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    current_sqrt_price: u128,
) -> (i128, i128) {
    amounts_for_liquidity(liquidity, sqrt_price_lower, sqrt_price_upper, current_sqrt_price, false)
}

/// Token amounts required to mint liquidity, rounded up. Use for deposits, so
/// that rounding can never leave the pool owing more than it received.
pub fn get_amounts_for_liquidity_rounding_up(
    _env: &Env,
    liquidity: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    current_sqrt_price: u128,
) -> (i128, i128) {
    amounts_for_liquidity(liquidity, sqrt_price_lower, sqrt_price_upper, current_sqrt_price, true)
}

fn amounts_for_liquidity(
    liquidity: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    current_sqrt_price: u128,
    round_up: bool,
) -> (i128, i128) {
    if liquidity <= 0 { return (0, 0); }

    let liq_u = i128_to_u128_safe(liquidity);

    let sp = current_sqrt_price
        .max(sqrt_price_lower)
        .min(sqrt_price_upper);

    let amount0_u = if sp < sqrt_price_upper {
        get_amount_0_delta(sp, sqrt_price_upper, liq_u, round_up)
    } else {
        0
    };

    let amount1_u = if sp > sqrt_price_lower {
        get_amount_1_delta(sqrt_price_lower, sp, liq_u, round_up)
    } else {
        0
    };

    (u128_to_i128_saturating(amount0_u), u128_to_i128_saturating(amount1_u))
}