//! Router type definitions

use soroban_sdk::{contracttype, Address, Vec};

// ============================================================
// ROUTER CONFIG
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct RouterConfig {
    /// Factory contract address
    pub factory: Address,
    /// Admin address
    pub admin: Address,
}

// ============================================================
// SWAP PARAMS
// ============================================================

/// Parameters for exact input swap
#[contracttype]
#[derive(Clone, Debug)]
pub struct ExactInputParams {
    /// Token to swap from
    pub token_in: Address,
    /// Token to swap to
    pub token_out: Address,
    /// Amount of token_in
    pub amount_in: i128,
    /// Minimum amount of token_out (slippage protection)
    pub amount_out_min: i128,
    /// Fee tiers to try (e.g., [5, 30, 100] for 0.05%, 0.3%, 1%)
    pub fee_tiers: Vec<u32>,
    /// Recipient address
    pub recipient: Address,
    /// Deadline (ledger sequence)
    pub deadline: u32,
}

/// Parameters for exact output swap
#[contracttype]
#[derive(Clone, Debug)]
pub struct ExactOutputParams {
    /// Token to swap from
    pub token_in: Address,
    /// Token to swap to
    pub token_out: Address,
    /// Exact amount of token_out desired
    pub amount_out: i128,
    /// Maximum amount of token_in willing to spend
    pub amount_in_max: i128,
    /// Fee tiers to try
    pub fee_tiers: Vec<u32>,
    /// Recipient address
    pub recipient: Address,
    /// Deadline (ledger sequence)
    pub deadline: u32,
}

/// Single hop in a multi-hop path
#[contracttype]
#[derive(Clone, Debug)]
pub struct Hop {
    /// Token address for this hop
    pub token: Address,
    /// Fee tier for the pool to use
    pub fee_bps: u32,
}

/// Parameters for multi-hop exact input swap
#[contracttype]
#[derive(Clone, Debug)]
pub struct MultihopExactInputParams {
    /// Token to start with
    pub token_in: Address,
    /// Amount of token_in
    pub amount_in: i128,
    /// Path of hops (intermediate tokens + fees)
    /// Final element's token is the output token
    pub path: Vec<Hop>,
    /// Minimum final output
    pub amount_out_min: i128,
    /// Recipient address
    pub recipient: Address,
    /// Deadline (ledger sequence)
    pub deadline: u32,
}

// ============================================================
// QUOTE RESULTS
// ============================================================

/// Quote result for a single pool
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolQuote {
    /// Pool address
    pub pool: Address,
    /// Fee tier in bps
    pub fee_bps: u32,
    /// Expected output amount
    pub amount_out: i128,
    /// Price impact in bps
    pub price_impact_bps: i128,
}

/// Best quote result aggregating multiple pools
#[contracttype]
#[derive(Clone, Debug)]
pub struct BestQuote {
    /// Best pool to use
    pub pool: Address,
    /// Fee tier of best pool
    pub fee_bps: u32,
    /// Best output amount
    pub amount_out: i128,
    /// Price impact in bps
    pub price_impact_bps: i128,
    /// All pool quotes for comparison
    pub all_quotes: Vec<PoolQuote>,
}

/// Split quote for splitting across multiple pools
#[contracttype]
#[derive(Clone, Debug)]
pub struct SplitQuote {
    /// Pool address
    pub pool: Address,
    /// Fee tier
    pub fee_bps: u32,
    /// Amount to route through this pool
    pub amount_in: i128,
    /// Expected output from this pool
    pub amount_out: i128,
}

/// Aggregated quote result with optional split routing
#[contracttype]
#[derive(Clone, Debug)]
pub struct AggregatedQuote {
    /// Total input amount
    pub total_amount_in: i128,
    /// Total expected output
    pub total_amount_out: i128,
    /// Individual splits
    pub splits: Vec<SplitQuote>,
    /// Whether split routing is recommended
    pub is_split_recommended: bool,
}

// ============================================================
// SWAP RESULT
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapResult {
    /// Actual amount of token_in used
    pub amount_in: i128,
    /// Actual amount of token_out received
    pub amount_out: i128,
    /// Pool(s) used
    pub pools_used: Vec<Address>,
    /// Fee tiers used
    pub fee_tiers_used: Vec<u32>,
}

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum DataKey {
    /// Router config
    Config,
    /// Initialization flag
    Initialized,
}