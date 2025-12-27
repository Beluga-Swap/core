#![allow(dead_code)]

use soroban_sdk::{Env, contracttype, Vec};
use crate::DataKey;

pub const TWAP_BUFFER_SIZE: u32 = 100;

#[contracttype]
#[derive(Clone, Debug)]
pub struct TWAPObservation {
    pub timestamp: u64,
    pub tick: i32,
    pub tick_cumulative: i64,
    pub fee_growth_global_0: u128,
    pub fee_growth_global_1: u128,
    pub liquidity: i128,
}

fn get_newest_index(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get::<_, u32>(&DataKey::TWAPNewestIndex)
        .unwrap_or(0)
}

fn set_newest_index(env: &Env, index: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::TWAPNewestIndex, &index);
}

fn is_twap_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<_, bool>(&DataKey::TWAPInitialized)
        .unwrap_or(false)
}

fn set_twap_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&DataKey::TWAPInitialized, &true);
}

fn read_observation(env: &Env, index: u32) -> Option<TWAPObservation> {
    env.storage()
        .persistent()
        .get::<_, TWAPObservation>(&DataKey::TWAPObservation(index % TWAP_BUFFER_SIZE))
}

fn write_observation(env: &Env, index: u32, obs: &TWAPObservation) {
    env.storage()
        .persistent()
        .set(&DataKey::TWAPObservation(index % TWAP_BUFFER_SIZE), obs);
}

pub fn initialize_twap(
    env: &Env,
    tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
    liquidity: i128,
) {
    let obs = TWAPObservation {
        timestamp: env.ledger().timestamp(),
        tick,
        tick_cumulative: 0,
        fee_growth_global_0,
        fee_growth_global_1,
        liquidity,
    };
    
    write_observation(env, 0, &obs);
    set_newest_index(env, 0);
    set_twap_initialized(env);
}

pub fn update_twap(
    env: &Env,
    new_tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
    liquidity: i128,
) {
    if !is_twap_initialized(env) {
        initialize_twap(env, new_tick, fee_growth_global_0, fee_growth_global_1, liquidity);
        return;
    }
    
    let current_index = get_newest_index(env);
    let last_obs = read_observation(env, current_index)
        .expect("twap: observation not found");
    
    let current_time = env.ledger().timestamp();
    let time_elapsed = current_time.saturating_sub(last_obs.timestamp);
    
    let tick_contribution = (last_obs.tick as i64)
        .saturating_mul(time_elapsed as i64);
    
    let new_tick_cumulative = last_obs.tick_cumulative
        .saturating_add(tick_contribution);
    
    let new_obs = TWAPObservation {
        timestamp: current_time,
        tick: new_tick,
        tick_cumulative: new_tick_cumulative,
        fee_growth_global_0,
        fee_growth_global_1,
        liquidity,
    };
    
    let next_index = (current_index + 1) % TWAP_BUFFER_SIZE;
    write_observation(env, next_index, &new_obs);
    set_newest_index(env, next_index);
}

pub fn get_twap_tick(env: &Env, seconds_ago: u64) -> Option<i32> {
    if !is_twap_initialized(env) {
        return None;
    }
    
    let current_index = get_newest_index(env);
    let newest_obs = read_observation(env, current_index)?;
    
    let target_time = env.ledger().timestamp().saturating_sub(seconds_ago);
    let oldest_obs = find_observation_at_time(env, target_time)?;
    
    let time_diff = newest_obs.timestamp.saturating_sub(oldest_obs.timestamp);
    if time_diff == 0 {
        return Some(newest_obs.tick);
    }
    
    let tick_diff = newest_obs.tick_cumulative
        .saturating_sub(oldest_obs.tick_cumulative);
    
    let avg_tick = tick_diff / (time_diff as i64);
    Some(avg_tick as i32)
}

fn find_observation_at_time(env: &Env, target_time: u64) -> Option<TWAPObservation> {
    let newest_index = get_newest_index(env);
    let newest_obs = read_observation(env, newest_index)?;
    
    if target_time >= newest_obs.timestamp {
        return Some(newest_obs);
    }
    
    let mut best_obs = newest_obs.clone();
    let mut best_diff = newest_obs.timestamp.saturating_sub(target_time);
    
    for i in 1..TWAP_BUFFER_SIZE {
        let index = (newest_index + TWAP_BUFFER_SIZE - i) % TWAP_BUFFER_SIZE;
        
        if let Some(obs) = read_observation(env, index) {
            let diff = if obs.timestamp > target_time {
                obs.timestamp - target_time
            } else {
                target_time - obs.timestamp
            };
            
            if diff < best_diff {
                best_diff = diff;
                best_obs = obs.clone();
            }
            
            if obs.timestamp <= target_time {
                break;
            }
        }
    }
    
    Some(best_obs)
}

pub fn get_observations_for_fee_calculation(
    env: &Env,
    position_last_update_time: u64,
) -> Vec<TWAPObservation> {
    let mut result = Vec::new(env);
    
    if !is_twap_initialized(env) {
        return result;
    }
    
    let newest_index = get_newest_index(env);
    
    for i in 0..TWAP_BUFFER_SIZE {
        let index = (newest_index + TWAP_BUFFER_SIZE - i) % TWAP_BUFFER_SIZE;
        
        if let Some(obs) = read_observation(env, index) {
            if obs.timestamp >= position_last_update_time {
                result.push_front(obs);
            } else {
                break;
            }
        }
    }
    
    result
}

pub fn calculate_fees_from_twap(
    env: &Env,
    liquidity: i128,
    lower_tick: i32,
    upper_tick: i32,
    last_fee_growth_inside_0: u128,
    last_fee_growth_inside_1: u128,
    last_update_time: u64,
) -> (u128, u128) {
    if liquidity <= 0 {
        return (0, 0);
    }
    
    let observations = get_observations_for_fee_calculation(env, last_update_time);
    
    if observations.is_empty() {
        return (0, 0);
    }
    
    let liquidity_u = liquidity as u128;
    let mut total_fee_0 = 0u128;
    let mut total_fee_1 = 0u128;
    
    // Start from last known checkpoint
    let mut last_global_0 = last_fee_growth_inside_0;
    let mut last_global_1 = last_fee_growth_inside_1;
    
    // Process each observation chronologically
    for obs in observations.iter() {
        // Check if position was active at this observation
        let position_was_active = obs.tick >= lower_tick && obs.tick < upper_tick;
        
        if position_was_active {
            // Position was active - accumulate fees
            let current_global_0 = obs.fee_growth_global_0;
            let current_global_1 = obs.fee_growth_global_1;
            
            // Calculate delta since last update
            if current_global_0 >= last_global_0 {
                let delta_0 = current_global_0 - last_global_0;
                let fee_0 = (liquidity_u * delta_0) >> 64;
                total_fee_0 = total_fee_0.saturating_add(fee_0);
            }
            
            if current_global_1 >= last_global_1 {
                let delta_1 = current_global_1 - last_global_1;
                let fee_1 = (liquidity_u * delta_1) >> 64;
                total_fee_1 = total_fee_1.saturating_add(fee_1);
            }
            
            // Update checkpoints
            last_global_0 = current_global_0;
            last_global_1 = current_global_1;
        } else {
            // Position went inactive at this observation
            // Stop accumulating - we've captured all fees while active
            break;
        }
    }
    
    (total_fee_0, total_fee_1)
}

pub fn get_latest_observation(env: &Env) -> Option<TWAPObservation> {
    if !is_twap_initialized(env) {
        return None;
    }
    
    let index = get_newest_index(env);
    read_observation(env, index)
}

pub fn get_observation_count(env: &Env) -> u32 {
    if !is_twap_initialized(env) {
        return 0;
    }
    
    let mut count = 0;
    for i in 0..TWAP_BUFFER_SIZE {
        if read_observation(env, i).is_some() {
            count += 1;
        } else {
            break;
        }
    }
    count
}