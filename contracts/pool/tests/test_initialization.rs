mod common;

use belugaswap_math::{constants::MAX_TICK, get_sqrt_ratio_at_tick};
use belugaswap_pool::BelugaPoolClient;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Deploy with default parameters, overriding fee, creator fee, price and spacing.
fn deploy(env: &Env, fee: u32, creator_fee: u32, sqrt_price: u128, spacing: i32) -> BelugaPoolClient<'_> {
    env.mock_all_auths();
    let creator = Address::generate(env);
    let token_a = common::create_token(env, &creator);
    let token_b = common::create_token(env, &creator);
    common::deploy_pool(
        env,
        &Address::generate(env),
        &Address::generate(env),
        &creator,
        &token_a,
        &token_b,
        &fee,
        &creator_fee,
        &sqrt_price,
        &spacing,
    )
}

fn deploy_with_defaults(env: &Env) -> BelugaPoolClient<'_> {
    deploy(
        env,
        common::DEFAULT_FEE_BPS,
        common::DEFAULT_CREATOR_FEE_BPS,
        common::DEFAULT_SQRT_PRICE_X64,
        common::DEFAULT_TICK_SPACING,
    )
}

#[test]
fn test_initialization_success() {
    let env = Env::default();
    let client = deploy_with_defaults(&env);

    let state = client.get_pool_state();
    assert_eq!(state.sqrt_price_x64, common::DEFAULT_SQRT_PRICE_X64);
    assert_eq!(state.current_tick, common::DEFAULT_TICK);
    assert_eq!(state.liquidity, 0);
    assert_eq!(state.tick_spacing, common::DEFAULT_TICK_SPACING);

    let config = client.get_pool_config();
    assert_eq!(config.fee_bps, common::DEFAULT_FEE_BPS);
    assert_eq!(config.creator_fee_bps, common::DEFAULT_CREATOR_FEE_BPS);
    assert!(state.token0 < state.token1);
}

#[test]
fn test_tick_is_derived_from_price() {
    for tick in [-200_040, -61, -1, 1, 13_920, 200_040] {
        let env = Env::default();
        let client = deploy(&env, 30, 100, get_sqrt_ratio_at_tick(tick), 60);
        assert_eq!(client.get_pool_state().current_tick, tick);
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_invalid_fee_zero() {
    let env = Env::default();
    deploy(&env, 0, 100, common::DEFAULT_SQRT_PRICE_X64, 60);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_invalid_fee_too_high() {
    let env = Env::default();
    deploy(&env, 10_001, 100, common::DEFAULT_SQRT_PRICE_X64, 60);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_invalid_creator_fee_too_low() {
    let env = Env::default();
    deploy(&env, 30, 0, common::DEFAULT_SQRT_PRICE_X64, 60);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_invalid_creator_fee_too_high() {
    let env = Env::default();
    deploy(&env, 30, 1_001, common::DEFAULT_SQRT_PRICE_X64, 60);
}

#[test]
#[should_panic(expected = "Error(Contract, #202)")]
fn test_invalid_tick_spacing_zero() {
    let env = Env::default();
    deploy(&env, 30, 100, common::DEFAULT_SQRT_PRICE_X64, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #202)")]
fn test_invalid_tick_spacing_negative() {
    let env = Env::default();
    deploy(&env, 30, 100, common::DEFAULT_SQRT_PRICE_X64, -10);
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_price_out_of_range() {
    let env = Env::default();
    deploy(&env, 30, 100, get_sqrt_ratio_at_tick(MAX_TICK) + 1, 60);
}
