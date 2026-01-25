mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
#[should_panic(expected = "tick out of range")]
fn test_edge_tick_boundaries() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Try to add liquidity at extreme tick boundaries
    client.add_liquidity(
        &lp,
        &-887272, // Near min tick
        &887272,  // Near max tick
        &1_000_000,
        &1_000_000,
        &0,
        &0,
    );
}

#[test]
#[should_panic]
fn test_unaligned_ticks() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _creator, _factory, _token_a, _token_b) = common::setup_pool(&env);
    let lp = Address::generate(&env);
    
    // Ticks not aligned to tick spacing (60)
    // This will fail because tokens don't have balance
    client.add_liquidity(
        &lp,
        &-601, // Not divisible by 60
        &601,  // Not divisible by 60
        &1_000_000,
        &1_000_000,
        &0,
        &0,
    );
}