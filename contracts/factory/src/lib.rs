#![no_std]

//! # BelugaSwap Factory
//! 
//! Permissionless pool deployment with creator incentives.
//! 
//! ## Responsibilities:
//! 1. Create pools (atomic: deploy + init + LP + lock)
//! 2. Fee tier standardization
//! 3. Duplicate prevention
//! 4. Creator lock management

use soroban_sdk::{
    contract, contractimpl, vec, Address, BytesN, Env, IntoVal, Symbol, Vec,
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

/// Minimum lock duration: ~7 days at 5s/ledger
const MIN_LOCK_DURATION: u32 = 120_960;

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
    // WRITE FUNCTIONS (3)
    // ========================================================
    
    /// Initialize factory
    pub fn initialize(
        env: Env,
        admin: Address,
        pool_wasm_hash: BytesN<32>,
    ) -> Result<(), FactoryError> {
        admin.require_auth();
        
        if is_initialized(&env) {
            return Err(FactoryError::AlreadyInitialized);
        }
        
        // Save config
        let config = FactoryConfig {
            admin: admin.clone(),
            pool_wasm_hash,
        };
        write_config(&env, &config);
        set_initialized(&env);
        init_pool_list(&env);
        
        // Init default fee tiers
        Self::init_fee_tiers(&env);
        
        emit_initialized(&env, &admin);
        
        Ok(())
    }
    
    /// Create pool (atomic: deploy + init + LP + lock)
    /// 
    /// # Arguments
    /// * `creator` - Pool creator (must provide initial LP)
    /// * `params` - Pool creation parameters (see CreatePoolParams)
    pub fn create_pool(
        env: Env,
        creator: Address,
        params: CreatePoolParams,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();
        
        // Unpack params
        let token_a = params.token_a;
        let token_b = params.token_b;
        let fee_bps = params.fee_bps;
        let creator_fee_bps = params.creator_fee_bps;
        let initial_sqrt_price_x64 = params.initial_sqrt_price_x64;
        let amount0_desired = params.amount0_desired;
        let amount1_desired = params.amount1_desired;
        let lower_tick = params.lower_tick;
        let upper_tick = params.upper_tick;
        let lock_duration = params.lock_duration;
        
        if !is_initialized(&env) {
            return Err(FactoryError::NotInitialized);
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
        
        // Validate lock duration
        let is_permanent = lock_duration == 0;
        if !is_permanent && lock_duration < MIN_LOCK_DURATION {
            return Err(FactoryError::InvalidLockDuration);
        }
        
        // Validate creator fee (0.1% - 10%)
        if creator_fee_bps < 10 || creator_fee_bps > 1000 {
            return Err(FactoryError::InvalidCreatorFee);
        }
        
        // Validate price
        if initial_sqrt_price_x64 == 0 {
            return Err(FactoryError::InvalidInitialPrice);
        }
        
        // === DEPLOY POOL ===
        let config = read_config(&env);
        let pool_address = Self::deploy_pool(&env, &config, &token0, &token1, fee_bps);
        
        // === INITIALIZE POOL ===
        let current_tick = Self::sqrt_price_to_tick(initial_sqrt_price_x64);
        
        let _: () = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "initialize"),
            vec![
                &env,
                creator.clone().into_val(&env),
                token_a.clone().into_val(&env),
                token_b.clone().into_val(&env),
                fee_bps.into_val(&env),
                creator_fee_bps.into_val(&env),
                initial_sqrt_price_x64.into_val(&env),
                current_tick.into_val(&env),
                tick_spacing.into_val(&env),
            ],
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
        
        // === REGISTER LOCK ===
        let current_ledger = env.ledger().sequence();
        let lock_end = if is_permanent {
            u32::MAX
        } else {
            current_ledger.saturating_add(lock_duration)
        };
        
        let creator_lock = CreatorLock {
            pool: pool_address.clone(),
            creator: creator.clone(),
            liquidity,
            lower_tick,
            upper_tick,
            lock_start: current_ledger,
            lock_end,
            is_permanent,
            is_unlocked: false,
            fee_revoked: false,
        };
        write_creator_lock(&env, &pool_address, &creator, &creator_lock);
        
        // === REGISTER POOL ===
        write_pool(&env, &token0, &token1, fee_bps, &pool_address);
        add_to_pool_list(&env, &pool_address);
        increment_pool_count(&env);
        
        // === EMIT EVENTS ===
        emit_pool_created(&env, &pool_address, &token0, &token1, &creator, fee_bps);
        emit_creator_locked(&env, &pool_address, &creator, liquidity, lock_end, is_permanent);
        
        Ok(pool_address)
    }
    
    /// Unlock creator liquidity (REVOKES CREATOR FEE PERMANENTLY!)
    pub fn unlock_creator_liquidity(
        env: Env,
        pool_address: Address,
        creator: Address,
    ) -> Result<i128, FactoryError> {
        creator.require_auth();
        
        let mut lock = read_creator_lock(&env, &pool_address, &creator)
            .ok_or(FactoryError::CreatorLockNotFound)?;
        
        if lock.creator != creator {
            return Err(FactoryError::NotPoolCreator);
        }
        
        if lock.is_unlocked {
            return Err(FactoryError::CreatorFeeRevoked);
        }
        
        if lock.is_permanent {
            return Err(FactoryError::LiquidityStillLocked);
        }
        
        let current_ledger = env.ledger().sequence();
        if current_ledger < lock.lock_end {
            return Err(FactoryError::LiquidityStillLocked);
        }
        
        // REVOKE PERMANENTLY
        lock.is_unlocked = true;
        lock.fee_revoked = true;
        write_creator_lock(&env, &pool_address, &creator, &lock);
        
        emit_creator_unlocked(&env, &pool_address, &creator, lock.liquidity);
        emit_creator_fee_revoked(&env, &pool_address, &creator);
        
        Ok(lock.liquidity)
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
    
    /// Get all deployed pool addresses
    pub fn get_all_pool_addresses(env: Env) -> Vec<Address> {
        read_pool_list(&env)
    }
    
    /// Get fee tier configuration
    pub fn get_fee_tier(env: Env, fee_bps: u32) -> Option<FeeTier> {
        read_fee_tier(&env, fee_bps)
    }
    
    /// Get creator lock info for a pool
    pub fn get_creator_lock(
        env: Env, 
        pool_address: Address, 
        creator: Address
    ) -> Option<CreatorLock> {
        read_creator_lock(&env, &pool_address, &creator)
    }
    
    /// Check if creator fee is still active (not revoked)
    /// 
    /// This is called by Pool contract during swaps to determine
    /// if creator should still receive fee share.
    /// 
    /// Returns true if:
    /// - Creator lock exists AND
    /// - fee_revoked == false
    /// 
    /// Returns false if:
    /// - Creator lock doesn't exist OR
    /// - fee_revoked == true
    pub fn is_creator_fee_active(
        env: Env,
        pool_address: Address,
        creator: Address,
    ) -> bool {
        match read_creator_lock(&env, &pool_address, &creator) {
            Some(lock) => !lock.fee_revoked,
            None => false,
        }
    }
    
    // ========================================================
    // ADMIN FUNCTIONS 
    // ========================================================
    
    /// Update pool WASM hash (for future pool deployments)
    pub fn set_pool_wasm_hash(
        env: Env, 
        new_hash: BytesN<32>
    ) -> Result<(), FactoryError> {
        let mut config = read_config(&env);
        config.admin.require_auth();
        config.pool_wasm_hash = new_hash;
        write_config(&env, &config);
        Ok(())
    }
    
    /// Transfer admin role to new address
    ///  Both old and new admin must authorize
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), FactoryError> {
        let mut config = read_config(&env);
        config.admin.require_auth();
        new_admin.require_auth();  // [FIX] New admin must also authorize
        
        emit_admin_updated(&env, &config.admin, &new_admin);
        
        config.admin = new_admin;
        write_config(&env, &config);
        Ok(())
    }
    
    /// Add or update fee tier configuration
    /// Added fee_bps validation
    pub fn set_fee_tier(
        env: Env,
        fee_bps: u32,
        tick_spacing: i32,
        enabled: bool,
    ) -> Result<(), FactoryError> {
        let config = read_config(&env);
        config.admin.require_auth();
        
        if tick_spacing <= 0 {
            return Err(FactoryError::InvalidTickSpacing);
        }
        
        // Validate fee_bps range
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
    ) -> Address {
        // Deterministic salt
        let mut salt_data = token0.clone().to_xdr(env);
        salt_data.append(&token1.clone().to_xdr(env));
        salt_data.append(&fee_bps.to_xdr(env));
        let salt = env.crypto().sha256(&salt_data);
        
        env.deployer()
            .with_current_contract(salt)
            .deploy(config.pool_wasm_hash.clone())
    }
    
    fn sqrt_price_to_tick(sqrt_price_x64: u128) -> i32 {
        const ONE_X64: u128 = 1u128 << 64;
        
        if sqrt_price_x64 == ONE_X64 {
            return 0;
        }
        
        if sqrt_price_x64 > ONE_X64 {
            let ratio = sqrt_price_x64 / ONE_X64;
            if ratio == 0 { return 0; }
            (ratio.ilog2() as i32) * 6932
        } else {
            let ratio = ONE_X64 / sqrt_price_x64.max(1);
            if ratio == 0 { return 0; }
            -((ratio.ilog2() as i32) * 6932)
        }
    }
}