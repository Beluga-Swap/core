// Compatible with OpenZeppelin Stellar Soroban Contracts patterns
//
// Events module following OpenZeppelin conventions:
// - Clear event structure with typed data
// - Consistent naming convention
// - Event topics for indexing

use soroban_sdk::{contracttype, Env, Symbol, Address};

// ============================================================
// EVENT TOPICS (OpenZeppelin Style)
// ============================================================
// Using Symbol for event topics, following Soroban best practices

/// Event topic constants
pub struct EventTopic;

impl EventTopic {
    /// Pool initialized event topic
    #[inline]
    pub fn initialized(env: &Env) -> Symbol {
        Symbol::new(env, "Initialized")
    }

    /// Pool init (with price info) event topic
    #[inline]
    pub fn pool_init(env: &Env) -> Symbol {
        Symbol::new(env, "PoolInit")
    }

    /// Liquidity added event topic
    #[inline]
    pub fn liquidity_added(env: &Env) -> Symbol {
        Symbol::new(env, "LiquidityAdded")
    }

    /// Liquidity removed event topic
    #[inline]
    pub fn liquidity_removed(env: &Env) -> Symbol {
        Symbol::new(env, "LiquidityRemoved")
    }

    /// Swap executed event topic
    #[inline]
    pub fn swap(env: &Env) -> Symbol {
        Symbol::new(env, "Swap")
    }

    /// Fees collected event topic
    #[inline]
    pub fn fees_collected(env: &Env) -> Symbol {
        Symbol::new(env, "FeesCollected")
    }

    /// Tick synced event topic (for debugging/indexing)
    #[inline]
    pub fn tick_synced(env: &Env) -> Symbol {
        Symbol::new(env, "TickSynced")
    }

    /// Creator fees claimed event topic
    #[inline]
    pub fn creator_fees_claimed(env: &Env) -> Symbol {
        Symbol::new(env, "CreatorFeesClaimed")
    }
}

// ============================================================
// EVENT DATA STRUCTURES (OpenZeppelin Style)
// ============================================================
// Typed event data structures for better documentation and type safety

/// Data for Initialized event
#[contracttype]
#[derive(Clone, Debug)]
pub struct InitializedEventData {
    pub fee_bps: u32,
    pub creator_fee_bps: u32,
    pub tick_spacing: i32,
}

/// Data for PoolInit event
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolInitEventData {
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub tick_spacing: i32,
}

/// Data for LiquidityAdded event
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityAddedEventData {
    pub owner: Address,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub liquidity: i128,
    pub amount0: i128,
    pub amount1: i128,
}

/// Data for LiquidityRemoved event
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityRemovedEventData {
    pub owner: Address,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub liquidity: i128,
    pub amount0: i128,
    pub amount1: i128,
}

/// Data for Swap event
#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapEventData {
    pub sender: Address,
    pub amount_in: i128,
    pub amount_out: i128,
    pub zero_for_one: bool,
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
}

/// Data for FeesCollected event
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeesCollectedEventData {
    pub owner: Address,
    pub amount0: u128,
    pub amount1: u128,
}

/// Data for CreatorFeesClaimed event
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatorFeesClaimedEventData {
    pub creator: Address,
    pub amount0: u128,
    pub amount1: u128,
}

// ============================================================
// EVENT EMITTERS (OpenZeppelin Style)
// ============================================================
// Functions to emit events with proper structure

/// Emit pool initialized event
pub fn emit_initialized(env: &Env, fee_bps: u32, creator_fee_bps: u32, tick_spacing: i32) {
    env.events().publish(
        (EventTopic::initialized(env),),
        InitializedEventData {
            fee_bps,
            creator_fee_bps,
            tick_spacing,
        },
    );
}

/// Emit pool init event (with price info)
pub fn emit_pool_init(env: &Env, sqrt_price_x64: u128, current_tick: i32, tick_spacing: i32) {
    env.events().publish(
        (EventTopic::pool_init(env),),
        PoolInitEventData {
            sqrt_price_x64,
            current_tick,
            tick_spacing,
        },
    );
}

/// Emit add liquidity event (simple version for backward compatibility)
pub fn emit_add_liquidity(env: &Env, liquidity: i128, amount0: i128, amount1: i128) {
    env.events().publish(
        (EventTopic::liquidity_added(env),),
        (liquidity, amount0, amount1),
    );
}

/// Emit remove liquidity event (simple version for backward compatibility)
pub fn emit_remove_liquidity(env: &Env, liquidity: i128, amount0: i128, amount1: i128) {
    env.events().publish(
        (EventTopic::liquidity_removed(env),),
        (liquidity, amount0, amount1),
    );
}

/// Emit swap event (simple version for backward compatibility)
pub fn emit_swap(env: &Env, amount_in: i128, amount_out: i128, zero_for_one: bool) {
    env.events().publish(
        (EventTopic::swap(env),),
        (amount_in, amount_out, zero_for_one),
    );
}

/// Emit collect fees event (simple version for backward compatibility)
pub fn emit_collect(env: &Env, amount0: u128, amount1: u128) {
    env.events().publish(
        (EventTopic::fees_collected(env),),
        (amount0, amount1),
    );
}

/// Emit tick sync event (for debugging/indexing)
pub fn emit_sync_tick(env: &Env, tick: i32, sqrt_price_x64: u128) {
    env.events().publish(
        (EventTopic::tick_synced(env),),
        (tick, sqrt_price_x64),
    );
}

/// Emit claim creator fees event (simple version for backward compatibility)
pub fn emit_claim_creator_fees(env: &Env, amount0: u128, amount1: u128) {
    env.events().publish(
        (EventTopic::creator_fees_claimed(env),),
        (amount0, amount1),
    );
}