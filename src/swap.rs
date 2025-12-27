use soroban_sdk::{Env, symbol_short};

use crate::pool::PoolState;
use crate::math::{compute_swap_step_with_target, get_sqrt_ratio_at_tick, div_q64};
use crate::tick::{find_next_initialized_tick, cross_tick};

// ============================================================
// CONSTANTS
// ============================================================

/// Minimum swap amount (1 stroop = 0.0000001 XLM)
pub const MIN_SWAP_AMOUNT: i128 = 1;

/// Minimum output amount to prevent dust swaps
pub const MIN_OUTPUT_AMOUNT: i128 = 1;

/// Maximum slippage tolerance for auto-reject (50% = 5000 bps)
pub const MAX_SLIPPAGE_BPS: i128 = 5000;

// ============================================================
// SWAP ENGINE
// ============================================================

/// Safe swap engine for preview (doesn't panic)
/// Returns (amount_in_including_fee, amount_out)
fn engine_swap_safe(
    env: &Env,
    pool: &mut PoolState,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
    protocol_fee_bps: i128,
) -> (i128, i128) {
    // Validate without panic
    if amount_specified < MIN_SWAP_AMOUNT || amount_specified <= 0 {
        return (0, 0);
    }

    if pool.liquidity <= 0 {
        return (0, 0);
    }

    // Use same logic as engine_swap but without panics
    engine_swap_internal(env, pool, amount_specified, zero_for_one, sqrt_price_limit_x64, fee_bps, protocol_fee_bps, false)
}

/// Main swap execution engine (with panic for actual swaps)
/// Returns (amount_in_including_fee, amount_out)
pub fn engine_swap(
    env: &Env,
    pool: &mut PoolState,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
    protocol_fee_bps: i128,
) -> (i128, i128) {
    // CRITICAL: Validate minimum amount
    if amount_specified < MIN_SWAP_AMOUNT {
        panic!("swap amount too small, minimum is {}", MIN_SWAP_AMOUNT);
    }

    if amount_specified <= 0 {
        return (0, 0);
    }

    if pool.liquidity <= 0 {
        panic!("no liquidity available");
    }

    engine_swap_internal(env, pool, amount_specified, zero_for_one, sqrt_price_limit_x64, fee_bps, protocol_fee_bps, true)
}

/// Internal swap engine implementation
fn engine_swap_internal(
    env: &Env,
    pool: &mut PoolState,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
    protocol_fee_bps: i128,
    allow_panic: bool,
) -> (i128, i128) {

    let mut amount_remaining = amount_specified;
    let mut amount_out_total: i128 = 0;
    let mut total_protocol_fee: i128 = 0;

    let mut sqrt_price = pool.sqrt_price_x64;
    let mut liquidity = pool.liquidity;
    let mut current_tick = pool.current_tick;

    // Set price limit if not specified
    let sqrt_limit = if sqrt_price_limit_x64 == 0 {
        if zero_for_one {
            100 // Minimum valid sqrt price
        } else {
            u128::MAX - 1000 // Maximum valid sqrt price
        }
    } else {
        sqrt_price_limit_x64
    };

    // Prevent infinite loops
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 1024;

    while iterations < MAX_ITERATIONS {
        iterations += 1;

        // Exit conditions
        if amount_remaining <= 0 {
            break;
        }

        if liquidity <= 0 {
            break;
        }

        // Check if we've reached price limit
        if zero_for_one && sqrt_price <= sqrt_limit {
            break;
        }
        if !zero_for_one && sqrt_price >= sqrt_limit {
            break;
        }

        // Find next initialized tick
        let next_tick = find_next_initialized_tick(
            env,
            current_tick,
            pool.tick_spacing,
            zero_for_one,
        );

        let mut sqrt_target = get_sqrt_ratio_at_tick(next_tick);

        // Adjust target to not exceed limit
        if zero_for_one {
            if sqrt_target < sqrt_limit {
                sqrt_target = sqrt_limit;
            }
        } else if sqrt_target > sqrt_limit {
            sqrt_target = sqrt_limit;
        }

        // Calculate amount available after fee deduction
        // amount_available = amount_remaining * (10000 - fee_bps) / 10000
        let fee_divisor = 10000 - fee_bps;
        if fee_divisor <= 0 {
            if allow_panic {
                panic!("fee too high");
            } else {
                break;  // Preview mode: just break the loop
            }
        }

        let amount_available = amount_remaining
            .saturating_mul(fee_divisor)
            .saturating_div(10000);

        // CRITICAL: Check if amount_available is too small
        if amount_available < MIN_OUTPUT_AMOUNT {
            break;
        }

        // Compute swap step
        let (sqrt_next, amount_in, amount_out) = if sqrt_price == sqrt_target {
            (sqrt_price, 0, 0)
        } else {
            compute_swap_step_with_target(
                env,
                sqrt_price,
                liquidity,
                amount_available,
                zero_for_one,
                sqrt_target,
            )
        };

        // CRITICAL: Enhanced dust protection
        if amount_in < MIN_OUTPUT_AMOUNT || amount_out < MIN_OUTPUT_AMOUNT {
            break;
        }

        // Calculate fee on this step
        let step_fee = if amount_in == amount_available {
            // Used all remaining amount - fee is the difference
            amount_remaining.saturating_sub(amount_in)
        } else {
            // Calculate proportional fee (round up)
            let fee_num = amount_in.saturating_mul(fee_bps);
            let fee = fee_num.saturating_div(fee_divisor);
            // Round up
            if fee_num % fee_divisor != 0 {
                fee.saturating_add(1)
            } else {
                fee
            }
        };

        // Validate fee is reasonable
        if step_fee < 0 || step_fee > amount_in {
            if allow_panic {
                panic!("invalid fee calculation");
            } else {
                break;  // Preview mode: break the loop
            }
        }

        // Calculate protocol fee portion
        let protocol_fee = if protocol_fee_bps > 0 && step_fee > 0 {
            let pf = step_fee.saturating_mul(protocol_fee_bps).saturating_div(10000);
            if pf > 0 {
                pf
            } else {
                0
            }
        } else {
            0
        };

        // LP fee = total fee - protocol fee
        let lp_fee = step_fee.saturating_sub(protocol_fee);

        // Update running totals
        amount_remaining = amount_remaining.saturating_sub(amount_in).saturating_sub(step_fee);
        amount_out_total = amount_out_total.saturating_add(amount_out);
        total_protocol_fee = total_protocol_fee.saturating_add(protocol_fee);

        // Update global fee growth for LPs
        if liquidity > 0 && lp_fee > 0 {
            let fee_u = lp_fee as u128;
            let liq_u = liquidity as u128;

            // fee_growth_delta = (fee * 2^64) / liquidity
            let growth_delta = div_q64(fee_u, liq_u);

            if zero_for_one {
                // Swapping Token0 -> Token1, fee is in Token0
                pool.fee_growth_global_0 = pool
                    .fee_growth_global_0
                    .wrapping_add(growth_delta);
            } else {
                // Swapping Token1 -> Token0, fee is in Token1
                pool.fee_growth_global_1 = pool
                    .fee_growth_global_1
                    .wrapping_add(growth_delta);
            }
        }

        // Check if we reached target and should cross tick
        let target_reached = sqrt_next == sqrt_target;
        let moving_forward = if zero_for_one {
            sqrt_target <= sqrt_price
        } else {
            sqrt_target >= sqrt_price
        };
        let at_user_limit = sqrt_price_limit_x64 != 0 && sqrt_target == sqrt_limit;

        if target_reached && moving_forward && !at_user_limit {
            // We reached an initialized tick - cross it
            sqrt_price = sqrt_target;

            cross_tick(
                env,
                next_tick,
                &mut liquidity,
                pool.fee_growth_global_0,
                pool.fee_growth_global_1,
                zero_for_one,
            );

            // Update current tick
            if zero_for_one {
                current_tick = next_tick.saturating_sub(1);
            } else {
                current_tick = next_tick;
            }
        } else if sqrt_next != sqrt_price {
            // Price moved but didn't reach target
            sqrt_price = sqrt_next;

            if amount_remaining <= 0 {
                break;
            }
        } else {
            // No price movement - exit
            break;
        }
    }

    // CRITICAL: Validate output amount
    if amount_out_total < MIN_OUTPUT_AMOUNT {
        if allow_panic {
            panic!("output amount too small, got {}, minimum is {}", 
                amount_out_total, MIN_OUTPUT_AMOUNT);
        } else {
            // Preview mode: return zero instead of panic
            return (0, 0);
        }
    }

    // Update pool state
    pool.sqrt_price_x64 = sqrt_price;
    pool.liquidity = liquidity;
    pool.current_tick = current_tick;

    // Update protocol fee accumulation
    if total_protocol_fee > 0 {
        if zero_for_one {
            pool.protocol_fees_0 = pool
                .protocol_fees_0
                .saturating_add(total_protocol_fee as u128);
        } else {
            pool.protocol_fees_1 = pool
                .protocol_fees_1
                .saturating_add(total_protocol_fee as u128);
        }
    }

    // Emit sync event
    env.events().publish(
        (symbol_short!("synctk"),),
        (pool.current_tick, pool.sqrt_price_x64),
    );

    // Return total amounts (including fees paid)
    let amount_in_total = amount_specified.saturating_sub(amount_remaining);
    (amount_in_total, amount_out_total)
}

// ============================================================
// SWAP QUOTE (DRY RUN)
// ============================================================

/// Calculate swap output without executing (for quotes)
/// Returns (amount_in_used, amount_out, final_sqrt_price)
pub fn quote_swap(
    env: &Env,
    pool: &PoolState,
    amount_in: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
) -> (i128, i128, u128) {
    // Validate minimum amount
    if amount_in < MIN_SWAP_AMOUNT {
        return (0, 0, pool.sqrt_price_x64);
    }

    if amount_in <= 0 || pool.liquidity <= 0 {
        return (0, 0, pool.sqrt_price_x64);
    }

    // Clone pool state for simulation
    let mut sim_pool = pool.clone();

    // Use safe version that doesn't panic
    let (amount_in_used, amount_out) = engine_swap_safe(
        env,
        &mut sim_pool,
        amount_in,
        zero_for_one,
        sqrt_price_limit_x64,
        fee_bps,
        0, // No protocol fee in quote
    );

    (amount_in_used, amount_out, sim_pool.sqrt_price_x64)
}

// ============================================================
// SWAP VALIDATION & PREVIEW
// ============================================================

use soroban_sdk::Symbol;

/// Validate swap parameters and return expected output
/// This should ALWAYS be called before executing a swap
/// Returns Result<(amount_in_used, amount_out, fee_paid, final_price), error_symbol>
pub fn validate_and_preview_swap(
    env: &Env,
    pool: &PoolState,
    amount_in: i128,
    min_amount_out: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
) -> Result<(i128, i128, i128, u128), Symbol> {
    // Check minimum input
    if amount_in < MIN_SWAP_AMOUNT {
        return Err(symbol_short!("AMT_LOW"));
    }

    // Check pool has liquidity
    if pool.liquidity <= 0 {
        return Err(symbol_short!("NO_LIQ"));
    }

    // Get quote
    let (amount_in_used, amount_out, final_price) = quote_swap(
        env,
        pool,
        amount_in,
        zero_for_one,
        sqrt_price_limit_x64,
        fee_bps,
    );

    // Validate output meets minimum
    if amount_out < min_amount_out {
        return Err(symbol_short!("SLIP_HI"));
    }

    // Validate output is above dust threshold
    if amount_out < MIN_OUTPUT_AMOUNT {
        return Err(symbol_short!("OUT_DUST"));
    }

    // Calculate fee paid
    let fee_paid = amount_in_used.saturating_sub(amount_out);
    let slippage_bps = if amount_in_used > 0 {
        (amount_in.saturating_sub(amount_out))
            .saturating_mul(10000)
            .saturating_div(amount_in)
    } else {
        0
    };

    // Check for excessive slippage
    if slippage_bps > MAX_SLIPPAGE_BPS {
        return Err(symbol_short!("SLIP_MAX"));
    }

    Ok((amount_in_used, amount_out, fee_paid, final_price))
}