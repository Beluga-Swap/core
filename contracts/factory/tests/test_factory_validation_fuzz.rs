// ============================================================
// FACTORY VALIDATION FUZZING
// Tests validation logic WITHOUT actual contract deployment
// ============================================================

use proptest::prelude::*;

// ============================================================
// VALIDATION LOGIC
// ============================================================

const MIN_LOCK_DURATION: u32 = 120_960; // ~7 days
const MIN_INITIAL_LIQUIDITY: i128 = 1_000_000;
const MAX_FEE_BPS: u32 = 100;

fn validate_tokens(token_a: &str, token_b: &str) -> bool {
    token_a != token_b
}

fn validate_fee_tier(fee_bps: u32) -> bool {
    fee_bps == 5 || fee_bps == 30 || fee_bps == 100
}

fn validate_tick_range(lower: i32, upper: i32) -> bool {
    lower < upper
}

fn validate_tick_spacing(tick: i32, spacing: i32) -> bool {
    tick % spacing == 0
}

fn validate_amounts(amt0: i128, amt1: i128) -> bool {
    amt0 >= MIN_INITIAL_LIQUIDITY && amt1 >= MIN_INITIAL_LIQUIDITY
}

fn validate_lock_duration(duration: u32) -> bool {
    duration == 0 || duration >= MIN_LOCK_DURATION
}

fn validate_creator_fee(fee_bps: u32) -> bool {
    fee_bps >= 10 && fee_bps <= 1000
}

fn validate_sqrt_price(price: u128) -> bool {
    price > 0
}

fn get_tick_spacing(fee_bps: u32) -> i32 {
    match fee_bps {
        5 => 10,
        30 => 60,
        100 => 200,
        _ => 0,
    }
}

// ============================================================
// PROPERTY TESTS
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // ========================================================
    // TOKEN VALIDATION
    // ========================================================
    
    #[test]
    fn fuzz_same_token_rejected(
        token in "[a-z]{3,10}",
    ) {
        let result = validate_tokens(&token, &token);
        prop_assert!(!result, "Same tokens should be rejected");
    }
    
    #[test]
    fn fuzz_different_tokens_accepted(
        token_a in "[a-z]{3,10}",
        token_b in "[a-z]{3,10}",
    ) {
        if token_a != token_b {
            let result = validate_tokens(&token_a, &token_b);
            prop_assert!(result, "Different tokens should be accepted");
        }
    }
    
    // ========================================================
    // FEE TIER VALIDATION
    // ========================================================
    
    #[test]
    fn fuzz_valid_fee_tiers(
        fee_bps in prop::sample::select(vec![5u32, 30, 100]),
    ) {
        let result = validate_fee_tier(fee_bps);
        prop_assert!(result, "Valid fee tiers should be accepted: 5, 30, 100");
    }
    
    #[test]
    fn fuzz_invalid_fee_tiers(
        fee_bps in 1u32..=200u32,
    ) {
        let result = validate_fee_tier(fee_bps);
        
        if fee_bps != 5 && fee_bps != 30 && fee_bps != 100 {
            prop_assert!(!result, "Invalid fee tier {} should be rejected", fee_bps);
        }
    }
    
    // ========================================================
    // TICK VALIDATION
    // ========================================================
    
    #[test]
    fn fuzz_tick_range_validation(
        lower in -100000i32..100000i32,
        upper in -100000i32..100000i32,
    ) {
        let result = validate_tick_range(lower, upper);
        
        if lower >= upper {
            prop_assert!(!result, "lower >= upper should be rejected");
        } else {
            prop_assert!(result, "lower < upper should be accepted");
        }
    }
    
    #[test]
    fn fuzz_tick_spacing_alignment(
        tick in -10000i32..10000i32,
        spacing in 1i32..=200i32,
    ) {
        let result = validate_tick_spacing(tick, spacing);
        
        if tick % spacing == 0 {
            prop_assert!(result, "Aligned tick should be valid");
        } else {
            prop_assert!(!result, "Misaligned tick should be invalid");
        }
    }
    
    #[test]
    fn fuzz_tick_spacing_per_fee_tier(
        fee_tier in prop::sample::select(vec![5u32, 30, 100]),
        tick_multiplier in -100i32..100i32,
    ) {
        let spacing = get_tick_spacing(fee_tier);
        let tick = tick_multiplier * spacing;
        
        let result = validate_tick_spacing(tick, spacing);
        prop_assert!(result, "Tick aligned to fee tier spacing should be valid");
    }
    
    // ========================================================
    // AMOUNT VALIDATION
    // ========================================================
    
    #[test]
    fn fuzz_insufficient_amounts(
        amt0 in 0i128..MIN_INITIAL_LIQUIDITY,
        amt1 in 0i128..MIN_INITIAL_LIQUIDITY,
    ) {
        let result = validate_amounts(amt0, amt1);
        prop_assert!(!result, "Amounts below minimum should be rejected");
    }
    
    #[test]
    fn fuzz_sufficient_amounts(
        amt0 in MIN_INITIAL_LIQUIDITY..=i128::MAX / 2,
        amt1 in MIN_INITIAL_LIQUIDITY..=i128::MAX / 2,
    ) {
        let result = validate_amounts(amt0, amt1);
        prop_assert!(result, "Amounts above minimum should be accepted");
    }
    
    #[test]
    fn fuzz_one_insufficient_amount(
        sufficient in MIN_INITIAL_LIQUIDITY..10_000_000i128,
        insufficient in 0i128..MIN_INITIAL_LIQUIDITY,
    ) {
        // Case 1: amt0 sufficient, amt1 insufficient
        let result1 = validate_amounts(sufficient, insufficient);
        prop_assert!(!result1, "One insufficient amount should reject");
        
        // Case 2: amt0 insufficient, amt1 sufficient
        let result2 = validate_amounts(insufficient, sufficient);
        prop_assert!(!result2, "One insufficient amount should reject");
    }
    
    // ========================================================
    // LOCK DURATION VALIDATION
    // ========================================================
    
    #[test]
    fn fuzz_permanent_lock(
        _seed in 0u8..10u8,
    ) {
        let result = validate_lock_duration(0);
        prop_assert!(result, "Permanent lock (0) should be valid");
    }
    
    #[test]
    fn fuzz_valid_lock_duration(
        duration in MIN_LOCK_DURATION..=31536000u32,
    ) {
        let result = validate_lock_duration(duration);
        prop_assert!(result, "Duration >= MIN_LOCK_DURATION should be valid");
    }
    
    #[test]
    fn fuzz_invalid_lock_duration(
        duration in 1u32..MIN_LOCK_DURATION,
    ) {
        let result = validate_lock_duration(duration);
        prop_assert!(!result, "Duration < MIN_LOCK_DURATION should be invalid");
    }
    
    // ========================================================
    // CREATOR FEE VALIDATION
    // ========================================================
    
    #[test]
    fn fuzz_valid_creator_fee(
        fee_bps in 10u32..=1000u32,
    ) {
        let result = validate_creator_fee(fee_bps);
        prop_assert!(result, "Creator fee 10-1000 bps should be valid");
    }
    
    #[test]
    fn fuzz_too_low_creator_fee(
        fee_bps in 0u32..10u32,
    ) {
        let result = validate_creator_fee(fee_bps);
        prop_assert!(!result, "Creator fee < 10 bps should be invalid");
    }
    
    #[test]
    fn fuzz_too_high_creator_fee(
        fee_bps in 1001u32..=5000u32,
    ) {
        let result = validate_creator_fee(fee_bps);
        prop_assert!(!result, "Creator fee > 1000 bps should be invalid");
    }
    
    // ========================================================
    // SQRT PRICE VALIDATION
    // ========================================================
    
    #[test]
    fn fuzz_zero_price_rejected(
        _seed in 0u8..10u8,
    ) {
        let result = validate_sqrt_price(0);
        prop_assert!(!result, "Zero price should be rejected");
    }
    
    #[test]
    fn fuzz_valid_prices(
        price in 1u128..=u128::MAX / 2,
    ) {
        let result = validate_sqrt_price(price);
        prop_assert!(result, "Non-zero price should be valid");
    }
    
    // ========================================================
    // COMBINED VALIDATION SCENARIOS
    // ========================================================
    
    #[test]
    fn fuzz_complete_valid_params(
        amt0 in MIN_INITIAL_LIQUIDITY..10_000_000i128,
        amt1 in MIN_INITIAL_LIQUIDITY..10_000_000i128,
        fee_tier in prop::sample::select(vec![5u32, 30, 100]),
        creator_fee in 10u32..=1000u32,
        lock_duration in prop::sample::select(vec![0u32, MIN_LOCK_DURATION, MIN_LOCK_DURATION * 2]),
    ) {
        let spacing = get_tick_spacing(fee_tier);
        let lower = -600;
        let upper = 600;
        
        // All validations should pass
        prop_assert!(validate_tokens("USDC", "XLM"));
        prop_assert!(validate_fee_tier(fee_tier));
        prop_assert!(validate_tick_range(lower, upper));
        prop_assert!(validate_tick_spacing(lower, spacing));
        prop_assert!(validate_tick_spacing(upper, spacing));
        prop_assert!(validate_amounts(amt0, amt1));
        prop_assert!(validate_lock_duration(lock_duration));
        prop_assert!(validate_creator_fee(creator_fee));
        prop_assert!(validate_sqrt_price(1u128 << 64));
    }
    
    #[test]
    fn fuzz_detect_any_invalid_param(
        valid_amt in MIN_INITIAL_LIQUIDITY..10_000_000i128,
        invalid_amt in 0i128..MIN_INITIAL_LIQUIDITY,
        valid_creator_fee in 10u32..=1000u32,
        invalid_creator_fee in 0u32..10u32,
        valid_lock in MIN_LOCK_DURATION..=1000000u32,
        invalid_lock in 1u32..MIN_LOCK_DURATION,
    ) {
        // At least one validation should fail
        let all_valid = 
            validate_amounts(valid_amt, valid_amt) &&
            validate_creator_fee(valid_creator_fee) &&
            validate_lock_duration(valid_lock);
            
        let has_invalid = 
            !validate_amounts(invalid_amt, valid_amt) ||
            !validate_amounts(valid_amt, invalid_amt) ||
            !validate_creator_fee(invalid_creator_fee) ||
            !validate_lock_duration(invalid_lock);
        
        prop_assert!(all_valid, "All valid params should pass");
        prop_assert!(has_invalid, "Any invalid param should fail");
    }
}

// ============================================================
// EDGE CASE TESTS
// ============================================================

#[test]
fn test_boundary_amounts() {
    // Exactly at minimum
    assert!(validate_amounts(MIN_INITIAL_LIQUIDITY, MIN_INITIAL_LIQUIDITY));
    
    // One below minimum
    assert!(!validate_amounts(MIN_INITIAL_LIQUIDITY - 1, MIN_INITIAL_LIQUIDITY));
    assert!(!validate_amounts(MIN_INITIAL_LIQUIDITY, MIN_INITIAL_LIQUIDITY - 1));
    
    // Zero
    assert!(!validate_amounts(0, 0));
}

#[test]
fn test_boundary_creator_fee() {
    // Lower bound
    assert!(validate_creator_fee(10));
    assert!(!validate_creator_fee(9));
    
    // Upper bound
    assert!(validate_creator_fee(1000));
    assert!(!validate_creator_fee(1001));
}

#[test]
fn test_boundary_lock_duration() {
    // Permanent
    assert!(validate_lock_duration(0));
    
    // Just below minimum
    assert!(!validate_lock_duration(MIN_LOCK_DURATION - 1));
    
    // Exactly minimum
    assert!(validate_lock_duration(MIN_LOCK_DURATION));
}

#[test]
fn test_tick_spacing_all_tiers() {
    // Fee tier 5 bps → spacing 10
    assert_eq!(get_tick_spacing(5), 10);
    assert!(validate_tick_spacing(-600, 10));
    assert!(validate_tick_spacing(600, 10));
    assert!(!validate_tick_spacing(605, 10));
    
    // Fee tier 30 bps → spacing 60
    assert_eq!(get_tick_spacing(30), 60);
    assert!(validate_tick_spacing(-600, 60));
    assert!(validate_tick_spacing(600, 60));
    assert!(!validate_tick_spacing(650, 60));
    
    // Fee tier 100 bps → spacing 200
    assert_eq!(get_tick_spacing(100), 200);
    assert!(validate_tick_spacing(-600, 200));
    assert!(validate_tick_spacing(600, 200));
    assert!(!validate_tick_spacing(700, 200));
}

#[test]
fn test_extreme_tick_values() {
    let max_tick = 887272;
    let min_tick = -887272;
    
    // Valid range
    assert!(validate_tick_range(min_tick, max_tick));
    
    // Same tick (invalid)
    assert!(!validate_tick_range(0, 0));
    
    // Reversed (invalid)
    assert!(!validate_tick_range(max_tick, min_tick));
}

#[test]
fn test_price_boundaries() {
    // Zero (invalid)
    assert!(!validate_sqrt_price(0));
    
    // Minimum valid
    assert!(validate_sqrt_price(1));
    
    // Typical value (2^64)
    assert!(validate_sqrt_price(1u128 << 64));
    
    // Very large (but valid)
    assert!(validate_sqrt_price(u128::MAX - 1));
}

// ============================================================
// STATISTICAL TESTS
// ============================================================

#[test]
fn statistics_fee_tier_distribution() {
    // Verify all 3 tiers are valid
    let valid_tiers = vec![5u32, 30, 100];
    
    for tier in valid_tiers {
        assert!(validate_fee_tier(tier), "Tier {} should be valid", tier);
        assert!(get_tick_spacing(tier) > 0, "Tier {} should have tick spacing", tier);
    }
    
    // Verify random values are mostly invalid
    let mut invalid_count = 0;
    for i in 1..=200 {
        if i != 5 && i != 30 && i != 100 {
            if !validate_fee_tier(i) {
                invalid_count += 1;
            }
        }
    }
    
    assert!(invalid_count >= 195, "Most random fee tiers should be invalid");
}

#[test]
fn statistics_creator_fee_range() {
    let valid_range = 10..=1000;
    let total_valid = valid_range.count();
    
    assert_eq!(total_valid, 991, "Should have 991 valid creator fee values");
    
    // Below range
    for i in 0..10 {
        assert!(!validate_creator_fee(i));
    }
    
    // Above range
    for i in 1001..1100 {
        assert!(!validate_creator_fee(i));
    }
}