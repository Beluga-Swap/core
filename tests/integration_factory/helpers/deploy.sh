#!/bin/bash

# ============================================================
# DEPLOYMENT HELPER FUNCTIONS
# ============================================================
# Location: tests/integration_factory/helpers/deploy.sh
# ============================================================

# Deploy pool WASM and save hash
deploy_pool_wasm() {
    log_info "Installing pool WASM..."
    
    local pool_wasm="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/belugaswap_pool.wasm"
    
    if [ ! -f "$pool_wasm" ]; then
        log_error "Pool WASM not found: $pool_wasm"
        return 1
    fi
    
    local wasm_hash=$(soroban contract install \
        --wasm "$pool_wasm" \
        --network "$NETWORK" \
        --source admin 2>&1 | tail -1)
    
    if [ -z "$wasm_hash" ]; then
        log_error "Failed to install pool WASM"
        return 1
    fi
    
    save_state "pool_wasm_hash" "$wasm_hash"
    log_success "Pool WASM installed: $wasm_hash"
}

# Deploy factory contract
deploy_factory_contract() {
    log_info "Deploying factory contract..."
    
    local factory_wasm="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/belugaswap_factory.wasm"
    
    if [ ! -f "$factory_wasm" ]; then
        log_error "Factory WASM not found: $factory_wasm"
        return 1
    fi
    
    local factory_id=$(soroban contract deploy \
        --wasm "$factory_wasm" \
        --network "$NETWORK" \
        --source admin 2>&1 | tail -1)
    
    if [ -z "$factory_id" ]; then
        log_error "Failed to deploy factory"
        return 1
    fi
    
    save_state "factory_id" "$factory_id"
    log_success "Factory deployed: $factory_id"
}

# Initialize factory
initialize_factory() {
    log_info "Initializing factory..."
    
    local factory_id=$(load_state "factory_id")
    local pool_wasm_hash=$(load_state "pool_wasm_hash")
    local admin_address=$(get_admin_address)
    
    if [ -z "$factory_id" ] || [ -z "$pool_wasm_hash" ] || [ -z "$admin_address" ]; then
        log_error "Missing deployment prerequisites"
        return 1
    fi
    
    soroban contract invoke \
        --id "$factory_id" \
        --network "$NETWORK" \
        --source admin \
        -- initialize \
        --admin "$admin_address" \
        --pool_wasm_hash "$pool_wasm_hash" \
        > "$STATE_DIR/logs/init_factory.log" 2>&1
    
    if [ $? -ne 0 ]; then
        log_error "Factory initialization failed"
        cat "$STATE_DIR/logs/init_factory.log"
        return 1
    fi
    
    log_success "Factory initialized"
}

# Deploy test token
deploy_test_token() {
    local name="$1"
    local symbol="$2"
    local decimals="${3:-7}"
    
    log_info "Deploying $symbol token..."
    
    # Use Stellar test token contract
    local token_wasm="$PROJECT_ROOT/contracts/token/soroban_token_contract.wasm"
    
    if [ ! -f "$token_wasm" ]; then
        # Download if not exists
        wget -q https://github.com/stellar/soroban-examples/raw/main/token/target/wasm32-unknown-unknown/release/soroban_token_contract.wasm \
            -O "$token_wasm"
    fi
    
    local token_id=$(soroban contract deploy \
        --wasm "$token_wasm" \
        --network "$NETWORK" \
        --source admin 2>&1 | tail -1)
    
    if [ -z "$token_id" ]; then
        log_error "Failed to deploy $symbol token"
        return 1
    fi
    
    # Initialize token
    local admin_address=$(get_admin_address)
    
    soroban contract invoke \
        --id "$token_id" \
        --network "$NETWORK" \
        --source admin \
        -- initialize \
        --admin "$admin_address" \
        --decimal "$decimals" \
        --name "$name" \
        --symbol "$symbol" \
        > /dev/null 2>&1
    
    save_state "token_${symbol,,}_id" "$token_id"
    log_success "$symbol token deployed: $token_id"
    
    echo "$token_id"
}

# Mint test tokens
mint_test_token() {
    local symbol="$1"
    local amount="$2"
    local recipient="${3:-$(get_admin_address)}"
    
    local token_id=$(load_state "token_${symbol,,}_id")
    
    if [ -z "$token_id" ]; then
        log_error "Token $symbol not found"
        return 1
    fi
    
    soroban contract invoke \
        --id "$token_id" \
        --network "$NETWORK" \
        --source admin \
        -- mint \
        --to "$recipient" \
        --amount "$amount" \
        > /dev/null 2>&1
    
    log_info "Minted $amount $symbol to $recipient"
}