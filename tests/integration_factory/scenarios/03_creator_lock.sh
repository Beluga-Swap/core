#!/bin/bash

# ============================================================
# SCENARIO 03: CREATOR LOCK
# ============================================================
# Location: tests/integration_factory/scenarios/03_creator_lock.sh
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/common.sh"
source "$SCRIPT_DIR/../helpers/assert.sh"

FACTORY_ID=$(load_state "factory_id")
POOL_ID=$(load_state "pool_usdc_xlm_30")
ADMIN_ADDRESS=$(get_admin_address)

log_section "SCENARIO 03: Creator Lock"

# Test 1: Get creator lock info
log_info "Test: Query creator lock"
LOCK_INFO=$(soroban contract invoke \
    --id "$FACTORY_ID" \
    --network "$NETWORK" \
    --source admin \
    -- get_creator_lock \
    --pool "$POOL_ID" \
    --creator "$ADMIN_ADDRESS" 2>&1)

assert_contains "$LOCK_INFO" "is_permanent" "Lock info contains is_permanent"
assert_contains "$LOCK_INFO" "liquidity" "Lock info contains liquidity"

# Test 2: Try to unlock permanent lock (should fail)
log_info "Test: Permanent lock cannot be unlocked"
assert_fails "Unlock permanent lock rejected" \
    soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- unlock_creator_liquidity \
        --pool_address "$POOL_ID" \
        --creator "$ADMIN_ADDRESS"

log_success "Scenario 03 completed"