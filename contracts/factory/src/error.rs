// Factory error module for BelugaSwap

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    AlreadyInitialized = 1000,
    NotInitialized = 1001,
    PoolAlreadyExists = 1100,
    InvalidTokenPair = 1101,
    InvalidFeeTier = 1102,
    InvalidTickSpacing = 1103,
    InvalidInitialPrice = 1104,
    InvalidTickRange = 1105,
    InsufficientInitialLiquidity = 1200,
    LiquidityLocked = 1201,
    LockDurationTooShort = 1202,
    PositionNotFound = 1203,
    InsufficientLiquidity = 1204,
    Unauthorized = 1300,
    NotPoolCreator = 1301,
    NotPositionOwner = 1302,
    NoFeesToClaim = 1400,
    CreatorNotEligible = 1401,
    CreatorUnlocked = 1402,
    PoolNotFound = 1500,
    PoolNotActive = 1501,
    PoolDeploymentFailed = 1502,
    PositionOutOfRange = 1600,
}

pub struct FactoryErrorMsg;

impl FactoryErrorMsg {
    pub const ALREADY_INITIALIZED: &'static str = "factory already initialized";
    pub const NOT_INITIALIZED: &'static str = "factory not initialized";
    pub const POOL_EXISTS: &'static str = "pool already exists";
    pub const INVALID_TOKEN_PAIR: &'static str = "invalid token pair";
    pub const INVALID_FEE_TIER: &'static str = "invalid fee tier";
    pub const INSUFFICIENT_INITIAL_LIQUIDITY: &'static str = "initial liquidity below minimum";
    pub const LIQUIDITY_LOCKED: &'static str = "liquidity is locked";
    pub const UNAUTHORIZED: &'static str = "unauthorized";
    pub const NOT_POOL_CREATOR: &'static str = "not pool creator";
    pub const CREATOR_NOT_ELIGIBLE: &'static str = "creator not eligible";
    pub const CREATOR_UNLOCKED: &'static str = "creator unlocked - no fee rights";
    pub const POOL_NOT_FOUND: &'static str = "pool not found";
    pub const LOCK_DURATION_TOO_SHORT: &'static str = "lock duration too short";
}