#!/bin/bash

# ============================================================
# SCENARIO 01: INITIALIZATION
# ============================================================
# Location: tests/integration_factory/scenarios/01_initialization.sh
#
# Tests factory initialization and fee tier setup.
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/common.sh"
source "$SCRIPT_DIR/../helpers/assert.sh"

FACTORY_ID=$(load_state "factory_id")
ADMIN_ADDRESS=$(get_admin_address)

log_section "SCENARIO 01: Initialization"

# Test 1: Factory should be initialized
log_info "Test: Factory is initialized"
assert_not_empty "$FACTORY_ID" "Factory ID should exist"

# Test 2: Configure fee tier 5 bps
log_info "Test: Set fee tier 5 bps"
assert_success "Fee tier 5 bps configured" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- set_fee_tier \
        --fee_bps 5 \
        --tick_spacing 10 \
        --enabled true

# Test 3: Configure fee tier 30 bps
log_info "Test: Set fee tier 30 bps"
assert_success "Fee tier 30 bps configured" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- set_fee_tier \
        --fee_bps 30 \
        --tick_spacing 60 \
        --enabled true

# Test 4: Configure fee tier 100 bps
log_info "Test: Set fee tier 100 bps"
assert_success "Fee tier 100 bps configured" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- set_fee_tier \
        --fee_bps 100 \
        --tick_spacing 200 \
        --enabled true

# Test 5: Double initialization should fail
log_info "Test: Double initialization rejected"
POOL_WASM_HASH=$(load_state "pool_wasm_hash")
assert_fails "Double initialization rejected" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --pool_wasm_hash "$POOL_WASM_HASH"

log_success "Scenario 01 completed"