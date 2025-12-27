use soroban_sdk::{Env, contracttype};
use crate::DataKey;
use crate::math::snap_tick_to_spacing;

// ============================================================
// TICK DATA STRUCTURE
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct TickInfo {
    pub liquidity_gross: i128, // Total liquidity referencing this tick
    pub liquidity_net: i128,   // Liquidity delta when crossing this tick

    // Fee growth on the "other side" of this tick
    // These are used to calculate fee_growth_inside for positions
    pub fee_growth_outside_0: u128,
    pub fee_growth_outside_1: u128,
}

// ============================================================
// STORAGE HELPERS
// ============================================================

pub fn read_tick_info(env: &Env, tick: i32) -> TickInfo {
    env.storage()
        .persistent()
        .get::<_, TickInfo>(&DataKey::Tick(tick))
        .unwrap_or(TickInfo {
            liquidity_gross: 0,
            liquidity_net: 0,
            fee_growth_outside_0: 0,
            fee_growth_outside_1: 0,
        })
}

pub fn write_tick_info(env: &Env, tick: i32, info: &TickInfo) {
    if info.liquidity_gross == 0 {
        // Remove tick from storage if no liquidity references it
        env.storage().persistent().remove(&DataKey::Tick(tick));
    } else {
        env.storage()
            .persistent()
            .set::<_, TickInfo>(&DataKey::Tick(tick), info);
    }
}

// ============================================================
// TICK TRAVERSAL
// ============================================================

/// Find the next initialized tick in the given direction
/// Returns the current tick if no initialized tick is found within max_steps
pub fn find_next_initialized_tick(
    env: &Env,
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

    // Snap current tick to spacing
    let mut tick = snap_tick_to_spacing(current_tick, tick_spacing);
    
    // CRITICAL FIX: Always move to NEXT tick before checking
    // This prevents returning the same tick we just crossed (double crossing bug)
    // 
    // Previous bug: For zero_for_one, it would check snapped tick first.
    // If current_tick = -51, snap(-51, 10) = -50, and tick -50 was just crossed.
    // The old code would return -50 again, causing double crossing!
    //
    // New behavior: Always skip to next tick before searching.
    tick = tick.saturating_add(step);

    // Search for next initialized tick
    let max_steps: i32 = 2000;
    for _ in 0..max_steps {
        // Check bounds
        if !(crate::math::MIN_TICK..=crate::math::MAX_TICK).contains(&tick) {
            return current_tick;
        }

        let maybe_info = env
            .storage()
            .persistent()
            .get::<_, TickInfo>(&DataKey::Tick(tick));
        
        if let Some(info) = maybe_info {
            if info.liquidity_gross > 0 {
                return tick;
            }
        }
        
        tick = tick.saturating_add(step);
    }

    // No initialized tick found
    current_tick
}

// ============================================================
// TICK CROSSING
// ============================================================

/// Cross a tick boundary, updating liquidity and flipping fee growth
pub fn cross_tick(
    env: &Env,
    tick: i32,
    liquidity: &mut i128,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
    zero_for_one: bool,
) {
    let mut info = read_tick_info(env, tick);

    // Update active liquidity based on direction
    if zero_for_one {
        // Moving down: subtract liquidity_net
        *liquidity = liquidity.saturating_sub(info.liquidity_net);
    } else {
        // Moving up: add liquidity_net
        *liquidity = liquidity.saturating_add(info.liquidity_net);
    }

    // ============================================================
    // CHECKPOINT: Save fee_growth BEFORE flipping
    // ============================================================
    // This checkpoint allows reconstruction of fees for positions
    // that went inactive during this tick cross.
    // Gas cost: ~500 gas (one persistent storage write)
    use crate::DataKey;
    use soroban_sdk::contracttype;
    
    #[contracttype]
    #[derive(Clone, Debug)]
    pub struct TickCrossData {
        pub fee_growth_global_0: u128,
        pub fee_growth_global_1: u128,
    }
    
    let cross_data = TickCrossData {
        fee_growth_global_0,
        fee_growth_global_1,
    };
    
    env.storage()
        .persistent()
        .set(&DataKey::TickCross(tick), &cross_data);
    // ============================================================

    // NEW: Update positions at this tick BEFORE flipping
    // This saves fees for positions going inactive
    use crate::{get_positions_at_tick, read_position, write_position, 
                update_position_fees, read_pool_state, get_fee_growth_inside};
    
    let positions = get_positions_at_tick(env, tick);
    let pool = read_pool_state(env);
    
    const MAX_UPDATES: usize = 10;
    for pos_key in positions.iter().take(MAX_UPDATES) {
        let mut pos = read_position(env, &pos_key.owner, pos_key.lower, pos_key.upper);
        
        if pos.liquidity > 0 {
            let (inside_0, inside_1) = get_fee_growth_inside(
                env,
                pos_key.lower,
                pos_key.upper,
                pool.current_tick,
                fee_growth_global_0,
                fee_growth_global_1,
            );
            
            update_position_fees(env, &mut pos, pos_key.lower, pos_key.upper, inside_0, inside_1);
            write_position(env, &pos_key.owner, pos_key.lower, pos_key.upper, &pos);
        }
    }

    // Flip fee growth outside
    info.fee_growth_outside_0 = fee_growth_global_0.wrapping_sub(info.fee_growth_outside_0);
    info.fee_growth_outside_1 = fee_growth_global_1.wrapping_sub(info.fee_growth_outside_1);

    write_tick_info(env, tick, &info);

}

// ============================================================
// TICK INITIALIZATION
// ============================================================

/// Initialize a tick's fee growth when first liquidity is added
pub fn initialize_tick_if_needed(
    env: &Env,
    tick: i32,
    current_tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> TickInfo {
    let mut info = read_tick_info(env, tick);

    // Only initialize if this is the first liquidity referencing this tick
    if info.liquidity_gross == 0 {
        // Set fee_growth_outside based on current tick position
        if current_tick >= tick {
            // Tick is below or at current price
            // Outside = all fees that have accumulated so far
            info.fee_growth_outside_0 = fee_growth_global_0;
            info.fee_growth_outside_1 = fee_growth_global_1;
        } else {
            // Tick is above current price
            // Outside = 0 (no fees have been earned "outside" yet)
            info.fee_growth_outside_0 = 0;
            info.fee_growth_outside_1 = 0;
        }
    }

    info
}

// ============================================================
// FEE GROWTH INSIDE CALCULATION
// ============================================================

/// Calculate fee growth inside a tick range
/// This is used to determine how much fees a position has earned
pub fn get_fee_growth_inside(
    env: &Env,
    lower_tick: i32,
    upper_tick: i32,
    current_tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> (u128, u128) {
    let lower_info = read_tick_info(env, lower_tick);
    let upper_info = read_tick_info(env, upper_tick);

    // Calculate fee growth below lower tick
    let (below_0, below_1) = if current_tick >= lower_tick {
        // Current tick is above lower tick
        // Below = outside (fees earned below lower tick)
        (lower_info.fee_growth_outside_0, lower_info.fee_growth_outside_1)
    } else {
        // Current tick is below lower tick
        // Below = global - outside (fees earned above lower tick, which is "below" from range perspective)
        (
            fee_growth_global_0.wrapping_sub(lower_info.fee_growth_outside_0),
            fee_growth_global_1.wrapping_sub(lower_info.fee_growth_outside_1),
        )
    };

    // Calculate fee growth above upper tick
    let (above_0, above_1) = if current_tick < upper_tick {
        // Current tick is below upper tick
        // Above = outside (fees earned above upper tick)
        (upper_info.fee_growth_outside_0, upper_info.fee_growth_outside_1)
    } else {
        // Current tick is above upper tick
        // Above = global - outside (fees earned below upper tick, which is "above" from range perspective)
        (
            fee_growth_global_0.wrapping_sub(upper_info.fee_growth_outside_0),
            fee_growth_global_1.wrapping_sub(upper_info.fee_growth_outside_1),
        )
    };

    // Fee growth inside = Global - Below - Above
    let inside_0 = fee_growth_global_0
        .wrapping_sub(below_0)
        .wrapping_sub(above_0);

    let inside_1 = fee_growth_global_1
        .wrapping_sub(below_1)
        .wrapping_sub(above_1);

    (inside_0, inside_1)
}