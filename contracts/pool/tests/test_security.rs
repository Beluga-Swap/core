mod common;

use soroban_sdk::{testutils::Address as _, Address, Env};

// Regression test for the unauthorized-mint pool drain.
//
// `mint()` credits a position without pulling any tokens (the factory transfers
// them beforehand in create_pool). It must only be callable when the factory
// authorizes the call. Here we strip all authorizations before calling mint
// directly as an attacker; the call must be rejected.
//
// Before the fix (mint had no auth check) this call would succeed, letting the
// attacker create a free position and drain the pool. After the fix it panics
// because the factory has not authorized the invocation.
#[test]
#[should_panic]
fn test_mint_rejects_unauthorized_direct_call() {
    // Arrange
    let env = Env::default();
    env.mock_all_auths();
    let (client, _creator, _factory, _router, _token_a, _token_b) = common::setup_pool(&env);
    let attacker = Address::generate(&env);

    // Act: drop all mocked auths so the direct mint carries no factory authorization.
    env.set_auths(&[]);
    client.mint(&attacker, &-600, &600, &10_000_000_000, &10_000_000_000);

    // Assert: handled by #[should_panic] — mint must not succeed here.
}
