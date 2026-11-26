//! Router storage operations

use soroban_sdk::Env;

use crate::types::{DataKey, RouterConfig};

// ============================================================
// TTL CONFIG
// ============================================================

const DAY_IN_LEDGERS: u32 = 17_280;

/// Entries are extended to `TTL_BUMP` once fewer than `TTL_THRESHOLD` ledgers
/// remain, so an active entry is re-extended about once a day instead of on
/// every access.
const TTL_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
const TTL_BUMP: u32 = 31 * DAY_IN_LEDGERS;

/// Keeps the contract instance and its code alive.
pub fn extend_instance(env: &Env) {
    env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_BUMP);
}

fn extend_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, TTL_THRESHOLD, TTL_BUMP);
}

// ============================================================
// CONFIG
// ============================================================

pub fn write_config(env: &Env, config: &RouterConfig) {
    env.storage().persistent().set(&DataKey::Config, config);
    extend_ttl(env, &DataKey::Config);
}

pub fn read_config(env: &Env) -> RouterConfig {
    let key = DataKey::Config;
    let config = env.storage().persistent().get(&key).unwrap();
    extend_ttl(env, &key);
    config
}