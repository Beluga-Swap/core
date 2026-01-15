// SPDX-License-Identifier: MIT
// BelugaSwap Math Package

#![no_std]

pub mod constants;
pub mod q64;
pub mod sqrt_price;
pub mod liquidity;

// Re-export commonly used items
pub use constants::*;
pub use q64::{mul_q64, div_q64, mul_div};
pub use sqrt_price::{
    get_sqrt_ratio_at_tick, 
    get_next_sqrt_price_from_input, 
    get_next_sqrt_price_from_output,
    compute_swap_step_with_target,  // <-- ADD THIS
};
pub use liquidity::{
    get_liquidity_for_amount0, get_liquidity_for_amount1,
    get_liquidity_for_amounts, get_amounts_for_liquidity,
    get_amount_0_delta, get_amount_1_delta
};

// Tick utility (moved from math module)
pub fn snap_tick_to_spacing(tick: i32, spacing: i32) -> i32 {
    if spacing <= 0 {
        panic!("tick_spacing must be positive");
    }
    let rem = tick.rem_euclid(spacing);
    tick - rem
}

// Export MIN_LIQUIDITY for backward compatibility
pub const MIN_LIQUIDITY: i128 = constants::MIN_LIQUIDITY;