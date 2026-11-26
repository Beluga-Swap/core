//! The two-level bitmap must agree with a brute-force ordered set.

use belugaswap_math::constants::{MAX_TICK, MIN_TICK};
use belugaswap_tick::{flip_tick, next_initialized_tick};
use soroban_sdk::Env;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

fn rng(state: &mut u64, n: u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state % n
}

#[test]
fn matches_brute_force() {
    let env = Env::default();
    for spacing in [1i32, 10, 60, 200] {
        let words = RefCell::new(BTreeMap::<i32, u128>::new());
        let summaries = RefCell::new(BTreeMap::<i32, u128>::new());
        let read_word = |_: &Env, p: i32| *words.borrow().get(&p).unwrap_or(&0);
        let write_word = |_: &Env, p: i32, v: u128| {
            words.borrow_mut().insert(p, v);
        };
        let read_summary = |_: &Env, p: i32| *summaries.borrow().get(&p).unwrap_or(&0);
        let write_summary = |_: &Env, p: i32, v: u128| {
            summaries.borrow_mut().insert(p, v);
        };

        let mut model = BTreeSet::<i32>::new();
        let mut s = 0x2545F4914F6CDD1Du64 ^ spacing as u64;
        let max = MAX_TICK / spacing;
        // Cluster most ticks around word and summary boundaries.
        let anchors = [0, 128, -128, 16_384, -16_384, max - 3, -(max - 3)];

        for _ in 0..4_000 {
            let mut c = anchors[rng(&mut s, anchors.len() as u64) as usize]
                + rng(&mut s, 7) as i32
                - 3;
            if rng(&mut s, 5) == 0 {
                c = rng(&mut s, (2 * max + 1) as u64) as i32 - max;
            }
            let tick = c.clamp(-max, max) * spacing;
            flip_tick(&env, read_word, write_word, read_summary, write_summary, tick, spacing);
            if !model.remove(&tick) {
                model.insert(tick);
            }

            let q = (tick + rng(&mut s, 5) as i32 - 2).clamp(MIN_TICK, MAX_TICK);
            let down = next_initialized_tick(&env, read_word, read_summary, q, spacing, true);
            let up = next_initialized_tick(&env, read_word, read_summary, q, spacing, false);
            assert_eq!(down, model.range(..=q).next_back().copied(), "down {q} spacing {spacing}");
            assert_eq!(up, model.range(q + 1..).next().copied(), "up {q} spacing {spacing}");
        }
    }
}
