#![no_std]

//! # BelugaSwap Factory
//! 
//! Permissionless pool deployment.
//! 
//! ## Deployment Flow:
//! 1. Deploy Factory → initialize(admin, pool_wasm_hash)
//! 2. Deploy Router
//! 3. Set router in Factory → set_router(router_address)
//! 4. Create pools via Factory
//! 
//! ## Responsibilities:
//! 1. Create pools (atomic: deploy + init + initial LP)
//! 2. Fee tier standardization
//! 3. Duplicate prevention
//! 4. Router registry

use soroban_sdk::{
    contract, contractimpl, vec, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
    token, xdr::ToXdr,
};

mod error;
mod events;
mod storage;
mod types;

pub use error::FactoryError;
use events::*;
use storage::*;
pub use types::*;

// ============================================================
// CONSTANTS
// ============================================================

/// Minimum initial liquidity per token (0.1 with 7 decimals)
const MIN_INITIAL_LIQUIDITY: i128 = 1_000_000;

/// Maximum fee in bps (1%)
const MAX_FEE_BPS: u32 = 100;

// ============================================================
// CONTRACT
// ============================================================

#[contract]
pub struct BelugaFactory;

#[contractimpl]
impl BelugaFactory {
    // ========================================================
    // INITIALIZATION (Phase 1)
    // ========================================================
    
/// Deploy-time setup; the factory cannot be initialized by anyone else.
    pub fn __constructor(env: Env, admin: Address, pool_wasm_hash: BytesN<32>) {
        write_config(&env, &FactoryConfig {
            admin: admin.clone(),
            pool_wasm_hash,
            router: None,
        });
        init_pool_list(&env);
        Self::init_fee_tiers(&env);
        extend_instance(&env);

        emit_initialized(&env, &admin);
    }
    
    // ========================================================
    // ROUTER SETUP (Phase 2)
    // ========================================================
    
    /// Set router address (admin only, one-time setup)
    /// Call this after deploying the router contract
    pub fn set_router(
        env: Env,
        router: Address,
    ) -> Result<(), FactoryError> {
        extend_instance(&env);
        let mut config = read_config(&env);
        config.admin.require_auth();
        
        // Allow setting router only once (or allow update - your choice)
        // Current: allow update for flexibility
        config.router = Some(router.clone());
        write_config(&env, &config);
        
        emit_router_set(&env, &router);
        
        Ok(())
    }
    
    /// Get router address
    pub fn get_router(env: Env) -> Option<Address> {
        read_config(&env).router
    }
    
    // ========================================================
    // POOL CREATION
    // ========================================================
    
    /// Create pool (atomic: deploy + init + initial LP). The initial
    /// position is an ordinary position the creator can withdraw at any time.
    /// 
    /// # Arguments
    /// * `creator` - Pool creator (must provide initial LP)
    /// * `params` - Pool creation parameters (see CreatePoolParams)
    pub fn create_pool(
        env: Env,
        creator: Address,
        params: CreatePoolParams,
    ) -> Result<Address, FactoryError> {
        extend_instance(&env);
        creator.require_auth();
        
        // Unpack params
        let token_a = params.token_a;
        let token_b = params.token_b;
        let fee_bps = params.fee_bps;
        let initial_sqrt_price_x64 = params.initial_sqrt_price_x64;
        let amount0_desired = params.amount0_desired;
        let amount1_desired = params.amount1_desired;
        let lower_tick = params.lower_tick;
        let upper_tick = params.upper_tick;
        
        let config = read_config(&env);
        
        // Router must be set before creating pools
        if config.router.is_none() {
            return Err(FactoryError::RouterNotSet);
        }
        
        // Validate tokens
        if token_a == token_b {
            return Err(FactoryError::InvalidTokenPair);
        }
        
        // Sort tokens
        let (token0, token1) = sort_tokens(&token_a, &token_b);
        let (amt0, amt1) = if token_a < token_b {
            (amount0_desired, amount1_desired)
        } else {
            (amount1_desired, amount0_desired)
        };
        
        // Check pool doesn't exist
        if pool_exists(&env, &token0, &token1, fee_bps) {
            return Err(FactoryError::PoolAlreadyExists);
        }
        
        // Validate fee tier
        let tier = read_fee_tier(&env, fee_bps)
            .filter(|t| t.enabled)
            .ok_or(FactoryError::InvalidFeeTier)?;
        let tick_spacing = tier.tick_spacing;
        
        // Validate ticks
        if lower_tick >= upper_tick {
            return Err(FactoryError::InvalidTickRange);
        }
        if lower_tick % tick_spacing != 0 || upper_tick % tick_spacing != 0 {
            return Err(FactoryError::InvalidTickSpacing);
        }
        
        // Validate amounts
        if amt0 < MIN_INITIAL_LIQUIDITY || amt1 < MIN_INITIAL_LIQUIDITY {
            return Err(FactoryError::InsufficientInitialLiquidity);
        }
        
        // Validate price
        if initial_sqrt_price_x64 == 0 {
            return Err(FactoryError::InvalidInitialPrice);
        }
        
        // === DEPLOY + INITIALIZE POOL (one step) ===
        let router_address = config.router.clone().unwrap(); // Safe: checked above
        let pool_address = Self::deploy_pool(
            &env,
            &config,
            &token0,
            &token1,
            fee_bps,
            (
                env.current_contract_address(),
                router_address,
                creator.clone(),
                token_a.clone(),
                token_b.clone(),
                fee_bps,
                initial_sqrt_price_x64,
                tick_spacing,
            )
                .into_val(&env),
        );
        
        // === TRANSFER TOKENS ===
        token::Client::new(&env, &token0).transfer(&creator, &pool_address, &amt0);
        token::Client::new(&env, &token1).transfer(&creator, &pool_address, &amt1);
        
        // === ADD LIQUIDITY ===
        let liquidity: i128 = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "mint"),
            vec![
                &env,
                creator.clone().into_val(&env),
                lower_tick.into_val(&env),
                upper_tick.into_val(&env),
                amt0.into_val(&env),
                amt1.into_val(&env),
            ],
        );
        
        if liquidity <= 0 {
            return Err(FactoryError::InsufficientInitialLiquidity);
        }
        
        // === REGISTER POOL ===
        write_pool(&env, &token0, &token1, fee_bps, &pool_address);
        add_to_pool_list(&env, &pool_address);
        
        // === EMIT EVENTS ===
        emit_pool_created(&env, &pool_address, &token0, &token1, &creator, fee_bps);
        
        Ok(pool_address)
    }
    
    // ========================================================
    // READ FUNCTIONS 
    // ========================================================
    
    /// Get pool contract address by token pair and fee tier
    pub fn get_pool_address(
        env: Env, 
        token_a: Address, 
        token_b: Address, 
        fee_bps: u32
    ) -> Option<Address> {
        let (token0, token1) = sort_tokens(&token_a, &token_b);
        read_pool(&env, &token0, &token1, fee_bps)
    }
    
    /// Check if pool is already deployed for this pair+fee
    pub fn is_pool_deployed(
        env: Env, 
        token_a: Address, 
        token_b: Address, 
        fee_bps: u32
    ) -> bool {
        let (token0, token1) = sort_tokens(&token_a, &token_b);
        pool_exists(&env, &token0, &token1, fee_bps)
    }
    
    /// Get total number of deployed pools
    pub fn get_total_pools(env: Env) -> u32 {
        read_pool_count(&env)
    }
    
    /// Get the first deployed pool addresses (at most 100). Use `get_pools`
    /// to page through the whole registry.
    pub fn get_all_pool_addresses(env: Env) -> Vec<Address> {
        read_pool_page(&env, 0, MAX_POOL_PAGE)
    }
    
    /// Get up to `limit` pool addresses (at most 100) from creation index `start`
    pub fn get_pools(env: Env, start: u32, limit: u32) -> Vec<Address> {
        read_pool_page(&env, start, limit)
    }
    
    /// Get fee tier configuration
    pub fn get_fee_tier(env: Env, fee_bps: u32) -> Option<FeeTier> {
        read_fee_tier(&env, fee_bps)
    }
    
    /// Check if factory is ready (initialized + router set)
    pub fn is_ready(env: Env) -> bool {
        read_config(&env).router.is_some()
    }
    
    // ========================================================
    // ADMIN FUNCTIONS 
    // ========================================================
    
    /// Update pool WASM hash (for future pool deployments)
    pub fn set_pool_wasm_hash(
        env: Env, 
        new_hash: BytesN<32>
    ) -> Result<(), FactoryError> {
        extend_instance(&env);
        let mut config = read_config(&env);
        config.admin.require_auth();
        config.pool_wasm_hash = new_hash;
        write_config(&env, &config);
        Ok(())
    }
    
    /// Transfer admin role to new address
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), FactoryError> {
        extend_instance(&env);
        let mut config = read_config(&env);
        config.admin.require_auth();
        new_admin.require_auth();
        
        emit_admin_updated(&env, &config.admin, &new_admin);
        
        config.admin = new_admin;
        write_config(&env, &config);
        Ok(())
    }
    
    /// Add or update fee tier configuration
    pub fn set_fee_tier(
        env: Env,
        fee_bps: u32,
        tick_spacing: i32,
        enabled: bool,
    ) -> Result<(), FactoryError> {
        extend_instance(&env);
        let config = read_config(&env);
        config.admin.require_auth();
        
        if tick_spacing <= 0 {
            return Err(FactoryError::InvalidTickSpacing);
        }
        
        if fee_bps == 0 || fee_bps > MAX_FEE_BPS {
            return Err(FactoryError::InvalidFeeTier);
        }
        
        let tier = FeeTier {
            fee_bps,
            tick_spacing,
            enabled,
        };
        write_fee_tier(&env, fee_bps, &tier);
        
        emit_fee_tier_updated(&env, fee_bps, tick_spacing, enabled);
        
        Ok(())
    }
    
    // ========================================================
    // INTERNAL HELPERS
    // ========================================================
    
    fn init_fee_tiers(env: &Env) {
        // Stablecoin: 0.05%
        write_fee_tier(env, 5, &FeeTier {
            fee_bps: 5,
            tick_spacing: 10,
            enabled: true,
        });
        
        // Volatile: 0.30%
        write_fee_tier(env, 30, &FeeTier {
            fee_bps: 30,
            tick_spacing: 60,
            enabled: true,
        });
        
        // Meme/Exotic: 1.00%
        write_fee_tier(env, 100, &FeeTier {
            fee_bps: 100,
            tick_spacing: 200,
            enabled: true,
        });
    }
    
    fn deploy_pool(
        env: &Env,
        config: &FactoryConfig,
        token0: &Address,
        token1: &Address,
        fee_bps: u32,
        constructor_args: Vec<Val>,
    ) -> Address {
        // Deterministic salt
        let mut salt_data = token0.clone().to_xdr(env);
        salt_data.append(&token1.clone().to_xdr(env));
        salt_data.append(&fee_bps.to_xdr(env));
        let salt = env.crypto().sha256(&salt_data);
        
        env.deployer()
            .with_current_contract(salt)
            .deploy_v2(config.pool_wasm_hash.clone(), constructor_args)
    }
    
}