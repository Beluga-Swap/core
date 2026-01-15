#![no_std]

use soroban_sdk::{contract, contractimpl, vec, Address, BytesN, Env, Symbol, Bytes, IntoVal, xdr::ToXdr};

mod error;
mod events;
mod storage;
mod types;

use error::FactoryError;
use events::*;
use storage::*;
use types::*;

#[contract]
pub struct BelugaFactory;

#[contractimpl]
impl BelugaFactory {
    // ============================================================
    // INITIALIZATION
    // ============================================================
    
    /// Initialize the factory contract
    /// Can only be called once by the admin
    pub fn initialize(
        env: Env,
        admin: Address,
        pool_wasm_hash: BytesN<32>,
        min_initial_liquidity: i128,
        min_lock_duration: u32,
        default_fee_bps: u32,
        default_creator_fee_bps: u32,
    ) -> Result<(), FactoryError> {
        admin.require_auth();
        
        if factory_is_initialized(&env) {
            return Err(FactoryError::AlreadyInitialized);
        }
        
        // Validate parameters
        if default_fee_bps > 10000 {
            return Err(FactoryError::InvalidFeeTier);
        }
        if default_creator_fee_bps > 5000 {
            return Err(FactoryError::InvalidFeeTier);
        }
        
        let config = FactoryConfig {
            admin: admin.clone(),
            pool_wasm_hash: pool_wasm_hash.clone(),
            min_initial_liquidity,
            min_lock_duration,
            default_fee_bps,
            default_creator_fee_bps,
        };
        
        write_factory_config(&env, &config);
        factory_set_initialized(&env);
        
        // Initialize default fee tiers
        Self::_init_default_fee_tiers(&env);
        
        emit_factory_initialized(&env, &admin, &pool_wasm_hash, min_initial_liquidity);
        
        Ok(())
    }
    
    fn _init_default_fee_tiers(env: &Env) {
        let tiers = [
            (1, 1),      // 0.01% fee, tick spacing 1
            (5, 10),     // 0.05% fee, tick spacing 10
            (30, 60),    // 0.30% fee, tick spacing 60
            (100, 200),  // 1.00% fee, tick spacing 200
        ];
        
        for (fee_bps, tick_spacing) in tiers {
            let tier = FeeTier {
                fee_bps,
                tick_spacing,
                enabled: true,
            };
            write_fee_tier(env, fee_bps, &tier);
        }
    }
    
    // ============================================================
    // PERMISSIONLESS POOL CREATION
    // ============================================================
    
    pub fn create_pool(
        env: Env,
        creator: Address,
        params: CreatePoolParams,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();
        
        if !factory_is_initialized(&env) {
            return Err(FactoryError::NotInitialized);
        }
        
        let config = read_factory_config(&env);
        
        // Validate token pair
        if params.token_a == params.token_b {
            return Err(FactoryError::InvalidTokenPair);
        }
        
        // Sort tokens
        let (token0, token1, amount0, amount1) = if params.token_a < params.token_b {
            (params.token_a.clone(), params.token_b.clone(), params.amount_a, params.amount_b)
        } else {
            (params.token_b.clone(), params.token_a.clone(), params.amount_b, params.amount_a)
        };
        
        // Check if pool already exists
        if pool_exists(&env, &token0, &token1, params.fee_bps) {
            return Err(FactoryError::PoolAlreadyExists);
        }
        
        // Validate fee tier
        let tick_spacing = match get_tick_spacing_for_fee(&env, params.fee_bps) {
            Some(ts) => ts,
            None => return Err(FactoryError::InvalidFeeTier),
        };
        
        // Validate tick range
        if params.initial_lower_tick >= params.initial_upper_tick {
            return Err(FactoryError::InvalidTickRange);
        }
        if params.initial_lower_tick % tick_spacing != 0 || params.initial_upper_tick % tick_spacing != 0 {
            return Err(FactoryError::InvalidTickSpacing);
        }
        
        // Validate initial liquidity
        if amount0 < config.min_initial_liquidity || amount1 < config.min_initial_liquidity {
            return Err(FactoryError::InsufficientInitialLiquidity);
        }
        
        // Validate lock duration
        let lock_duration = if params.permanent_lock {
            u32::MAX
        } else {
            if params.lock_duration < config.min_lock_duration {
                return Err(FactoryError::LockDurationTooShort);
            }
            params.lock_duration
        };
        
        // Validate creator fee
        let creator_fee_bps = if params.creator_fee_bps == 0 {
            config.default_creator_fee_bps
        } else if params.creator_fee_bps > 5000 {
            return Err(FactoryError::InvalidFeeTier);
        } else {
            params.creator_fee_bps
        };
        
        // Deploy the pool contract
        let pool_address = Self::_deploy_pool(
            &env,
            &config.pool_wasm_hash,
            &token0,
            &token1,
            params.fee_bps,
        );
        
        // Initialize the pool
        Self::_initialize_pool(
            &env,
            &pool_address,
            &token0,
            &token1,
            params.fee_bps,
            tick_spacing,
            params.initial_sqrt_price_x64,
            creator_fee_bps,
            &creator,
        );
        
        // Transfer initial liquidity from creator to pool
        Self::_transfer_tokens(&env, &creator, &pool_address, &token0, amount0);
        Self::_transfer_tokens(&env, &creator, &pool_address, &token1, amount1);
        
        // Add initial liquidity position via pool contract
        let liquidity: i128 = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "mint"),
            vec![
                &env,
                creator.clone().into_val(&env),
                params.initial_lower_tick.into_val(&env),
                params.initial_upper_tick.into_val(&env),
                amount0.into_val(&env),
                amount1.into_val(&env),
            ],
        );
        
        if liquidity <= 0 {
            return Err(FactoryError::InsufficientInitialLiquidity);
        }
        
        // Register locked liquidity
        let current_ledger = env.ledger().sequence();
        let lock_end = if params.permanent_lock {
            u32::MAX
        } else {
            current_ledger.saturating_add(lock_duration)
        };
        
        let locked = LockedLiquidity {
            pool_address: pool_address.clone(),
            owner: creator.clone(),
            lower_tick: params.initial_lower_tick,
            upper_tick: params.initial_upper_tick,
            liquidity,
            lock_start: current_ledger,
            lock_end,
            is_permanent: params.permanent_lock,
            is_unlocked: false,
            initial_amount0: amount0,
            initial_amount1: amount1,
        };
        
        write_locked_liquidity(
            &env,
            &pool_address,
            &creator,
            params.initial_lower_tick,
            params.initial_upper_tick,
            &locked,
        );
        
        // Register pool in factory
        let pool_info = PoolInfo {
            pool_address: pool_address.clone(),
            token0: token0.clone(),
            token1: token1.clone(),
            creator: creator.clone(),
            fee_bps: params.fee_bps,
            creator_fee_bps,
            tick_spacing,
            created_at: current_ledger,
            is_active: true,
        };
        
        register_pool(&env, &token0, &token1, params.fee_bps, &pool_address, &pool_info);
        update_creator_total_locked(&env, &pool_address, &creator, liquidity);
        
        // Emit events
        emit_pool_created_simple(&env, &pool_address, &token0, &token1, &creator, params.fee_bps);
        emit_liquidity_locked_simple(
            &env,
            &pool_address,
            &creator,
            liquidity,
            params.initial_lower_tick,
            params.initial_upper_tick,
            params.permanent_lock,
        );
        
        Ok(pool_address)
    }
    
    fn _deploy_pool(
        env: &Env,
        wasm_hash: &BytesN<32>,
        token0: &Address,
        token1: &Address,
        fee_bps: u32,
    ) -> Address {
        let salt = Self::_compute_pool_salt(env, token0, token1, fee_bps);
        env.deployer().with_current_contract(salt).deploy(wasm_hash.clone())
    }
    
    fn _compute_pool_salt(
        env: &Env,
        token0: &Address,
        token1: &Address,
        fee_bps: u32,
    ) -> BytesN<32> {
        let mut salt_data = Bytes::new(env);
        
        // Convert addresses to bytes using to_xdr
        let token0_bytes = token0.to_xdr(env);
        let token1_bytes = token1.to_xdr(env);
        
        salt_data.append(&token0_bytes);
        salt_data.append(&token1_bytes);
        
        // Add fee to salt
        let fee_bytes = fee_bps.to_be_bytes();
        for b in fee_bytes {
            salt_data.push_back(b);
        }
        
        env.crypto().sha256(&salt_data).into()
    }
    
    fn _initialize_pool(
        env: &Env,
        pool_address: &Address,
        token0: &Address,
        token1: &Address,
        fee_bps: u32,
        tick_spacing: i32,
        sqrt_price_x64: u128,
        creator_fee_bps: u32,
        creator: &Address,
    ) {
        let factory_address = env.current_contract_address();
        
        env.invoke_contract::<()>(
            pool_address,
            &Symbol::new(env, "initialize"),
            vec![
                env,
                factory_address.into_val(env),
                token0.clone().into_val(env),
                token1.clone().into_val(env),
                fee_bps.into_val(env),
                tick_spacing.into_val(env),
                sqrt_price_x64.into_val(env),
                creator_fee_bps.into_val(env),
                creator.clone().into_val(env),
            ],
        );
    }
    
    fn _transfer_tokens(
        env: &Env,
        from: &Address,
        to: &Address,
        token: &Address,
        amount: i128,
    ) {
        if amount > 0 {
            env.invoke_contract::<()>(
                token,
                &Symbol::new(env, "transfer"),
                vec![
                    env,
                    from.clone().into_val(env),
                    to.clone().into_val(env),
                    amount.into_val(env),
                ],
            );
        }
    }
    
    // ============================================================
    // CREATOR FEE MANAGEMENT
    // ============================================================
    
    pub fn claim_creator_fees(
        env: Env,
        pool_address: Address,
        creator: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<(u128, u128), FactoryError> {
        creator.require_auth();
        
        let pool_info = get_pool_info(&env, &pool_address)
            .ok_or(FactoryError::PoolNotFound)?;
        
        if pool_info.creator != creator {
            return Err(FactoryError::NotPoolCreator);
        }
        
        if !is_creator_fee_eligible(&env, &pool_address, &creator, lower_tick, upper_tick) {
            return Err(FactoryError::CreatorNotEligible);
        }
        
        // Get creator fees info from pool
        let (is_in_range, pending_fees_0, pending_fees_1): (bool, u128, u128) = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "get_creator_fees"),
            vec![
                &env,
                creator.clone().into_val(&env),
                lower_tick.into_val(&env),
                upper_tick.into_val(&env),
            ],
        );
        
        if pending_fees_0 == 0 && pending_fees_1 == 0 {
            return Err(FactoryError::NoFeesToClaim);
        }
        
        // Claim fees from pool
        let (claimed_0, claimed_1): (u128, u128) = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "claim_creator_fees"),
            vec![
                &env,
                creator.clone().into_val(&env),
                lower_tick.into_val(&env),
                upper_tick.into_val(&env),
            ],
        );
        
        emit_creator_fees_claimed_simple(&env, &pool_address, &creator, claimed_0, claimed_1, is_in_range);
        
        Ok((claimed_0, claimed_1))
    }
    
    pub fn get_creator_status(
        env: Env,
        pool_address: Address,
        creator: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<CreatorStatus, FactoryError> {
        let pool_info = get_pool_info(&env, &pool_address)
            .ok_or(FactoryError::PoolNotFound)?;
        
        if pool_info.creator != creator {
            return Err(FactoryError::NotPoolCreator);
        }
        
        let locked = read_locked_liquidity(&env, &pool_address, &creator, lower_tick, upper_tick);
        
        let (is_locked, locked_liquidity) = match &locked {
            Some(l) => (!l.is_unlocked && (l.is_permanent || env.ledger().sequence() < l.lock_end), l.liquidity),
            None => (false, 0),
        };
        
        let (is_in_range, pending_fees_0, pending_fees_1): (bool, u128, u128) = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "get_creator_fees"),
            vec![
                &env,
                creator.clone().into_val(&env),
                lower_tick.into_val(&env),
                upper_tick.into_val(&env),
            ],
        );
        
        let is_eligible = is_locked && is_in_range;
        
        Ok(CreatorStatus {
            pool_address: pool_address.clone(),
            creator: creator.clone(),
            is_in_range,
            is_eligible,
            is_locked,
            locked_liquidity,
            pending_fees_0,
            pending_fees_1,
        })
    }
    
    // ============================================================
    // LIQUIDITY LOCK MANAGEMENT
    // ============================================================
    
    pub fn unlock_liquidity(
        env: Env,
        pool_address: Address,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<i128, FactoryError> {
        owner.require_auth();
        
        let pool_info = get_pool_info(&env, &pool_address)
            .ok_or(FactoryError::PoolNotFound)?;
        
        let mut locked = read_locked_liquidity(&env, &pool_address, &owner, lower_tick, upper_tick)
            .ok_or(FactoryError::PositionNotFound)?;
        
        if locked.owner != owner {
            return Err(FactoryError::NotPositionOwner);
        }
        
        if locked.is_unlocked {
            return Err(FactoryError::PositionNotFound);
        }
        
        if locked.is_permanent {
            return Err(FactoryError::LiquidityLocked);
        }
        
        if !is_lock_expired(&env, &pool_address, &owner, lower_tick, upper_tick) {
            return Err(FactoryError::LiquidityLocked);
        }
        
        locked.is_unlocked = true;
        write_locked_liquidity(&env, &pool_address, &owner, lower_tick, upper_tick, &locked);
        
        update_creator_total_locked(&env, &pool_address, &owner, -locked.liquidity);
        
        if pool_info.creator == owner {
            emit_creator_rights_revoked(&env, &pool_address, &owner, "unlocked");
        }
        
        emit_liquidity_unlocked(&env, &pool_address, &owner, locked.liquidity, lower_tick, upper_tick);
        
        Ok(locked.liquidity)
    }
    
    pub fn extend_lock(
        env: Env,
        pool_address: Address,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
        additional_duration: u32,
    ) -> Result<u32, FactoryError> {
        owner.require_auth();
        
        get_pool_info(&env, &pool_address).ok_or(FactoryError::PoolNotFound)?;
        
        let mut locked = read_locked_liquidity(&env, &pool_address, &owner, lower_tick, upper_tick)
            .ok_or(FactoryError::PositionNotFound)?;
        
        if locked.is_permanent {
            return Err(FactoryError::LiquidityLocked);
        }
        
        if locked.is_unlocked {
            return Err(FactoryError::PositionNotFound);
        }
        
        let current_ledger = env.ledger().sequence();
        let new_lock_end = if locked.lock_end > current_ledger {
            locked.lock_end.saturating_add(additional_duration)
        } else {
            current_ledger.saturating_add(additional_duration)
        };
        
        locked.lock_end = new_lock_end;
        write_locked_liquidity(&env, &pool_address, &owner, lower_tick, upper_tick, &locked);
        
        Ok(new_lock_end)
    }
    
    pub fn make_permanent(
        env: Env,
        pool_address: Address,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<(), FactoryError> {
        owner.require_auth();
        
        get_pool_info(&env, &pool_address).ok_or(FactoryError::PoolNotFound)?;
        
        let mut locked = read_locked_liquidity(&env, &pool_address, &owner, lower_tick, upper_tick)
            .ok_or(FactoryError::PositionNotFound)?;
        
        if locked.is_unlocked {
            return Err(FactoryError::PositionNotFound);
        }
        
        if locked.is_permanent {
            return Ok(());
        }
        
        locked.is_permanent = true;
        locked.lock_end = u32::MAX;
        write_locked_liquidity(&env, &pool_address, &owner, lower_tick, upper_tick, &locked);
        
        emit_liquidity_locked_simple(
            &env,
            &pool_address,
            &owner,
            locked.liquidity,
            lower_tick,
            upper_tick,
            true,
        );
        
        Ok(())
    }
    
    // ============================================================
    // FEE TIER MANAGEMENT (ADMIN ONLY)
    // ============================================================
    
    pub fn set_fee_tier(
        env: Env,
        fee_bps: u32,
        tick_spacing: i32,
        enabled: bool,
    ) -> Result<(), FactoryError> {
        let config = read_factory_config(&env);
        config.admin.require_auth();
        
        if fee_bps > 10000 {
            return Err(FactoryError::InvalidFeeTier);
        }
        if tick_spacing <= 0 {
            return Err(FactoryError::InvalidTickSpacing);
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
    
    pub fn get_fee_tier(env: Env, fee_bps: u32) -> Option<FeeTier> {
        read_fee_tier(&env, fee_bps)
    }
    
    // ============================================================
    // ADMIN FUNCTIONS
    // ============================================================
    
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), FactoryError> {
        let config = read_factory_config(&env);
        config.admin.require_auth();
        
        let old_admin = config.admin.clone();
        
        let new_config = FactoryConfig {
            admin: new_admin.clone(),
            ..config
        };
        
        write_factory_config(&env, &new_config);
        emit_admin_updated(&env, &old_admin, &new_admin);
        
        Ok(())
    }
    
    pub fn set_pool_wasm_hash(env: Env, new_hash: BytesN<32>) -> Result<(), FactoryError> {
        let config = read_factory_config(&env);
        config.admin.require_auth();
        
        let new_config = FactoryConfig {
            pool_wasm_hash: new_hash,
            ..config
        };
        
        write_factory_config(&env, &new_config);
        
        Ok(())
    }
    
    pub fn set_min_initial_liquidity(env: Env, new_min: i128) -> Result<(), FactoryError> {
        let config = read_factory_config(&env);
        config.admin.require_auth();
        
        let new_config = FactoryConfig {
            min_initial_liquidity: new_min,
            ..config
        };
        
        write_factory_config(&env, &new_config);
        
        Ok(())
    }
    
    // ============================================================
    // QUERY FUNCTIONS
    // ============================================================
    
    pub fn get_config(env: Env) -> FactoryConfig {
        read_factory_config(&env)
    }
    
    pub fn get_stats(env: Env) -> FactoryStats {
        read_factory_stats(&env)
    }
    
    pub fn get_pool(env: Env, token_a: Address, token_b: Address, fee_bps: u32) -> Option<Address> {
        get_pool_address(&env, &token_a, &token_b, fee_bps)
    }
    
    pub fn get_pool_info(env: Env, pool_address: Address) -> Option<PoolInfo> {
        get_pool_info(&env, &pool_address)
    }
    
    pub fn get_pool_count(env: Env) -> u32 {
        get_pool_count(&env)
    }
    
    pub fn pool_exists(env: Env, token_a: Address, token_b: Address, fee_bps: u32) -> bool {
        pool_exists(&env, &token_a, &token_b, fee_bps)
    }
    
    pub fn get_locked_liquidity(
        env: Env,
        pool_address: Address,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Option<LockedLiquidity> {
        read_locked_liquidity(&env, &pool_address, &owner, lower_tick, upper_tick)
    }
    
    pub fn is_liquidity_locked(
        env: Env,
        pool_address: Address,
        owner: Address,
        lower_tick: i32,
        upper_tick: i32,
    ) -> bool {
        is_liquidity_locked(&env, &pool_address, &owner, lower_tick, upper_tick)
    }
    
    pub fn compute_pool_address(
        env: Env,
        token_a: Address,
        token_b: Address,
        fee_bps: u32,
    ) -> BytesN<32> {
        let (token0, token1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        
        Self::_compute_pool_salt(&env, &token0, &token1, fee_bps)
    }
}