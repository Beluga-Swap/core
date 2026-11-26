// Tick Update and Crossing Logic

use soroban_sdk::Env;

use crate::TickStore;

/// Apply a liquidity change to both boundaries of a position's range.
///
/// Follows Uniswap V3's `Tick.update()`: a tick that gains its first liquidity
/// records the fee growth on its far side, and a tick that changes between
/// initialized and uninitialized is flipped in the bitmap.
pub fn update_range(
    env: &Env,
    store: &impl TickStore,
    lower_tick: i32,
    upper_tick: i32,
    current_tick: i32,
    liquidity_delta: i128,
    fee_growth_global: (u128, u128),
) {
    for (tick, upper) in [(lower_tick, false), (upper_tick, true)] {
        if update_tick(env, store, tick, current_tick, liquidity_delta, fee_growth_global, upper) {
            store.flip_tick(env, tick);
        }
    }
}

/// Update one boundary tick. Returns whether it was flipped.
fn update_tick(
    env: &Env,
    store: &impl TickStore,
    tick: i32,
    current_tick: i32,
    liquidity_delta: i128,
    (fee_growth_global_0, fee_growth_global_1): (u128, u128),
    upper: bool,
) -> bool {
    let mut info = store.read_tick(env, tick);

    let liquidity_gross_before = info.liquidity_gross;
    let liquidity_gross_after = if liquidity_delta > 0 {
        liquidity_gross_before
            .checked_add(liquidity_delta)
            .expect("tick liquidity overflow")
    } else {
        liquidity_gross_before.saturating_sub(liquidity_delta.abs())
    };

    let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

    if liquidity_gross_before == 0 && liquidity_gross_after > 0 {
        // By convention all fees so far were earned below the tick if the
        // price is at or above it, and above it otherwise.
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

    // Crossing upward enters the range at the lower tick and leaves it at the upper.
    if upper {
        info.liquidity_net = info.liquidity_net.saturating_sub(liquidity_delta);
    } else {
        info.liquidity_net = info.liquidity_net.saturating_add(liquidity_delta);
    }

    if liquidity_gross_after == 0 {
        info.initialized = false;
    }

    store.write_tick(env, tick, &info);

    flipped
}

/// Cross a tick during a swap: flip its outside fee growth
/// (`outside = global - outside`) and return its `liquidity_net`.
pub fn cross_tick(
    env: &Env,
    store: &impl TickStore,
    tick: i32,
    (fee_growth_global_0, fee_growth_global_1): (u128, u128),
) -> i128 {
    let mut info = store.read_tick(env, tick);

    info.fee_growth_outside_0 = fee_growth_global_0.wrapping_sub(info.fee_growth_outside_0);
    info.fee_growth_outside_1 = fee_growth_global_1.wrapping_sub(info.fee_growth_outside_1);

    store.write_tick(env, tick, &info);

    info.liquidity_net
}
