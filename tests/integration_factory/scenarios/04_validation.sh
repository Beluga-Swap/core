#!/bin/bash

# ============================================================
# SCENARIO 04: VALIDATION
# ============================================================
# Location: tests/integration_factory/scenarios/04_validation.sh
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/common.sh"
source "$SCRIPT_DIR/../helpers/assert.sh"

FACTORY_ID=$(load_state "factory_id")
USDC_ID=$(load_state "token_usdc_id")
XLM_ID=$(load_state "token_xlm_id")
ADMIN_ADDRESS=$(get_admin_address)

log_section "SCENARIO 04: Validation"

# Test 1: Same token pair rejected
log_info "Test: Same token pair rejected"
assert_fails "Same token rejected" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- create_pool \
        --creator "$ADMIN_ADDRESS" \
        --params '{
            "token_a": "'"$USDC_ID"'",
            "token_b": "'"$USDC_ID"'",
            "fee_bps": 30,
            "creator_fee_bps": 100,
            "initial_sqrt_price_x64": "'$((1 << 64))'",
            "amount0_desired": 10000000,
            "amount1_desired": 10000000,
            "lower_tick": -600,
            "upper_tick": 600,
            "lock_duration": 0
        }'

# Test 2: Invalid fee tier rejected
log_info "Test: Invalid fee tier rejected"
assert_fails "Invalid fee tier rejected" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- create_pool \
        --creator "$ADMIN_ADDRESS" \
        --params '{
            "token_a": "'"$USDC_ID"'",
            "token_b": "'"$XLM_ID"'",
            "fee_bps": 999,
            "creator_fee_bps": 100,
            "initial_sqrt_price_x64": "'$((1 << 64))'",
            "amount0_desired": 10000000,
            "amount1_desired": 10000000,
            "lower_tick": -600,
            "upper_tick": 600,
            "lock_duration": 0
        }'

# Test 3: Duplicate pool rejected
log_info "Test: Duplicate pool rejected"
assert_fails "Duplicate pool rejected" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- create_pool \
        --creator "$ADMIN_ADDRESS" \
        --params '{
            "token_a": "'"$USDC_ID"'",
            "token_b": "'"$XLM_ID"'",
            "fee_bps": 30,
            "creator_fee_bps": 100,
            "initial_sqrt_price_x64": "'$((1 << 64))'",
            "amount0_desired": 10000000,
            "amount1_desired": 10000000,
            "lower_tick": -600,
            "upper_tick": 600,
            "lock_duration": 0
        }'

# Test 4: Invalid creator fee rejected
log_info "Test: Invalid creator fee rejected (too low)"
assert_fails "Creator fee too low rejected" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- create_pool \
        --creator "$ADMIN_ADDRESS" \
        --params '{
            "token_a": "'"$USDC_ID"'",
            "token_b": "'"$XLM_ID"'",
            "fee_bps": 100,
            "creator_fee_bps": 5,
            "initial_sqrt_price_x64": "'$((1 << 64))'",
            "amount0_desired": 10000000,
            "amount1_desired": 10000000,
            "lower_tick": -600,
            "upper_tick": 600,
            "lock_duration": 0
        }'

log_success "Scenario 04 completed"