use soroban_sdk::{Env, contracttype, Address};
use crate::DataKey;

// ============================================================
// POSITION DATA STRUCTURE
// ============================================================

#[contracttype]
#[derive(Clone, Debug)]
pub struct Position {
    pub liquidity: i128,

    // Fee tracking checkpoints (Q64.64 format)
    // These store the fee_growth_inside value when position was last modified
    pub fee_growth_inside_last_0: u128,
    pub fee_growth_inside_last_1: u128,

    // Accumulated fees ready to be collected
    pub tokens_owed_0: u128,
    pub tokens_owed_1: u128,
    pub last_update_timestamp: u64,

}

// ============================================================
// STORAGE HELPERS
// ============================================================

pub fn read_position(env: &Env, owner: &Address, lower: i32, upper: i32) -> Position {
    env.storage()
        .persistent()
        .get::<_, Position>(&DataKey::Position(owner.clone(), lower, upper))
        .unwrap_or(Position {
            liquidity: 0,
            fee_growth_inside_last_0: 0,
            fee_growth_inside_last_1: 0,
            tokens_owed_0: 0,
            tokens_owed_1: 0,
            last_update_timestamp: 0,
        })
}

pub fn write_position(
    env: &Env,
    owner: &Address,
    lower: i32,
    upper: i32,
    pos: &Position,
) {
    // Only delete if completely empty (no liquidity and no pending fees)
    if pos.liquidity == 0 && pos.tokens_owed_0 == 0 && pos.tokens_owed_1 == 0 {
        env.storage()
            .persistent()
            .remove(&DataKey::Position(owner.clone(), lower, upper));
    } else {
        env.storage()
            .persistent()
            .set::<_, Position>(
                &DataKey::Position(owner.clone(), lower, upper),
                pos,
            );
    }
}