// Tick Types

use soroban_sdk::contracttype;

/// Information stored for each initialized tick
#[contracttype]
#[derive(Clone, Debug)]
pub struct TickInfo {
    /// Total liquidity referencing this tick
    pub liquidity_gross: i128,
    /// Net liquidity change when crossing left-to-right
    pub liquidity_net: i128,
    /// Fee growth outside this tick for token0
    pub fee_growth_outside_0: u128,
    /// Fee growth outside this tick for token1
    pub fee_growth_outside_1: u128,
    /// Whether this tick is initialized
    pub initialized: bool,
}

impl Default for TickInfo {
    fn default() -> Self {
        Self {
            liquidity_gross: 0,
            liquidity_net: 0,
            fee_growth_outside_0: 0,
            fee_growth_outside_1: 0,
            initialized: false,
        }
    }
}
