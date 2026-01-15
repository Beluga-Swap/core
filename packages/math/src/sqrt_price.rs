// SPDX-License-Identifier: MIT
// Sqrt Price Calculations

use soroban_sdk::Env;
use crate::constants::{MIN_TICK, MAX_TICK};
use crate::q64::{mul_q64, div_q64, mul_div, ONE_X64};

/// Convert tick to sqrt price in Q64.64 format
/// Formula: sqrt(1.0001^tick) * 2^64
pub fn get_sqrt_ratio_at_tick(tick: i32) -> u128 {
    if !(MIN_TICK..=MAX_TICK).contains(&tick) {
        panic!("tick out of range");
    }

    if tick == 0 { return ONE_X64; }

    let abs_tick = tick.unsigned_abs();
    let mut ratio: u128 = ONE_X64;

    // Binary decomposition constants for sqrt(1.0001^(2^n)) * 2^64
    if abs_tick & 0x1 != 0 { ratio = mul_q64(ratio, 18447666387855958016); }
    if abs_tick & 0x2 != 0 { ratio = mul_q64(ratio, 18448588748116922368); }
    if abs_tick & 0x4 != 0 { ratio = mul_q64(ratio, 18450433606991732736); }
    if abs_tick & 0x8 != 0 { ratio = mul_q64(ratio, 18454123878217469952); }
    if abs_tick & 0x10 != 0 { ratio = mul_q64(ratio, 18461506635090006016); }
    if abs_tick & 0x20 != 0 { ratio = mul_q64(ratio, 18476281010653908992); }
    if abs_tick & 0x40 != 0 { ratio = mul_q64(ratio, 18505849059060717568); }
    if abs_tick & 0x80 != 0 { ratio = mul_q64(ratio, 18565033932859791360); }
    if abs_tick & 0x100 != 0 { ratio = mul_q64(ratio, 18683636815981789184); }
    if abs_tick & 0x200 != 0 { ratio = mul_q64(ratio, 18922376066158198784); }
    if abs_tick & 0x400 != 0 { ratio = mul_q64(ratio, 19403906064415539200); }
    if abs_tick & 0x800 != 0 { ratio = mul_q64(ratio, 20388321338895749120); }
    if abs_tick & 0x1000 != 0 { ratio = mul_q64(ratio, 22486086334269071360); }
    if abs_tick & 0x2000 != 0 { ratio = mul_q64(ratio, 27241267204663885824); }
    if abs_tick & 0x4000 != 0 { ratio = mul_q64(ratio, 40198444615281172480); }
    if abs_tick & 0x8000 != 0 { ratio = mul_q64(ratio, 87150709742682460160); }
    if abs_tick & 0x10000 != 0 { ratio = mul_q64(ratio, 409916713094318874624); }

    if tick < 0 {
        if ratio == 0 { return u128::MAX; }
        let numerator = ONE_X64.saturating_mul(ONE_X64);
        ratio = numerator / ratio;
    }

    ratio
}

/// Alias for get_sqrt_ratio_at_tick
#[allow(dead_code)]
pub fn tick_to_sqrt_price_x64(_env: &Env, tick: i32) -> u128 {
    get_sqrt_ratio_at_tick(tick)
}

/// Calculate next sqrt price given input amount
pub fn get_next_sqrt_price_from_input(
    env: &Env,
    sqrt_price: u128,
    liquidity: u128,
    amount_in: u128,
    zero_for_one: bool,
) -> u128 {
    if amount_in == 0 || liquidity == 0 {
        return sqrt_price;
    }

    if zero_for_one {
        let product = amount_in.saturating_mul(sqrt_price);
        let numerator = liquidity.saturating_mul(sqrt_price);
        let liq_shifted = liquidity << 64;
        let denominator = liq_shifted.saturating_add(product);

        if denominator == 0 { return sqrt_price; }
        mul_div(env, numerator, ONE_X64, denominator)
    } else {
        let quotient = div_q64(amount_in, liquidity);
        sqrt_price.saturating_add(quotient)
    }
}

/// Calculate next sqrt price given output amount
#[allow(dead_code)]
pub fn get_next_sqrt_price_from_output(
    env: &Env,
    sqrt_price: u128,
    liquidity: u128,
    amount_out: u128,
    zero_for_one: bool,
) -> u128 {
    if amount_out == 0 || liquidity == 0 {
        return sqrt_price;
    }

    if zero_for_one {
        let quotient = div_q64(amount_out, liquidity);
        sqrt_price.saturating_sub(quotient)
    } else {
        let product = amount_out.saturating_mul(sqrt_price);
        let numerator = liquidity.saturating_mul(sqrt_price);
        let liq_shifted = liquidity << 64;
        let denominator = liq_shifted.saturating_sub(product);

        if denominator == 0 { return u128::MAX; }
        mul_div(env, numerator, ONE_X64, denominator)
    }
}

/// Compute swap step with a target price
pub fn compute_swap_step_with_target(
    env: &Env,
    sqrt_price_current: u128,
    liquidity: i128,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_target: u128,
) -> (u128, i128, i128) {
    use crate::q64::{i128_to_u128_safe, u128_to_i128_saturating};
    use crate::liquidity::{get_amount_0_delta, get_amount_1_delta};
    
    let liq_u = i128_to_u128_safe(liquidity);
    let amount_rem_u = i128_to_u128_safe(amount_specified);

    let next_price_input = get_next_sqrt_price_from_input(
        env, sqrt_price_current, liq_u, amount_rem_u, zero_for_one
    );

    let target_reached = if zero_for_one {
        next_price_input <= sqrt_price_target
    } else {
        next_price_input >= sqrt_price_target
    };

    let sqrt_price_next = if target_reached {
        sqrt_price_target
    } else {
        next_price_input
    };

    let (amount_in, amount_out) = if zero_for_one {
        (
            get_amount_0_delta(sqrt_price_current, sqrt_price_next, liq_u, true),
            get_amount_1_delta(sqrt_price_current, sqrt_price_next, liq_u, false),
        )
    } else {
        (
            get_amount_1_delta(sqrt_price_current, sqrt_price_next, liq_u, true),
            get_amount_0_delta(sqrt_price_current, sqrt_price_next, liq_u, false),
        )
    };

    let final_amount_in = if !target_reached && amount_in > amount_rem_u {
        amount_rem_u
    } else {
        amount_in
    };

    (
        sqrt_price_next,
        u128_to_i128_saturating(final_amount_in),
        u128_to_i128_saturating(amount_out)
    )
}
