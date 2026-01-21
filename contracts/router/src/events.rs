//! Router events

use soroban_sdk::{Address, Env, Symbol, Vec};

/// Emitted when router is initialized
pub fn emit_initialized(env: &Env, factory: &Address, admin: &Address) {
    env.events().publish(
        (Symbol::new(env, "RouterInit"),),
        (factory.clone(), admin.clone()),
    );
}

/// Emitted on successful swap
pub fn emit_swap(
    env: &Env,
    sender: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    pools_used: &Vec<Address>,
) {
    env.events().publish(
        (Symbol::new(env, "Swap"),),
        (
            sender.clone(),
            token_in.clone(),
            token_out.clone(),
            amount_in,
            amount_out,
            pools_used.clone(),
        ),
    );
}

/// Emitted on split swap
pub fn emit_split_swap(
    env: &Env,
    sender: &Address,
    token_in: &Address,
    token_out: &Address,
    total_in: i128,
    total_out: i128,
    num_splits: u32,
) {
    env.events().publish(
        (Symbol::new(env, "SplitSwap"),),
        (
            sender.clone(),
            token_in.clone(),
            token_out.clone(),
            total_in,
            total_out,
            num_splits,
        ),
    );
}

/// Emitted on multihop swap
pub fn emit_multihop_swap(
    env: &Env,
    sender: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    num_hops: u32,
) {
    env.events().publish(
        (Symbol::new(env, "MultihopSwap"),),
        (
            sender.clone(),
            token_in.clone(),
            token_out.clone(),
            amount_in,
            amount_out,
            num_hops,
        ),
    );
}