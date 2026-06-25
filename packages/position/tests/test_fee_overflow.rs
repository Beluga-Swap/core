// Regression: fee accrual must use full-width math. The old
// `liquidity.checked_mul(delta).unwrap_or(0) >> 64` silently dropped ALL fees
// for a position whose liquidity * delta exceeded u128. Here liquidity * delta
// = 2^70 * 2^64 = 2^134 overflows u128, so the buggy code returned 0; the
// 256-bit path must return the exact fee = (2^70 * 2^64) >> 64 = 2^70.

use belugaswap_position::{calculate_pending_fees, update_position, Position};
use soroban_sdk::Env;

#[test]
fn test_large_position_fees_not_dropped_on_overflow() {
    // Arrange
    let env = Env::default();
    let pos = Position {
        liquidity: 1 << 70,
        fee_growth_inside_last_0: 0,
        fee_growth_inside_last_1: 0,
        tokens_owed_0: 0,
        tokens_owed_1: 0,
    };

    // Act: fee growth delta of 1.0 (Q64.64) on token0.
    let (pending_0, pending_1) = calculate_pending_fees(&env, &pos, 1u128 << 64, 0);

    // Assert
    assert_eq!(pending_0, 1u128 << 70, "fee must be exact, not dropped to 0");
    assert_eq!(pending_1, 0);
}

#[test]
fn test_update_position_credits_large_fee() {
    // Arrange
    let env = Env::default();
    let mut pos = Position {
        liquidity: 1 << 70,
        fee_growth_inside_last_0: 0,
        fee_growth_inside_last_1: 0,
        tokens_owed_0: 0,
        tokens_owed_1: 0,
    };

    // Act
    update_position(&env, &mut pos, 1u128 << 64, 0);

    // Assert
    assert_eq!(pos.tokens_owed_0, 1u128 << 70);
    assert_eq!(pos.fee_growth_inside_last_0, 1u128 << 64);
}
