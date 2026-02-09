#!/bin/bash

# ============================================================
# BELUGASWAP FACTORY - INTEGRATION TEST (TESTNET)
# ============================================================
# Location: tests/integration_factory/test_factory_integration.sh
#
# Full end-to-end integration testing with deployment to testnet.
#
# Usage:
#   cd tests/integration_factory
#   ./test_factory_integration.sh           # Run all
#   ./test_factory_integration.sh quick     # Skip deploy
#   ./test_factory_integration.sh cleanup   # Clean state
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Load helpers
source "$SCRIPT_DIR/helpers/common.sh"
source "$SCRIPT_DIR/helpers/deploy.sh"
source "$SCRIPT_DIR/helpers/assert.sh"

# Config
NETWORK="${NETWORK:-testnet}"
STATE_DIR="$SCRIPT_DIR/.state"
SCENARIOS_DIR="$SCRIPT_DIR/scenarios"

main() {
    case "${1:-all}" in
        quick)
            log_section "QUICK TEST (Skip Deploy)"
            check_state
            run_scenarios
            generate_report
            ;;
        cleanup)
            cleanup_state
            ;;
        *)
            log_section "FULL INTEGRATION TEST"
            setup_environment
            build_contracts
            deploy_contracts
            run_scenarios
            generate_report
            ;;
    esac
    
    log_success "Integration tests completed! 🎉"
}

setup_environment() {
    log_info "Setting up..."
    check_dependencies
    mkdir -p "$STATE_DIR" "$STATE_DIR/logs"
    setup_keypairs
    log_success "Environment ready"
}

check_dependencies() {
    for dep in stellar soroban jq curl; do
        command -v "$dep" &> /dev/null || {
            log_error "$dep not found"
            exit 1
        }
    done
}

setup_keypairs() {
    [ -f "$STATE_DIR/admin.json" ] && return
    
    log_info "Generating admin keypair..."
    stellar keys generate admin --network "$NETWORK"
    stellar keys show admin > "$STATE_DIR/admin.json"
    
    if [ "$NETWORK" == "testnet" ]; then
        ADMIN_ADDRESS=$(stellar keys address admin)
        curl -X POST "https://friendbot.stellar.org?addr=$ADMIN_ADDRESS" \
            > "$STATE_DIR/logs/friendbot.log" 2>&1 || true
        sleep 2
    fi
}

build_contracts() {
    log_info "Building contracts..."
    cd "$PROJECT_ROOT"
    
    cd contracts/factory
    cargo build --target wasm32-unknown-unknown --release \
        > "$STATE_DIR/logs/build_factory.log" 2>&1
    cd ../..
    
    cd contracts/pool
    cargo build --target wasm32-unknown-unknown --release \
        > "$STATE_DIR/logs/build_pool.log" 2>&1
    cd ../..
    
    log_success "Contracts built"
}

deploy_contracts() {
    log_info "Deploying to $NETWORK..."
    cd "$PROJECT_ROOT"
    deploy_pool_wasm
    deploy_factory_contract
    initialize_factory
    log_success "Deployed"
}

run_scenarios() {
    log_section "RUNNING SCENARIOS"
    
    local passed=0 failed=0
    
    for scenario in "$SCENARIOS_DIR"/*.sh; do
        [ -f "$scenario" ] || continue
        
        local name=$(basename "$scenario")
        log_info "Running $name..."
        
        if bash "$scenario"; then
            log_success "$name passed"
            ((passed++))
        else
            log_error "$name failed"
            ((failed++))
        fi
    done
    
    echo "$passed" > "$STATE_DIR/passed.txt"
    echo "$failed" > "$STATE_DIR/failed.txt"
    
    [ "$failed" -eq 0 ] || return 1
}

generate_report() {
    log_section "REPORT"
    
    local passed=$(cat "$STATE_DIR/passed.txt" 2>/dev/null || echo "0")
    local failed=$(cat "$STATE_DIR/failed.txt" 2>/dev/null || echo "0")
    
    cat > "$STATE_DIR/test_report.txt" <<EOF
============================================================
BELUGASWAP FACTORY - INTEGRATION TEST REPORT
============================================================
Date: $(date)
Network: $NETWORK
Factory: $(cat "$STATE_DIR/factory_id.txt" 2>/dev/null || echo "N/A")
Results: $passed passed, $failed failed
Status: $([ "$failed" -eq 0 ] && echo "✓ PASS" || echo "✗ FAIL")
============================================================
EOF
    
    cat "$STATE_DIR/test_report.txt"
}

check_state() {
    [ -f "$STATE_DIR/factory_id.txt" ] || {
        log_error "No factory deployed. Run full test first."
        exit 1
    }
}

cleanup_state() {
    rm -rf "$STATE_DIR"
    log_success "State cleaned"
}

main "$@"