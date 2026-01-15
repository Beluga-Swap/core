// Factory error module for BelugaSwap

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    // Initialization errors (1000-1099)
    AlreadyInitialized = 1000,
    NotInitialized = 1001,
    
    // Pool creation errors (1100-1199)
    PoolAlreadyExists = 1100,
    InvalidTokenPair = 1101,
    InvalidFeeTier = 1102,
    InvalidTickSpacing = 1103,
    InvalidInitialPrice = 1104,
    InvalidTickRange = 1105,
    PoolDeploymentFailed = 1106,
    
    // Liquidity errors (1200-1299)
    InsufficientInitialLiquidity = 1200,
    LiquidityLocked = 1201,
    LockDurationTooShort = 1202,
    PositionNotFound = 1203,
    InsufficientLiquidity = 1204,
    
    // Authorization errors (1300-1399)
    Unauthorized = 1300,
    NotPoolCreator = 1301,
    NotPositionOwner = 1302,
    
    // Fee errors (1400-1499)
    NoFeesToClaim = 1400,
    CreatorNotEligible = 1401,
    CreatorUnlocked = 1402,
    
    // Pool state errors (1500-1599)
    PoolNotFound = 1500,
    PoolNotActive = 1501,
    
    // Position errors (1600-1699)
    PositionOutOfRange = 1600,
}

/// Human-readable error messages for debugging
pub struct FactoryErrorMsg;

impl FactoryErrorMsg {
    // Initialization
    pub const ALREADY_INITIALIZED: &'static str = "Factory: already initialized";
    pub const NOT_INITIALIZED: &'static str = "Factory: not initialized";
    
    // Pool creation
    pub const POOL_EXISTS: &'static str = "Factory: pool already exists for this pair and fee tier";
    pub const INVALID_TOKEN_PAIR: &'static str = "Factory: token addresses must be different";
    pub const INVALID_FEE_TIER: &'static str = "Factory: fee tier not enabled or invalid";
    pub const INVALID_TICK_SPACING: &'static str = "Factory: ticks must align with tick spacing";
    pub const INVALID_TICK_RANGE: &'static str = "Factory: lower tick must be less than upper tick";
    pub const INVALID_INITIAL_PRICE: &'static str = "Factory: initial price must be positive";
    pub const POOL_DEPLOYMENT_FAILED: &'static str = "Factory: pool contract deployment failed";
    
    // Liquidity
    pub const INSUFFICIENT_INITIAL_LIQUIDITY: &'static str = "Factory: initial liquidity below minimum required";
    pub const LIQUIDITY_LOCKED: &'static str = "Factory: liquidity is still locked";
    pub const LOCK_DURATION_TOO_SHORT: &'static str = "Factory: lock duration below minimum";
    pub const POSITION_NOT_FOUND: &'static str = "Factory: position not found or already unlocked";
    pub const INSUFFICIENT_LIQUIDITY: &'static str = "Factory: insufficient liquidity";
    
    // Authorization
    pub const UNAUTHORIZED: &'static str = "Factory: caller not authorized";
    pub const NOT_POOL_CREATOR: &'static str = "Factory: caller is not the pool creator";
    pub const NOT_POSITION_OWNER: &'static str = "Factory: caller is not the position owner";
    
    // Fees
    pub const NO_FEES_TO_CLAIM: &'static str = "Factory: no fees available to claim";
    pub const CREATOR_NOT_ELIGIBLE: &'static str = "Factory: creator not eligible for fees (liquidity unlocked or out of range)";
    pub const CREATOR_UNLOCKED: &'static str = "Factory: creator unlocked liquidity, no longer eligible for fees";
    
    // Pool state
    pub const POOL_NOT_FOUND: &'static str = "Factory: pool does not exist";
    pub const POOL_NOT_ACTIVE: &'static str = "Factory: pool is not active";
    
    // Position
    pub const POSITION_OUT_OF_RANGE: &'static str = "Factory: position is out of current price range";
}