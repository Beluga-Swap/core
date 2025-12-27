#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, token, Env, Address, Symbol, symbol_short,
};

// ============================================================
// MODULE DECLARATIONS
// ============================================================
mod pool;
mod tick;
mod math;
mod swap;
mod position;
mod twap;

// ============================================================
// INTERNAL IMPORTS
// ============================================================
use crate::tick::{
    TickInfo, read_tick_info, write_tick_info, initialize_tick_if_needed,
    get_fee_growth_inside,
};
use crate::math::{
    ONE_X64, get_liquidity_for_amounts, get_amounts_for_liquidity,
    snap_tick_to_spacing, MIN_LIQUIDITY,
};
use crate::swap::{engine_swap, validate_and_preview_swap, MIN_SWAP_AMOUNT};
use crate::pool::{
    PoolState, PoolConfig, init_pool, read_pool_state, write_pool_state,
    read_pool_config, write_pool_config,
};
use crate::position::{Position, read_position, write_position};

// ============================================================
// DATA KEYS
// ============================================================
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    PoolState,
    PoolConfig,
    Initialized,
    Tick(i32),
    Position(Address, i32, i32),
    TickCross(i32), 
    TWAPObservation(u32),  
    TWAPNewestIndex,       
    TWAPInitialized,
    TickPositions(i32), 
}

// ============================================================
// RETURN TYPES
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub current_tick: i32,
    pub sqrt_price_x64: u128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PositionInfo {
    pub liquidity: i128,
    pub amount0: i128,
    pub amount1: i128,
    pub fees_owed_0: u128,
    pub fees_owed_1: u128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct QuoteResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub final_sqrt_price: u128,
    pub final_tick: i32,
    pub fee_paid: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TickCrossData {
    pub fee_growth_global_0: u128,
    pub fee_growth_global_1: u128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapPreview {
    pub amount_in_used: i128,
    pub amount_out_expected: i128,
    pub fee_paid: i128,
    pub price_impact_bps: i128,
    pub is_valid: bool,
    pub error_message: Option<Symbol>,
}

// ============================================================
// TICK POSITIONS TRACKING
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct PositionKey {
    pub owner: Address,
    pub lower: i32,
    pub upper: i32,
}

/// Register position at tick boundary
fn register_position_at_tick(
    env: &Env,
    tick: i32,
    owner: Address,
    lower: i32,
    upper: i32,
) {
    use soroban_sdk::Vec;
    
    let key = PositionKey {
        owner: owner.clone(),
        lower,
        upper,
    };
    
    let mut positions: Vec<PositionKey> = env
        .storage()
        .persistent()
        .get(&DataKey::TickPositions(tick))
        .unwrap_or(Vec::new(env));
    
    // Check if already exists
    let mut exists = false;
    for p in positions.iter() {
        if p.owner == owner && p.lower == lower && p.upper == upper {
            exists = true;
            break;
        }
    }
    
    if !exists {
        positions.push_back(key);
        env.storage()
            .persistent()
            .set(&DataKey::TickPositions(tick), &positions);
    }
}

/// Unregister position from tick boundary
fn unregister_position_at_tick(
    env: &Env,
    tick: i32,
    owner: &Address,
    lower: i32,
    upper: i32,
) {
    use soroban_sdk::Vec;
    
    let positions_opt: Option<Vec<PositionKey>> = env
        .storage()
        .persistent()
        .get(&DataKey::TickPositions(tick));
    
    if positions_opt.is_none() {
        return;
    }
    
    let old_positions = positions_opt.unwrap();
    let mut new_positions = Vec::new(env);
    
    // Filter out the matching position
    for p in old_positions.iter() {
        if !(p.owner == *owner && p.lower == lower && p.upper == upper) {
            new_positions.push_back(p);
        }
    }
    
    if new_positions.is_empty() {
        env.storage()
            .persistent()
            .remove(&DataKey::TickPositions(tick));
    } else {
        env.storage()
            .persistent()
            .set(&DataKey::TickPositions(tick), &new_positions);
    }
}

/// Get positions at tick
fn get_positions_at_tick(env: &Env, tick: i32) -> soroban_sdk::Vec<PositionKey> {
    use soroban_sdk::Vec;
    
    env.storage()
        .persistent()
        .get(&DataKey::TickPositions(tick))
        .unwrap_or(Vec::new(env))
}
// ============================================================
// FEE UPDATE HELPER
// ============================================================

/// Update position fees before modifying liquidity
fn update_position_fees(
    env: &Env,
    pos: &mut Position,
    lower_tick: i32,
    upper_tick: i32,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) {
    if pos.liquidity == 0 {
        pos.fee_growth_inside_last_0 = fee_growth_inside_0;
        pos.fee_growth_inside_last_1 = fee_growth_inside_1;
        pos.last_update_timestamp = env.ledger().timestamp();
        return;
    }

    let pool = read_pool_state(env);
    let current_tick = pool.current_tick;
    let position_inactive = current_tick < lower_tick || current_tick >= upper_tick;

    let liquidity_u = pos.liquidity as u128;
    let delta_0 = fee_growth_inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
    let delta_1 = fee_growth_inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

    const MAX_REASONABLE_DELTA: u128 = u128::MAX / 2;
    
    let should_use_twap = position_inactive && pos.last_update_timestamp > 0;
    
    if !should_use_twap && delta_0 < MAX_REASONABLE_DELTA && delta_1 < MAX_REASONABLE_DELTA {
        let fee_0 = liquidity_u.wrapping_mul(delta_0) >> 64;
        let fee_1 = liquidity_u.wrapping_mul(delta_1) >> 64;
        pos.tokens_owed_0 = pos.tokens_owed_0.saturating_add(fee_0);
        pos.tokens_owed_1 = pos.tokens_owed_1.saturating_add(fee_1);
    } else {
        use crate::twap;
        
        let (fee_0, fee_1) = twap::calculate_fees_from_twap(
            env,
            pos.liquidity,
            lower_tick,
            upper_tick,
            pos.fee_growth_inside_last_0,
            pos.fee_growth_inside_last_1,
            pos.last_update_timestamp,
        );
        
        pos.tokens_owed_0 = pos.tokens_owed_0.saturating_add(fee_0);
        pos.tokens_owed_1 = pos.tokens_owed_1.saturating_add(fee_1);
    }

    pos.fee_growth_inside_last_0 = fee_growth_inside_0;
    pos.fee_growth_inside_last_1 = fee_growth_inside_1;
    pos.last_update_timestamp = env.ledger().timestamp();
}

// ============================================================
// MAIN CONTRACT
// ============================================================

#[contract]
pub struct ClmmPool;

#[contractimpl]
impl ClmmPool {
    // ========================================================
    // INITIALIZATION
    // ========================================================

    pub fn initialize(
        env: Env,
        admin: Address,
        token_a: Address,
        token_b: Address,
        fee_bps: u32,
        protocol_fee_bps: u32,
        sqrt_price_x64: u128,
        current_tick: i32,
        tick_spacing: i32,
    ) {
        admin.require_auth();

        // Validate not already initialized
        if env.storage().persistent().has(&DataKey::Initialized) {
            panic!("pool already initialized");
        }

        // Validate inputs
        if token_a == token_b {
            panic!("tokens must be different");
        }
        if tick_spacing <= 0 {
            panic!("invalid tick spacing");
        }
        if fee_bps == 0 || fee_bps >= 10000 {
            panic!("invalid fee bps");
        }
        if protocol_fee_bps > 5000 {
            panic!("protocol fee too high");
        }

        let initial_sqrt = if sqrt_price_x64 == 0 {
            ONE_X64
        } else {
            sqrt_price_x64
        };

        // Initialize pool
        init_pool(
            &env,
            initial_sqrt,
            current_tick,
            tick_spacing,
            token_a.clone(),
            token_b.clone(),
        );
         use crate::twap;
    
        let snapped_tick = snap_tick_to_spacing(current_tick, tick_spacing);
    
        twap::initialize_twap(
        &env,
        snapped_tick,
        0,  // fee_growth_global_0 start at 0
        0,  // fee_growth_global_1 start at 0
        0,  // liquidity start at 0
    );

        // Save config
        let cfg = PoolConfig {
            admin,
            token_a,
            token_b,
            fee_bps,
            protocol_fee_bps,
        };
        write_pool_config(&env, &cfg);

        env.storage().persistent().set(&DataKey::Initialized, &true);

        env.events().publish(
            (Symbol::new(&env, "initialized"),),
            (fee_bps, tick_spacing),
        );
    }

    // ========================================================
    // VIEW FUNCTIONS
    // ========================================================

    pub fn get_pool_state(env: Env) -> PoolState {
        read_pool_state(&env)
    }

    pub fn get_pool_config(env: Env) -> PoolConfig {
        read_pool_config(&env)
    }

    pub fn get_tick_info(env: Env, tick: i32) -> TickInfo {
        read_tick_info(&env, tick)
    }

    pub fn get_position(
        env: Env,
        owner: Address,
        lower: i32,
        upper: i32,
    ) -> PositionInfo {
        let pos = read_position(&env, &owner, lower, upper);
        let pool = read_pool_state(&env);

        if pos.liquidity == 0 {
            return PositionInfo {
                liquidity: 0,
                amount0: 0,
                amount1: 0,
                fees_owed_0: pos.tokens_owed_0,
                fees_owed_1: pos.tokens_owed_1,
            };
        }

        // Calculate current amounts
        let sqrt_lower = math::get_sqrt_ratio_at_tick(lower);
        let sqrt_upper = math::get_sqrt_ratio_at_tick(upper);

        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            pos.liquidity,
            sqrt_lower,
            sqrt_upper,
            pool.sqrt_price_x64,
        );

        // Calculate uncollected fees
        let (inside_0, inside_1) = get_fee_growth_inside(
            &env,
            lower,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );

        let delta_0 = inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
        let delta_1 = inside_1.wrapping_sub(pos.fee_growth_inside_last_1);

        // Only calculate fees if delta is reasonable
        const MAX_REASONABLE_DELTA: u128 = u128::MAX / 2;
        
        let pending_0 = if delta_0 < MAX_REASONABLE_DELTA {
            (pos.liquidity as u128).wrapping_mul(delta_0) >> 64
        } else {
            0 // Position was inactive, skip fee calculation
        };
        
        let pending_1 = if delta_1 < MAX_REASONABLE_DELTA {
            (pos.liquidity as u128).wrapping_mul(delta_1) >> 64
        } else {
            0 // Position was inactive, skip fee calculation
        };

        PositionInfo {
            liquidity: pos.liquidity,
            amount0,
            amount1,
            fees_owed_0: pos.tokens_owed_0.saturating_add(pending_0),
            fees_owed_1: pos.tokens_owed_1.saturating_add(pending_1),
        }
    }

    /// Get pending fees for a position (view-only, does not modify state)
    /// 
    /// Returns total claimable fees including:
    /// - Already accrued fees (tokens_owed_0, tokens_owed_1)
    /// - Newly accumulated fees since last update
    /// 
    /// # Arguments
    /// * `owner` - Position owner address
    /// * `lower_tick` - Lower tick boundary
    /// * `upper_tick` - Upper tick boundary
    /// 
    /// # Returns
    /// `(total_fee_0, total_fee_1)` - Total claimable fees for token0 and token1
    pub fn get_pending_fees(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> (u128, u128) {
        let pool = read_pool_state(&env);
        
        // Snap ticks to spacing
        let lower = snap_tick_to_spacing(lower_tick, pool.tick_spacing);
        let upper = snap_tick_to_spacing(upper_tick, pool.tick_spacing);
        
        // Load position
        let pos = read_position(&env, &owner, lower, upper);
        
        // If no liquidity, return only already accrued fees
        if pos.liquidity == 0 {
            return (pos.tokens_owed_0, pos.tokens_owed_1);
        }
        
        // Calculate fee growth inside the position's range
        let (inside_0, inside_1) = get_fee_growth_inside(
            &env,
            lower,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );
        
        // Calculate delta in fee growth since last update
        // Use wrapping_sub to handle fee growth overflow (u128 wraps around)
        let delta_0 = inside_0.wrapping_sub(pos.fee_growth_inside_last_0);
        let delta_1 = inside_1.wrapping_sub(pos.fee_growth_inside_last_1);
        
        // Calculate newly accumulated fees
        // Formula: fee = (liquidity * delta_fee_growth_Q64) / 2^64
        // CRITICAL FIX: Use wrapping_mul (same as update_position_fees)
        let liquidity_u = pos.liquidity as u128;
        let new_fee_0 = liquidity_u.wrapping_mul(delta_0) >> 64;
        let new_fee_1 = liquidity_u.wrapping_mul(delta_1) >> 64;
        
        // Total fees = already accrued + newly accumulated
        let total_fee_0 = pos.tokens_owed_0.saturating_add(new_fee_0);
        let total_fee_1 = pos.tokens_owed_1.saturating_add(new_fee_1);
        
        (total_fee_0, total_fee_1)
    }

    // ========================================================
    // SWAP PREVIEW & QUOTE
    // ========================================================

    pub fn preview_swap(
        env: Env,
        amount_specified: i128,
        min_amount_out: i128,
        zero_for_one: bool,
        sqrt_price_limit_x64: u128,
    ) -> SwapPreview {
        let pool = read_pool_state(&env);
        let config = read_pool_config(&env);

        // Check minimum amount
        if amount_specified < MIN_SWAP_AMOUNT {
            return SwapPreview {
                amount_in_used: 0,
                amount_out_expected: 0,
                fee_paid: 0,
                price_impact_bps: 0,
                is_valid: false,
                error_message: Some(symbol_short!("AMT_LOW")),
            };
        }

        // Check pool liquidity
        if pool.liquidity <= 0 {
            return SwapPreview {
                amount_in_used: 0,
                amount_out_expected: 0,
                fee_paid: 0,
                price_impact_bps: 0,
                is_valid: false,
                error_message: Some(symbol_short!("NO_LIQ")),
            };
        }

        // Try to validate and get preview
        let result = validate_and_preview_swap(
            &env,
            &pool,
            amount_specified,
            min_amount_out,
            zero_for_one,
            sqrt_price_limit_x64,
            config.fee_bps as i128,
        );

        match result {
            Ok((amount_in_used, amount_out, fee_paid, final_price)) => {
                // Calculate price impact
                let price_before = pool.sqrt_price_x64;
                let price_change = if final_price > price_before {
                    final_price - price_before
                } else {
                    price_before - final_price
                };
                
                let price_impact_u128 = if price_before > 0 {
                    price_change.saturating_mul(10000).saturating_div(price_before)
                } else {
                    0
                };
                
                let price_impact_bps = if price_impact_u128 > i128::MAX as u128 {
                    i128::MAX
                } else {
                    price_impact_u128 as i128
                };

                SwapPreview {
                    amount_in_used,
                    amount_out_expected: amount_out,
                    fee_paid,
                    price_impact_bps,
                    is_valid: true,
                    error_message: None,
                }
            }
            Err(err_symbol) => {
                SwapPreview {
                    amount_in_used: 0,
                    amount_out_expected: 0,
                    fee_paid: 0,
                    price_impact_bps: 0,
                    is_valid: false,
                    error_message: Some(err_symbol),
                }
            }
        }
    }

    pub fn quote(
        env: Env,
        amount_in: i128,
        zero_for_one: bool,
        sqrt_price_limit: u128,
    ) -> QuoteResult {
        let pool = read_pool_state(&env);
        let config = read_pool_config(&env);

        let (amount_in_used, amount_out, final_sqrt) = swap::quote_swap(
            &env,
            &pool,
            amount_in,
            zero_for_one,
            sqrt_price_limit,
            config.fee_bps as i128,
        );

        let fee_paid = amount_in_used.saturating_sub(amount_out);
        let mut final_tick = pool.current_tick;
        
        if final_sqrt < pool.sqrt_price_x64 {
            final_tick -= pool.tick_spacing;
        } else if final_sqrt > pool.sqrt_price_x64 {
            final_tick += pool.tick_spacing;
        }

        QuoteResult {
            amount_in: amount_in_used,
            amount_out,
            final_sqrt_price: final_sqrt,
            final_tick,
            fee_paid,
        }
    }

    // ========================================================
    // SWAP
    // ========================================================

pub fn swap(
    env: Env,
    caller: Address,
    amount_specified: i128,
    min_amount_out: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
) -> SwapResult {
    caller.require_auth();

    let mut pool = read_pool_state(&env);
    let config = read_pool_config(&env);
    let pool_addr = env.current_contract_address();

    if amount_specified < MIN_SWAP_AMOUNT {
        panic!("swap amount too small");
    }

    let (amount_in_total, amount_out_total) = engine_swap(
        &env,
        &mut pool,
        amount_specified,
        zero_for_one,
        sqrt_price_limit_x64,
        config.fee_bps as i128,
        config.protocol_fee_bps as i128,
    );

    if amount_in_total <= 0 || amount_out_total <= 0 {
        panic!("swap failed");
    }

    if amount_out_total < min_amount_out {
        panic!("insufficient output");
    }

    write_pool_state(&env, &pool);

    // ============================================================
    // TWAP UPDATE 
    // ============================================================
    use crate::twap;
    twap::update_twap(
        &env,
        pool.current_tick,
        pool.fee_growth_global_0,
        pool.fee_growth_global_1,
        pool.liquidity,
    );
    // ============================================================

    let token0 = token::Client::new(&env, &pool.token0);
    let token1 = token::Client::new(&env, &pool.token1);

    if zero_for_one {
        token0.transfer(&caller, &pool_addr, &amount_in_total);
        token1.transfer(&pool_addr, &caller, &amount_out_total);
    } else {
        token1.transfer(&caller, &pool_addr, &amount_in_total);
        token0.transfer(&pool_addr, &caller, &amount_out_total);
    }

    env.events().publish(
        (Symbol::new(&env, "swap"),),
        (amount_in_total, amount_out_total, zero_for_one),
    );

    SwapResult {
        amount_in: amount_in_total,
        amount_out: amount_out_total,
        current_tick: pool.current_tick,
        sqrt_price_x64: pool.sqrt_price_x64,
    }
}

    // ========================================================
    // ADD LIQUIDITY
    // ========================================================

    pub fn add_liquidity(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        amount0_desired: i128,
        amount1_desired: i128,
        amount0_min: i128,
        amount1_min: i128,
    ) -> (i128, i128, i128) {
        owner.require_auth();

        let config = read_pool_config(&env);
        let mut pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();

        let lower = snap_tick_to_spacing(lower_tick, pool.tick_spacing);
        let upper = snap_tick_to_spacing(upper_tick, pool.tick_spacing);

        if lower >= upper {
            panic!("invalid tick range");
        }

        let sqrt_lower = math::get_sqrt_ratio_at_tick(lower);
        let sqrt_upper = math::get_sqrt_ratio_at_tick(upper);

        let liquidity = get_liquidity_for_amounts(
            &env,
            amount0_desired,
            amount1_desired,
            sqrt_lower,
            sqrt_upper,
            pool.sqrt_price_x64,
        );

        if liquidity < MIN_LIQUIDITY {
            panic!("liquidity too low");
        }

        let (amount0_actual, amount1_actual) = get_amounts_for_liquidity(
            &env,
            liquidity,
            sqrt_lower,
            sqrt_upper,
            pool.sqrt_price_x64,
        );

        if amount0_actual < amount0_min || amount1_actual < amount1_min {
            panic!("slippage exceeded");
        }

        let lower_info_init = initialize_tick_if_needed(
            &env,
            lower,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );
        write_tick_info(&env, lower, &lower_info_init);

        let upper_info_init = initialize_tick_if_needed(
            &env,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );
        write_tick_info(&env, upper, &upper_info_init);

        let (inside_0, inside_1) = get_fee_growth_inside(
            &env,
            lower,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );

        let mut pos = read_position(&env, &owner, lower, upper);
        update_position_fees(&env, &mut pos, lower, upper, inside_0, inside_1);
        pos.liquidity = pos.liquidity.saturating_add(liquidity);

        let mut lower_info = read_tick_info(&env, lower);
        let mut upper_info = read_tick_info(&env, upper);

        lower_info.liquidity_gross = lower_info.liquidity_gross.saturating_add(liquidity);
        lower_info.liquidity_net = lower_info.liquidity_net.saturating_add(liquidity);
        write_tick_info(&env, lower, &lower_info);

        upper_info.liquidity_gross = upper_info.liquidity_gross.saturating_add(liquidity);
        upper_info.liquidity_net = upper_info.liquidity_net.saturating_sub(liquidity);
        write_tick_info(&env, upper, &upper_info);

        if pool.current_tick >= lower && pool.current_tick < upper {
            pool.liquidity = pool.liquidity.saturating_add(liquidity);
        }
        write_pool_state(&env, &pool);
        write_position(&env, &owner, lower, upper, &pos);

        register_position_at_tick(&env, lower, owner.clone(), lower, upper);
        register_position_at_tick(&env, upper, owner.clone(), lower, upper);

        if amount0_actual > 0 {
            token::Client::new(&env, &config.token_a)
                .transfer(&owner, &pool_addr, &amount0_actual);
        }

        if amount1_actual > 0 {
            token::Client::new(&env, &config.token_b)
                .transfer(&owner, &pool_addr, &amount1_actual);
        }

        env.events().publish(
            (Symbol::new(&env, "add_liq"),),
            (liquidity, amount0_actual, amount1_actual),
        );

        (liquidity, amount0_actual, amount1_actual)
    }

    // ========================================================
    // REMOVE LIQUIDITY
    // ========================================================

    pub fn remove_liquidity(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_to_remove: i128,
    ) -> (i128, i128) {
        owner.require_auth();

        let config = read_pool_config(&env);
        let mut pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();

        let lower = snap_tick_to_spacing(lower_tick, pool.tick_spacing);
        let upper = snap_tick_to_spacing(upper_tick, pool.tick_spacing);

        let (inside_0, inside_1) = get_fee_growth_inside(
            &env,
            lower,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );

        let mut pos = read_position(&env, &owner, lower, upper);

        if pos.liquidity < liquidity_to_remove {
            panic!("insufficient liquidity");
        }

        update_position_fees(&env, &mut pos, lower, upper, inside_0, inside_1);

        let sqrt_lower = math::get_sqrt_ratio_at_tick(lower);
        let sqrt_upper = math::get_sqrt_ratio_at_tick(upper);

        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            liquidity_to_remove,
            sqrt_lower,
            sqrt_upper,
            pool.sqrt_price_x64,
        );

        pos.liquidity = pos.liquidity.saturating_sub(liquidity_to_remove);
        write_position(&env, &owner, lower, upper, &pos);

        if pos.liquidity == 0 {
        unregister_position_at_tick(&env, lower, &owner, lower, upper);
        unregister_position_at_tick(&env, upper, &owner, lower, upper);
        }

        let mut lower_info = read_tick_info(&env, lower);
        lower_info.liquidity_gross = lower_info.liquidity_gross.saturating_sub(liquidity_to_remove);
        lower_info.liquidity_net = lower_info.liquidity_net.saturating_sub(liquidity_to_remove);
        write_tick_info(&env, lower, &lower_info);

        let mut upper_info = read_tick_info(&env, upper);
        upper_info.liquidity_gross = upper_info.liquidity_gross.saturating_sub(liquidity_to_remove);
        upper_info.liquidity_net = upper_info.liquidity_net.saturating_add(liquidity_to_remove);
        write_tick_info(&env, upper, &upper_info);

        if pool.current_tick >= lower && pool.current_tick < upper {
            pool.liquidity = pool.liquidity.saturating_sub(liquidity_to_remove);
        }
        write_pool_state(&env, &pool);

        if amount0 > 0 {
            token::Client::new(&env, &config.token_a)
                .transfer(&pool_addr, &owner, &amount0);
        }

        if amount1 > 0 {
            token::Client::new(&env, &config.token_b)
                .transfer(&pool_addr, &owner, &amount1);
        }

        env.events().publish(
            (Symbol::new(&env, "rm_liq"),),
            (liquidity_to_remove, amount0, amount1),
        );

        (amount0, amount1)
    }

    // ========================================================
    // COLLECT FEES
    // ========================================================

    /// Collect accumulated fees from a position
    /// 
    pub fn collect(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> (i128, i128) {
        owner.require_auth();

        let config = read_pool_config(&env);
        let pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();

        let lower = snap_tick_to_spacing(lower_tick, pool.tick_spacing);
        let upper = snap_tick_to_spacing(upper_tick, pool.tick_spacing);

        let (inside_0, inside_1) = get_fee_growth_inside(
            &env,
            lower,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );

        let mut pos = read_position(&env, &owner, lower, upper);
        
        update_position_fees(&env, &mut pos, lower, upper, inside_0, inside_1);

        // Convert u128 to i128 for token transfer
        let amount0 = pos.tokens_owed_0 as i128;
        let amount1 = pos.tokens_owed_1 as i128;

        // Validate amounts are reasonable (not overflow bugs)
        if amount0 < 0 || amount1 < 0 {
            panic!("invalid fee amounts detected");
        }

        // Reset fees after collecting
        pos.tokens_owed_0 = 0;
        pos.tokens_owed_1 = 0;
        write_position(&env, &owner, lower, upper, &pos);

        // Transfer fees to owner
        if amount0 > 0 {
            token::Client::new(&env, &config.token_a)
                .transfer(&pool_addr, &owner, &amount0);
        }

        if amount1 > 0 {
            token::Client::new(&env, &config.token_b)
                .transfer(&pool_addr, &owner, &amount1);
        }

        env.events().publish(
            (Symbol::new(&env, "collect"),),
            (amount0, amount1),
        );

        (amount0, amount1)
    }

    // ========================================================
    // ADMIN FUNCTIONS
    // ========================================================

    pub fn collect_protocol(env: Env, caller: Address) -> (u128, u128) {
        caller.require_auth();

        let config = read_pool_config(&env);
        if caller != config.admin {
            panic!("unauthorized");
        }

        let mut pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();

        let amount0 = pool.protocol_fees_0;
        let amount1 = pool.protocol_fees_1;

        pool.protocol_fees_0 = 0;
        pool.protocol_fees_1 = 0;
        write_pool_state(&env, &pool);

        if amount0 > 0 {
            token::Client::new(&env, &config.token_a)
                .transfer(&pool_addr, &caller, &(amount0 as i128));
        }

        if amount1 > 0 {
            token::Client::new(&env, &config.token_b)
                .transfer(&pool_addr, &caller, &(amount1 as i128));
        }

        (amount0, amount1)
    }
}