//! The pool registry stores one entry per pool and is read in pages.

use beluga_factory::{BelugaFactory, BelugaFactoryClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn empty_registry_pages() {
    let env = Env::default();
    env.mock_all_auths();
    let hash = BytesN::from_array(&env, &[0; 32]);
    let client =
        BelugaFactoryClient::new(&env, &env.register(BelugaFactory, (&Address::generate(&env), &hash)));

    assert_eq!(client.get_total_pools(), 0);
    assert_eq!(client.get_pools(&0, &10).len(), 0);
    assert_eq!(client.get_pools(&u32::MAX, &u32::MAX).len(), 0);
}
