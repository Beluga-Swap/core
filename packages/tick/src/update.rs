// Tick Update and Crossing Logic

use soroban_sdk::Env;
use belugaswap_math::{constants::{MAX_TICK, MAX_TICK_SEARCH_STEPS, MIN_TICK}, snap_tick_to_spacing};
use crate::types::TickInfo;

/// Storage trait for tick operations
/// This allows the update module to work with any storage implementation
pub trait TickStorage {
    fn read_tick_info(&self, env: &Env, tick: i32) -> TickInfo;
    fn write_tick_info(&self, env: &Env, tick: i32, info: &TickInfo);
}

/// Update a tick when liquidity is added or removed
pub fn update_tick(
    env: &Env,
    read_tick: impl Fn(&Env, i32) -> TickInfo,
    write_tick: impl Fn(&Env, i32, &TickInfo),
    tick: i32,
    current_tick: i32,
    liquidity_delta: i128,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
    upper: bool,
) -> bool {
    let mut info = read_tick(env, tick);

    let liquidity_gross_before = info.liquidity_gross;
    let liquidity_gross_after = if liquidity_delta > 0 {
        liquidity_gross_before.saturating_add(liquidity_delta)
    } else {
        liquidity_gross_before.saturating_sub(liquidity_delta.abs())
    };

    let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

    if liquidity_gross_before == 0 && liquidity_gross_after > 0 {
        if current_tick >= tick {
            info.fee_growth_outside_0 = fee_growth_global_0;
            info.fee_growth_outside_1 = fee_growth_global_1;
        } else {
            info.fee_growth_outside_0 = 0;
            info.fee_growth_outside_1 = 0;
        }
        info.initialized = true;
    }

    info.liquidity_gross = liquidity_gross_after;

    if upper {
        info.liquidity_net = info.liquidity_net.saturating_sub(liquidity_delta);
    } else {
        info.liquidity_net = info.liquidity_net.saturating_add(liquidity_delta);
    }

    if liquidity_gross_after == 0 {
        info.initialized = false;
    }

    write_tick(env, tick, &info);

    flipped
}

/// Cross a tick boundary during a swap
pub fn cross_tick(
    env: &Env,
    read_tick: impl Fn(&Env, i32) -> TickInfo,
    write_tick: impl Fn(&Env, i32, &TickInfo),
    tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> i128 {
    let mut info = read_tick(env, tick);

    info.fee_growth_outside_0 = fee_growth_global_0.wrapping_sub(info.fee_growth_outside_0);
    info.fee_growth_outside_1 = fee_growth_global_1.wrapping_sub(info.fee_growth_outside_1);

    write_tick(env, tick, &info);

    info.liquidity_net
}

/// Find the next initialized tick in the given direction
pub fn find_next_initialized_tick(
    env: &Env,
    read_tick: impl Fn(&Env, i32) -> TickInfo,
    current_tick: i32,
    tick_spacing: i32,
    zero_for_one: bool,
) -> i32 {
    if tick_spacing <= 0 {
        return current_tick;
    }

    let step = if zero_for_one {
        -tick_spacing
    } else {
        tick_spacing
    };

    let mut tick = snap_tick_to_spacing(current_tick, tick_spacing);
    tick = tick.saturating_add(step);

    for _ in 0..MAX_TICK_SEARCH_STEPS {
        if !(MIN_TICK..=MAX_TICK).contains(&tick) {
            return current_tick;
        }

        let info = read_tick(env, tick);

        if info.initialized && info.liquidity_gross > 0 {
            return tick;
        }

        tick = tick.saturating_add(step);
    }

    current_tick
}

/// Check if a tick is within valid range
#[inline]
pub fn is_valid_tick(tick: i32) -> bool {
    tick >= MIN_TICK && tick <= MAX_TICK
}
