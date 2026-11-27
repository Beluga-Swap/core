#![no_std]
// Entrypoint arguments are the contract's public interface; grouping them into
// structs would only move the count into client code. The contractimpl macro
// generates code outside the impl block, so the allow has to be crate-wide.
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, Address, Env, Symbol,
};

// External packages
use belugaswap_math::{
    get_amounts_for_liquidity, get_amounts_for_liquidity_rounding_up, get_liquidity_for_amounts,
    get_tick_at_sqrt_ratio,snap_tick_to_spacing, 
    MIN_LIQUIDITY, get_sqrt_ratio_at_tick, 
    constants::{MAX_FEE_BPS, MAX_TICK, MIN_TICK}
};
use belugaswap_position::{PositionInfo, modify_position, update_position, calculate_pending_fees};
use belugaswap_swap::{
    engine_swap, validate_and_preview_swap, PoolSwapResult, PreviewResult, SwapOutcome, SwapParams,
    SwapState,
};

// Local modules
mod error;
mod events;
mod storage;
pub mod types;

pub use error::PoolError;
use events::*;
use storage::*;
use types::{PoolConfig, PoolState};

#[contract]
pub struct BelugaPool;

#[contractimpl]
impl BelugaPool {
    // ========================================================
    // INITIALIZATION
    // ========================================================
    
/// Deploy-time setup. The factory deploys and initializes a pool in one
    /// step, so nobody can initialize it first.
    pub fn __constructor(
        env: Env,
        factory: Address,
        router: Address,
        creator: Address,
        token_a: Address,
        token_b: Address,
        fee_bps: u32,
        sqrt_price_x64: u128,
        tick_spacing: i32,
    ) {
        if fee_bps == 0 || fee_bps > MAX_FEE_BPS {
            panic_with_error!(&env, PoolError::InvalidFee);
        }
        if tick_spacing <= 0 {
            panic_with_error!(&env, PoolError::InvalidTickSpacing);
        }
        if !(get_sqrt_ratio_at_tick(MIN_TICK)..=get_sqrt_ratio_at_tick(MAX_TICK)).contains(&sqrt_price_x64) {
            panic_with_error!(&env, PoolError::InvalidTick);
        }
        // Derived here so tick and price can never disagree.
        let current_tick = get_tick_at_sqrt_ratio(sqrt_price_x64);

        let (token0, token1) = if token_a < token_b {
            (token_a.clone(), token_b.clone())
        } else {
            (token_b.clone(), token_a.clone())
        };

        write_pool_config(&env, &PoolConfig {
            factory,
            router,
            creator,
            token_a,
            token_b,
            fee_bps,
        });
        init_pool_state(&env, sqrt_price_x64, current_tick, tick_spacing, token0, token1);
        extend_instance(&env);

        emit_initialized(&env, fee_bps, tick_spacing);
        emit_pool_init(&env, sqrt_price_x64, current_tick, tick_spacing);
    }
    
    // ========================================================
    // VIEW FUNCTIONS 
    // ========================================================
    
    
    /// Get current pool state (price, tick, liquidity)
    pub fn get_pool_state(env: Env) -> PoolState {
        read_pool_state(&env)
    }
    
    /// Get pool configuration (tokens, fees, creator, router)
    pub fn get_pool_config(env: Env) -> PoolConfig {
        read_pool_config(&env)
    }
    
    /// Get router address
    pub fn get_router(env: Env) -> Address {
        read_pool_config(&env).router
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
        
        let fee_growth_inside = get_fee_growth_inside_local(&env, &state, lower_tick, upper_tick);
        
        let (pending0, pending1) = calculate_pending_fees(&env, &pos, fee_growth_inside.0, fee_growth_inside.1);
        
        PositionInfo {
            liquidity: pos.liquidity,
            amount0,
            amount1,
            fees_owed_0: pos.tokens_owed_0.saturating_add(pending0),
            fees_owed_1: pos.tokens_owed_1.saturating_add(pending1),
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
            panic_with_error!(&env, PoolError::InvalidToken);
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
        
        let params = SwapParams {
            amount: amount_in,
            zero_for_one,
            sqrt_price_limit_x64,
            fee_bps: config.fee_bps as i128,
        };
        let ticks = PoolTicks { tick_spacing: pool.tick_spacing };

        match validate_and_preview_swap(&env, &swap_state, &ticks, &params, min_amount_out) {
            Ok((amount_in_used, amount_out, fee_paid, price_impact_bps, _final_price)) => {
                PreviewResult::valid(amount_in_used, amount_out, fee_paid, price_impact_bps)
            }
            Err(error_symbol) => PreviewResult::invalid(error_symbol),
        }
    }
    
    // ========================================================
    // SWAP FUNCTION
    // ========================================================
    
    /// Execute a swap
    /// 
    /// Can be called by:
    /// - Direct user (sender.require_auth)
    /// - Router contract (for multi-hop swaps)
    pub fn swap(
        env: Env,
        sender: Address,
        token_in: Address,
        amount_in: i128,
        amount_out_min: i128,
        sqrt_price_limit_x64: u128,
    ) -> PoolSwapResult {
        extend_instance(&env);
        sender.require_auth();
        
        // Validate amount_in early
        if amount_in <= 0 {
            panic_with_error!(&env, PoolError::SwapAmountTooSmall);
        }
        
        let pool_state = read_pool_state(&env);
        let config = read_pool_config(&env);
        
        let zero_for_one = if token_in == pool_state.token0 {
            true
        } else if token_in == pool_state.token1 {
            false
        } else {
            panic_with_error!(&env, PoolError::InvalidToken);
        };
        
        
        let mut swap_state = SwapState {
            sqrt_price_x64: pool_state.sqrt_price_x64,
            current_tick: pool_state.current_tick,
            liquidity: pool_state.liquidity,
            tick_spacing: pool_state.tick_spacing,
            fee_growth_global_0: pool_state.fee_growth_global_0,
            fee_growth_global_1: pool_state.fee_growth_global_1,
        };
        
        let params = SwapParams {
            amount: amount_in,
            zero_for_one,
            sqrt_price_limit_x64,
            fee_bps: config.fee_bps as i128,
        };
        let ticks = PoolTicks { tick_spacing: pool_state.tick_spacing };
        let SwapOutcome { amount_in: amount_in_used, amount_out } =
            engine_swap(&env, &mut swap_state, &ticks, &params)
                .unwrap_or_else(|err| panic_with_error!(&env, PoolError::from(err)));
        emit_sync_tick(&env, swap_state.current_tick, swap_state.sqrt_price_x64);
        
        if amount_out < amount_out_min {
            panic_with_error!(&env, PoolError::SlippageExceeded);
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
        
        PoolSwapResult {
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
        extend_instance(&env);
        owner.require_auth();
        
        // Delegate to internal function
        let (liquidity, amount0, amount1) = Self::internal_add_liquidity(
            &env,
            &owner,
            lower_tick,
            upper_tick,
            (amount0_desired, amount1_desired),
            (amount0_min, amount1_min),
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
    
    /// Mint liquidity (called by factory during pool creation)
    /// Tokens must already be transferred to pool
    pub fn mint(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        amount0_desired: i128,
        amount1_desired: i128,
    ) -> i128 {
        extend_instance(&env);
        // SECURITY: mint credits a position WITHOUT pulling tokens (the factory
        // transfers them beforehand during create_pool). It must therefore only
        // be callable by the factory, otherwise anyone could mint a free position
        // and drain the pool via remove_liquidity. Require the factory to
        // authorize this call (mirrors `initialize`).
        read_pool_config(&env).factory.require_auth();
        // Tokens already transferred by factory

        let (liquidity, amount0, amount1) = Self::internal_add_liquidity(
            &env,
            &owner,
            lower_tick,
            upper_tick,
            (amount0_desired, amount1_desired),
            (0, 0), // no slippage check for the factory mint
        );
        
        // Return whatever the position did not need; nothing else could ever
        // recover it from the pool.
        let state = read_pool_state(&env);
        let pool = env.current_contract_address();
        if amount0_desired > amount0 {
            token::Client::new(&env, &state.token0).transfer(&pool, &owner, &(amount0_desired - amount0));
        }
        if amount1_desired > amount1 {
            token::Client::new(&env, &state.token1).transfer(&pool, &owner, &(amount1_desired - amount1));
        }

        emit_add_liquidity(&env, liquidity, amount0, amount1);
        
        liquidity
    }
    
    /// Remove liquidity from a position
    pub fn remove_liquidity(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        liquidity: i128,
        amount0_min: i128,
        amount1_min: i128,
    ) -> (i128, i128) {
        extend_instance(&env);
        owner.require_auth();
        
        if liquidity <= 0 {
            panic_with_error!(&env, PoolError::InvalidLiquidityAmount);
        }
        
        let mut state = read_pool_state(&env);
        
        let lower_aligned = snap_tick_to_spacing(lower_tick, state.tick_spacing);
        let upper_aligned = snap_tick_to_spacing(upper_tick, state.tick_spacing);
        
        let mut pos = read_position(&env, &owner, lower_aligned, upper_aligned);
        
        if pos.liquidity < liquidity {
            panic_with_error!(&env, PoolError::InsufficientLiquidity);
        }
        
        // Calculate amounts to withdraw
        let (amount0, amount1) = get_amounts_for_liquidity(
            &env,
            liquidity,
            get_sqrt_ratio_at_tick(lower_aligned),
            get_sqrt_ratio_at_tick(upper_aligned),
            state.sqrt_price_x64,
        );
        
        // Slippage check
        if amount0 < amount0_min || amount1 < amount1_min {
            panic_with_error!(&env, PoolError::SlippageExceeded);
        }
        
        // Update fee growth
        let fee_growth_inside = get_fee_growth_inside_local(&env, &state, lower_aligned, upper_aligned);
        
        update_position(&env, &mut pos, fee_growth_inside.0, fee_growth_inside.1);
        
        // Subtract liquidity from position
        pos.liquidity = pos.liquidity.saturating_sub(liquidity);
        
        // Update ticks
        belugaswap_tick::update_range(
            &env,
            &PoolTicks { tick_spacing: state.tick_spacing },
            lower_aligned,
            upper_aligned,
            state.current_tick,
            -liquidity,
            (state.fee_growth_global_0, state.fee_growth_global_1),
        );
        
        // Update pool liquidity if in range
        if state.current_tick >= lower_aligned && state.current_tick < upper_aligned {
            state.liquidity = state.liquidity.saturating_sub(liquidity);
        }
        
        write_pool_state(&env, &state);
        write_position(&env, &owner, lower_aligned, upper_aligned, &pos);
        
        // Transfer tokens to owner
        if amount0 > 0 {
            token::Client::new(&env, &state.token0).transfer(&env.current_contract_address(), &owner, &amount0);
        }
        if amount1 > 0 {
            token::Client::new(&env, &state.token1).transfer(&env.current_contract_address(), &owner, &amount1);
        }
        
        emit_remove_liquidity(&env, liquidity, amount0, amount1);
        
        (amount0, amount1)
    }
    
    /// Collect accumulated fees from a position
    pub fn collect_fees(
        env: Env,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> (u128, u128) {
        extend_instance(&env);
        owner.require_auth();
        
        let state = read_pool_state(&env);
        
        let lower_aligned = snap_tick_to_spacing(lower_tick, state.tick_spacing);
        let upper_aligned = snap_tick_to_spacing(upper_tick, state.tick_spacing);
        
        let mut pos = read_position(&env, &owner, lower_aligned, upper_aligned);
        
        // Update fee growth
        let fee_growth_inside = get_fee_growth_inside_local(&env, &state, lower_aligned, upper_aligned);
        
        let (pending0, pending1) = calculate_pending_fees(&env, &pos, fee_growth_inside.0, fee_growth_inside.1);
        
        let fees0 = pos.tokens_owed_0.saturating_add(pending0);
        let fees1 = pos.tokens_owed_1.saturating_add(pending1);
        
        // Reset owed tokens
        pos.tokens_owed_0 = 0;
        pos.tokens_owed_1 = 0;
        pos.fee_growth_inside_last_0 = fee_growth_inside.0;
        pos.fee_growth_inside_last_1 = fee_growth_inside.1;
        
        write_position(&env, &owner, lower_aligned, upper_aligned, &pos);
        
        // Transfer fees
        if fees0 > 0 {
            token::Client::new(&env, &state.token0).transfer(
                &env.current_contract_address(),
                &owner,
                &safe_u128_to_i128(fees0),
            );
        }
        if fees1 > 0 {
            token::Client::new(&env, &state.token1).transfer(
                &env.current_contract_address(),
                &owner,
                &safe_u128_to_i128(fees1),
            );
        }
        
        emit_collect(&env, fees0, fees1);
        
        (fees0, fees1)
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
        (amount0_desired, amount1_desired): (i128, i128),
        (amount0_min, amount1_min): (i128, i128),
    ) -> (i128, i128, i128) {
        let mut state = read_pool_state(env);
        
        let lower_aligned = snap_tick_to_spacing(lower_tick, state.tick_spacing);
        let upper_aligned = snap_tick_to_spacing(upper_tick, state.tick_spacing);
        
        if lower_aligned < MIN_TICK || upper_aligned > MAX_TICK {
            panic_with_error!(env, PoolError::InvalidTick);
        }
        if lower_aligned >= upper_aligned {
            panic_with_error!(env, PoolError::InvalidTickRange);
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
            panic_with_error!(env, PoolError::LiquidityTooLow);
        }
        
        let (amount0, amount1) = get_amounts_for_liquidity_rounding_up(
            env,
            liquidity,
            get_sqrt_ratio_at_tick(lower_aligned),
            get_sqrt_ratio_at_tick(upper_aligned),
            state.sqrt_price_x64,
        );
        
        // Slippage check (skip if min = 0, e.g., factory mint)
        if (amount0_min > 0 || amount1_min > 0)
            && (amount0 < amount0_min || amount1 < amount1_min) {
                panic_with_error!(env, PoolError::SlippageExceeded);
            }
        
        // Update ticks
        belugaswap_tick::update_range(
            env,
            &PoolTicks { tick_spacing: state.tick_spacing },
            lower_aligned,
            upper_aligned,
            state.current_tick,
            liquidity,
            (state.fee_growth_global_0, state.fee_growth_global_1),
        );
        
        // Update pool liquidity if in range
        if state.current_tick >= lower_aligned && state.current_tick < upper_aligned {
            state.liquidity = state
                .liquidity
                .checked_add(liquidity)
                .unwrap_or_else(|| panic_with_error!(env, PoolError::Overflow));
        }
        write_pool_state(env, &state);
        
        // Update position
        let fee_growth_inside = get_fee_growth_inside_local(env, &state, lower_aligned, upper_aligned);
        
        let mut pos = read_position(env, owner, lower_aligned, upper_aligned);
        modify_position(env, &mut pos, liquidity, fee_growth_inside.0, fee_growth_inside.1);
        write_position(env, owner, lower_aligned, upper_aligned, &pos);
        
        (liquidity, amount0, amount1)
    }
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

fn get_fee_growth_inside_local(
    env: &Env,
    state: &PoolState,
    lower_tick: i32,
    upper_tick: i32,
) -> (u128, u128) {
    belugaswap_tick::get_fee_growth_inside(
        env,
        &PoolTicks { tick_spacing: state.tick_spacing },
        lower_tick,
        upper_tick,
        state.current_tick,
        (state.fee_growth_global_0, state.fee_growth_global_1),
    )
}

/// Safe conversion from u128 to i128
#[inline]
fn safe_u128_to_i128(value: u128) -> i128 {
    if value > i128::MAX as u128 {
        i128::MAX
    } else {
        value as i128
    }
}