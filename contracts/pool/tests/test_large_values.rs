//! Amounts and tick ranges whose intermediate products exceed 128 bits.

mod common;

use belugaswap_math::get_sqrt_ratio_at_tick;
use belugaswap_pool::BelugaPoolClient;
use soroban_sdk::{
    testutils::Address as _, token::TokenClient, Address, Env,
};

struct Pool<'a> {
    env: &'a Env,
    c: BelugaPoolClient<'a>,
    t0: Address,
    t1: Address,
}

impl<'a> Pool<'a> {
    fn new(env: &'a Env, tick: i32) -> Self {
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited();
        let factory = Address::generate(env);
        let creator = Address::generate(env);
        let a = common::create_token(env, &creator);
        let b = common::create_token(env, &creator);
        let c = common::deploy_pool(env, &factory, &Address::generate(env), &creator, &a, &b, &30, &get_sqrt_ratio_at_tick(tick), &60);
        let s = c.get_pool_state();
        Pool { env, c, t0: s.token0, t1: s.token1 }
    }

    fn funded(&self, amount: i128) -> Address {
        let u = Address::generate(self.env);
        common::mint_tokens(self.env, &self.t0, &u, amount);
        common::mint_tokens(self.env, &self.t1, &u, amount);
        u
    }

    fn balances(&self, who: &Address) -> (i128, i128) {
        (
            TokenClient::new(self.env, &self.t0).balance(who),
            TokenClient::new(self.env, &self.t1).balance(who),
        )
    }
}

/// Swap there and back, then the LP leaves. The trader must not gain.
fn round_trip(tick: i32, lower: i32, upper: i32, amount: i128, size: i128) {
    let env = Env::default();
    let p = Pool::new(&env, tick);
    let lp = p.funded(amount * 4);
    let (l, a0, a1) = p.c.add_liquidity(&lp, &lower, &upper, &amount, &amount, &0, &0);
    assert!(a0 > 0 || a1 > 0, "position cost nothing: L={l}");

    for zero_for_one in [true, false] {
        let trader = p.funded(amount * 4);
        let before = p.balances(&trader);
        let (tin, tout) = if zero_for_one { (&p.t0, &p.t1) } else { (&p.t1, &p.t0) };
        env.cost_estimate().budget().reset_unlimited();
        let there = p.c.swap(&trader, tin, &size, &0, &0);
        env.cost_estimate().budget().reset_unlimited();
        p.c.swap(&trader, tout, &there.amount_out, &0, &0);
        let after = p.balances(&trader);
        // Value both deltas in token1 at the starting price. Dust left over in
        // one token is fine as long as the trade as a whole lost money.
        let price = 1.0001f64.powi(tick);
        let value = (after.0 - before.0) as f64 * price + (after.1 - before.1) as f64;
        assert!(
            value <= 0.0,
            "round trip profit at 1e{} tick {tick}: {:?} -> {:?}",
            amount.ilog10(), before, after
        );
    }

    env.cost_estimate().budget().reset_unlimited();
    let (back0, back1) = p.c.remove_liquidity(&lp, &lower, &upper, &l, &0, &0);
    p.c.collect_fees(&lp, &lower, &upper);
    assert!(back0 > 0 || back1 > 0);
}

#[test]
fn swaps_work_with_large_liquidity() {
    for exp in [7u32, 12, 16, 17, 19, 22, 25] {
        let amount = 10i128.pow(exp);
        round_trip(0, -60, 60, amount, amount / 10);
    }
}

#[test]
fn full_range_positions_work() {
    for amount in [1_000_000_000i128, 10i128.pow(18)] {
        round_trip(0, -887_220, 887_220, amount, amount / 10);
    }
}

#[test]
fn full_range_is_priced_like_x_times_y() {
    let env = Env::default();
    let p = Pool::new(&env, 0);
    let lp = p.funded(2_000_000_000);
    let (l, a0, a1) = p.c.add_liquidity(&lp, &-887_220, &887_220, &1_000_000_000, &1_000_000_000, &0, &0);

    // At price 1 a full range position holds L of each token.
    for v in [l, a0, a1] {
        assert!((v - 1_000_000_000).abs() < 1_000, "L={l} a0={a0} a1={a1}");
    }
}

#[test]
fn swaps_work_far_from_par() {
    // Price up to ~5e8 either way. Much further out, 1e9 units of the cheap
    // token are worth less than one unit of the other and rightly revert.
    for tick in [-200_040, -100_020, 100_020, 200_040] {
        round_trip(tick, tick - 600, tick + 600, 10i128.pow(14), 1_000_000_000);
    }
}
