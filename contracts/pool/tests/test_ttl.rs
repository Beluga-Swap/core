mod common;

use common::*;
use soroban_sdk::{
    contracttype,
    testutils::{
        storage::{Instance as _, Persistent as _},
        Ledger,
    },
    Env,
};

/// Mirror of the pool's private storage key. A `contracttype` enum is stored
/// as its variant name plus fields, so this addresses the same entries.
#[contracttype]
enum DataKey {
    PoolState,
    PoolConfig,
}

const DAY_IN_LEDGERS: u32 = 17_280;
const EXPECTED_TTL: u32 = 120 * DAY_IN_LEDGERS;
/// `max_entry_ttl` on Stellar mainnet and testnet (~180 days).
const NETWORK_MAX_ENTRY_TTL: u32 = 3_110_400;

#[test]
fn pool_entries_live_120_days_within_network_limit() {
    // Arrange: a ledger with the live network's TTL ceiling.
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.max_entry_ttl = NETWORK_MAX_ENTRY_TTL);

    // Act
    let (client, ..) = setup_pool(&env);

    // Assert
    assert!(EXPECTED_TTL < NETWORK_MAX_ENTRY_TTL);
    env.as_contract(&client.address, || {
        assert_eq!(env.storage().instance().get_ttl(), EXPECTED_TTL);
        assert_eq!(env.storage().persistent().get_ttl(&DataKey::PoolState), EXPECTED_TTL);
        assert_eq!(env.storage().persistent().get_ttl(&DataKey::PoolConfig), EXPECTED_TTL);
    });
}
