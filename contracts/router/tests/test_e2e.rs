//! Router against a real pool and real tokens. `mock_all_auths` does not
//! satisfy non-root contract authorizations, matching network behaviour.

use belugaswap_pool::{BelugaPool, BelugaPoolClient};
use belugaswap_router::{
    BelugaRouter, BelugaRouterClient, ExactInputParams, Hop, MultihopExactInputParams, SplitQuote,
};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token::{StellarAssetClient, TokenClient},
    vec, Address, Env, Symbol,
};

#[contract]
pub struct MockFactory;

#[contractimpl]
impl MockFactory {
    pub fn set(env: Env, pool: Address) {
        env.storage().instance().set(&Symbol::new(&env, "pool"), &pool);
    }
    pub fn get_pool_address(env: Env, _a: Address, _b: Address, fee_bps: u32) -> Option<Address> {
        if fee_bps != 30 {
            return None;
        }
        env.storage().instance().get(&Symbol::new(&env, "pool"))
    }
}

struct World<'a> {
    router: BelugaRouterClient<'a>,
    pool: BelugaPoolClient<'a>,
    t0: Address,
    t1: Address,
    user: Address,
}

fn world(env: &Env) -> World<'_> {
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(env);
    let factory = env.register(MockFactory, ());
    let router_id = env.register(BelugaRouter, (&factory, &admin));
    let router = BelugaRouterClient::new(env, &router_id);
    let a = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let b = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let pool_id = env.register(
        BelugaPool,
        (&factory, &router_id, &admin, &a, &b, 30u32, 1u128 << 64, 60i32),
    );
    let pool = BelugaPoolClient::new(env, &pool_id);
    MockFactoryClient::new(env, &factory).set(&pool_id);
    let s = pool.get_pool_state();
    let user = Address::generate(env);
    for t in [&s.token0, &s.token1] {
        StellarAssetClient::new(env, t).mint(&user, &10_000_000_000);
    }
    pool.add_liquidity(&user, &-600, &600, &100_000_000, &100_000_000, &0, &0);
    World { router, pool, t0: s.token0, t1: s.token1, user }
}

#[test]
fn quote_through_router() {
    let env = Env::default();
    let w = world(&env);
    let q = w.router.try_get_best_quote(&w.t0, &w.t1, &1_000_000, &vec![&env]);
    assert!(matches!(q, Ok(Ok(_))), "quote failed: {:?}", q.map(|r| r.map(|_| ())));
}

#[test]
fn exact_input_through_router() {
    let env = Env::default();
    let w = world(&env);
    let r = w.router.try_swap_exact_input(&w.user, &ExactInputParams {
        token_in: w.t0.clone(),
        token_out: w.t1.clone(),
        amount_in: 1_000_000,
        amount_out_min: 1,
        recipient: w.user.clone(),
        deadline: 1000,
        fee_tiers: vec![&env, 30],
    });
    assert!(matches!(r, Ok(Ok(_))), "swap failed: {:?}", r.map(|r| r.map(|_| ())));
}

fn one_hop(env: &Env, w: &World, amount_in: i128) -> bool {
    let r = w.router.try_swap_multihop(&w.user, &MultihopExactInputParams {
        token_in: w.t0.clone(),
        path: vec![env, Hop { token: w.t1.clone(), fee_bps: 30 }],
        amount_in,
        amount_out_min: 1,
        recipient: w.user.clone(),
        deadline: 1000,
    });
    matches!(r, Ok(Ok(_)))
}

#[test]
fn multihop_through_router() {
    let env = Env::default();
    let w = world(&env);
    assert!(one_hop(&env, &w, 1_000_000), "router swap failed");
}

#[test]
fn partial_fill_leaves_nothing_in_router() {
    let env = Env::default();
    let w = world(&env);
    // More than the position can absorb, within the preview's impact cap.
    let before = TokenClient::new(&env, &w.t0).balance(&w.user);
    assert!(one_hop(&env, &w, 150_000_000), "router swap failed");
    let spent = before - TokenClient::new(&env, &w.t0).balance(&w.user);
    assert!(spent < 150_000_000, "expected a partial fill, spent {spent}");
    let stuck = TokenClient::new(&env, &w.t0).balance(&w.router.address);
    assert_eq!(stuck, 0, "input stranded in router");
    let _ = &w.pool;
}

#[test]
fn split_through_router_and_rejects_unregistered_pool() {
    let env = Env::default();
    let w = world(&env);
    let split = |pool: Address| {
        w.router.try_swap_split(
            &w.user, &w.t0, &w.t1, &1_000_000, &1,
            &vec![&env, SplitQuote { pool, fee_bps: 30, amount_in: 1_000_000, amount_out: 0 }],
            &w.user, &1000,
        )
    };
    assert!(matches!(split(w.pool.address.clone()), Ok(Ok(_))), "split failed");
    assert!(split(Address::generate(&env)).is_err(), "unregistered pool accepted");
}
