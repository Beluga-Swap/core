// Compatible with OpenZeppelin Stellar Soroban Contracts patterns
//
// Error handling module following OpenZeppelin conventions:
// - Uses contracterror derive macro for typed errors (recommended by OZ)
// - Provides both error codes and symbols for different use cases
// - Separates validation errors from runtime errors

use soroban_sdk::{contracterror, symbol_short, Symbol};

// ============================================================
// CONTRACT ERRORS (OpenZeppelin Style)
// ============================================================
// Using contracterror derive macro for typed error handling
// This is the recommended pattern in OpenZeppelin contracts

/// Contract-level errors following OpenZeppelin contracterror pattern
/// These errors are returned from contract functions and can be caught by callers
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BelugaError {
    // Initialization errors (100-199)
    /// Pool has already been initialized
    AlreadyInitialized = 100,
    /// Pool has not been initialized
    NotInitialized = 101,

    // Configuration errors (200-299)
    /// Invalid fee: must be 1-10000 bps
    InvalidFee = 200,
    /// Invalid creator fee: must be 1-1000 bps
    InvalidCreatorFee = 201,
    /// Invalid tick spacing: must be positive
    InvalidTickSpacing = 202,
    /// Invalid tick range: lower must be < upper
    InvalidTickRange = 203,
    /// Tick out of valid range
    InvalidTick = 204,

    // Token errors (300-399)
    /// Invalid token for this pool
    InvalidToken = 300,
    /// Input and output tokens are the same
    SameToken = 301,

    // Liquidity errors (400-499)
    /// Liquidity amount too low (minimum required)
    LiquidityTooLow = 400,
    /// Insufficient liquidity in position
    InsufficientLiquidity = 401,
    /// Liquidity amount must be positive
    InvalidLiquidityAmount = 402,

    // Swap errors (500-599)
    /// Swap amount too small
    SwapAmountTooSmall = 500,
    /// Slippage tolerance exceeded
    SlippageExceeded = 501,
    /// Output amount too small (dust)
    OutputDust = 502,
    /// No liquidity available for swap
    NoLiquidity = 503,
    /// Maximum slippage exceeded
    MaxSlippageExceeded = 504,

    // Authorization errors (600-699)
    /// Unauthorized: only pool creator can perform this action
    Unauthorized = 600,

    // Math errors (700-799)
    /// Division by zero
    DivisionByZero = 700,
    /// Arithmetic overflow
    Overflow = 701,
}

// ============================================================
// ERROR SYMBOLS (For PreviewResult and swap validation)
// ============================================================
// Symbols are used for lightweight error indication in preview/validation
// following Soroban conventions for Symbol-based error messaging

pub struct ErrorSymbol;

impl ErrorSymbol {
    /// Amount too low
    #[inline]
    pub fn amt_low() -> Symbol {
        symbol_short!("AMT_LOW")
    }

    /// No liquidity
    #[inline]
    pub fn no_liq() -> Symbol {
        symbol_short!("NO_LIQ")
    }

    /// Slippage too high
    #[inline]
    pub fn slip_hi() -> Symbol {
        symbol_short!("SLIP_HI")
    }

    /// Output dust (too small)
    #[inline]
    pub fn out_dust() -> Symbol {
        symbol_short!("OUT_DUST")
    }

    /// Max slippage exceeded
    #[inline]
    pub fn slip_max() -> Symbol {
        symbol_short!("SLIP_MAX")
    }

    /// Invalid token
    #[inline]
    pub fn bad_token() -> Symbol {
        symbol_short!("BAD_TKN")
    }

    /// Same token for input and output
    #[inline]
    pub fn same_token() -> Symbol {
        symbol_short!("SAME_TKN")
    }
}

// ============================================================
// ERROR MESSAGES (Legacy - for panic messages)
// Kept for backward compatibility but prefer BelugaError
// ============================================================

pub struct ErrorMsg;

impl ErrorMsg {
    pub const ALREADY_INITIALIZED: &'static str = "pool already initialized";
    pub const INVALID_FEE: &'static str = "invalid fee: must be 1-10000 bps";
    pub const INVALID_CREATOR_FEE: &'static str = "invalid creator fee: must be 1-1000 bps";
    pub const INVALID_TICK_SPACING: &'static str = "invalid tick spacing: must be positive";
    pub const INVALID_TICK_RANGE: &'static str = "invalid tick range: lower must be < upper";
    pub const INVALID_TOKEN: &'static str = "invalid token for this pool";
    pub const SAME_TOKEN: &'static str = "input and output tokens are the same";
    pub const SLIPPAGE_EXCEEDED: &'static str = "slippage tolerance exceeded";
    pub const LIQUIDITY_TOO_LOW: &'static str = "liquidity amount too low";
    pub const INSUFFICIENT_LIQUIDITY: &'static str = "insufficient liquidity in position";
    pub const INVALID_LIQUIDITY_AMOUNT: &'static str = "liquidity amount must be positive";
    pub const UNAUTHORIZED: &'static str = "unauthorized: only pool creator can perform this action";
}

// ============================================================
// ERROR CONVERSION HELPERS
// ============================================================

impl BelugaError {
    /// Convert error to Symbol for use in PreviewResult
    pub fn to_symbol(&self) -> Symbol {
        match self {
            BelugaError::SwapAmountTooSmall => ErrorSymbol::amt_low(),
            BelugaError::NoLiquidity => ErrorSymbol::no_liq(),
            BelugaError::SlippageExceeded => ErrorSymbol::slip_hi(),
            BelugaError::OutputDust => ErrorSymbol::out_dust(),
            BelugaError::MaxSlippageExceeded => ErrorSymbol::slip_max(),
            BelugaError::InvalidToken => ErrorSymbol::bad_token(),
            BelugaError::SameToken => ErrorSymbol::same_token(),
            _ => symbol_short!("ERROR"),
        }
    }
}