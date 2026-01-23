#![no_std]

use soroban_sdk::{contract, contractimpl, token, vec, Address, Env, IntoVal, Symbol};

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
    
    /// Initialize pool
    /// 
    /// # Arguments
    /// * `factory` - Factory contract address (MUST be a contract, not EOA)
    /// * `creator` - Pool creator address
    /// * `token_a` - First token
    /// * `token_b` - Second token
    /// * `fee_bps` - Trading fee in basis points
    /// * `creator_fee_bps` - Creator's share of fees
    /// * `sqrt_price_x64` - Initial sqrt price
    /// * `current_tick` - Initial tick
    /// * `tick_spacing` - Tick spacing for this fee tier
    pub fn initialize(
        env: Env,
        factory: Address,      
        creator: Address,
        token_a: Address,
        token_b: Address,
        fee_bps: u32,
        creator_fee_bps: u32,
        sqrt_price_x64: u128,
        current_tick: i32,
        tick_spacing: i32,
    ) {
        factory.require_auth();
        
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
            factory,             
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
    
    /// Check if pool is initialized
    pub fn is_initialized(env: Env) -> bool {
        is_initialized(&env)
    }
    
    /// Get current pool state (price, tick, liquidity)
    pub fn get_pool_state(env: Env) -> PoolState {
        read_pool_state(&env)
    }
    
    /// Get pool configuration (tokens, fees, creator)
    pub fn get_pool_config(env: Env) -> PoolConfig {
        read_pool_config(&env)
    }
    
    /// Get position info for an LP
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
    
    /// Get accumulated creator fees
    pub fn get_creator_fees(env: Env) -> CreatorFeesInfo {
        let pool = read_pool_state(&env);
        CreatorFeesInfo {
            fees_token0: pool.creator_fees_0,
            fees_token1: pool.creator_fees_1,
        }
    }
    
    /// Get swap direction based on input token
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
    
    /// Preview swap output without executing
    pub fn preview_swap(
        env: Env,
        token_in: Address,
        amount_in: i128,
        min_amount_out: i128,
        sqrt_price_limit_x64: u128,
    ) -> PreviewResult {
        let config = read_pool_config(&env);
        let pool = read_pool_state(&env);
        
        // Validate token_in
        let zero_for_one = if token_in == pool.token0 {
            true
        } else if token_in == pool.token1 {
            false
        } else {
            return PreviewResult::invalid(Symbol::new(&env, "BAD_TOKEN"));
        };
        
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
    // SWAP FUNCTION
    // ========================================================
    
    /// Execute a swap
    pub fn swap(
        env: Env,
        sender: Address,
        token_in: Address,
        amount_in: i128,
        amount_out_min: i128,
        sqrt_price_limit_x64: u128,
    ) -> SwapResult {
        sender.require_auth();
        
        // Validate amount_in early
        if amount_in <= 0 {
            panic!("amount_in must be positive");
        }
        
        let pool_state = read_pool_state(&env);
        let config = read_pool_config(&env);
        
        let zero_for_one = if token_in == pool_state.token0 {
            true
        } else if token_in == pool_state.token1 {
            false
        } else {
            panic!("{}", ErrorMsg::INVALID_TOKEN);
        };
        
        // Safe creator fee check - won't panic if factory call fails
        let creator_fee_bps = Self::get_active_creator_fee_bps_safe(&env, &config);
        
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
            creator_fee_bps,
        );
        
        if amount_out < amount_out_min {
            panic!("{}", ErrorMsg::SLIPPAGE_EXCEEDED);
        }
        
        // Update pool state
        let mut updated_pool = pool_state.clone();
        updated_pool.sqrt_price_x64 = swap_state.sqrt_price_x64;
        updated_pool.current_tick = swap_state.current_tick;
        updated_pool.liquidity = swap_state.liquidity;
        updated_pool.fee_growth_global_0 = swap_state.fee_growth_global_0;
        updated_pool.fee_growth_global_1 = swap_state.fee_growth_global_1;
        write_pool_state(&env, &updated_pool);
        
        // Transfer tokens
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
    
    // ========================================================
    // LIQUIDITY FUNCTIONS
    // ========================================================
    
    /// Add liquidity to a position
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
        
        // Delegate to internal function
        let (liquidity, amount0, amount1) = Self::internal_add_liquidity(
            &env,
            &owner,
            lower_tick,
            upper_tick,
            amount0_desired,
            amount1_desired,
            amount0_min,
            amount1_min,
        );
        
        // Transfer tokens from owner to pool
        let state = read_pool_state(&env);
        if amount0 > 0 {
            token::Client::new(&env, &state.token0).transfer(&owner, &env.current_contract_address(), &amount0);
        }
        if amount1 > 0 {
            token::Client::new(&env, &state.token1).transfer(&owner, &env.current_contract_address(), &amount1);
        }
        
        emit_add_liquidity(&env, liquidity, amount0, amount1);
        
        (liquidity, amount0, amount1)
    }

    // ========================================================
    // MINT FUNCTION (For Factory Integration)
    // ========================================================
    
    /// Mint liquidity - used by Factory during pool creation
    /// 
    /// # Arguments
    /// * `owner` - Position owner
    /// * `lower_tick` - Lower tick boundary
    /// * `upper_tick` - Upper tick boundary
    /// * `amount0_desired` - Amount of token0 expected in pool
    /// * `amount1_desired` - Amount of token1 expected in pool
    /// 
    /// # Returns
    /// Liquidity minted
    pub fn mint(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        amount0_desired: i128,
        amount1_desired: i128,
    ) -> i128 {
        let config = read_pool_config(&env);
        
        // Only factory can call mint
        config.factory.require_auth();
        
        // Verify tokens are actually in pool
        let state = read_pool_state(&env);
        let pool_addr = env.current_contract_address();
        
        let balance0 = token::Client::new(&env, &state.token0).balance(&pool_addr);
        let balance1 = token::Client::new(&env, &state.token1).balance(&pool_addr);
        
        // Must have at least the desired amounts
        if balance0 < amount0_desired || balance1 < amount1_desired {
            panic!("insufficient token balance in pool");
        }
        
        // Delegate to internal function (no slippage check for factory mint)
        let (liquidity, amount0, amount1) = Self::internal_add_liquidity(
            &env,
            &owner,
            lower_tick,
            upper_tick,
            amount0_desired,
            amount1_desired,
            0, 
            0, 
        );
        
        // No transfer - factory already transferred tokens
        
        emit_add_liquidity(&env, liquidity, amount0, amount1);
        
        liquidity
    }
    
    /// Remove liquidity from a position

    pub fn remove_liquidity(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: i128,
    ) -> (i128, i128) {
        owner.require_auth();
        
        let config = read_pool_config(&env);
        let mut pool = read_pool_state(&env);
        let pool_addr = env.current_contract_address();
        
        let lower = snap_tick_to_spacing(lower_tick, pool.tick_spacing);
        let upper = snap_tick_to_spacing(upper_tick, pool.tick_spacing);
        
        if liquidity_delta <= 0 {
            panic!("{}", ErrorMsg::INVALID_LIQUIDITY_AMOUNT);
        }
        
        // Check if this position is locked (creator lock enforcement)
        // Query factory to check if owner has active lock on this position
        if Self::is_position_locked(&env, &config, &owner, lower, upper) {
            panic!("position is locked - use factory.unlock_creator_liquidity first");
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
        
        // Update ticks
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
        
        // Update pool liquidity if in range
        if pool.current_tick >= lower && pool.current_tick < upper {
            pool.liquidity = pool.liquidity.saturating_sub(liquidity_delta);
        }
        write_pool_state(&env, &pool);
        
        // Calculate amounts to return
        let sqrt_lower = get_sqrt_ratio_at_tick(lower);
        let sqrt_upper = get_sqrt_ratio_at_tick(upper);
        
        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            liquidity_delta,
            sqrt_lower,
            sqrt_upper,
            pool.sqrt_price_x64,
        );
        
        // Transfer tokens to owner
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
    
    /// Collect accumulated LP fees
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
        
        // Cap by pool balance
        let pool_balance_0 = token::Client::new(&env, &pool.token0).balance(&pool_addr) as u128;
        let pool_balance_1 = token::Client::new(&env, &pool.token1).balance(&pool_addr) as u128;
        
        let amount0_capped = amount0.min(pool_balance_0);
        let amount1_capped = amount1.min(pool_balance_1);
        
        pos.tokens_owed_0 = pos.tokens_owed_0.saturating_sub(amount0_capped);
        pos.tokens_owed_1 = pos.tokens_owed_1.saturating_sub(amount1_capped);
        
        write_position(&env, &owner, lower, upper, &pos);
        
        // Safe cast with bounds check
        let transfer0 = safe_u128_to_i128(amount0_capped);
        let transfer1 = safe_u128_to_i128(amount1_capped);
        
        // Transfer fees
        if transfer0 > 0 {
            token::Client::new(&env, &pool.token0)
                .transfer(&pool_addr, &owner, &transfer0);
        }
        if transfer1 > 0 {
            token::Client::new(&env, &pool.token1)
                .transfer(&pool_addr, &owner, &transfer1);
        }
        
        emit_collect(&env, amount0_capped, amount1_capped);
        
        (amount0_capped, amount1_capped)
    }
    
    /// Claim creator fees
    /// 
    /// Note: Creator can claim accumulated fees even after revocation.
    /// However, no NEW fees will accumulate after revocation.
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
        
        // Cap by pool balance
        let pool_balance_0 = token::Client::new(&env, &pool.token0).balance(&pool_addr) as u128;
        let pool_balance_1 = token::Client::new(&env, &pool.token1).balance(&pool_addr) as u128;
        
        let amount0_capped = amount0.min(pool_balance_0);
        let amount1_capped = amount1.min(pool_balance_1);
        
        pool.creator_fees_0 = pool.creator_fees_0.saturating_sub(amount0_capped);
        pool.creator_fees_1 = pool.creator_fees_1.saturating_sub(amount1_capped);
        
        write_pool_state(&env, &pool);
        
        // Safe cast with bounds check
        let transfer0 = safe_u128_to_i128(amount0_capped);
        let transfer1 = safe_u128_to_i128(amount1_capped);
        
        // Transfer fees
        if transfer0 > 0 {
            token::Client::new(&env, &pool.token0)
                .transfer(&pool_addr, &claimer, &transfer0);
        }
        if transfer1 > 0 {
            token::Client::new(&env, &pool.token1)
                .transfer(&pool_addr, &claimer, &transfer1);
        }
        
        emit_claim_creator_fees(&env, amount0_capped, amount1_capped);
        
        (amount0_capped, amount1_capped)
    }
    
    // ========================================================
    // INTERNAL HELPERS
    // ========================================================
    
    /// Shared internal logic for add_liquidity and mint
    fn internal_add_liquidity(
        env: &Env,
        owner: &Address,
        lower_tick: i32,
        upper_tick: i32,
        amount0_desired: i128,
        amount1_desired: i128,
        amount0_min: i128,
        amount1_min: i128,
    ) -> (i128, i128, i128) {
        let mut state = read_pool_state(env);
        
        let lower_aligned = snap_tick_to_spacing(lower_tick, state.tick_spacing);
        let upper_aligned = snap_tick_to_spacing(upper_tick, state.tick_spacing);
        
        if lower_aligned >= upper_aligned {
            panic!("{}", ErrorMsg::INVALID_TICK_RANGE);
        }
        
        let liquidity = get_liquidity_for_amounts(
            env,
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
            env,
            liquidity,
            get_sqrt_ratio_at_tick(lower_aligned),
            get_sqrt_ratio_at_tick(upper_aligned),
            state.sqrt_price_x64,
        );
        
        // Slippage check (skip if min = 0, e.g., factory mint)
        if amount0_min > 0 || amount1_min > 0 {
            if amount0 < amount0_min || amount1 < amount1_min {
                panic!("{}", ErrorMsg::SLIPPAGE_EXCEEDED);
            }
        }
        
        // Update ticks
        belugaswap_tick::update_tick(
            env,
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
            env,
            |e, t| read_tick_info(e, t),
            |e, t, info| write_tick_info(e, t, info),
            upper_aligned,
            state.current_tick,
            liquidity,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
            true,
        );
        
        // Update pool liquidity if in range
        if state.current_tick >= lower_aligned && state.current_tick < upper_aligned {
            state.liquidity = state.liquidity.saturating_add(liquidity);
        }
        write_pool_state(env, &state);
        
        // Update position
        let fee_growth_inside = get_fee_growth_inside_local(
            env,
            lower_aligned,
            upper_aligned,
            state.current_tick,
            state.fee_growth_global_0,
            state.fee_growth_global_1,
        );
        
        let mut pos = read_position(env, owner, lower_aligned, upper_aligned);
        modify_position(&mut pos, liquidity, fee_growth_inside.0, fee_growth_inside.1);
        write_position(env, owner, lower_aligned, upper_aligned, &pos);
        
        (liquidity, amount0, amount1)
    }
    
    /// Check if a position is locked by querying factory
    /// 
    /// Returns true if:
    /// - Owner is the pool creator AND
    /// - Position matches locked position (same ticks) AND
    /// - Lock is still active (not unlocked and not expired)
    /// 
    /// Returns false if:
    /// - Factory call fails (graceful degradation)
    /// - Owner is not creator
    /// - Position is not the locked one
    /// - Lock has been unlocked
    fn is_position_locked(
        env: &Env,
        config: &PoolConfig,
        owner: &Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> bool {
        // Only check lock for creator - other LPs are not affected
        if owner != &config.creator {
            return false;
        }
        
        let pool_addr = env.current_contract_address();
        
        // Query factory: is_liquidity_locked(pool, creator, lower_tick, upper_tick) -> bool
        let result = env.try_invoke_contract::<bool, soroban_sdk::Error>(
            &config.factory,
            &Symbol::new(env, "is_liquidity_locked"),
            vec![
                env,
                pool_addr.into_val(env),
                owner.clone().into_val(env),
                lower_tick.into_val(env),
                upper_tick.into_val(env),
            ],
        );
        
        match result {
            Ok(Ok(is_locked)) => is_locked,
            _ => false, // Factory call failed - allow withdrawal (graceful degradation)
        }
    }
    
    /// [FIX] Safe version - returns 0 if factory call fails (prevents DoS)
    /// 
    /// This prevents swap DoS if factory address is invalid or
    /// doesn't have the is_creator_fee_active function
    fn get_active_creator_fee_bps_safe(env: &Env, config: &PoolConfig) -> i128 {
        let pool_addr = env.current_contract_address();
        
        // Try to query factory - if it fails, just return 0 (no creator fee)
        // This prevents DoS if factory is misconfigured
        let result = env.try_invoke_contract::<bool, soroban_sdk::Error>(
            &config.factory,
            &Symbol::new(env, "is_creator_fee_active"),
            vec![
                env,
                pool_addr.into_val(env),
                config.creator.clone().into_val(env),
            ],
        );
        
        match result {
            Ok(Ok(true)) => config.creator_fee_bps as i128,
            _ => 0, // Factory call failed or returned false - no creator fee
        }
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

/// Safe conversion from u128 to i128
/// Returns i128::MAX if value exceeds i128 range
#[inline]
fn safe_u128_to_i128(value: u128) -> i128 {
    if value > i128::MAX as u128 {
        i128::MAX
    } else {
        value as i128
    }
}