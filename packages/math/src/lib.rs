// BelugaSwap Math Package

#![no_std]

pub mod constants;
pub mod q64;
pub mod sqrt_price;
pub mod liquidity;

// Re-export commonly used items from constants
pub use constants::*;

// Re-export Q64 arithmetic functions
pub use q64::{
    mul_q64, 
    div_q64, 
    mul_div,
    div_round_up,           // <-- ADDED: was missing from export
    i128_to_u128_safe,      // <-- ADDED: helper function
    u128_to_i128_saturating, // <-- ADDED: helper function
    ONE_X64,                // <-- ADDED: constant
};

// Re-export sqrt price functions
pub use sqrt_price::{
    get_sqrt_ratio_at_tick, 
    tick_to_sqrt_price_x64,  // <-- ADDED: alias function
    get_next_sqrt_price_from_input, 
    get_next_sqrt_price_from_output,
    compute_swap_step_with_target,
};

// Re-export liquidity functions
pub use liquidity::{
    get_liquidity_for_amount0, 
    get_liquidity_for_amount1,
    get_liquidity_for_amounts, 
    get_amounts_for_liquidity,
    get_amount_0_delta, 
    get_amount_1_delta,
};

// Tick utility (kept in lib for backward compatibility)
pub fn snap_tick_to_spacing(tick: i32, spacing: i32) -> i32 {
    if spacing <= 0 {
        panic!("tick_spacing must be positive");
    }
    let rem = tick.rem_euclid(spacing);
    tick - rem
}

// Export MIN_LIQUIDITY for backward compatibility
pub const MIN_LIQUIDITY: i128 = constants::MIN_LIQUIDITY;

// ============================================================
// COMPUTE SWAP STEP (was missing from package)
// ============================================================

use soroban_sdk::Env;

/// Compute a single swap step (without target price)
/// This was in the original math.rs but missing from the refactored package
pub fn compute_swap_step(
    env: &Env,
    sqrt_price_current: u128,
    liquidity: i128,
    amount_remaining: i128,
    zero_for_one: bool,
) -> (u128, i128, i128) {
    const MIN_PRICE_DELTA: u128 = 1;
    
    if liquidity <= 0 || amount_remaining <= 0 {
        return (sqrt_price_current, 0, 0);
    }

    let liq_u = q64::i128_to_u128_safe(liquidity);
    let amt_in_remaining = q64::i128_to_u128_safe(amount_remaining);

    let next_sqrt_price = sqrt_price::get_next_sqrt_price_from_input(
        env, sqrt_price_current, liq_u, amt_in_remaining, zero_for_one
    );

    let price_delta = next_sqrt_price.abs_diff(sqrt_price_current);

    if price_delta < MIN_PRICE_DELTA {
        return (sqrt_price_current, 0, 0);
    }

    let amount_0 = liquidity::get_amount_0_delta(sqrt_price_current, next_sqrt_price, liq_u, true);
    let amount_1 = liquidity::get_amount_1_delta(sqrt_price_current, next_sqrt_price, liq_u, true);

    let (amount_in, amount_out) = if zero_for_one {
        (amount_0, amount_1)
    } else {
        (amount_1, amount_0)
    };

    let final_amount_in = amount_in.min(amt_in_remaining);

    (
        next_sqrt_price,
        q64::u128_to_i128_saturating(final_amount_in),
        q64::u128_to_i128_saturating(amount_out)
    )
}