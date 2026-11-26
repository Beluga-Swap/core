//! `mul_div_wide` against the host's 256-bit integers.

use belugaswap_math::mul_div_wide;
use soroban_sdk::{Env, U256};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// A non-zero value of exactly `bits` bits.
    fn sized(&mut self, bits: u32) -> u128 {
        let raw = ((self.next() as u128) << 64) | self.next() as u128;
        (raw >> (128 - bits)) | (1u128 << (bits - 1))
    }
}

fn expected(env: &Env, a: u128, b: u128, shift: u32, d1: u128, d2: u128, up: bool) -> u128 {
    let big = |v: u128| U256::from_u128(env, v);
    let x = big(a).mul(&big(b)).shl(shift);
    let q1 = x.div(&big(d1));
    let q = q1.div(&big(d2));
    let exact = q1.mul(&big(d1)) == x && q.mul(&big(d2)) == q1;
    let q = q.to_u128().unwrap_or(u128::MAX);
    if up && !exact {
        q.saturating_add(1)
    } else {
        q
    }
}

#[test]
fn matches_u256() {
    let env = Env::default();
    let mut rng = Rng(0x9E3779B97F4A7C15);
    for i in 0..20_000 {
        env.cost_estimate().budget().reset_unlimited();
        let a_bits = 1 + (rng.next() % 128) as u32;
        let b_bits = 1 + (rng.next() % 128) as u32;
        // Keep the shifted product within 256 bits so U256 can check it.
        let room = 256 - a_bits - b_bits;
        let shift = (rng.next() % (room.min(128) as u64 + 1)) as u32;
        let d1_bits = 1 + (rng.next() % 128) as u32;
        let d2_bits = 1 + (rng.next() % 128) as u32;
        let (a, b) = (rng.sized(a_bits), rng.sized(b_bits));
        let (d1, d2) = (rng.sized(d1_bits), rng.sized(d2_bits));
        for up in [false, true] {
            assert_eq!(
                mul_div_wide(a, b, shift, d1, d2, up),
                expected(&env, a, b, shift, d1, d2, up),
                "case {i}: {a} * {b} << {shift} / {d1} / {d2} up={up}"
            );
        }
    }
}

#[test]
fn handles_products_beyond_256_bits() {
    let max = u128::MAX;
    assert_eq!(mul_div_wide(max, max, 128, max, max, false), u128::MAX); // 2^128 saturates
    assert_eq!(mul_div_wide(max, max, 64, max, max, false), 1u128 << 64);
    assert_eq!(mul_div_wide(1 << 127, 1 << 127, 128, 1 << 127, 1 << 127, false), u128::MAX);
    assert_eq!(mul_div_wide(1 << 120, 1 << 100, 100, 1 << 110, 1 << 110, false), 1 << 100);
}
