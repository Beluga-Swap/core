//! Two-level bitmap of initialized ticks.
//!
//! A tick index is compressed to `tick / tick_spacing`. Each bitmap word marks
//! 128 compressed ticks and each summary word marks which of 128 bitmap words
//! are non-empty. A search therefore skips 16,384 compressed ticks per storage
//! read, keeping the transaction footprint small however far apart ticks are.

use belugaswap_math::constants::{MAX_TICK, MIN_TICK};
use soroban_sdk::Env;

const WORD_BITS: i32 = 128;

/// Bits at or below `bit`.
fn mask_lte(bit: i32) -> u128 {
    u128::MAX >> (WORD_BITS - 1 - bit)
}

/// Bits at or above `bit`.
fn mask_gte(bit: i32) -> u128 {
    u128::MAX << bit
}

fn highest(word: u128) -> i32 {
    WORD_BITS - 1 - word.leading_zeros() as i32
}

fn lowest(word: u128) -> i32 {
    word.trailing_zeros() as i32
}

/// Toggle a tick. Call exactly when the tick flips between initialized and not.
pub fn flip_tick(
    env: &Env,
    read_word: impl Fn(&Env, i32) -> u128,
    write_word: impl Fn(&Env, i32, u128),
    read_summary: impl Fn(&Env, i32) -> u128,
    write_summary: impl Fn(&Env, i32, u128),
    tick: i32,
    tick_spacing: i32,
) {
    let compressed = tick.div_euclid(tick_spacing);
    let word_pos = compressed.div_euclid(WORD_BITS);
    let before = read_word(env, word_pos);
    let after = before ^ (1u128 << compressed.rem_euclid(WORD_BITS));
    write_word(env, word_pos, after);

    if (before == 0) != (after == 0) {
        let summary_pos = word_pos.div_euclid(WORD_BITS);
        let summary = read_summary(env, summary_pos);
        write_summary(
            env,
            summary_pos,
            summary ^ (1u128 << word_pos.rem_euclid(WORD_BITS)),
        );
    }
}

/// The next initialized tick in the swap direction, or `None` if there is none.
///
/// * `zero_for_one` (price falling): the greatest initialized tick `<= current_tick`.
/// * otherwise (price rising): the smallest initialized tick `> current_tick`.
pub fn next_initialized_tick(
    env: &Env,
    read_word: impl Fn(&Env, i32) -> u128,
    read_summary: impl Fn(&Env, i32) -> u128,
    current_tick: i32,
    tick_spacing: i32,
    zero_for_one: bool,
) -> Option<i32> {
    let mut start = current_tick.div_euclid(tick_spacing);
    if !zero_for_one {
        start += 1;
    }
    let word_pos = start.div_euclid(WORD_BITS);
    let bit = start.rem_euclid(WORD_BITS);

    // 1. The word containing the starting point.
    let mask = if zero_for_one { mask_lte(bit) } else { mask_gte(bit) };
    let word = read_word(env, word_pos) & mask;
    if word != 0 {
        let found = if zero_for_one { highest(word) } else { lowest(word) };
        return Some((word_pos * WORD_BITS + found) * tick_spacing);
    }

    // 2. Walk the summary to the next non-empty word.
    let step: i32 = if zero_for_one { -1 } else { 1 };
    let first_word = word_pos + step;
    let mut summary_pos = first_word.div_euclid(WORD_BITS);
    let first_bit = first_word.rem_euclid(WORD_BITS);
    let mut mask = if zero_for_one { mask_lte(first_bit) } else { mask_gte(first_bit) };

    let edge = if zero_for_one { MIN_TICK } else { MAX_TICK };
    let last_summary = edge
        .div_euclid(tick_spacing)
        .div_euclid(WORD_BITS)
        .div_euclid(WORD_BITS);

    while (zero_for_one && summary_pos >= last_summary)
        || (!zero_for_one && summary_pos <= last_summary)
    {
        let summary = read_summary(env, summary_pos) & mask;
        if summary != 0 {
            let found = if zero_for_one { highest(summary) } else { lowest(summary) };
            let target = summary_pos * WORD_BITS + found;
            let word = read_word(env, target);
            let found = if zero_for_one { highest(word) } else { lowest(word) };
            return Some((target * WORD_BITS + found) * tick_spacing);
        }
        summary_pos += step;
        mask = u128::MAX;
    }

    None
}
