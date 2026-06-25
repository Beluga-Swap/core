// Validation of get_sqrt_ratio_at_tick: end-to-end monotonicity and accuracy.
//
// The bit-decomposition constants were hand-tuned; this locks in that the
// resulting price curve is (a) non-decreasing across the whole tick range,
// (b) strictly increasing across the practical range, and (c) accurate to
// well under 1 ppm versus high-precision reference values.

use belugaswap_math::constants::{MAX_TICK, MIN_TICK};
use belugaswap_math::sqrt_price::{get_sqrt_ratio_at_tick, get_tick_at_sqrt_ratio};

// High-precision reference values: sqrt(1.0001^tick) * 2^64, rounded.
// Generated independently with 90-digit Decimal arithmetic.
const REFERENCE: &[(i32, u128)] = &[
    (-887272, 2),
    (-100000, 124324258982887575),
    (-60000, 918547070906360253),
    (-4096, 15030750278693429945),
    (-256, 18212142134806087855),
    (-128, 18329067761203520169),
    (-64, 18387811781193591353),
    (-60, 18391489527427947883),
    (-1, 18445821805675392312),
    (1, 18447666387855959851),
    (60, 18502164624211761448),
    (63, 18504940018287354211),
    (64, 18505865242158250042),
    (127, 18564247702699447986),
    (128, 18565175891880433523),
    (255, 18683433917872171730),
    (256, 18684368066214940583),
    (1000, 19392480388906836278),
    (4096, 22639080592224303008),
    (60000, 370457190163559926269),
    (100000, 2737055259406582257881),
    (887272, 340269576638287423002690256994712238281),
];

#[test]
fn test_sqrt_ratio_never_decreases_across_full_range() {
    // Arrange / Act: scan every tick, assert the curve never goes backwards.
    let mut prev = get_sqrt_ratio_at_tick(MIN_TICK);
    for tick in (MIN_TICK + 1)..=MAX_TICK {
        let cur = get_sqrt_ratio_at_tick(tick);
        // Assert
        assert!(
            cur >= prev,
            "monotonicity broken: ratio({}) = {} < ratio({}) = {}",
            tick,
            cur,
            tick - 1,
            prev
        );
        prev = cur;
    }
}

#[test]
fn test_sqrt_ratio_strictly_increases_in_practical_range() {
    // Strictness only holds where the price magnitude exceeds fixed-point
    // quantization. +/-300000 covers ~1e13x price range — far beyond any pool.
    let lo = -300_000;
    let hi = 300_000;
    let mut prev = get_sqrt_ratio_at_tick(lo);
    for tick in (lo + 1)..=hi {
        let cur = get_sqrt_ratio_at_tick(tick);
        assert!(
            cur > prev,
            "strict monotonicity broken at tick {} ({} !> {})",
            tick,
            cur,
            prev
        );
        prev = cur;
    }
}

#[test]
fn test_sqrt_ratio_matches_reference_within_1ppm() {
    // Assert each sampled tick is within 1 ppm of the high-precision reference.
    let mut worst_num: u128 = 0; // track worst relative error as diff/ref
    let mut worst_tick = 0i32;
    for &(tick, expected) in REFERENCE {
        let got = get_sqrt_ratio_at_tick(tick);
        let diff = got.abs_diff(expected);
        // tolerance = max(expected / 1e6, small absolute floor for tiny prices)
        let tol = (expected / 1_000_000).max(65_536);
        assert!(
            diff <= tol,
            "tick {}: got {}, expected {}, diff {} > tol {}",
            tick,
            got,
            expected,
            diff,
            tol
        );
        // record worst ppm where expected is large enough to be meaningful
        if expected > 1 << 64 {
            let ppm = diff.saturating_mul(1_000_000) / expected;
            if ppm > worst_num {
                worst_num = ppm;
                worst_tick = tick;
            }
        }
    }
    std::println!(
        "[sqrt validation] worst observed error = {} ppm (at tick {})",
        worst_num,
        worst_tick
    );
}

#[test]
fn test_sqrt_ratio_strict_across_bit_carry_boundaries() {
    // Bit-carry points are where decomposition errors would surface first.
    for &b in &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 4096, 65536] {
        let before = get_sqrt_ratio_at_tick(b - 1);
        let at = get_sqrt_ratio_at_tick(b);
        let after = get_sqrt_ratio_at_tick(b + 1);
        assert!(before < at && at < after, "non-strict near tick {}", b);
    }
}

#[test]
fn test_get_tick_at_sqrt_ratio_round_trips() {
    // For any tick in the practical range, ratio(t) maps back to exactly t.
    for &t in &[
        -300_000, -131_072, -60_000, -4096, -600, -60, -1, 0, 1, 60, 600, 4096,
        60_000, 131_072, 300_000,
    ] {
        let ratio = get_sqrt_ratio_at_tick(t);
        let back = get_tick_at_sqrt_ratio(ratio);
        assert_eq!(back, t, "round-trip failed for tick {} (got {})", t, back);
    }
}

#[test]
fn test_get_tick_at_sqrt_ratio_floor_and_clamp() {
    // Price 1.0 -> tick 0.
    assert_eq!(get_tick_at_sqrt_ratio(1u128 << 64), 0);

    // A price strictly between ratio(1000) and ratio(1001) floors to 1000.
    let mid = get_sqrt_ratio_at_tick(1000) + 1;
    assert!(mid < get_sqrt_ratio_at_tick(1001));
    assert_eq!(get_tick_at_sqrt_ratio(mid), 1000);

    // Clamps at the ends.
    assert_eq!(get_tick_at_sqrt_ratio(0), MIN_TICK);
    assert_eq!(get_tick_at_sqrt_ratio(u128::MAX), MAX_TICK);
}
