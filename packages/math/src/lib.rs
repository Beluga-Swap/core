// BelugaSwap Math Package

#![no_std]

pub mod constants;
pub mod q64;
pub mod wide;
pub mod sqrt_price;
pub mod liquidity;

// Re-export commonly used items from constants
pub use constants::*;

// Re-export Q64 arithmetic functions
pub use q64::{
    mul_q64, 
    mul_div,
    i128_to_u128_safe,
    u128_to_i128_saturating,
    ONE_X64,
};

pub use wide::mul_div_wide;

// Re-export sqrt price functions
pub use sqrt_price::{
    get_sqrt_ratio_at_tick,
    get_tick_at_sqrt_ratio,
    get_next_sqrt_price_from_input, 
    compute_swap_step_with_target,
};

// Re-export liquidity functions
pub use liquidity::{
    get_liquidity_for_amount0, 
    get_liquidity_for_amount1,
    get_liquidity_for_amounts, 
    get_amounts_for_liquidity,
    get_amounts_for_liquidity_rounding_up,
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

