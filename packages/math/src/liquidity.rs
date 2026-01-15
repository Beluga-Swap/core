// SPDX-License-Identifier: MIT
// Liquidity Calculations

use soroban_sdk::Env;
use crate::q64::{mul_q64, div_q64, div_round_up, i128_to_u128_safe, u128_to_i128_saturating};

/// Calculate token0 amount for a liquidity and price range
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

    let delta_price = sqrt_upper.saturating_sub(sqrt_lower);
    let product_prices = mul_q64(sqrt_upper, sqrt_lower);

    if product_prices == 0 { return 0; }

    let numerator = liquidity.saturating_mul(delta_price);

    if round_up {
        div_round_up(numerator, product_prices)
    } else {
        numerator / product_prices
    }
}

/// Calculate token1 amount for a liquidity and price range
pub fn get_amount_1_delta(
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

    let delta = sqrt_upper.saturating_sub(sqrt_lower);
    let product = liquidity.saturating_mul(delta);

    if round_up {
        if product & 0xFFFFFFFFFFFFFFFF != 0 {
            (product >> 64) + 1
        } else {
            product >> 64
        }
    } else {
        product >> 64
    }
}

/// Calculate liquidity from token0 amount
pub fn get_liquidity_for_amount0(
    _env: &Env,
    amount0: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128
) -> i128 {
    if amount0 <= 0 || sqrt_price_lower >= sqrt_price_upper { return 0; }

    let amt0_u = i128_to_u128_safe(amount0);
    let product = mul_q64(sqrt_price_upper, sqrt_price_lower);
    let numerator = amt0_u.saturating_mul(product);
    let denominator = sqrt_price_upper.saturating_sub(sqrt_price_lower);

    if denominator == 0 { return 0; }
    u128_to_i128_saturating(numerator / denominator)
}

/// Calculate liquidity from token1 amount
pub fn get_liquidity_for_amount1(
    _env: &Env,
    amount1: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128
) -> i128 {
    if amount1 <= 0 || sqrt_price_lower >= sqrt_price_upper { return 0; }

    let amt1_u = i128_to_u128_safe(amount1);
    let diff = sqrt_price_upper.saturating_sub(sqrt_price_lower);

    if diff == 0 { return 0; }
    let liq_u = div_q64(amt1_u, diff);
    u128_to_i128_saturating(liq_u)
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

/// Calculate token amounts from liquidity
pub fn get_amounts_for_liquidity(
    _env: &Env,
    liquidity: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    current_sqrt_price: u128,
) -> (i128, i128) {
    if liquidity <= 0 { return (0, 0); }

    let liq_u = i128_to_u128_safe(liquidity);

    let sp = current_sqrt_price
        .max(sqrt_price_lower)
        .min(sqrt_price_upper);

    let amount0_u = if sp < sqrt_price_upper {
        get_amount_0_delta(sp, sqrt_price_upper, liq_u, false)
    } else {
        0
    };

    let amount1_u = if sp > sqrt_price_lower {
        get_amount_1_delta(sqrt_price_lower, sp, liq_u, false)
    } else {
        0
    };

    (u128_to_i128_saturating(amount0_u), u128_to_i128_saturating(amount1_u))
}