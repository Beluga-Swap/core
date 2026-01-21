//! Router storage operations

use soroban_sdk::Env;

use crate::types::{DataKey, RouterConfig};

// ============================================================
// TTL CONFIG
// ============================================================

const PERSISTENT_TTL: u32 = 6_307_200; // ~1 year

fn extend_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, PERSISTENT_TTL, PERSISTENT_TTL);
}

// ============================================================
// INITIALIZATION
// ============================================================

pub fn is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage().persistent().set(&DataKey::Initialized, &true);
    extend_ttl(env, &DataKey::Initialized);
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