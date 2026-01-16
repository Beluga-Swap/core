//! Factory error types

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FactoryError {
    // Initialization
    AlreadyInitialized = 1,
    NotInitialized = 2,
    
    // Pool creation
    PoolAlreadyExists = 10,
    InvalidTokenPair = 11,
    InvalidFeeTier = 12,
    InvalidTickSpacing = 13,
    InvalidTickRange = 14,
    InvalidInitialPrice = 15,
    InvalidCreatorFee = 16,
    
    // Liquidity
    InsufficientInitialLiquidity = 20,
    InvalidLockDuration = 21,
    LiquidityStillLocked = 22,
    
    // Creator
    NotPoolCreator = 30,
    CreatorFeeRevoked = 31,
    CreatorLockNotFound = 32,
    
    // Admin
    Unauthorized = 50,
}