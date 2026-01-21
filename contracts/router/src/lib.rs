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
//! ## Functions:
//! - Write (4): initialize, swap_exact_input, swap_split, swap_multihop
//! - Read (4): get_best_quote, get_all_quotes, get_split_quote, quote_multihop

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
    // SWAP FUNCTIONS (Write)
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
        
        // Execute swap
        let result = Self::execute_single_swap(
            &env,
            &sender,
            &params.recipient,
            &params.token_in,
            &params.token_out,
            params.amount_in,
            params.amount_out_min,
            &best_quote.pool,
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
            return Err(RouterError::NoPoolsFound);
        }
        
        // Validate total split amounts
        let mut total_split_in: i128 = 0;
        for i in 0..splits.len() {
            let split = splits.get(i).unwrap();
            total_split_in = total_split_in.saturating_add(split.amount_in);
        }
        
        if total_split_in != amount_in {
            return Err(RouterError::InvalidAmount);
        }
        
        // Execute each split
        let mut total_out: i128 = 0;
        let mut pools_used = Vec::new(&env);
        let mut fee_tiers_used = Vec::new(&env);
        
        for i in 0..splits.len() {
            let split = splits.get(i).unwrap();
            
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
                MIN_OUTPUT, // Each split has minimal slippage check
                &split.pool,
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
            splits.len(),
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
            let hop_recipient = if i == params.path.len() - 1 {
                params.recipient.clone()
            } else {
                router_addr.clone()
            };
            
            // Min out for intermediate hops is 1 (just dust protection)
            let min_out = if i == params.path.len() - 1 {
                params.amount_out_min
            } else {
                MIN_OUTPUT
            };
            
            // Execute swap
            let result = Self::execute_single_swap(
                &env,
                &sender,
                &hop_recipient,
                &current_token,
                &next_token,
                current_amount,
                min_out,
                &pool,
            )?;
            
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
    // QUOTE FUNCTIONS (Read)
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
    
    /// Get optimized split quote for large orders
    /// 
    /// Returns recommended splits to minimize price impact
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
        
        // Get quotes for all pools
        let quotes = Self::get_quotes_for_tiers(
            &env,
            &config.factory,
            &token_in,
            &token_out,
            amount_in,
            &tiers,
        )?;
        
        if quotes.is_empty() {
            return Err(RouterError::NoPoolsFound);
        }
        
        // Simple split strategy: if price impact > 100 bps (1%), consider splitting
        // Check if single best pool has low price impact
        let mut best_idx: u32 = 0;
        let mut best_out: i128 = 0;
        
        for i in 0..quotes.len() {
            let q = quotes.get(i).unwrap();
            if q.amount_out > best_out {
                best_out = q.amount_out;
                best_idx = i;
            }
        }
        
        let best_quote = quotes.get(best_idx).unwrap();
        
        // If price impact is acceptable or only one pool, don't split
        if best_quote.price_impact_bps < 100 || quotes.len() == 1 {
            let split = SplitQuote {
                pool: best_quote.pool.clone(),
                fee_bps: best_quote.fee_bps,
                amount_in,
                amount_out: best_quote.amount_out,
            };
            
            return Ok(AggregatedQuote {
                total_amount_in: amount_in,
                total_amount_out: best_quote.amount_out,
                splits: vec![&env, split],
                is_split_recommended: false,
            });
        }
        
        // Calculate optimal split
        // Simple 2-way split between top 2 pools by output
        let mut sorted_quotes: Vec<PoolQuote> = Vec::new(&env);
        for i in 0..quotes.len() {
            sorted_quotes.push_back(quotes.get(i).unwrap());
        }
        
        // Find top 2 pools
        let pool1 = quotes.get(best_idx).unwrap();
        let mut second_best_idx: u32 = 0;
        let mut second_best_out: i128 = 0;
        
        for i in 0..quotes.len() {
            if i == best_idx {
                continue;
            }
            let q = quotes.get(i).unwrap();
            if q.amount_out > second_best_out {
                second_best_out = q.amount_out;
                second_best_idx = i;
            }
        }
        
        let pool2 = quotes.get(second_best_idx).unwrap();
        
        // 70/30 split favoring the better pool
        let amount1 = amount_in.saturating_mul(70).saturating_div(100);
        let amount2 = amount_in.saturating_sub(amount1);
        
        // Get quotes for split amounts
        let quote1 = Self::get_single_quote(
            &env,
            &config.factory,
            &token_in,
            &token_out,
            amount1,
            pool1.fee_bps,
        )?;
        
        let quote2 = Self::get_single_quote(
            &env,
            &config.factory,
            &token_in,
            &token_out,
            amount2,
            pool2.fee_bps,
        )?;
        
        let split1 = SplitQuote {
            pool: quote1.pool,
            fee_bps: quote1.fee_bps,
            amount_in: amount1,
            amount_out: quote1.amount_out,
        };
        
        let split2 = SplitQuote {
            pool: quote2.pool,
            fee_bps: quote2.fee_bps,
            amount_in: amount2,
            amount_out: quote2.amount_out,
        };
        
        let total_split_out = split1.amount_out.saturating_add(split2.amount_out);
        
        // Only recommend split if it's actually better
        let is_split_better = total_split_out > best_out;
        
        if is_split_better {
            Ok(AggregatedQuote {
                total_amount_in: amount_in,
                total_amount_out: total_split_out,
                splits: vec![&env, split1, split2],
                is_split_recommended: true,
            })
        } else {
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
        // Call pool's preview_swap
        // Returns PreviewResult { is_valid, amount_in, amount_out, fee_amount, price_impact_bps, error_code }
        let result: PreviewResultRaw = env.invoke_contract(
            pool,
            &Symbol::new(env, "preview_swap"),
            vec![
                env,
                token_in.clone().into_val(env),
                amount_in.into_val(env),
                0i128.into_val(env), // min_amount_out = 0 for quote
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
    
    fn execute_single_swap(
        env: &Env,
        sender: &Address,
        recipient: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        amount_out_min: i128,
        pool: &Address,
    ) -> Result<SwapResult, RouterError> {
        // Transfer token_in from sender to router first
        let router_addr = env.current_contract_address();
        token::Client::new(env, token_in).transfer(sender, &router_addr, &amount_in);
        
        // Approve pool to spend tokens
        token::Client::new(env, token_in).approve(&router_addr, pool, &amount_in, &(env.ledger().sequence() + 1000));
        
        // Call pool swap
        // Pool.swap(sender, token_in, amount_in, amount_out_min, sqrt_price_limit)
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
        
        // Transfer output to recipient
        if recipient != &router_addr {
            token::Client::new(env, token_out).transfer(&router_addr, recipient, &swap_result.amount_out);
        }
        
        Ok(SwapResult {
            amount_in: swap_result.amount_in,
            amount_out: swap_result.amount_out,
            pools_used: vec![env, pool.clone()],
            fee_tiers_used: vec![env, 0u32], // Will be filled by caller
        })
    }
}

// ============================================================
// HELPER TYPES FOR CROSS-CONTRACT CALLS
// ============================================================

/// Raw preview result from pool (matching pool's PreviewResult)
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