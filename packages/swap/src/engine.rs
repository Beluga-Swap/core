use soroban_sdk::{Env, Symbol};
use belugaswap_math::{
    constants::{MAX_SLIPPAGE_BPS, MAX_SWAP_ITERATIONS, MIN_OUTPUT_AMOUNT, MIN_SWAP_AMOUNT},
    compute_swap_step_with_target, div_q64, get_sqrt_ratio_at_tick,
    mul_div,
};
use belugaswap_tick::TickInfo;

// ============================================================
// SWAP STATE
// ============================================================

/// Swap state passed from contract
/// Contains the minimal pool state needed for swap calculations
#[derive(Clone)]
pub struct SwapState {
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub liquidity: i128,
    pub tick_spacing: i32,
    pub fee_growth_global_0: u128,
    pub fee_growth_global_1: u128,
}

// ============================================================
// PUBLIC SWAP FUNCTIONS
// ============================================================

/// Execute a swap with callbacks for storage access
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `state` - Mutable swap state
/// * `read_tick` - Callback to read tick info from storage
/// * `write_tick` - Callback to write tick info to storage
/// * `emit_sync` - Callback to emit sync event
/// * `amount_specified` - Input amount
/// * `zero_for_one` - Direction (true = token0 -> token1)
/// * `sqrt_price_limit_x64` - Price limit (0 for no limit)
/// * `fee_bps` - Fee in basis points
/// * `creator_fee_bps` - Creator fee in basis points
/// 
/// # Returns
/// `(amount_in, amount_out)`
pub fn engine_swap<F1, F2, F3>(
    env: &Env,
    state: &mut SwapState,
    read_tick: F1,
    write_tick: F2,
    emit_sync: F3,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
    creator_fee_bps: i128,
) -> (i128, i128)
where
    F1: Fn(&Env, i32) -> TickInfo,
    F2: Fn(&Env, i32, &TickInfo),
    F3: Fn(&Env, i32, u128),
{
    if amount_specified < MIN_SWAP_AMOUNT {
        panic!("swap amount too small");
    }

    if amount_specified <= 0 {
        return (0, 0);
    }

    if state.liquidity <= 0 {
        panic!("no liquidity available");
    }

    engine_swap_internal(
        env,
        state,
        read_tick,
        write_tick,
        emit_sync,
        amount_specified,
        zero_for_one,
        sqrt_price_limit_x64,
        fee_bps,
        creator_fee_bps,
        true,  
        false, 
    )
}

/// Quote a swap without executing (read-only simulation)
/// 
/// # Returns
/// `(amount_in_used, amount_out, final_sqrt_price)`
pub fn quote_swap<F>(
    env: &Env,
    state: &SwapState,
    read_tick: F,
    amount_in: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
) -> (i128, i128, u128)
where
    F: Fn(&Env, i32) -> TickInfo,
{
    if amount_in < MIN_SWAP_AMOUNT || amount_in <= 0 || state.liquidity <= 0 {
        return (0, 0, state.sqrt_price_x64);
    }

    // Clone state for simulation (doesn't modify original)
    let mut sim_state = state.clone();

    let (amount_in_used, amount_out) = engine_swap_internal(
        env,
        &mut sim_state,
        read_tick,
        |_, _, _| {},
        |_, _, _| {}, 
        amount_in,
        zero_for_one,
        sqrt_price_limit_x64,
        fee_bps,
        0,    
        false, 
        true, 
    );

    (amount_in_used, amount_out, sim_state.sqrt_price_x64)
}

/// Validate and preview a swap
/// 
/// # Returns
/// `Ok((amount_in_used, amount_out, fee_paid, final_price))` or `Err(Symbol)`
/// 
/// # Fee Calculation
/// fee_paid = amount_in_used * fee_bps / 10000
/// This is calculated in input token units (correct)
/// 
/// # Price Impact Calculation
/// price_impact = (expected_out - actual_out) / expected_out * 10000
/// Where expected_out is calculated from spot price before swap
/// Both values are in output token units (apples to apples)
pub fn validate_and_preview_swap<F>(
    env: &Env,
    state: &SwapState,
    read_tick: F,
    amount_in: i128,
    min_amount_out: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
) -> Result<(i128, i128, i128, u128), Symbol>
where
    F: Fn(&Env, i32) -> TickInfo + Clone,
{
    // Validate input amount
    if amount_in < MIN_SWAP_AMOUNT {
        return Err(Symbol::new(env, "AMT_LOW"));
    }

    // Validate liquidity
    if state.liquidity <= 0 {
        return Err(Symbol::new(env, "NO_LIQ"));
    }

    // Calculate expected output at spot price (before swap)
    // This gives us the "ideal" output without any price impact
    let expected_out = calculate_expected_output(
        state.sqrt_price_x64,
        amount_in,
        zero_for_one,
        fee_bps,
    );

    // Get actual quote (includes price impact from moving through ticks)
    let (amount_in_used, amount_out, final_price) =
        quote_swap(env, state, read_tick, amount_in, zero_for_one, sqrt_price_limit_x64, fee_bps);

    // Check slippage against user's minimum
    if amount_out < min_amount_out {
        return Err(Symbol::new(env, "SLIP_HI"));
    }

    // Check minimum output (dust protection)
    if amount_out < MIN_OUTPUT_AMOUNT {
        return Err(Symbol::new(env, "OUT_DUST"));
    }

    // Calculate fee paid in INPUT token units
    // fee_paid = amount_in_used * fee_bps / 10000
    let fee_paid = amount_in_used
        .saturating_mul(fee_bps)
        .saturating_div(10000);

    // Calculate price impact in basis points
    // price_impact = (expected_out - actual_out) / expected_out * 10000
    // Both are in OUTPUT token units (apples to apples)
    let price_impact_bps = if expected_out > 0 && expected_out > amount_out {
        (expected_out.saturating_sub(amount_out))
            .saturating_mul(10000)
            .saturating_div(expected_out)
    } else {
        0 // No impact or positive impact (unlikely but possible with rounding)
    };

    // Check maximum price impact
    if price_impact_bps > MAX_SLIPPAGE_BPS {
        return Err(Symbol::new(env, "SLIP_MAX"));
    }

    Ok((amount_in_used, amount_out, fee_paid, final_price))
}

/// Calculate expected output at current spot price (no price impact)
/// 
/// This is used to measure price impact by comparing with actual output.
/// 
/// Formula for token0 -> token1 (zero_for_one = true):
///   price = (sqrt_price_x64 / 2^64)^2
///   output = input * price * (1 - fee)
/// 
/// Formula for token1 -> token0 (zero_for_one = false):
///   price = (2^64 / sqrt_price_x64)^2  
///   output = input * price * (1 - fee)
fn calculate_expected_output(
    sqrt_price_x64: u128,
    amount_in: i128,
    zero_for_one: bool,
    fee_bps: i128,
) -> i128 {
    if amount_in <= 0 || sqrt_price_x64 == 0 {
        return 0;
    }

    let amount_in_u = amount_in as u128;
    const Q64: u128 = 1u128 << 64;
    
    // Calculate amount after fee
    let fee_multiplier = (10000 - fee_bps) as u128;
    let amount_after_fee = amount_in_u
        .saturating_mul(fee_multiplier)
        .saturating_div(10000);

    // Calculate output based on direction
    let output = if zero_for_one {
        // token0 -> token1: multiply by price
        // output = amount * (sqrt_price / Q64)^2
        // Simplified: output = amount * sqrt_price^2 / Q64^2
        let price_squared = sqrt_price_x64.saturating_mul(sqrt_price_x64);
        amount_after_fee
            .saturating_mul(price_squared)
            .saturating_div(Q64)
            .saturating_div(Q64)
    } else {
        // token1 -> token0: divide by price
        // output = amount / (sqrt_price / Q64)^2
        // Simplified: output = amount * Q64^2 / sqrt_price^2
        let price_squared = sqrt_price_x64.saturating_mul(sqrt_price_x64);
        if price_squared == 0 {
            return 0;
        }
        amount_after_fee
            .saturating_mul(Q64)
            .saturating_mul(Q64)
            .saturating_div(price_squared)
    };

    // Cap at i128::MAX and return
    if output > i128::MAX as u128 {
        i128::MAX
    } else {
        output as i128
    }
}

// ============================================================
// INTERNAL SWAP LOGIC
// ============================================================

/// Core swap logic following Uniswap V3 pattern
/// 
/// Creator fee is taken from LP fee (not from total swap amount)
/// Formula: creator_fee = (lp_fee * creator_fee_bps) / 10000
fn engine_swap_internal<F1, F2, F3>(
    env: &Env,
    state: &mut SwapState,
    read_tick: F1,
    write_tick: F2,
    emit_sync: F3,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
    creator_fee_bps: i128,
    allow_panic: bool,
    dry_run: bool,
) -> (i128, i128)
where
    F1: Fn(&Env, i32) -> TickInfo,
    F2: Fn(&Env, i32, &TickInfo),
    F3: Fn(&Env, i32, u128),
{
    // Initialize swap state
    let mut amount_remaining = amount_specified;
    let mut amount_out_total: i128 = 0;
    let mut total_creator_fee: i128 = 0;

    let mut sqrt_price = state.sqrt_price_x64;
    let mut liquidity = state.liquidity;
    let mut current_tick = state.current_tick;

    // Set default price limits
    let sqrt_limit = if sqrt_price_limit_x64 == 0 {
        if zero_for_one {
            100 // Minimum price
        } else {
            u128::MAX - 1000 // Maximum price
        }
    } else {
        sqrt_price_limit_x64
    };

    // Main swap loop
    let mut iterations = 0;

    while iterations < MAX_SWAP_ITERATIONS {
        iterations += 1;

        // Exit conditions
        if amount_remaining <= 0 {
            break;
        }

        if liquidity <= 0 {
            break;
        }

        // Check price limit
        if zero_for_one && sqrt_price <= sqrt_limit {
            break;
        }
        if !zero_for_one && sqrt_price >= sqrt_limit {
            break;
        }

        // Find next initialized tick
        let next_tick = belugaswap_tick::find_next_initialized_tick(
            env,
            &read_tick,
            current_tick,
            state.tick_spacing,
            zero_for_one,
        );

        // Get sqrt price at next tick
        let mut sqrt_target = get_sqrt_ratio_at_tick(next_tick);

        // Clamp target to user's price limit
        if zero_for_one {
            if sqrt_target < sqrt_limit {
                sqrt_target = sqrt_limit;
            }
        } else if sqrt_target > sqrt_limit {
            sqrt_target = sqrt_limit;
        }

        // Calculate fee divisor
        let fee_divisor = 10000 - fee_bps;
        if fee_divisor <= 0 {
            if allow_panic {
                panic!("fee too high");
            } else {
                break;
            }
        }

        // Amount available after fee reservation
        let amount_available = amount_remaining
            .saturating_mul(fee_divisor)
            .saturating_div(10000);

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

        // Check minimum amounts
        if amount_in < MIN_OUTPUT_AMOUNT || amount_out < MIN_OUTPUT_AMOUNT {
            break;
        }

        // Calculate step fee
        let step_fee = calculate_step_fee(
            amount_in,
            amount_remaining,
            amount_available,
            fee_bps,
            fee_divisor,
        );

        // Validate fee
        if step_fee < 0 || step_fee > amount_in {
            if allow_panic {
                panic!("invalid fee calculation");
            } else {
                break;
            }
        }

        // Calculate creator fee (percentage of LP fee)
        let creator_fee = if creator_fee_bps > 0 && step_fee > 0 {
            step_fee
                .saturating_mul(creator_fee_bps)
                .saturating_div(10000)
        } else {
            0
        };

        // LP fee = total fee - creator fee
        let lp_fee = step_fee.saturating_sub(creator_fee);

        // Update amounts
        amount_remaining = amount_remaining
            .saturating_sub(amount_in)
            .saturating_sub(step_fee);
        amount_out_total = amount_out_total.saturating_add(amount_out);
        total_creator_fee = total_creator_fee.saturating_add(creator_fee);

        // Update fee growth global for LP
        if liquidity > 0 && lp_fee > 0 {
            let fee_u = lp_fee as u128;
            let liq_u = liquidity as u128;
            let growth_delta = div_q64(fee_u, liq_u);

            if zero_for_one {
                state.fee_growth_global_0 = state.fee_growth_global_0.wrapping_add(growth_delta);
            } else {
                state.fee_growth_global_1 = state.fee_growth_global_1.wrapping_add(growth_delta);
            }
        }

        // Handle tick crossing
        let target_reached = sqrt_next == sqrt_target;
        let should_cross = if zero_for_one {
            sqrt_target <= sqrt_price
        } else {
            sqrt_target >= sqrt_price
        };
        let at_user_limit = sqrt_price_limit_x64 != 0 && sqrt_target == sqrt_limit;

        if target_reached && should_cross && !at_user_limit {
            sqrt_price = sqrt_target;

            // Cross tick - only modify storage if NOT dry_run
            let liquidity_net = if dry_run {
                let tick_info = read_tick(env, next_tick);
                tick_info.liquidity_net
            } else {
                belugaswap_tick::cross_tick(
                    env,
                    &read_tick,
                    &write_tick,
                    next_tick,
                    state.fee_growth_global_0,
                    state.fee_growth_global_1,
                )
            };

            // Update liquidity based on direction
            if zero_for_one {
                liquidity = liquidity.saturating_sub(liquidity_net);
            } else {
                liquidity = liquidity.saturating_add(liquidity_net);
            }

            // Update current tick
            current_tick = if zero_for_one {
                next_tick.saturating_sub(1)
            } else {
                next_tick
            };
        } else if sqrt_next != sqrt_price {
            sqrt_price = sqrt_next;

            if amount_remaining <= 0 {
                break;
            }
        } else {
            break;
        }
    }

    // Validate output
    if amount_out_total < MIN_OUTPUT_AMOUNT {
        if allow_panic {
            panic!("output amount too small");
        } else {
            return (0, 0);
        }
    }

    // Update state
    state.sqrt_price_x64 = sqrt_price;
    state.liquidity = liquidity;
    state.current_tick = current_tick;

    // Emit sync event (no-op in dry_run via callback)
    emit_sync(env, state.current_tick, state.sqrt_price_x64);

    let amount_in_total = amount_specified.saturating_sub(amount_remaining);
    (amount_in_total, amount_out_total)
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

/// Calculate the fee for a swap step
#[inline]
fn calculate_step_fee(
    amount_in: i128,
    amount_remaining: i128,
    amount_available: i128,
    fee_bps: i128,
    fee_divisor: i128,
) -> i128 {
    if amount_in == amount_available {
        // Used all available amount
        amount_remaining.saturating_sub(amount_in)
    } else {
        // Calculate fee on amount_in
        let fee_num = amount_in.saturating_mul(fee_bps);
        let fee = fee_num.saturating_div(fee_divisor);
        if fee_num % fee_divisor != 0 {
            fee.saturating_add(1) // Round up
        } else {
            fee
        }
    }
}