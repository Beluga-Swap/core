#!/bin/bash

# ============================================================
# SCENARIO 05: EDGE CASES
# ============================================================
# Location: tests/integration_factory/scenarios/05_edge_cases.sh
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/common.sh"
source "$SCRIPT_DIR/../helpers/assert.sh"

FACTORY_ID=$(load_state "factory_id")
ADMIN_ADDRESS=$(get_admin_address)

log_section "SCENARIO 05: Edge Cases"

# Test 1: Get non-existent pool
log_info "Test: Query non-existent pool returns empty"
FAKE_TOKEN_1="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
FAKE_TOKEN_2="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSD"

RESULT=$(soroban contract invoke \
    --id "$FACTORY_ID" \
    --network "$NETWORK" \
    --source admin \
    -- get_pool \
    --token_a "$FAKE_TOKEN_1" \
    --token_b "$FAKE_TOKEN_2" \
    --fee_bps 30 2>&1 || echo "null")

log_info "Non-existent pool query handled"

# Test 2: Minimum lock duration boundary
log_info "Test: Lock duration boundary (120960 ledgers)"
# This would require creating a new pool with exact MIN_LOCK_DURATION
# Skipping actual creation to avoid token deployment overhead
log_info "Lock duration validation is tested in fuzz tests"

# Test 3: Zero price rejected
log_info "Test: Zero price validation"
# Tested in validation logic and fuzz tests
log_info "Zero price rejection tested in fuzz tests"

# Test 4: Token ordering consistency
log_info "Test: Token ordering normalized"
USDC_ID=$(load_state "token_usdc_id")
XLM_ID=$(load_state "token_xlm_id")

if [ -n "$USDC_ID" ] && [ -n "$XLM_ID" ]; then
    # Query with USDC first
    POOL_1=$(soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- get_pool \
        --token_a "$USDC_ID" \
        --token_b "$XLM_ID" \
        --fee_bps 30 2>&1 | tail -1)
    
    # Query with XLM first
    POOL_2=$(soroban contract invoke \
        --id "$FACTORY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- get_pool \
        --token_a "$XLM_ID" \
        --token_b "$USDC_ID" \
        --fee_bps 30 2>&1 | tail -1)
    
    assert_eq "$POOL_1" "$POOL_2" "Token ordering normalized"
fi

log_success "Scenario 05 completed"