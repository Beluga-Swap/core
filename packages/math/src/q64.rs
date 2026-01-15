// SPDX-License-Identifier: MIT
// Q64.64 Fixed-Point Arithmetic Operations

use soroban_sdk::{Env, U256};
use crate::constants::Q64;

pub const ONE_X64: u128 = Q64;

/// Type conversion helpers
#[inline]
pub fn i128_to_u128_safe(x: i128) -> u128 {
    if x <= 0 { 0 } else { x as u128 }
}

#[inline]
pub fn u128_to_i128_saturating(x: u128) -> i128 {
    if x > i128::MAX as u128 { i128::MAX } else { x as i128 }
}

/// Multiply two Q64.64 numbers, returning Q64.64 result
/// Uses decomposition to avoid overflow
#[inline]
pub fn mul_q64(a: u128, b: u128) -> u128 {
    let a_hi = a >> 64;
    let a_lo = a & 0xFFFFFFFFFFFFFFFF;
    let b_hi = b >> 64;
    let b_lo = b & 0xFFFFFFFFFFFFFFFF;

    let term_hh = a_hi * b_hi;
    let term_hl = a_hi * b_lo;
    let term_lh = a_lo * b_hi;
    let term_ll = a_lo * b_lo;

    (term_hh << 64) + term_hl + term_lh + (term_ll >> 64)
}

/// Divide in Q64.64 format: (a * 2^64) / b
#[inline]
pub fn div_q64(a: u128, b: u128) -> u128 {
    if b == 0 { return u128::MAX; }

    if a <= (u128::MAX >> 64) {
        return (a << 64) / b;
    }

    let q = a / b;
    let r = a % b;

    let q_part = q << 64;
    let r_part = if r <= (u128::MAX >> 64) {
        (r << 64) / b
    } else {
        ((r >> 32) << 32) / (b >> 32).max(1)
    };

    q_part.saturating_add(r_part)
}

/// Safe multiply-divide using U256 to prevent overflow
/// Calculates: (a * b) / denominator
pub fn mul_div(env: &Env, a: u128, b: u128, denominator: u128) -> u128 {
    if denominator == 0 { panic!("mul_div: divide by zero"); }

    let a_256 = U256::from_u128(env, a);
    let b_256 = U256::from_u128(env, b);
    let den_256 = U256::from_u128(env, denominator);

    let product = a_256.mul(&b_256);
    let result = product.div(&den_256);

    result.to_u128().unwrap_or(u128::MAX)
}

/// Divide with rounding up
#[inline]
pub fn div_round_up(numerator: u128, denominator: u128) -> u128 {
    if denominator == 0 { return 0; }
    let result = numerator / denominator;
    if numerator % denominator != 0 {
        result.saturating_add(1)
    } else {
        result
    }
}
