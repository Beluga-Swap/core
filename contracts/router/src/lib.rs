#![no_std]

//! # BelugaSwap Router
//! 
//! Smart routing with multi-fee tier support.
//! 
//! ## Features:
//! 1. Single swap with auto best-fee selection
//! 2. Split routing across multiple fee tiers
//! 3. Multi-hop swaps (A -> B -> C)
//! 4. Quote aggregation for comparison
//!
//! ## Deployment Flow:
//! 1. Deploy Factory → factory.initialize()
//! 2. Deploy Router → router.initialize(factory, admin)
//! 3. Set router in Factory → factory.set_router(router)
//! 4. Ready to swap!

use soroban_sdk::{
    contract, contractimpl, vec, Address, Env, IntoVal, Symbol, Vec, token,
};

mod error;
mod events;
mod storage;
mod types;

pub use error::RouterError;
use events::*;
use storage::*;
pub use types::*;

// ============================================================
// CONSTANTS
// ============================================================

/// Maximum hops in a path
const MAX_HOPS: u32 = 4;

/// Default fee tiers to check (0.05%, 0.30%, 1.00%)
const DEFAULT_FEE_TIERS: [u32; 3] = [5, 30, 100];

/// Minimum output threshold (dust protection)
const MIN_OUTPUT: i128 = 1;

/// Approval expiry buffer (minimal - just for the current tx)
const APPROVAL_LEDGER_BUFFER: u32 = 100;

// ============================================================
// CONTRACT
// ============================================================

#[contract]
pub struct BelugaRouter;

#[contractimpl]
impl BelugaRouter {
    // ========================================================
    // INITIALIZATION
    // ========================================================
    
    /// Initialize router with factory address
    /// 
    /// After initialization, call factory.set_router(router_address)
    pub fn initialize(
        env: Env,
        factory: Address,
        admin: Address,
    ) -> Result<(), RouterError> {
        admin.require_auth();
        
        if is_initialized(&env) {
            return Err(RouterError::AlreadyInitialized);
        }
        
        let config = RouterConfig {
            factory,
            admin: admin.clone(),
        };
        write_config(&env, &config);
        set_initialized(&env);
        
        emit_initialized(&env, &config.factory, &admin);
        
        Ok(())
    }
    
    // ========================================================
    // SWAP FUNCTIONS 
    // ========================================================
    
    /// Swap exact input with automatic best fee selection
    /// 
    /// Tries all specified fee tiers and uses the one with best output
    pub fn swap_exact_input(
        env: Env,
        sender: Address,
        params: ExactInputParams,
    ) -> Result<SwapResult, RouterError> {
        sender.require_auth();
        
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        
        // Check deadline
        if env.ledger().sequence() > params.deadline {
            return Err(RouterError::DeadlineExpired);
        }
        
        if params.amount_in <= 0 {
            return Err(RouterError::InvalidAmount);
        }
        
        let config = read_config(&env);
        
        // Get best quote
        let fee_tiers = if params.fee_tiers.is_empty() {
            Self::default_fee_tiers(&env)
        } else {
            params.fee_tiers.clone()
        };
        
        let best_quote = Self::find_best_pool(
            &env,
            &config.factory,
            &params.token_in,
            &params.token_out,
            params.amount_in,
            &fee_tiers,
        )?;
        
        // Slippage check
        if best_quote.amount_out < params.amount_out_min {
            return Err(RouterError::SlippageExceeded);
        }
        
        // Execute swap - pull from user
        let result = Self::execute_single_swap(
            &env,
            &sender,              
            &params.recipient,   
            &params.token_in,
            &params.token_out,
            params.amount_in,
            params.amount_out_min,
            &best_quote.pool,
            best_quote.fee_bps,
        )?;
        
        emit_swap(
            &env,
            &sender,
            &params.token_in,
            &params.token_out,
            result.amount_in,
            result.amount_out,
            &result.pools_used,
        );
        
        Ok(result)
    }
    
    /// Split swap across multiple fee tiers for better execution on large orders
    /// 
    /// Splits the input across pools to minimize price impact
    pub fn swap_split(
        env: Env,
        sender: Address,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out_min: i128,
        splits: Vec<SplitQuote>,
        recipient: Address,
        deadline: u32,
    ) -> Result<SwapResult, RouterError> {
        sender.require_auth();
        
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        
        if env.ledger().sequence() > deadline {
            return Err(RouterError::DeadlineExpired);
        }
        
        if splits.is_empty() {
            return Err(RouterError::EmptySplits);
        }
        
        // Validate total split amounts
        let mut total_split_in: i128 = 0;
        let mut valid_splits: u32 = 0;
        
        for i in 0..splits.len() {
            let split = splits.get(i).unwrap();
            if split.amount_in > 0 {
                total_split_in = total_split_in.saturating_add(split.amount_in);
                valid_splits += 1;
            }
        }
        
        if total_split_in != amount_in {
            return Err(RouterError::SplitAmountMismatch);
        }
        
        if valid_splits == 0 {
            return Err(RouterError::EmptySplits);
        }
        
        // Execute each split - all pull from user (same token)
        let mut total_out: i128 = 0;
        let mut pools_used = Vec::new(&env);
        let mut fee_tiers_used = Vec::new(&env);
        
        for i in 0..splits.len() {
            let split = splits.get(i).unwrap();
            
            // Skip zero-amount splits
            if split.amount_in <= 0 {
                continue;
            }
            
            let result = Self::execute_single_swap(
                &env,
                &sender,
                &recipient,
                &token_in,
                &token_out,
                split.amount_in,
                MIN_OUTPUT, 
                &split.pool,
                split.fee_bps,
            )?;
            
            total_out = total_out.saturating_add(result.amount_out);
            pools_used.push_back(split.pool.clone());
            fee_tiers_used.push_back(split.fee_bps);
        }
        
        // Final slippage check on total output
        if total_out < amount_out_min {
            return Err(RouterError::SlippageExceeded);
        }
        
        emit_split_swap(
            &env,
            &sender,
            &token_in,
            &token_out,
            amount_in,
            total_out,
            valid_splits,
        );
        
        Ok(SwapResult {
            amount_in,
            amount_out: total_out,
            pools_used,
            fee_tiers_used,
        })
    }
    
    /// Multi-hop swap (A -> B -> C -> ...)
    pub fn swap_multihop(
        env: Env,
        sender: Address,
        params: MultihopExactInputParams,
    ) -> Result<SwapResult, RouterError> {
        sender.require_auth();
        
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        
        if env.ledger().sequence() > params.deadline {
            return Err(RouterError::DeadlineExpired);
        }
        
        if params.path.is_empty() {
            return Err(RouterError::InvalidPath);
        }
        
        if params.path.len() > MAX_HOPS {
            return Err(RouterError::PathTooLong);
        }
        
        if params.amount_in <= 0 {
            return Err(RouterError::InvalidAmount);
        }
        
        let config = read_config(&env);
        let router_addr = env.current_contract_address();
        
        let mut current_token = params.token_in.clone();
        let mut current_amount = params.amount_in;
        let mut pools_used = Vec::new(&env);
        let mut fee_tiers_used = Vec::new(&env);
        
        // Process each hop
        for i in 0..params.path.len() {
            let hop = params.path.get(i).unwrap();
            let next_token = hop.token.clone();
            let fee_bps = hop.fee_bps;
            
            // Get pool for this hop
            let pool = Self::get_pool_address(
                &env,
                &config.factory,
                &current_token,
                &next_token,
                fee_bps,
            ).ok_or(RouterError::PoolNotFound)?;
            
            // Determine recipient for this hop
            let is_final_hop = i == params.path.len() - 1;
            let hop_recipient = if is_final_hop {
                params.recipient.clone()
            } else {
                router_addr.clone()
            };
            
            // Min out for intermediate hops is 1 (dust protection)
            let min_out = if is_final_hop {
                params.amount_out_min
            } else {
                MIN_OUTPUT
            };
            
            // First hop: pull from user, subsequent: use router balance
            let is_first_hop = i == 0;
            
            let result = if is_first_hop {
                Self::execute_single_swap(
                    &env,
                    &sender,          
                    &hop_recipient,
                    &current_token,
                    &next_token,
                    current_amount,
                    min_out,
                    &pool,
                    fee_bps,
                )?
            } else {
                Self::execute_swap_from_router(
                    &env,
                    &hop_recipient,
                    &current_token,
                    &next_token,
                    current_amount,
                    min_out,
                    &pool,
                    fee_bps,
                )?
            };
            
            pools_used.push_back(pool);
            fee_tiers_used.push_back(fee_bps);
            
            current_token = next_token;
            current_amount = result.amount_out;
        }
        
        emit_multihop_swap(
            &env,
            &sender,
            &params.token_in,
            &current_token,
            params.amount_in,
            current_amount,
            params.path.len(),
        );
        
        Ok(SwapResult {
            amount_in: params.amount_in,
            amount_out: current_amount,
            pools_used,
            fee_tiers_used,
        })
    }
    
    // ========================================================
    // QUOTE FUNCTIONS 
    // ========================================================
    
    /// Get best quote across all fee tiers
    pub fn get_best_quote(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        fee_tiers: Vec<u32>,
    ) -> Result<BestQuote, RouterError> {
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        
        let config = read_config(&env);
        
        let tiers = if fee_tiers.is_empty() {
            Self::default_fee_tiers(&env)
        } else {
            fee_tiers
        };
        
        Self::find_best_pool(&env, &config.factory, &token_in, &token_out, amount_in, &tiers)
    }
    
    /// Get quotes from all available pools
    pub fn get_all_quotes(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        fee_tiers: Vec<u32>,
    ) -> Result<Vec<PoolQuote>, RouterError> {
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        
        let config = read_config(&env);
        
        let tiers = if fee_tiers.is_empty() {
            Self::default_fee_tiers(&env)
        } else {
            fee_tiers
        };
        
        Self::get_quotes_for_tiers(&env, &config.factory, &token_in, &token_out, amount_in, &tiers)
    }
    
    /// Get optimal split quote for large orders
    pub fn get_split_quote(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        fee_tiers: Vec<u32>,
    ) -> Result<AggregatedQuote, RouterError> {
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        
        let config = read_config(&env);
        
        let tiers = if fee_tiers.is_empty() {
            Self::default_fee_tiers(&env)
        } else {
            fee_tiers
        };
        
        let best_quote = Self::find_best_pool(&env, &config.factory, &token_in, &token_out, amount_in, &tiers)?;
        
        // Simple implementation: single pool for now
        // TODO: Implement proper split logic based on liquidity depth
        let split = SplitQuote {
            pool: best_quote.pool.clone(),
            fee_bps: best_quote.fee_bps,
            amount_in,
            amount_out: best_quote.amount_out,
        };
        
        Ok(AggregatedQuote {
            total_amount_in: amount_in,
            total_amount_out: best_quote.amount_out,
            splits: vec![&env, split],
            is_split_recommended: false,
        })
    }
    
    /// Quote for multi-hop path
    pub fn quote_multihop(
        env: Env,
        token_in: Address,
        amount_in: i128,
        path: Vec<Hop>,
    ) -> Result<i128, RouterError> {
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        
        if path.is_empty() {
            return Err(RouterError::InvalidPath);
        }
        
        if path.len() > MAX_HOPS {
            return Err(RouterError::PathTooLong);
        }
        
        let config = read_config(&env);
        
        let mut current_token = token_in;
        let mut current_amount = amount_in;
        
        for i in 0..path.len() {
            let hop = path.get(i).unwrap();
            
            let quote = Self::get_single_quote(
                &env,
                &config.factory,
                &current_token,
                &hop.token,
                current_amount,
                hop.fee_bps,
            )?;
            
            current_token = hop.token;
            current_amount = quote.amount_out;
        }
        
        Ok(current_amount)
    }
    
    // ========================================================
    // VIEW FUNCTIONS
    // ========================================================
    
    /// Get router configuration
    pub fn get_config(env: Env) -> Result<RouterConfig, RouterError> {
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        Ok(read_config(&env))
    }
    
    /// Get factory address
    pub fn get_factory(env: Env) -> Result<Address, RouterError> {
        if !is_initialized(&env) {
            return Err(RouterError::NotInitialized);
        }
        Ok(read_config(&env).factory)
    }
    
    /// Check if router is initialized
    pub fn is_initialized(env: Env) -> bool {
        is_initialized(&env)
    }
    
    // ========================================================
    // INTERNAL HELPERS
    // ========================================================
    
    fn default_fee_tiers(env: &Env) -> Vec<u32> {
        vec![env, DEFAULT_FEE_TIERS[0], DEFAULT_FEE_TIERS[1], DEFAULT_FEE_TIERS[2]]
    }
    
    fn get_pool_address(
        env: &Env,
        factory: &Address,
        token_a: &Address,
        token_b: &Address,
        fee_bps: u32,
    ) -> Option<Address> {
        env.invoke_contract(
            factory,
            &Symbol::new(env, "get_pool_address"),
            vec![
                env,
                token_a.clone().into_val(env),
                token_b.clone().into_val(env),
                fee_bps.into_val(env),
            ],
        )
    }
    
    fn preview_pool_swap(
        env: &Env,
        pool: &Address,
        token_in: &Address,
        amount_in: i128,
        sqrt_price_limit: u128,
    ) -> Option<(i128, i128)> {
        let result: PreviewResultRaw = env.invoke_contract(
            pool,
            &Symbol::new(env, "preview_swap"),
            vec![
                env,
                token_in.clone().into_val(env),
                amount_in.into_val(env),
                0i128.into_val(env),
                sqrt_price_limit.into_val(env),
            ],
        );
        
        if result.is_valid {
            Some((result.amount_out, result.price_impact_bps))
        } else {
            None
        }
    }
    
    fn find_best_pool(
        env: &Env,
        factory: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        fee_tiers: &Vec<u32>,
    ) -> Result<BestQuote, RouterError> {
        let quotes = Self::get_quotes_for_tiers(env, factory, token_in, token_out, amount_in, fee_tiers)?;
        
        if quotes.is_empty() {
            return Err(RouterError::NoPoolsFound);
        }
        
        // Find best quote by output amount
        let mut best_idx: u32 = 0;
        let mut best_out: i128 = 0;
        
        for i in 0..quotes.len() {
            let q = quotes.get(i).unwrap();
            if q.amount_out > best_out {
                best_out = q.amount_out;
                best_idx = i;
            }
        }
        
        let best = quotes.get(best_idx).unwrap();
        
        Ok(BestQuote {
            pool: best.pool,
            fee_bps: best.fee_bps,
            amount_out: best.amount_out,
            price_impact_bps: best.price_impact_bps,
            all_quotes: quotes,
        })
    }
    
    fn get_quotes_for_tiers(
        env: &Env,
        factory: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        fee_tiers: &Vec<u32>,
    ) -> Result<Vec<PoolQuote>, RouterError> {
        let mut quotes = Vec::new(env);
        
        for i in 0..fee_tiers.len() {
            let fee_bps = fee_tiers.get(i).unwrap();
            
            if let Ok(quote) = Self::get_single_quote(env, factory, token_in, token_out, amount_in, fee_bps) {
                quotes.push_back(quote);
            }
        }
        
        Ok(quotes)
    }
    
    fn get_single_quote(
        env: &Env,
        factory: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        fee_bps: u32,
    ) -> Result<PoolQuote, RouterError> {
        let pool = Self::get_pool_address(env, factory, token_in, token_out, fee_bps)
            .ok_or(RouterError::PoolNotFound)?;
        
        let (amount_out, price_impact) = Self::preview_pool_swap(env, &pool, token_in, amount_in, 0)
            .ok_or(RouterError::QuoteFailed)?;
        
        Ok(PoolQuote {
            pool,
            fee_bps,
            amount_out,
            price_impact_bps: price_impact,
        })
    }
    
    /// Execute swap pulling tokens from an external source (user)
    fn execute_single_swap(
        env: &Env,
        token_source: &Address,
        recipient: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        amount_out_min: i128,
        pool: &Address,
        fee_bps: u32,
    ) -> Result<SwapResult, RouterError> {
        let router_addr = env.current_contract_address();
        
        // Transfer token_in from source (user) to router
        token::Client::new(env, token_in).transfer(token_source, &router_addr, &amount_in);
        
        // Execute the actual swap via pool
        Self::execute_pool_swap(
            env,
            recipient,
            token_in,
            token_out,
            amount_in,
            amount_out_min,
            pool,
            fee_bps,
        )
    }
    
    /// Execute swap using tokens already in router
    fn execute_swap_from_router(
        env: &Env,
        recipient: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        amount_out_min: i128,
        pool: &Address,
        fee_bps: u32,
    ) -> Result<SwapResult, RouterError> {
        Self::execute_pool_swap(
            env,
            recipient,
            token_in,
            token_out,
            amount_in,
            amount_out_min,
            pool,
            fee_bps,
        )
    }
    
    /// Common pool swap execution logic
    fn execute_pool_swap(
        env: &Env,
        recipient: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        amount_out_min: i128,
        pool: &Address,
        fee_bps: u32,
    ) -> Result<SwapResult, RouterError> {
        let router_addr = env.current_contract_address();
        let current_ledger = env.ledger().sequence();
        
        // Minimal approval - just enough for this transaction
        token::Client::new(env, token_in).approve(
            &router_addr,
            pool,
            &amount_in,
            &(current_ledger + APPROVAL_LEDGER_BUFFER),
        );
        
        // Call pool swap - router is the sender
        let swap_result: PoolSwapResult = env.invoke_contract(
            pool,
            &Symbol::new(env, "swap"),
            vec![
                env,
                router_addr.clone().into_val(env),
                token_in.clone().into_val(env),
                amount_in.into_val(env),
                amount_out_min.into_val(env),
                0u128.into_val(env), // no price limit
            ],
        );
        
        // Transfer output to final recipient if not router
        if recipient != &router_addr {
            token::Client::new(env, token_out).transfer(
                &router_addr,
                recipient,
                &swap_result.amount_out,
            );
        }
        
        Ok(SwapResult {
            amount_in: swap_result.amount_in,
            amount_out: swap_result.amount_out,
            pools_used: vec![env, pool.clone()],
            fee_tiers_used: vec![env, fee_bps],
        })
    }
}

// ============================================================
// HELPER TYPES FOR CROSS-CONTRACT CALLS
// ============================================================

/// Raw preview result from pool
#[soroban_sdk::contracttype]
#[derive(Clone, Debug)]
struct PreviewResultRaw {
    pub is_valid: bool,
    pub amount_in: i128,
    pub amount_out: i128,
    pub fee_amount: i128,
    pub price_impact_bps: i128,
    pub error_code: Symbol,
}

/// Raw swap result from pool
#[soroban_sdk::contracttype]
#[derive(Clone, Debug)]
struct PoolSwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub current_tick: i32,
    pub sqrt_price_x64: u128,
}