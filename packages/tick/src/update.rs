// Tick Update and Crossing Logic

use soroban_sdk::Env;
use crate::types::TickInfo;

// ============================================================
// TICK UPDATE (Called when modifying liquidity)
// ============================================================

/// Update a tick when liquidity is added or removed
/// 
/// This follows Uniswap V3's Tick.update() pattern:
/// - Initializes fee_growth_outside based on current tick position
/// - Updates liquidity_gross and liquidity_net
/// - Returns true if tick was flipped (initialized/uninitialized)
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `read_tick` - Callback to read tick info from storage
/// * `write_tick` - Callback to write tick info to storage
/// * `tick` - Tick index to update
/// * `current_tick` - Current pool tick
/// * `liquidity_delta` - Liquidity change (positive = add, negative = remove)
/// * `fee_growth_global_0` - Current global fee growth for token0
/// * `fee_growth_global_1` - Current global fee growth for token1
/// * `upper` - True if this is an upper tick boundary
/// 
/// # Returns
/// `true` if the tick was flipped from uninitialized to initialized (or vice versa)
pub fn update_tick<F1, F2>(
    env: &Env,
    read_tick: F1,
    write_tick: F2,
    tick: i32,
    current_tick: i32,
    liquidity_delta: i128,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
    upper: bool,
) -> bool 
where
    F1: Fn(&Env, i32) -> TickInfo,
    F2: Fn(&Env, i32, &TickInfo),
{
    let mut info = read_tick(env, tick);

    let liquidity_gross_before = info.liquidity_gross;
    let liquidity_gross_after = if liquidity_delta > 0 {
        liquidity_gross_before
            .checked_add(liquidity_delta)
            .expect("tick liquidity overflow")
    } else {
        liquidity_gross_before.saturating_sub(liquidity_delta.abs())
    };

    // Check if tick was flipped (changed between initialized/uninitialized)
    let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

    // Initialize tick if crossing from 0 liquidity
    if liquidity_gross_before == 0 && liquidity_gross_after > 0 {
        // Initialize fee_growth_outside based on current tick position
        // Convention: if current_tick >= tick, assume all fees were earned BELOW this tick
        if current_tick >= tick {
            info.fee_growth_outside_0 = fee_growth_global_0;
            info.fee_growth_outside_1 = fee_growth_global_1;
        } else {
            // All fees were earned ABOVE this tick
            info.fee_growth_outside_0 = 0;
            info.fee_growth_outside_1 = 0;
        }
        info.initialized = true;
    }

    info.liquidity_gross = liquidity_gross_after;

    // Update liquidity_net
    // For lower tick: add liquidity (entering range from left)
    // For upper tick: subtract liquidity (exiting range from left)
    if upper {
        info.liquidity_net = info.liquidity_net.saturating_sub(liquidity_delta);
    } else {
        info.liquidity_net = info.liquidity_net.saturating_add(liquidity_delta);
    }

    // Clear initialized flag if no more liquidity
    if liquidity_gross_after == 0 {
        info.initialized = false;
    }

    write_tick(env, tick, &info);

    flipped
}

// ============================================================
// TICK CROSSING (Uniswap V3 Style)
// ============================================================

/// Cross a tick boundary during a swap
/// 
/// This is the core Uniswap V3 fee tracking mechanism:
/// - `fee_growth_outside` represents fees accumulated on the "other side" of the tick
/// - When crossing, we flip it: `new_outside = global - old_outside`
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `read_tick` - Callback to read tick info from storage
/// * `write_tick` - Callback to write tick info to storage
/// * `tick` - Tick being crossed
/// * `fee_growth_global_0` - Current global fee growth for token0
/// * `fee_growth_global_1` - Current global fee growth for token1
/// 
/// # Returns
/// The `liquidity_net` to add/subtract from active liquidity
pub fn cross_tick<F1, F2>(
    env: &Env,
    read_tick: F1,
    write_tick: F2,
    tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> i128 
where
    F1: Fn(&Env, i32) -> TickInfo,
    F2: Fn(&Env, i32, &TickInfo),
{
    let mut info = read_tick(env, tick);

    // Flip fee_growth_outside using wrapping subtraction
    // After crossing: outside = global - previous_outside
    info.fee_growth_outside_0 = fee_growth_global_0.wrapping_sub(info.fee_growth_outside_0);
    info.fee_growth_outside_1 = fee_growth_global_1.wrapping_sub(info.fee_growth_outside_1);

    write_tick(env, tick, &info);

    info.liquidity_net
}

