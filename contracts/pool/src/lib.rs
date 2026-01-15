#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env, Symbol};

// External packages
use belugaswap_math::{
    get_amounts_for_liquidity, get_liquidity_for_amounts, snap_tick_to_spacing, 
    MIN_LIQUIDITY, get_sqrt_ratio_at_tick, 
    constants::{MAX_FEE_BPS, MIN_CREATOR_FEE_BPS, MAX_CREATOR_FEE_BPS}
};
use belugaswap_position::{PositionInfo, modify_position, update_position, calculate_pending_fees};
use belugaswap_swap::{SwapState, SwapResult, PreviewResult, engine_swap, validate_and_preview_swap};

// Local modules
mod error;
mod events;
mod storage;
pub mod types;

use error::ErrorMsg;
use events::*;
use storage::*;
use types::{PoolConfig, PoolState, CreatorFeesInfo};

#[contract]
pub struct BelugaPool;

#[contractimpl]
impl BelugaPool {
    // ========================================================
    // INITIALIZATION
    // ========================================================
    
    pub fn initialize(
        env: Env,
        creator: Address,
        token_a: Address,
        token_b: Address,
        fee_bps: u32,
        creator_fee_bps: u32,
        sqrt_price_x64: u128,
        current_tick: i32,
        tick_spacing: i32,
    ) {
        creator.require_auth();
        
        if is_initialized(&env) {
            panic!("{}", ErrorMsg::ALREADY_INITIALIZED);
        }
        
        if fee_bps == 0 || fee_bps > MAX_FEE_BPS {
            panic!("{}", ErrorMsg::INVALID_FEE);
        }
        
        if creator_fee_bps < MIN_CREATOR_FEE_BPS || creator_fee_bps > MAX_CREATOR_FEE_BPS {
            panic!("{}", ErrorMsg::INVALID_CREATOR_FEE);
        }
        
        if tick_spacing <= 0 {
            panic!("{}", ErrorMsg::INVALID_TICK_SPACING);
        }
        
        let (token0, token1) = if token_a < token_b {
            (token_a.clone(), token_b.clone())
        } else {
            (token_b.clone(), token_a.clone())
        };
        
        let config = PoolConfig {
            factory: creator.clone(), // When deployed directly, creator acts as factory
            creator,
            token_a,
            token_b,
            fee_bps,
            creator_fee_bps,
        };
        
        write_pool_config(&env, &config);
        init_pool_state(&env, sqrt_price_x64, current_tick, tick_spacing, token0, token1);
        set_initialized(&env);
        
        emit_initialized(&env, fee_bps, creator_fee_bps, tick_spacing);
        emit_pool_init(&env, sqrt_price_x64, current_tick, tick_spacing);
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
    
    pub fn get_tick_info(env: Env, tick: i32) -> types::TickInfo {
        read_tick_info(&env, tick)
    }
    
    pub fn get_position(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> PositionInfo {
        let state = read_pool_state(&env);
        let pos = read_position(&env, &owner, lower_tick, upper_tick);
        
        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            pos.liquidity,
            get_sqrt_ratio_at_tick(lower_tick),
            get_sqrt_ratio_at_tick(upper_tick),
            state.sqrt_price_x64,
        );
        
        let fee_growth_inside = get_fee_growth_inside_local(
            &env,
            lower_tick,
            upper_tick,
            state.current_tick,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
        );
        
        let (pending0, pending1) = calculate_pending_fees(&pos, fee_growth_inside.0, fee_growth_inside.1);
        
        PositionInfo {
            liquidity: pos.liquidity,
            amount0,
            amount1,
            fees_owed_0: pos.tokens_owed_0.saturating_add(pending0),
            fees_owed_1: pos.tokens_owed_1.saturating_add(pending1),
        }
    }
    
    pub fn get_creator_fees(env: Env) -> CreatorFeesInfo {
        let pool = read_pool_state(&env);
        CreatorFeesInfo {
            fees_token0: pool.creator_fees_0,
            fees_token1: pool.creator_fees_1,
        }
    }
    
    /// Get creator fees with position check - for factory integration
    /// Returns (is_in_range, pending_fees_0, pending_fees_1)
    pub fn get_creator_fees_ex(
        env: Env,
        creator: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> (bool, u128, u128) {
        let state = read_pool_state(&env);
        let config = read_pool_config(&env);
        
        if creator != config.creator {
            return (false, 0, 0);
        }
        
        let is_in_range = state.current_tick >= lower_tick && state.current_tick < upper_tick;
        (is_in_range, state.creator_fees_0, state.creator_fees_1)
    }
    
    pub fn get_swap_direction(env: Env, token_in: Address) -> bool {
        let pool = read_pool_state(&env);
        if token_in == pool.token0 {
            true
        } else if token_in == pool.token1 {
            false
        } else {
            panic!("{}", ErrorMsg::INVALID_TOKEN);
        }
    }
    
    // ========================================================
    // SWAP FUNCTIONS
    // ========================================================
    
    pub fn swap(
        env: Env,
        sender: Address,
        token_in: Address,
        amount_in: i128,
        amount_out_min: i128,
        sqrt_price_limit_x64: u128,
    ) -> SwapResult {
        sender.require_auth();
        
        let pool_state = read_pool_state(&env);
        let config = read_pool_config(&env);
        
        let zero_for_one = if token_in == pool_state.token0 {
            true
        } else if token_in == pool_state.token1 {
            false
        } else {
            panic!("{}", ErrorMsg::INVALID_TOKEN);
        };
        
        let mut swap_state = SwapState {
            sqrt_price_x64: pool_state.sqrt_price_x64,
            current_tick: pool_state.current_tick,
            liquidity: pool_state.liquidity,
            tick_spacing: pool_state.tick_spacing,
            fee_growth_global_0: pool_state.fee_growth_global_0,
            fee_growth_global_1: pool_state.fee_growth_global_1,
        };
        
        let (amount_in_used, amount_out) = engine_swap(
            &env,
            &mut swap_state,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            |e, tick, price| emit_sync_tick(e, tick, price),
            amount_in,
            zero_for_one,
            sqrt_price_limit_x64,
            config.fee_bps as i128,
            config.creator_fee_bps as i128,
        );
        
        if amount_out < amount_out_min {
            panic!("{}", ErrorMsg::SLIPPAGE_EXCEEDED);
        }
        
        let mut updated_pool = pool_state.clone();
        updated_pool.sqrt_price_x64 = swap_state.sqrt_price_x64;
        updated_pool.current_tick = swap_state.current_tick;
        updated_pool.liquidity = swap_state.liquidity;
        updated_pool.fee_growth_global_0 = swap_state.fee_growth_global_0;
        updated_pool.fee_growth_global_1 = swap_state.fee_growth_global_1;
        
        write_pool_state(&env, &updated_pool);
        
        let (token_in_addr, token_out_addr) = if zero_for_one {
            (&pool_state.token0, &pool_state.token1)
        } else {
            (&pool_state.token1, &pool_state.token0)
        };
        
        token::Client::new(&env, token_in_addr).transfer(&sender, &env.current_contract_address(), &amount_in_used);
        token::Client::new(&env, token_out_addr).transfer(&env.current_contract_address(), &sender, &amount_out);
        
        emit_swap(&env, amount_in_used, amount_out, zero_for_one);
        
        SwapResult {
            amount_in: amount_in_used,
            amount_out,
            current_tick: swap_state.current_tick,
            sqrt_price_x64: swap_state.sqrt_price_x64,
        }
    }
    
    pub fn preview_swap(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        min_amount_out: i128,
        sqrt_price_limit_x64: u128,
    ) -> PreviewResult {
        let pool = read_pool_state(&env);
        
        if token_in != pool.token0 && token_in != pool.token1 {
            return PreviewResult::invalid(Symbol::new(&env, "BAD_TOKEN"));
        }
        if token_out != pool.token0 && token_out != pool.token1 {
            return PreviewResult::invalid(Symbol::new(&env, "BAD_TOKEN"));
        }
        if token_in == token_out {
            return PreviewResult::invalid(Symbol::new(&env, "SAME_TKN"));
        }
        
        let zero_for_one = token_in == pool.token0;
        Self::preview_swap_advanced(env, amount_in, min_amount_out, zero_for_one, sqrt_price_limit_x64)
    }
    
    pub fn swap_advanced(
        env: Env,
        caller: Address,
        amount_in: i128,
        min_amount_out: i128,
        zero_for_one: bool,
        sqrt_price_limit_x64: u128,
    ) -> SwapResult {
        caller.require_auth();
        
        let config = read_pool_config(&env);
        let pool_state = read_pool_state(&env);
        
        let mut swap_state = SwapState {
            sqrt_price_x64: pool_state.sqrt_price_x64,
            current_tick: pool_state.current_tick,
            liquidity: pool_state.liquidity,
            tick_spacing: pool_state.tick_spacing,
            fee_growth_global_0: pool_state.fee_growth_global_0,
            fee_growth_global_1: pool_state.fee_growth_global_1,
        };
        
        let (amount_in_actual, amount_out) = engine_swap(
            &env,
            &mut swap_state,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            |e, tick, price| emit_sync_tick(e, tick, price),
            amount_in,
            zero_for_one,
            sqrt_price_limit_x64,
            config.fee_bps as i128,
            config.creator_fee_bps as i128,
        );
        
        if amount_out < min_amount_out {
            panic!("{}", ErrorMsg::SLIPPAGE_EXCEEDED);
        }
        
        let mut updated_pool = pool_state.clone();
        updated_pool.sqrt_price_x64 = swap_state.sqrt_price_x64;
        updated_pool.current_tick = swap_state.current_tick;
        updated_pool.liquidity = swap_state.liquidity;
        updated_pool.fee_growth_global_0 = swap_state.fee_growth_global_0;
        updated_pool.fee_growth_global_1 = swap_state.fee_growth_global_1;
        
        write_pool_state(&env, &updated_pool);
        
        let (token_in_addr, token_out_addr) = if zero_for_one {
            (&pool_state.token0, &pool_state.token1)
        } else {
            (&pool_state.token1, &pool_state.token0)
        };
        
        if amount_in_actual > 0 {
            token::Client::new(&env, token_in_addr).transfer(&caller, &env.current_contract_address(), &amount_in_actual);
        }
        if amount_out > 0 {
            token::Client::new(&env, token_out_addr).transfer(&env.current_contract_address(), &caller, &amount_out);
        }
        
        emit_swap(&env, amount_in_actual, amount_out, zero_for_one);
        
        SwapResult {
            amount_in: amount_in_actual,
            amount_out,
            current_tick: swap_state.current_tick,
            sqrt_price_x64: swap_state.sqrt_price_x64,
        }
    }
    
    pub fn preview_swap_advanced(
        env: Env,
        amount_in: i128,
        min_amount_out: i128,
        zero_for_one: bool,
        sqrt_price_limit_x64: u128,
    ) -> PreviewResult {
        let config = read_pool_config(&env);
        let pool = read_pool_state(&env);
        
        let swap_state = SwapState {
            sqrt_price_x64: pool.sqrt_price_x64,
            current_tick: pool.current_tick,
            liquidity: pool.liquidity,
            tick_spacing: pool.tick_spacing,
            fee_growth_global_0: pool.fee_growth_global_0,
            fee_growth_global_1: pool.fee_growth_global_1,
        };
        
        match validate_and_preview_swap(
            &env,
            &swap_state,
            |e, t| read_tick_info(e, t),
            amount_in,
            min_amount_out,
            zero_for_one,
            sqrt_price_limit_x64,
            config.fee_bps as i128,
        ) {
            Ok((amount_in_used, amount_out, fee_paid, _final_price)) => {
                let price_impact_bps = if amount_in > 0 {
                    (amount_in.saturating_sub(amount_out))
                        .saturating_mul(10000)
                        .saturating_div(amount_in)
                } else {
                    0
                };
                PreviewResult::valid(amount_in_used, amount_out, fee_paid, price_impact_bps)
            }
            Err(error_symbol) => PreviewResult::invalid(error_symbol),
        }
    }
    
    // ========================================================
    // LIQUIDITY FUNCTIONS
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
        
        let mut state = read_pool_state(&env);
        
        let lower_aligned = snap_tick_to_spacing(lower_tick, state.tick_spacing);
        let upper_aligned = snap_tick_to_spacing(upper_tick, state.tick_spacing);
        
        if lower_aligned >= upper_aligned {
            panic!("{}", ErrorMsg::INVALID_TICK_RANGE);
        }
        
        let liquidity = get_liquidity_for_amounts(
            &env,
            amount0_desired,
            amount1_desired,
            get_sqrt_ratio_at_tick(lower_aligned),
            get_sqrt_ratio_at_tick(upper_aligned),
            state.sqrt_price_x64,
        );
        
        if liquidity < MIN_LIQUIDITY {
            panic!("{}", ErrorMsg::LIQUIDITY_TOO_LOW);
        }
        
        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            liquidity,
            get_sqrt_ratio_at_tick(lower_aligned),
            get_sqrt_ratio_at_tick(upper_aligned),
            state.sqrt_price_x64,
        );
        
        if amount0 < amount0_min || amount1 < amount1_min {
            panic!("{}", ErrorMsg::SLIPPAGE_EXCEEDED);
        }
        
        // Update ticks
        belugaswap_tick::update_tick(
            &env,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            lower_aligned,
            state.current_tick,
            liquidity,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
            false,
        );
        
        belugaswap_tick::update_tick(
            &env,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            upper_aligned,
            state.current_tick,
            liquidity,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
            true,
        );
        
        if state.current_tick >= lower_aligned && state.current_tick < upper_aligned {
            state.liquidity = state.liquidity.saturating_add(liquidity);
        }
        
        write_pool_state(&env, &state);
        
        let fee_growth_inside = get_fee_growth_inside_local(
            &env,
            lower_aligned,
            upper_aligned,
            state.current_tick,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
        );
        
        let mut pos = read_position(&env, &owner, lower_aligned, upper_aligned);
        modify_position(&mut pos, liquidity, fee_growth_inside.0, fee_growth_inside.1);
        write_position(&env, &owner, lower_aligned, upper_aligned, &pos);
        
        if amount0 > 0 {
            token::Client::new(&env, &state.token0).transfer(&owner, &env.current_contract_address(), &amount0);
        }
        if amount1 > 0 {
            token::Client::new(&env, &state.token1).transfer(&owner, &env.current_contract_address(), &amount1);
        }
        
        emit_add_liquidity(&env, liquidity, amount0, amount1);
        
        (liquidity, amount0, amount1)
    }
    
    /// Mint liquidity - for factory integration
    /// Assumes tokens are already transferred to pool by factory
    pub fn mint(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        amount0_desired: i128,
        amount1_desired: i128,
    ) -> i128 {
        owner.require_auth();
        
        let mut state = read_pool_state(&env);
        
        let lower_aligned = snap_tick_to_spacing(lower_tick, state.tick_spacing);
        let upper_aligned = snap_tick_to_spacing(upper_tick, state.tick_spacing);
        
        if lower_aligned >= upper_aligned {
            panic!("{}", ErrorMsg::INVALID_TICK_RANGE);
        }
        
        let liquidity = get_liquidity_for_amounts(
            &env,
            amount0_desired,
            amount1_desired,
            get_sqrt_ratio_at_tick(lower_aligned),
            get_sqrt_ratio_at_tick(upper_aligned),
            state.sqrt_price_x64,
        );
        
        if liquidity < MIN_LIQUIDITY {
            panic!("{}", ErrorMsg::LIQUIDITY_TOO_LOW);
        }
        
        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            liquidity,
            get_sqrt_ratio_at_tick(lower_aligned),
            get_sqrt_ratio_at_tick(upper_aligned),
            state.sqrt_price_x64,
        );
        
        // Update ticks
        belugaswap_tick::update_tick(
            &env,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            lower_aligned,
            state.current_tick,
            liquidity,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
            false,
        );
        
        belugaswap_tick::update_tick(
            &env,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            upper_aligned,
            state.current_tick,
            liquidity,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
            true,
        );
        
        if state.current_tick >= lower_aligned && state.current_tick < upper_aligned {
            state.liquidity = state.liquidity.saturating_add(liquidity);
        }
        
        write_pool_state(&env, &state);
        
        let fee_growth_inside = get_fee_growth_inside_local(
            &env,
            lower_aligned,
            upper_aligned,
            state.current_tick,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
        );
        
        let mut pos = read_position(&env, &owner, lower_aligned, upper_aligned);
        modify_position(&mut pos, liquidity, fee_growth_inside.0, fee_growth_inside.1);
        write_position(&env, &owner, lower_aligned, upper_aligned, &pos);
        
        // NOTE: No token transfer here - factory already transferred tokens
        
        emit_add_liquidity(&env, liquidity, amount0, amount1);
        
        liquidity
    }
    
    pub fn remove_liquidity(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: i128,
    ) -> (i128, i128) {
        owner.require_auth();
        
        let mut pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();
        
        let lower = snap_tick_to_spacing(lower_tick, pool.tick_spacing);
        let upper = snap_tick_to_spacing(upper_tick, pool.tick_spacing);
        
        if liquidity_delta <= 0 {
            panic!("{}", ErrorMsg::INVALID_LIQUIDITY_AMOUNT);
        }
        
        let (inside_0, inside_1) = get_fee_growth_inside_local(
            &env,
            lower,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );
        
        let mut pos = read_position(&env, &owner, lower, upper);
        
        if liquidity_delta > pos.liquidity {
            panic!("{}", ErrorMsg::INSUFFICIENT_LIQUIDITY);
        }
        
        modify_position(&mut pos, -liquidity_delta, inside_0, inside_1);
        write_position(&env, &owner, lower, upper, &pos);
        
        belugaswap_tick::update_tick(
            &env,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            lower,
            pool.current_tick,
            -liquidity_delta,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
            false,
        );
        
        belugaswap_tick::update_tick(
            &env,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            upper,
            pool.current_tick,
            -liquidity_delta,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
            true,
        );
        
        if pool.current_tick >= lower && pool.current_tick < upper {
            pool.liquidity = pool.liquidity.saturating_sub(liquidity_delta);
        }
        write_pool_state(&env, &pool);
        
        let sqrt_lower = get_sqrt_ratio_at_tick(lower);
        let sqrt_upper = get_sqrt_ratio_at_tick(upper);
        
        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            liquidity_delta,
            sqrt_lower,
            sqrt_upper,
            pool.sqrt_price_x64,
        );
        
        if amount0 > 0 {
            token::Client::new(&env, &pool.token0).transfer(&pool_addr, &owner, &amount0);
        }
        if amount1 > 0 {
            token::Client::new(&env, &pool.token1).transfer(&pool_addr, &owner, &amount1);
        }
        
        emit_remove_liquidity(&env, liquidity_delta, amount0, amount1);
        
        (amount0, amount1)
    }
    
    // ========================================================
    // FEE COLLECTION
    // ========================================================
    
    pub fn collect(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> (u128, u128) {
        owner.require_auth();
        
        let pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();
        
        let lower = snap_tick_to_spacing(lower_tick, pool.tick_spacing);
        let upper = snap_tick_to_spacing(upper_tick, pool.tick_spacing);
        
        let mut pos = read_position(&env, &owner, lower, upper);
        
        let (inside_0, inside_1) = get_fee_growth_inside_local(
            &env,
            lower,
            upper,
            pool.current_tick,
            pool.fee_growth_global_0,
            pool.fee_growth_global_1,
        );
        
        update_position(&mut pos, inside_0, inside_1);
        
        let amount0 = pos.tokens_owed_0;
        let amount1 = pos.tokens_owed_1;
        
        let pool_balance_0 = token::Client::new(&env, &pool.token0).balance(&pool_addr) as u128;
        let pool_balance_1 = token::Client::new(&env, &pool.token1).balance(&pool_addr) as u128;
        
        let amount0_capped = amount0.min(pool_balance_0);
        let amount1_capped = amount1.min(pool_balance_1);
        
        pos.tokens_owed_0 = pos.tokens_owed_0.saturating_sub(amount0_capped);
        pos.tokens_owed_1 = pos.tokens_owed_1.saturating_sub(amount1_capped);
        
        write_position(&env, &owner, lower, upper, &pos);
        
        if amount0_capped > 0 {
            token::Client::new(&env, &pool.token0)
                .transfer(&pool_addr, &owner, &(amount0_capped as i128));
        }
        if amount1_capped > 0 {
            token::Client::new(&env, &pool.token1)
                .transfer(&pool_addr, &owner, &(amount1_capped as i128));
        }
        
        emit_collect(&env, amount0_capped, amount1_capped);
        
        (amount0_capped, amount1_capped)
    }
    
    pub fn claim_creator_fees(env: Env, claimer: Address) -> (u128, u128) {
        claimer.require_auth();
        
        let config = read_pool_config(&env);
        let mut pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();
        
        if claimer != config.creator {
            panic!("{}", ErrorMsg::UNAUTHORIZED);
        }
        
        let amount0 = pool.creator_fees_0;
        let amount1 = pool.creator_fees_1;
        
        let pool_balance_0 = token::Client::new(&env, &pool.token0).balance(&pool_addr) as u128;
        let pool_balance_1 = token::Client::new(&env, &pool.token1).balance(&pool_addr) as u128;
        
        let amount0_capped = amount0.min(pool_balance_0);
        let amount1_capped = amount1.min(pool_balance_1);
        
        pool.creator_fees_0 = pool.creator_fees_0.saturating_sub(amount0_capped);
        pool.creator_fees_1 = pool.creator_fees_1.saturating_sub(amount1_capped);
        
        write_pool_state(&env, &pool);
        
        if amount0_capped > 0 {
            token::Client::new(&env, &pool.token0)
                .transfer(&pool_addr, &claimer, &(amount0_capped as i128));
        }
        if amount1_capped > 0 {
            token::Client::new(&env, &pool.token1)
                .transfer(&pool_addr, &claimer, &(amount1_capped as i128));
        }
        
        emit_claim_creator_fees(&env, amount0_capped, amount1_capped);
        
        (amount0_capped, amount1_capped)
    }
    
    /// Claim creator fees - extended version for factory integration
    pub fn claim_creator_fees_ex(
        env: Env, 
        claimer: Address,
        _lower_tick: i32,
        _upper_tick: i32,
    ) -> (u128, u128) {
        // Just delegate to the original function
        Self::claim_creator_fees(env, claimer)
    }
}

// ========================================================
// HELPER FUNCTIONS
// ========================================================

fn get_fee_growth_inside_local(
    env: &Env,
    lower_tick: i32,
    upper_tick: i32,
    current_tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> (u128, u128) {
    belugaswap_tick::get_fee_growth_inside(
        env,
        |e, t| read_tick_info(e, t),
        lower_tick,
        upper_tick,
        current_tick,
        fee_growth_global_0,
        fee_growth_global_1,
    )
}