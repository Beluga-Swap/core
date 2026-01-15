#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

mod error;
mod events;
mod storage;
mod types;

use error::FactoryErrorMsg;
use events::*;
use storage::*;
use types::*;

#[contract]
pub struct BelugaFactory;

#[contractimpl]
impl BelugaFactory {
    pub fn initialize(
        env: Env,
        admin: Address,
        pool_wasm_hash: BytesN<32>,
        min_initial_liquidity: i128,
        min_lock_duration: u32,
        default_fee_bps: u32,
        default_creator_fee_bps: u32,
    ) {
        admin.require_auth();
        
        if factory_is_initialized(&env) {
            panic!("{}", FactoryErrorMsg::ALREADY_INITIALIZED);
        }
        
        let config = FactoryConfig {
            admin: admin.clone(),
            pool_wasm_hash: pool_wasm_hash.clone(),
            min_initial_liquidity,
            min_lock_duration,
            default_fee_bps,
            default_creator_fee_bps,
        };
        
        write_factory_config(&env, &config);
        factory_set_initialized(&env);
        
        emit_factory_initialized(&env, &admin, &pool_wasm_hash, min_initial_liquidity);
    }
    
    pub fn get_config(env: Env) -> FactoryConfig {
        read_factory_config(&env)
    }
    
    pub fn get_stats(env: Env) -> FactoryStats {
        read_factory_stats(&env)
    }
}
