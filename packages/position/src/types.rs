use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Position {
    pub liquidity: i128,
    pub fee_growth_inside_last_0: u128,
    pub fee_growth_inside_last_1: u128,
    pub tokens_owed_0: u128,
    pub tokens_owed_1: u128,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            liquidity: 0,
            fee_growth_inside_last_0: 0,
            fee_growth_inside_last_1: 0,
            tokens_owed_0: 0,
            tokens_owed_1: 0,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PositionInfo {
    pub liquidity: i128,
    pub amount0: i128,
    pub amount1: i128,
    pub fees_owed_0: u128,
    pub fees_owed_1: u128,
}

impl Default for PositionInfo {
    fn default() -> Self {
        Self {
            liquidity: 0,
            amount0: 0,
            amount1: 0,
            fees_owed_0: 0,
            fees_owed_1: 0,
        }
    }
}
