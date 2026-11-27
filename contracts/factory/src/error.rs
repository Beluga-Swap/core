//! Factory error types

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FactoryError {
    // Initialization
    
    // Router
    RouterNotSet = 5,
    RouterAlreadySet = 6,
    
    // Pool creation
    PoolAlreadyExists = 10,
    InvalidTokenPair = 11,
    InvalidFeeTier = 12,
    InvalidTickSpacing = 13,
    InvalidTickRange = 14,
    InvalidInitialPrice = 15,
    
    // Liquidity
    InsufficientInitialLiquidity = 20,
    InvalidLockDuration = 21,
    LiquidityStillLocked = 22,
    
    // Creator
    NotPoolCreator = 30,
    /// Creator lock was already released
    AlreadyUnlocked = 31,
    CreatorLockNotFound = 32,
    
    // Admin
    Unauthorized = 50,
}