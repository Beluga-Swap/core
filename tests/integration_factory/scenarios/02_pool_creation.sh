#!/bin/bash

# ============================================================
# SCENARIO 02: POOL CREATION
# ============================================================
# Location: tests/integration_factory/scenarios/02_pool_creation.sh
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/common.sh"
source "$SCRIPT_DIR/../helpers/deploy.sh"
source "$SCRIPT_DIR/../helpers/assert.sh"

FACTORY_ID=$(load_state "factory_id")
ADMIN_ADDRESS=$(get_admin_address)

log_section "SCENARIO 02: Pool Creation"

# Deploy test tokens
log_info "Deploying test tokens..."
USDC_ID=$(deploy_test_token "USD Coin" "USDC" 7)
XLM_ID=$(deploy_test_token "Stellar Lumens" "XLM" 7)

# Mint tokens
mint_test_token "USDC" 1000000000
mint_test_token "XLM" 1000000000

# Test 1: Create USDC/XLM pool
log_info "Test: Create USDC/XLM pool (30 bps)"

SQRT_PRICE=$((1 << 64))  # 1:1 price

POOL_RESULT=$(soroban contract invoke \
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
        "initial_sqrt_price_x64": "'"$SQRT_PRICE"'",
        "amount0_desired": 10000000,
        "amount1_desired": 10000000,
        "lower_tick": -600,
        "upper_tick": 600,
        "lock_duration": 0
    }' 2>&1)

POOL_ID=$(echo "$POOL_RESULT" | jq -r '.result // .' | tail -1)

assert_not_empty "$POOL_ID" "Pool created successfully"
save_state "pool_usdc_xlm_30" "$POOL_ID"

# Test 2: Get pool address
log_info "Test: Query pool address"
POOL_ADDR=$(soroban contract invoke \
    --id "$FACTORY_ID" \
    --network "$NETWORK" \
    --source admin \
    -- get_pool \
    --token_a "$USDC_ID" \
    --token_b "$XLM_ID" \
    --fee_bps 30 2>&1 | tail -1)

assert_eq "$POOL_ADDR" "$POOL_ID" "Pool address matches"

# Test 3: Get pool count
log_info "Test: Pool count increased"
POOL_COUNT=$(soroban contract invoke \
    --id "$FACTORY_ID" \
    --network "$NETWORK" \
    --source admin \
    -- get_pool_count 2>&1 | tail -1)

assert_gt "$POOL_COUNT" 0 "Pool count > 0"

log_success "Scenario 02 completed"