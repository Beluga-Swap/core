use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug)]
#[derive(Default)]
pub struct Position {
    pub liquidity: i128,
    pub fee_growth_inside_last_0: u128,
    pub fee_growth_inside_last_1: u128,
    pub tokens_owed_0: u128,
    pub tokens_owed_1: u128,
}


#[contracttype]
#[derive(Clone, Debug)]
#[derive(Default)]
pub struct PositionInfo {
    pub liquidity: i128,
    pub amount0: i128,
    pub amount1: i128,
    pub fees_owed_0: u128,
    pub fees_owed_1: u128,
}

