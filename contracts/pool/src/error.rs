pub struct ErrorMsg;

impl ErrorMsg {
    pub const ALREADY_INITIALIZED: &'static str = "pool already initialized";
    pub const INVALID_FEE: &'static str = "invalid fee: must be 1-10000 bps";
    pub const INVALID_CREATOR_FEE: &'static str = "invalid creator fee: must be 1-1000 bps";
    pub const INVALID_TICK_SPACING: &'static str = "invalid tick spacing: must be positive";
    pub const INVALID_TICK_RANGE: &'static str = "invalid tick range: lower must be < upper";
    pub const INVALID_TOKEN: &'static str = "invalid token for this pool";
    pub const SLIPPAGE_EXCEEDED: &'static str = "slippage tolerance exceeded";
    pub const LIQUIDITY_TOO_LOW: &'static str = "liquidity amount too low";
    pub const INSUFFICIENT_LIQUIDITY: &'static str = "insufficient liquidity in position";
    pub const INVALID_LIQUIDITY_AMOUNT: &'static str = "liquidity amount must be positive";
}