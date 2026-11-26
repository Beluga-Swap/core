use soroban_sdk::Env;

use crate::TickStore;

/// Fee growth per unit of liquidity earned inside `[lower_tick, upper_tick)`.
pub fn get_fee_growth_inside(
    env: &Env,
    store: &impl TickStore,
    lower_tick: i32,
    upper_tick: i32,
    current_tick: i32,
    (fee_growth_global_0, fee_growth_global_1): (u128, u128),
) -> (u128, u128) {
    let lower_info = store.read_tick(env, lower_tick);
    let upper_info = store.read_tick(env, upper_tick);

    let (fee_growth_below_0, fee_growth_below_1) = if current_tick >= lower_tick {
        (lower_info.fee_growth_outside_0, lower_info.fee_growth_outside_1)
    } else {
        (
            fee_growth_global_0.wrapping_sub(lower_info.fee_growth_outside_0),
            fee_growth_global_1.wrapping_sub(lower_info.fee_growth_outside_1),
        )
    };

    let (fee_growth_above_0, fee_growth_above_1) = if current_tick < upper_tick {
        (upper_info.fee_growth_outside_0, upper_info.fee_growth_outside_1)
    } else {
        (
            fee_growth_global_0.wrapping_sub(upper_info.fee_growth_outside_0),
            fee_growth_global_1.wrapping_sub(upper_info.fee_growth_outside_1),
        )
    };

    (
        fee_growth_global_0
            .wrapping_sub(fee_growth_below_0)
            .wrapping_sub(fee_growth_above_0),
        fee_growth_global_1
            .wrapping_sub(fee_growth_below_1)
            .wrapping_sub(fee_growth_above_1),
    )
}
