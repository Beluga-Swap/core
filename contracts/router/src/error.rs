//! Router error types

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RouterError {
    // Initialization
    AlreadyInitialized = 1,
    NotInitialized = 2,
    
    // Swap errors
    InvalidPath = 10,
    PathTooLong = 11,
    NoPoolsFound = 12,
    InsufficientOutput = 13,
    SlippageExceeded = 14,
    DeadlineExpired = 15,
    
    // Pool errors
    PoolNotFound = 20,
    InvalidTokenPair = 21,
    NoLiquidityAvailable = 22,
    
    // Quote errors
    QuoteFailed = 30,
    InvalidAmount = 31,
    
    // Split errors
    EmptySplits = 40,
    SplitAmountMismatch = 41,
    
    // Authorization
    Unauthorized = 50,
}