//! Attempts to break the LP and swap flows. Each test tries one way to take
//! value that is not owed, and asserts it fails.

mod common;

use belugaswap_math::get_sqrt_ratio_at_tick;
use belugaswap_pool::BelugaPoolClient;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token::TokenClient, Address, Env, Symbol,
};

/// Factory stub whose creator lock can be set per test.
#[contract]
pub struct MockFactory;

#[contractimpl]
impl MockFactory {
    pub fn set_lock(env: Env, lower: i32, upper: i32) {
        env.storage().instance().set(&Symbol::new(&env, "lock"), &(lower, upper));
    }
    pub fn is_liquidity_locked(env: Env, _p: Address, _c: Address, lower: i32, upper: i32) -> bool {
        env.storage().instance().get(&Symbol::new(&env, "lock")) == Some((lower, upper))
    }
}

struct Pool<'a> {
    env: &'a Env,
    c: BelugaPoolClient<'a>,
    factory: Address,
    creator: Address,
    t0: Address,
    t1: Address,
}

const FEE_BPS: u32 = 30;

impl<'a> Pool<'a> {
    fn new(env: &'a Env) -> Self {
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited();
        let factory = env.register(MockFactory, ());
        let creator = Address::generate(env);
        let a = common::create_token(env, &creator);
        let b = common::create_token(env, &creator);
        let c = common::deploy_pool(
            env, &factory, &Address::generate(env), &creator, &a, &b,
            &FEE_BPS, &(1u128 << 64), &60,
        );
        let s = c.get_pool_state();
        Pool { env, c, factory, creator, t0: s.token0, t1: s.token1 }
    }

    fn user(&self, amount: i128) -> Address {
        let u = Address::generate(self.env);
        common::mint_tokens(self.env, &self.t0, &u, amount);
        common::mint_tokens(self.env, &self.t1, &u, amount);
        u
    }

    fn bal(&self, who: &Address) -> (i128, i128) {
        (
            TokenClient::new(self.env, &self.t0).balance(who),
            TokenClient::new(self.env, &self.t1).balance(who),
        )
    }

    fn swap(&self, who: &Address, zero_for_one: bool, amount: i128) -> (i128, i128) {
        self.env.cost_estimate().budget().reset_unlimited();
        let t = if zero_for_one { &self.t0 } else { &self.t1 };
        let r = self.c.swap(who, t, &amount, &0, &0);
        (r.amount_in, r.amount_out)
    }

    /// Value of (token0, token1) in token1 at the current price.
    fn value(&self, (a0, a1): (i128, i128)) -> f64 {
        let p = self.c.get_pool_state().sqrt_price_x64 as f64 / 18446744073709551616.0;
        a0 as f64 * p * p + a1 as f64
    }
}

fn delta(a: (i128, i128), b: (i128, i128)) -> (i128, i128) {
    (a.0 - b.0, a.1 - b.1)
}

// --- earn ---------------------------------------------------------------

#[test]
fn lp_earns_the_whole_fee() {
    let env = Env::default();
    let p = Pool::new(&env);
    let lp = p.user(10_000_000_000);
    p.c.add_liquidity(&lp, &-6000, &6000, &5_000_000_000, &5_000_000_000, &0, &0);
    let trader = p.user(10_000_000_000);

    let (in0, _) = p.swap(&trader, true, 100_000_000);
    let (in1, _) = p.swap(&trader, false, 100_000_000);
    let (f0, f1) = p.c.collect_fees(&lp, &-6000, &6000);

    // The whole 0.3% goes to the only LP, rounding against it.
    for (input, fee) in [(in0, f0 as i128), (in1, f1 as i128)] {
        let total = input * FEE_BPS as i128 / 10_000;
        assert!(fee <= total + 2, "paid out {fee}, more than the {total} charged");
        assert!(fee >= total - 2, "fees lost: {fee} of {total}");
    }
}

#[test]
fn late_lp_cannot_take_earlier_fees() {
    for (lo, hi) in [(-6000, 6000), (-60, 60), (-1200, -600), (600, 1200), (-12000, 12000)] {
        let env = Env::default();
        let p = Pool::new(&env);
        let early = p.user(10_000_000_000);
        p.c.add_liquidity(&early, &-6000, &6000, &5_000_000_000, &5_000_000_000, &0, &0);
        let trader = p.user(10_000_000_000);
        for _ in 0..3 {
            p.swap(&trader, true, 300_000_000);
            p.swap(&trader, false, 300_000_000);
        }

        let late = p.user(10_000_000_000);
        p.c.add_liquidity(&late, &lo, &hi, &1_000_000_000, &1_000_000_000, &0, &0);

        assert_eq!(p.c.collect_fees(&late, &lo, &hi), (0, 0), "range [{lo},{hi})");
        // The fees were there to take.
        let earned = p.c.collect_fees(&early, &-6000, &6000);
        assert!(earned.0 > 0 && earned.1 > 0);
    }
}

#[test]
fn out_of_range_lp_earns_nothing() {
    let env = Env::default();
    let p = Pool::new(&env);
    let active = p.user(10_000_000_000);
    p.c.add_liquidity(&active, &-600, &600, &1_000_000_000, &1_000_000_000, &0, &0);
    let idle = p.user(10_000_000_000);
    p.c.add_liquidity(&idle, &1200, &2400, &1_000_000_000, &1_000_000_000, &0, &0);
    let trader = p.user(10_000_000_000);
    for _ in 0..5 {
        p.swap(&trader, true, 50_000_000);
        p.swap(&trader, false, 50_000_000);
    }
    assert_eq!(p.c.collect_fees(&idle, &1200, &2400), (0, 0));
}

// --- claim ---------------------------------------------------------------

#[test]
fn fees_are_paid_once() {
    let env = Env::default();
    let p = Pool::new(&env);
    let lp = p.user(10_000_000_000);
    let (l, _, _) = p.c.add_liquidity(&lp, &-6000, &6000, &5_000_000_000, &5_000_000_000, &0, &0);
    let trader = p.user(10_000_000_000);
    p.swap(&trader, true, 200_000_000);
    p.swap(&trader, false, 200_000_000);

    let first = p.c.collect_fees(&lp, &-6000, &6000);
    assert!(first.0 > 0 && first.1 > 0);
    assert_eq!(p.c.collect_fees(&lp, &-6000, &6000), (0, 0), "second collect paid again");

    // Fees earned before a full withdrawal are kept and paid exactly once.
    p.swap(&trader, true, 200_000_000);
    p.c.remove_liquidity(&lp, &-6000, &6000, &l, &0, &0);
    let after_exit = p.c.collect_fees(&lp, &-6000, &6000);
    assert!(after_exit.0 > 0);
    assert_eq!(p.c.collect_fees(&lp, &-6000, &6000), (0, 0));
    // Re-entering the same range does not resurrect old fees.
    p.c.add_liquidity(&lp, &-6000, &6000, &1_000_000_000, &1_000_000_000, &0, &0);
    assert_eq!(p.c.collect_fees(&lp, &-6000, &6000), (0, 0));
}

#[test]
fn only_the_owner_moves_a_position() {
    let env = Env::default();
    let p = Pool::new(&env);
    let lp = p.user(10_000_000_000);
    p.c.add_liquidity(&lp, &-600, &600, &1_000_000_000, &1_000_000_000, &0, &0);
    let thief = p.user(0);

    env.set_auths(&[]);
    assert!(p.c.try_remove_liquidity(&lp, &-600, &600, &1000, &0, &0).is_err());
    assert!(p.c.try_collect_fees(&lp, &-600, &600).is_err());
    // The thief's own (empty) position yields nothing.
    env.mock_all_auths();
    assert!(p.c.try_remove_liquidity(&thief, &-600, &600, &1000, &0, &0).is_err());
    assert_eq!(p.c.collect_fees(&thief, &-600, &600), (0, 0));
}

// --- add / withdraw ------------------------------------------------------

#[test]
fn deposit_then_withdraw_never_returns_more() {
    let env = Env::default();
    let p = Pool::new(&env);
    let anchor = p.user(10_000_000_000);
    p.c.add_liquidity(&anchor, &-6000, &6000, &1_000_000_000, &1_000_000_000, &0, &0);
    let u = p.user(1_000_000_000_000);
    let start = p.bal(&u);

    let mut seed = 0x2545F4914F6CDD1Du64;
    let mut cycles = 0;
    for _ in 0..200 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let lo = ((seed % 41) as i32 - 20) * 60;
        let hi = lo + ((seed >> 8) % 6 + 1) as i32 * 60;
        let amt = 1_000 + (seed >> 16) as i128 % 5_000_000_000;
        env.cost_estimate().budget().reset_unlimited();
        if let Ok(Ok((l, _, _))) = p.c.try_add_liquidity(&u, &lo, &hi, &amt, &amt, &0, &0) {
            p.c.remove_liquidity(&u, &lo, &hi, &l, &0, &0);
            cycles += 1;
        }
    }
    assert!(cycles > 150, "only {cycles} cycles ran");
    let d = delta(p.bal(&u), start);
    assert!(d.0 <= 0 && d.1 <= 0, "add/remove cycles extracted {d:?}");
    assert!(d.0 > -1_000 && d.1 > -1_000, "rounding cost too high: {d:?}");
}

#[test]
fn factory_mint_never_leaves_the_pool_short() {
    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut minted = 0;
    for _ in 0..60 {
        let env = Env::default();
        let p = Pool::new(&env);
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let lo = ((seed % 21) as i32 - 10) * 60;
        let hi = lo + ((seed >> 8) % 8 + 1) as i32 * 60;
        let (a0, a1) = (1_000_000 + (seed >> 20) as i128 % 10i128.pow(12), 1_000_000 + (seed >> 3) as i128 % 10i128.pow(12));

        // What factory::create_pool does: transfer the desired amounts, then mint.
        common::mint_tokens(&env, &p.t0, &p.c.address, a0);
        common::mint_tokens(&env, &p.t1, &p.c.address, a1);
        let Ok(Ok(l)) = p.c.try_mint(&p.creator, &lo, &hi, &a0, &a1) else { continue };

        // Everything handed back plus everything withdrawable is at most what was sent.
        let refunded = p.bal(&p.creator);
        let (w0, w1) = p.c.remove_liquidity(&p.creator, &lo, &hi, &l, &0, &0);
        assert!(refunded.0 + w0 <= a0 && refunded.1 + w1 <= a1, "pool paid out more than it received");
        assert!(a0 - refunded.0 - w0 <= 2 && a1 - refunded.1 - w1 <= 2);
        minted += 1;
    }
    assert!(minted > 50, "only {minted} mints ran");
}

// --- self-dealing --------------------------------------------------------

#[test]
fn lp_who_trades_against_itself_does_not_profit() {
    let env = Env::default();
    let p = Pool::new(&env);
    let me = p.user(1_000_000_000_000);
    let start = p.bal(&me);
    let (l, _, _) = p.c.add_liquidity(&me, &-120, &120, &10_000_000_000, &10_000_000_000, &0, &0);
    for i in 0..20 {
        // Cross the range edges and back.
        p.swap(&me, i % 2 == 0, 40_000_000_000);
        let tick = p.c.get_pool_state().current_tick;
        assert!(!(-120..120).contains(&tick), "swap {i} stayed inside the range at {tick}");
    }
    p.c.remove_liquidity(&me, &-120, &120, &l, &0, &0);
    p.c.collect_fees(&me, &-120, &120);
    let d = delta(p.bal(&me), start);
    assert!(p.value(d) <= 0.0, "self-dealing profit {d:?}");
}

// --- swap ----------------------------------------------------------------

#[test]
fn swap_moves_exactly_the_reported_amounts() {
    let env = Env::default();
    let p = Pool::new(&env);
    let lp = p.user(10_000_000_000);
    p.c.add_liquidity(&lp, &-600, &600, &1_000_000_000, &1_000_000_000, &0, &0);
    p.c.add_liquidity(&lp, &-6000, &-600, &1_000_000_000, &1_000_000_000, &0, &0);
    let trader = p.user(10_000_000_000);
    for zero_for_one in [true, false, true] {
        let (before_t, before_p) = (p.bal(&trader), p.bal(&p.c.address));
        let (amount_in, amount_out) = p.swap(&trader, zero_for_one, 1_500_000_000);
        let (dt, dp) = (delta(p.bal(&trader), before_t), delta(p.bal(&p.c.address), before_p));
        let expect = if zero_for_one { (-amount_in, amount_out) } else { (amount_out, -amount_in) };
        assert_eq!(dt, expect);
        assert_eq!(dp, (-expect.0, -expect.1));
    }
}

#[test]
fn swap_needs_the_senders_signature() {
    let env = Env::default();
    let p = Pool::new(&env);
    let lp = p.user(10_000_000_000);
    p.c.add_liquidity(&lp, &-600, &600, &1_000_000_000, &1_000_000_000, &0, &0);
    let victim = p.user(10_000_000_000);
    env.set_auths(&[]);
    assert!(p.c.try_swap(&victim, &p.t0, &1_000_000, &0, &0).is_err());
}

// --- creator lock --------------------------------------------------------

#[test]
fn locked_creator_position_cannot_be_withdrawn() {
    let env = Env::default();
    let p = Pool::new(&env);
    MockFactoryClient::new(&env, &p.factory).set_lock(&-600, &600);
    common::mint_tokens(&env, &p.t0, &p.c.address, 1_000_000_000);
    common::mint_tokens(&env, &p.t1, &p.c.address, 1_000_000_000);
    let l = p.c.mint(&p.creator, &-600, &600, &1_000_000_000, &1_000_000_000);

    // Every spelling that snaps to the locked range is refused.
    for (lo, hi) in [(-600, 600), (-599, 601), (-541, 659), (-600, 659)] {
        assert!(p.c.try_remove_liquidity(&p.creator, &lo, &hi, &l, &0, &0).is_err(), "[{lo},{hi})");
    }
    // Fees stay claimable while locked.
    let trader = p.user(10_000_000_000);
    p.swap(&trader, true, 100_000_000);
    assert!(p.c.collect_fees(&p.creator, &-600, &600).0 > 0);
}
