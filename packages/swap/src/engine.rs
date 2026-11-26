use soroban_sdk::{Env, Symbol};
use belugaswap_math::{
    constants::{MAX_SLIPPAGE_BPS, MAX_SWAP_ITERATIONS, MIN_OUTPUT_AMOUNT, MIN_SWAP_AMOUNT},
    compute_swap_step_with_target, mul_div_wide, get_sqrt_ratio_at_tick, get_tick_at_sqrt_ratio,
    mul_div, ONE_X64,
};
use belugaswap_tick::{ReadOnly, TickStore};

/// Why a swap cannot be executed. The pool maps it to its contract error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwapError {
    AmountTooSmall,
    InvalidFee,
    /// A fee step broke `0 <= fee <= amount_in`; indicates a bug.
    FeeInvariant,
    NoLiquidity,
    OutputTooSmall,
}

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

/// What to swap and at which fees.
#[derive(Clone, Copy, Debug)]
pub struct SwapParams {
    /// Input amount, before fees
    pub amount: i128,
    /// true sells token0 for token1
    pub zero_for_one: bool,
    /// Price the swap may not cross; 0 for none
    pub sqrt_price_limit_x64: u128,
    pub fee_bps: i128,
    /// Creator's share of the fee, in basis points of the fee
    pub creator_fee_bps: i128,
}

/// Result of an executed swap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwapOutcome {
    pub amount_in: i128,
    pub amount_out: i128,
    pub creator_fee: i128,
}

// ============================================================
// PUBLIC SWAP FUNCTIONS
// ============================================================

/// Execute a swap, updating `state` and the ticks it crosses.
pub fn engine_swap(
    env: &Env,
    state: &mut SwapState,
    store: &impl TickStore,
    params: &SwapParams,
) -> Result<SwapOutcome, SwapError> {
    if params.amount < MIN_SWAP_AMOUNT {
        return Err(SwapError::AmountTooSmall);
    }
    run_swap(env, state, store, params, true)
}

/// Simulate a swap without writing anything; creator fee is ignored.
///
/// # Returns
/// `(amount_in_used, amount_out, final_sqrt_price)`
pub fn quote_swap(
    env: &Env,
    state: &SwapState,
    store: &impl TickStore,
    params: &SwapParams,
) -> (i128, i128, u128) {
    if params.amount < MIN_SWAP_AMOUNT {
        return (0, 0, state.sqrt_price_x64);
    }

    let mut sim_state = state.clone();
    let params = SwapParams { creator_fee_bps: 0, ..*params };
    match run_swap(env, &mut sim_state, &ReadOnly(store), &params, false) {
        Ok(outcome) => (outcome.amount_in, outcome.amount_out, sim_state.sqrt_price_x64),
        Err(_) => (0, 0, state.sqrt_price_x64),
    }
}

/// Validate and preview a swap
/// 
/// # Returns
/// `Ok((amount_in_used, amount_out, fee_paid, price_impact_bps, final_price))` or `Err(Symbol)`
/// 
/// # Fee Calculation 
/// fee_paid = amount_in_used * fee_bps / 10000
/// This is calculated in input token units (correct)
///
/// # Price Impact Calculation 
/// price_impact = (expected_out - actual_out) / expected_out * 10000
/// Where expected_out is calculated from spot price before swap
/// Both values are in output token units (apples to apples)
pub fn validate_and_preview_swap(
    env: &Env,
    state: &SwapState,
    store: &impl TickStore,
    params: &SwapParams,
    min_amount_out: i128,
) -> Result<(i128, i128, i128, i128, u128), Symbol> {
    let SwapParams { amount: amount_in, zero_for_one, fee_bps, .. } = *params;

    // Validate input amount
    if amount_in < MIN_SWAP_AMOUNT {
        return Err(Symbol::new(env, "AMT_LOW"));
    }

    // Get actual quote (includes price impact from moving through ticks)
    let (amount_in_used, amount_out, final_price) = quote_swap(env, state, store, params);

    if amount_out < MIN_OUTPUT_AMOUNT && state.liquidity <= 0 {
        return Err(Symbol::new(env, "NO_LIQ"));
    }

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

    // Output at the starting spot price, for the input actually consumed
    let expected_out =
        calculate_expected_output(env, state.sqrt_price_x64, amount_in_used, zero_for_one, fee_bps);

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

    Ok((amount_in_used, amount_out, fee_paid, price_impact_bps, final_price))
}

/// Output of a swap at the current spot price, after the fee.
///
/// The Q64.64 price is applied one square root at a time in 256 bits, because
/// `sqrt_price * sqrt_price` does not fit in u128 once the price reaches 1.
fn calculate_expected_output(
    env: &Env,
    sqrt_price_x64: u128,
    amount_in: i128,
    zero_for_one: bool,
    fee_bps: i128,
) -> i128 {
    if amount_in <= 0 || sqrt_price_x64 == 0 {
        return 0;
    }

    let after_fee = mul_div(env, amount_in as u128, (10000 - fee_bps) as u128, 10000);
    let output = if zero_for_one {
        let half = mul_div(env, after_fee, sqrt_price_x64, ONE_X64);
        mul_div(env, half, sqrt_price_x64, ONE_X64)
    } else {
        let half = mul_div(env, after_fee, ONE_X64, sqrt_price_x64);
        mul_div(env, half, ONE_X64, sqrt_price_x64)
    };

    output.min(i128::MAX as u128) as i128
}

// ============================================================
// INTERNAL SWAP LOGIC
// ============================================================

/// Core swap logic following Uniswap V3 pattern
/// 
/// Creator fee is taken from LP fee (not from total swap amount)
/// Formula: creator_fee = (lp_fee * creator_fee_bps) / 10000
fn run_swap(
    env: &Env,
    state: &mut SwapState,
    store: &impl TickStore,
    params: &SwapParams,
    // false for quotes: stop quietly instead of returning an error
    allow_panic: bool,
) -> Result<SwapOutcome, SwapError> {
    let SwapParams {
        amount: amount_specified,
        zero_for_one,
        sqrt_price_limit_x64,
        fee_bps,
        creator_fee_bps,
    } = *params;

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

        // Check price limit
        if zero_for_one && sqrt_price <= sqrt_limit {
            break;
        }
        if !zero_for_one && sqrt_price >= sqrt_limit {
            break;
        }

        // Find next initialized tick. With none left in this direction there is
        // no liquidity to trade against.
        let next_tick = match store.next_initialized_tick(env, current_tick, zero_for_one) {
            Some(tick) => tick,
            None => break,
        };
        let tick_price = get_sqrt_ratio_at_tick(next_tick);
        let mut sqrt_target = tick_price;

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
                return Err(SwapError::InvalidFee);
            }
            break;
        }

        // Amount available after fee reservation
        let amount_available = amount_remaining
            .saturating_mul(fee_divisor)
            .saturating_div(10000);

        if amount_available < MIN_OUTPUT_AMOUNT {
            break;
        }

        // Compute swap step. An empty range costs nothing: move straight to the
        // next boundary.
        let (sqrt_next, amount_in, amount_out) = if sqrt_price == sqrt_target || liquidity == 0 {
            (sqrt_target, 0, 0)
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

        // A step that reaches the boundary must still cross it, even when it moves
        // no tokens. A partial step that yields nothing is dust: stop.
        let reached_tick = sqrt_next == tick_price;
        if !reached_tick && (amount_in < MIN_OUTPUT_AMOUNT || amount_out < MIN_OUTPUT_AMOUNT) {
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
        if step_fee < 0 || (amount_in > 0 && step_fee > amount_in) {
            if allow_panic {
                return Err(SwapError::FeeInvariant);
            }
            break;
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

        // Update fee growth global for LP (Uniswap V3 style)
        if liquidity > 0 && lp_fee > 0 {
            let fee_u = lp_fee as u128;
            let liq_u = liquidity as u128;
            let growth_delta = mul_div_wide(fee_u, 1, 64, liq_u, 1, false);

            if zero_for_one {
                state.fee_growth_global_0 = state.fee_growth_global_0.wrapping_add(growth_delta);
            } else {
                state.fee_growth_global_1 = state.fee_growth_global_1.wrapping_add(growth_delta);
            }
        }

        // Handle tick crossing
        if reached_tick {
            sqrt_price = sqrt_next;

            let liquidity_net = belugaswap_tick::cross_tick(
                env,
                store,
                next_tick,
                (state.fee_growth_global_0, state.fee_growth_global_1),
            );

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
            // Stopped inside a range: keep the tick in step with the price.
            sqrt_price = sqrt_next;
            current_tick = get_tick_at_sqrt_ratio(sqrt_price);
        } else {
            break;
        }
    }

    // Validate output
    if amount_out_total < MIN_OUTPUT_AMOUNT {
        if !allow_panic {
            return Ok(SwapOutcome { amount_in: 0, amount_out: 0, creator_fee: 0 });
        }
        return Err(if state.liquidity <= 0 {
            SwapError::NoLiquidity
        } else {
            SwapError::OutputTooSmall
        });
    }

    // Update state
    state.sqrt_price_x64 = sqrt_price;
    state.liquidity = liquidity;
    state.current_tick = current_tick;

    Ok(SwapOutcome {
        amount_in: amount_specified.saturating_sub(amount_remaining),
        amount_out: amount_out_total,
        creator_fee: total_creator_fee,
    })
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