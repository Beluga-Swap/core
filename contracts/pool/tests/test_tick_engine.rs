//! Regression and invariant tests for tick traversal.

mod common;

use belugaswap_math::get_sqrt_ratio_at_tick;
use belugaswap_pool::BelugaPoolClient;
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, EnvTestConfig},
    token::TokenClient,
    Address, Env,
};

#[contract]
pub struct MockFactory;

#[contractimpl]
impl MockFactory {
    pub fn is_liquidity_locked(_e: Env, _p: Address, _c: Address, _l: i32, _u: i32) -> bool {
        false
    }
}

struct Pool<'a> {
    env: &'a Env,
    c: BelugaPoolClient<'a>,
    t0: Address,
    t1: Address,
}

impl<'a> Pool<'a> {
    fn new(env: &'a Env) -> Self {
        Self::with(env, 60, 30, 0)
    }

    fn with(env: &'a Env, spacing: i32, fee_bps: u32, tick: i32) -> Self {
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited();
        let factory = env.register(MockFactory, ());
        let router = Address::generate(env);
        let creator = Address::generate(env);
        let a = common::create_token(env, &creator);
        let b = common::create_token(env, &creator);
        let c = common::deploy_pool(env, &factory, &router, &creator, &a, &b, &fee_bps, &get_sqrt_ratio_at_tick(tick), &spacing);
        let s = c.get_pool_state();
        Pool { env, c, t0: s.token0, t1: s.token1 }
    }

    fn funded(&self, amount: i128) -> Address {
        let u = Address::generate(self.env);
        common::mint_tokens(self.env, &self.t0, &u, amount);
        common::mint_tokens(self.env, &self.t1, &u, amount);
        u
    }

    /// Returns (owner, liquidity).
    fn add(&self, lower: i32, upper: i32, amount: i128) -> (Address, i128) {
        let u = self.funded(amount * 2);
        let (l, _, _) = self.c.add_liquidity(&u, &lower, &upper, &amount, &amount, &0, &0);
        (u, l)
    }

    /// Returns Some((amount_in, amount_out)) when the swap succeeds.
    fn swap(&self, zero_for_one: bool, amount: i128) -> Option<(i128, i128)> {
        let u = self.funded(amount);
        let token = if zero_for_one { &self.t0 } else { &self.t1 };
        self.env.cost_estimate().budget().reset_unlimited();
        match self.c.try_swap(&u, token, &amount, &0, &0) {
            Ok(Ok(r)) => Some((r.amount_in, r.amount_out)),
            _ => None,
        }
    }

    fn assert_tick_matches_price(&self) {
        let s = self.c.get_pool_state();
        let lo = get_sqrt_ratio_at_tick(s.current_tick);
        let hi = get_sqrt_ratio_at_tick(s.current_tick + 1);
        assert!(
            lo <= s.sqrt_price_x64 && s.sqrt_price_x64 <= hi,
            "current_tick {} does not contain price {}",
            s.current_tick,
            s.sqrt_price_x64
        );
    }
}

#[test]
fn swap_crosses_empty_gap() {
    let env = Env::default();
    let p = Pool::new(&env);
    p.add(-60, 60, 2_000_000);
    p.add(-6060, -120, 1_000_000_000_000);

    let (used, _) = p.swap(true, 50_000_000).expect("swap");

    assert!(used > 49_000_000, "stopped at the gap, used only {used}");
    assert!(p.c.get_pool_state().current_tick < -120);
    assert!(p.c.get_pool_state().liquidity > 0);
    p.assert_tick_matches_price();
}

#[test]
fn pool_can_trade_back_after_leaving_all_liquidity() {
    let env = Env::default();
    let p = Pool::new(&env);
    let (_, l) = p.add(-60, 60, 2_000_000);
    p.swap(true, 50_000_000).expect("drain");
    assert_eq!(p.c.get_pool_state().liquidity, 0);

    p.swap(false, 1_000_000).expect("reverse swap must work");

    assert_eq!(p.c.get_pool_state().liquidity, l);
    p.assert_tick_matches_price();
}

#[test]
fn current_tick_follows_price_inside_a_range() {
    let env = Env::default();
    let p = Pool::new(&env);
    p.add(-6000, 6000, 1_000_000_000);

    p.swap(true, 10_000_000).expect("swap");

    assert!(p.c.get_pool_state().current_tick < 0);
    p.assert_tick_matches_price();
}

#[test]
fn liquidity_is_not_used_outside_its_range() {
    let env = Env::default();
    let p = Pool::new(&env);
    p.add(-60, 60, 2_000_000);
    p.swap(true, 50_000_000).expect("drain");
    p.add(-120, 0, 50_000_000);
    p.add(-6000, -3000, 1_000_000_000);
    p.swap(true, 1_000_000).expect("small");

    p.swap(true, 200_000_000).expect("big");

    let s = p.c.get_pool_state();
    assert!(s.current_tick < -120);
    if s.current_tick >= -3000 {
        assert_eq!(s.liquidity, 0, "liquidity active in an empty range");
    }
    p.assert_tick_matches_price();
}

// +-180,000 ticks is 3,000 spacing steps at spacing 60.
#[test]
fn wide_position_keeps_footprint_small() {
    let env = Env::default();
    let p = Pool::new(&env);
    p.add(-180_000, 180_000, 1_000_000_000);

    p.swap(true, 10_000).expect("swap on a wide position");

    let r = env.cost_estimate().resources();
    assert!(r.read_entries + r.write_entries < 40, "footprint too large: {r:?}");
}

#[test]
fn distant_ticks_keep_footprint_small() {
    let env = Env::default();
    let p = Pool::new(&env);
    p.add(-60, 60, 2_000_000);
    p.add(-200_040, -180_000, 1_000_000_000_000);

    p.swap(true, 5_000_000).expect("swap across a very wide gap");
    assert!(p.c.get_pool_state().current_tick < -179_999, "did not cross the gap");
    p.assert_tick_matches_price();

    let r = env.cost_estimate().resources();
    assert!(r.read_entries + r.write_entries < 60, "footprint too large: {r:?}");
}

#[test]
fn preview_matches_swap_across_ticks() {
    let env = Env::default();
    let p = Pool::new(&env);
    p.add(-60, 60, 2_000_000);
    p.add(-1200, -600, 100_000_000);

    let quote = p.c.preview_swap(&p.t0, &5_000_000, &0, &0);
    let (used, out) = p.swap(true, 5_000_000).expect("swap");

    assert!(quote.is_valid);
    assert_eq!((quote.amount_in_used, quote.amount_out_expected), (used, out));
}

#[test]
fn preview_reports_price_impact_at_any_price() {
    for tick in [0, 13_920, -13_920] {
        let env = Env::default();
        let p = Pool::new(&env);
        p.add(-30_000, 30_000, 10_000_000_000);
        if tick != 0 {
            let u = p.funded(100_000_000_000);
            let limit = get_sqrt_ratio_at_tick(tick);
            let token = if tick < 0 { &p.t0 } else { &p.t1 };
            p.c.swap(&u, token, &100_000_000_000, &0, &limit);
        }
        for token in [&p.t0, &p.t1] {
            let quote = p.c.preview_swap(token, &3_000_000_000, &0, &0);
            assert!(quote.is_valid, "tick {tick}: {:?}", quote.error_message);
            assert!(
                (500..5_000).contains(&quote.price_impact_bps),
                "tick {tick}: impact {} bps",
                quote.price_impact_bps
            );
        }
    }
}

// factory::create_pool transfers the desired amounts before calling mint.
#[test]
fn mint_returns_unused_amounts_to_owner() {
    let env = Env::default();
    let p = Pool::new(&env);
    let owner = Address::generate(&env);
    common::mint_tokens(&env, &p.t0, &p.c.address, 5_000_000_000);
    common::mint_tokens(&env, &p.t1, &p.c.address, 1_000_000);

    p.c.mint(&owner, &-600, &600, &5_000_000_000, &1_000_000);

    let t0 = TokenClient::new(&env, &p.t0);
    let held = t0.balance(&p.c.address);
    assert_eq!(held + t0.balance(&owner), 5_000_000_000);
    assert!(held <= 1_000_001, "excess left in pool: {held}");
}

struct Rng(u64);
impl Rng {
    fn next(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % n
    }
}

/// Pool shape and trade size for one family of random runs.
#[derive(Clone, Copy, Debug)]
struct Profile {
    spacing: i32,
    fee_bps: u32,
    base_tick: i32,
    /// Largest deposit per token; swaps are drawn up to 1.6x this.
    scale: u64,
}

const PROFILES: [Profile; 6] = [
    Profile { spacing: 60, fee_bps: 30, base_tick: 0, scale: 50_000_000 },
    Profile { spacing: 10, fee_bps: 5, base_tick: 0, scale: 50_000_000 },
    Profile { spacing: 200, fee_bps: 100, base_tick: 0, scale: 50_000_000 },
    Profile { spacing: 60, fee_bps: 30, base_tick: 99_960, scale: 50_000_000 },
    Profile { spacing: 60, fee_bps: 30, base_tick: -99_960, scale: 50_000_000 },
    Profile { spacing: 60, fee_bps: 30, base_tick: 0, scale: 10_000_000_000_000_000_000 },
];

fn random_run(seed: u64, profile: Profile) {
    let Profile { spacing, fee_bps, base_tick, scale } = profile;
    // No snapshot file per run: the random tests create thousands of envs.
    let env = Env::new_with_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });
    let p = Pool::with(&env, spacing, fee_bps, base_tick);
    let mut rng = Rng(seed);
    let mut positions: std::vec::Vec<(Address, i32, i32, i128)> = std::vec::Vec::new();

    for step in 0..60 {
        env.cost_estimate().budget().reset_unlimited();
        match rng.next(10) {
            0..=2 => {
                let lower = base_tick + (rng.next(40) as i32 - 20) * spacing;
                let upper = lower + (rng.next(8) as i32 + 1) * spacing;
                let amount = (scale / 50 + rng.next(scale)) as i128;
                let u = p.funded(amount * 2);
                // Some ranges are too thin for the amount; that may revert.
                if let Ok(Ok((l, _, _))) =
                    p.c.try_add_liquidity(&u, &lower, &upper, &amount, &amount, &0, &0)
                {
                    positions.push((u, lower, upper, l));
                }
            }
            // Remove all or part of a position.
            3 if !positions.is_empty() => {
                let i = rng.next(positions.len() as u64) as usize;
                let (owner, lower, upper, l) = positions[i].clone();
                let part = if rng.next(2) == 0 { l } else { (l / 3).max(1) };
                p.c.remove_liquidity(&owner, &lower, &upper, &part, &0, &0);
                if part == l {
                    p.c.collect_fees(&owner, &lower, &upper);
                    positions.swap_remove(i);
                } else {
                    positions[i].3 -= part;
                }
            }
            // Collect fees and keep the position.
            4 if !positions.is_empty() => {
                let i = rng.next(positions.len() as u64) as usize;
                let (owner, lower, upper, _) = &positions[i];
                p.c.collect_fees(owner, lower, upper);
            }
            pick => {
                let zero_for_one = rng.next(2) == 0;
                let amount = (1_000 + rng.next(scale / 5 * 8)) as i128;
                // Sometimes stop at a price limit near the current price.
                let limit = if pick >= 8 {
                    let offset = (rng.next(10 * spacing as u64) + 1) as i32;
                    let tick = p.c.get_pool_state().current_tick;
                    get_sqrt_ratio_at_tick(if zero_for_one { tick - offset } else { tick + offset })
                } else {
                    0
                };
                let u = p.funded(amount);
                let token = if zero_for_one { &p.t0 } else { &p.t1 };
                let before = p.c.get_pool_state().sqrt_price_x64;
                // A swap with nothing to trade against may revert; that is fine.
                if p.c.try_swap(&u, token, &amount, &0, &limit).is_ok_and(|r| r.is_ok()) {
                    let after = p.c.get_pool_state().sqrt_price_x64;
                    assert!(
                        if zero_for_one { after <= before } else { after >= before },
                        "seed {seed} step {step}: price moved the wrong way"
                    );
                    if limit != 0 {
                        assert!(
                            if zero_for_one { after >= limit } else { after <= limit },
                            "seed {seed} step {step}: price limit crossed"
                        );
                    }
                }
            }
        }

        let s = p.c.get_pool_state();
        p.assert_tick_matches_price();
        let expected: i128 = positions
            .iter()
            .filter(|(_, lo, hi, _)| *lo <= s.current_tick && s.current_tick < *hi)
            .map(|x| x.3)
            .sum();
        assert_eq!(
            s.liquidity, expected,
            "seed {seed} step {step} {profile:?}: active liquidity mismatch at tick {}",
            s.current_tick
        );
    }

    // Solvency: every LP and the creator can take out what they are owed.
    for (owner, lower, upper, l) in positions {
        env.cost_estimate().budget().reset_unlimited();
        p.c.remove_liquidity(&owner, &lower, &upper, &l, &0, &0);
        p.c.collect_fees(&owner, &lower, &upper);
    }
    assert_eq!(p.c.get_pool_state().liquidity, 0);
}

fn run_seeds(profile: Profile, count: u64) {
    for n in 1..=count {
        std::eprintln!("RUN {n} {profile:?}");
        random_run(n.wrapping_mul(0x9E3779B97F4A7C15), profile);
    }
}

fn long_seed_count() -> u64 {
    std::env::var("FUZZ_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(200)
}

// One test per profile so the harness runs them on separate cores.
//
// Quick:  cargo test -p belugaswap-pool --test test_tick_engine
// Long:   FUZZ_SEEDS=300 cargo test --release -p belugaswap-pool --test test_tick_engine -- --ignored
macro_rules! fuzz_profile {
    ($quick:ident, $long:ident, $index:expr) => {
        #[test]
        fn $quick() {
            run_seeds(PROFILES[$index], 7);
        }

        #[test]
        #[ignore = "long running"]
        fn $long() {
            run_seeds(PROFILES[$index], long_seed_count());
        }
    };
}

fuzz_profile!(random_spacing_60, random_long_spacing_60, 0);
fuzz_profile!(random_spacing_10, random_long_spacing_10, 1);
fuzz_profile!(random_spacing_200, random_long_spacing_200, 2);
fuzz_profile!(random_high_price, random_long_high_price, 3);
fuzz_profile!(random_low_price, random_long_low_price, 4);
fuzz_profile!(random_large_amounts, random_long_large_amounts, 5);
