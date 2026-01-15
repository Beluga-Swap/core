// Fee Growth Calculations

use soroban_sdk::Env;
use crate::types::TickInfo;

// Note: This function reads from storage, so we need storage trait
// For now, this is a placeholder that assumes read_tick_info is available

pub fn get_fee_growth_inside(
    env: &Env,
    read_tick: impl Fn(&Env, i32) -> TickInfo,
    lower_tick: i32,
    upper_tick: i32,
    current_tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> (u128, u128) {
    let lower_info = read_tick(env, lower_tick);
    let upper_info = read_tick(env, upper_tick);

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

    let fee_growth_inside_0 = fee_growth_global_0
        .wrapping_sub(fee_growth_below_0)
        .wrapping_sub(fee_growth_above_0);

    let fee_growth_inside_1 = fee_growth_global_1
        .wrapping_sub(fee_growth_below_1)
        .wrapping_sub(fee_growth_above_1);

    (fee_growth_inside_0, fee_growth_inside_1)
}
