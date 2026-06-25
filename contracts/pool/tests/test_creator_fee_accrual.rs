mod common;

use belugaswap_pool::{BelugaPool, BelugaPoolClient};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, token::TokenClient, Address, Env};

// Minimal factory stub: creator fee is gated on the factory reporting it active.
#[contract]
pub struct MockFactory;

#[contractimpl]
impl MockFactory {
    pub fn is_creator_fee_active(_env: Env, _pool: Address, _creator: Address) -> bool {
        true
    }
    pub fn is_liquidity_locked(
        _env: Env,
        _pool: Address,
        _creator: Address,
        _lower: i32,
        _upper: i32,
    ) -> bool {
        false
    }
}

// Regression: a swap must credit the creator fee (carved out of the LP fee,
// denominated in the input token) to creator_fees_*, and claim_creator_fees
// must pay it out. Before the fix the engine dropped total_creator_fee and the
// pool never credited it, so claim always returned 0 and the tokens stranded.
#[test]
fn test_creator_fee_accrues_and_is_claimable() {
    let env = Env::default();
    env.mock_all_auths();

    // Pool wired to a real factory stub that reports the creator fee active.
    let factory = env.register(MockFactory, ());
    let router = Address::generate(&env);
    let creator = Address::generate(&env);
    let token_a = common::create_token(&env, &creator);
    let token_b = common::create_token(&env, &creator);

    let pool_id = env.register(BelugaPool, ());
    let client = BelugaPoolClient::new(&env, &pool_id);
    client.initialize(
        &factory, &router, &creator, &token_a, &token_b,
        &30,   // fee_bps
        &1000, // creator_fee_bps (10% of LP fee)
        &common::DEFAULT_SQRT_PRICE_X64,
        &0, &60,
    );

    let state = client.get_pool_state();
    let t0 = state.token0.clone();

    // Honest LP provides liquidity around current price.
    let lp = Address::generate(&env);
    let liq_amt: i128 = 100_000_000_000;
    common::mint_tokens(&env, &token_a, &lp, liq_amt);
    common::mint_tokens(&env, &token_b, &lp, liq_amt);
    client.add_liquidity(&lp, &-600, &600, &liq_amt, &liq_amt, &0, &0);

    // Swapper swaps token0 -> token1 (so any creator fee is in token0).
    let swapper = Address::generate(&env);
    let amt_in: i128 = 10_000_000_000;
    common::mint_tokens(&env, &t0, &swapper, amt_in);
    let res = client.swap(&swapper, &t0, &amt_in, &0, &0);
    std::println!("[PoC] swap in={} out={}", res.amount_in, res.amount_out);

    let cf = client.get_creator_fees();
    std::println!("[PoC] get_creator_fees -> token0={} token1={}", cf.fees_token0, cf.fees_token1);

    let claimed = client.claim_creator_fees();
    let creator_bal = TokenClient::new(&env, &t0).balance(&creator);
    std::println!("[PoC] claim -> ({}, {}); creator t0 balance = {}", claimed.0, claimed.1, creator_bal);

    assert!(cf.fees_token0 > 0, "creator fee charged but never credited");
    assert_eq!(claimed.0, cf.fees_token0, "claim must pay out the recorded creator fee");
    assert_eq!(creator_bal as u128, cf.fees_token0, "creator must actually receive the tokens");
}
