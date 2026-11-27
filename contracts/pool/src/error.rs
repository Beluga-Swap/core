use belugaswap_swap::SwapError;
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PoolError {
    NotInitialized = 101,

    InvalidFee = 200,
    InvalidTickSpacing = 202,
    InvalidTickRange = 203,
    InvalidTick = 204,

    InvalidToken = 300,

    LiquidityTooLow = 400,
    InsufficientLiquidity = 401,
    InvalidLiquidityAmount = 402,

    SwapAmountTooSmall = 500,
    SlippageExceeded = 501,
    OutputDust = 502,
    NoLiquidity = 503,

    Overflow = 701,
    InvariantViolated = 702,
}

impl From<SwapError> for PoolError {
    fn from(e: SwapError) -> Self {
        match e {
            SwapError::AmountTooSmall => PoolError::SwapAmountTooSmall,
            SwapError::InvalidFee => PoolError::InvalidFee,
            SwapError::FeeInvariant => PoolError::InvariantViolated,
            SwapError::NoLiquidity => PoolError::NoLiquidity,
            SwapError::OutputTooSmall => PoolError::OutputDust,
        }
    }
}
