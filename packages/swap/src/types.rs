use soroban_sdk::{contracttype, Symbol};

#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub current_tick: i32,
    pub sqrt_price_x64: u128,
}

impl Default for SwapResult {
    fn default() -> Self {
        Self {
            amount_in: 0,
            amount_out: 0,
            current_tick: 0,
            sqrt_price_x64: 0,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PreviewResult {
    pub amount_in_used: i128,
    pub amount_out_expected: i128,
    pub fee_paid: i128,
    pub price_impact_bps: i128,
    pub is_valid: bool,
    pub error_message: Option<Symbol>,
}

impl Default for PreviewResult {
    fn default() -> Self {
        Self {
            amount_in_used: 0,
            amount_out_expected: 0,
            fee_paid: 0,
            price_impact_bps: 0,
            is_valid: false,
            error_message: None,
        }
    }
}

impl PreviewResult {
    pub fn valid(
        amount_in_used: i128,
        amount_out_expected: i128,
        fee_paid: i128,
        price_impact_bps: i128,
    ) -> Self {
        Self {
            amount_in_used,
            amount_out_expected,
            fee_paid,
            price_impact_bps,
            is_valid: true,
            error_message: None,
        }
    }

    pub fn invalid(error: Symbol) -> Self {
        Self {
            amount_in_used: 0,
            amount_out_expected: 0,
            fee_paid: 0,
            price_impact_bps: 0,
            is_valid: false,
            error_message: Some(error),
        }
    }
}