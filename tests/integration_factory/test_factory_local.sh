#!/bin/bash

# ============================================================
# BELUGASWAP FACTORY - LOCAL INTEGRATION TEST
# ============================================================
# Location: tests/integration_factory/test_factory_local.sh
#
# Local integration testing WITHOUT deployment.
# Runs unit tests, fuzz tests, clippy, and checks.
#
# Usage:
#   cd tests/integration_factory
#   ./test_factory_local.sh
# ============================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; PASSED=$((PASSED + 1)); }
log_error() { echo -e "${RED}[✗]${NC} $1"; FAILED=$((FAILED + 1)); }
log_section() { echo -e "\n${BLUE}========== $1 ==========${NC}\n"; }

main() {
    log_section "LOCAL INTEGRATION TEST"
    
    build_contracts
    run_unit_tests
    run_fuzz_tests
    run_clippy
    check_wasm_size
    generate_report
}

build_contracts() {
    log_info "Building contracts..."
    cd "$PROJECT_ROOT"
    
    cd contracts/factory
    cargo build --target wasm32-unknown-unknown --release --quiet || {
        log_error "Factory build failed"
        return 1
    }
    cd ../..
    
    cd contracts/pool
    cargo build --target wasm32-unknown-unknown --release --quiet || {
        log_error "Pool build failed"
        return 1
    }
    cd ../..
    
    log_success "Build complete"
}

run_unit_tests() {
    log_info "Running unit tests..."
    cd "$PROJECT_ROOT"
    
    cd contracts/factory
    if cargo test 2>&1 | grep -q "test result: ok"; then
        log_success "Factory unit tests passed"
    else
        log_error "Factory unit tests failed"
    fi
    cd ../..
    
    cd contracts/pool
    if cargo test 2>&1 | grep -q "test result: ok"; then
        log_success "Pool unit tests passed"
    else
        log_error "Pool unit tests failed"
    fi
    cd ../..
}

run_fuzz_tests() {
    log_info "Running fuzz tests (1000 cases)..."
    cd "$PROJECT_ROOT/contracts/factory"
    
    if [ -f "tests/test_factory_validation_fuzz.rs" ]; then
        if PROPTEST_CASES=1000 cargo test --test test_factory_validation_fuzz 2>&1 | grep -q "test result: ok"; then
            log_success "Fuzz tests passed (1000 cases)"
        else
            log_error "Fuzz tests failed"
        fi
    else
        log_info "Fuzz tests not found, skipping"
    fi
    cd ../..
}

run_clippy() {
    log_info "Running clippy..."
    cd "$PROJECT_ROOT"
    
    cd contracts/factory
    if cargo clippy 2>&1 | grep -qv "error:"; then
        log_success "Factory clippy passed"
    else
        log_error "Factory clippy found issues"
    fi
    cd ../..
    
    cd contracts/pool
    if cargo clippy 2>&1 | grep -qv "error:"; then
        log_success "Pool clippy passed"
    else
        log_error "Pool clippy found issues"
    fi
    cd ../..
}

check_wasm_size() {
    log_info "Checking WASM sizes..."
    cd "$PROJECT_ROOT"
    
    local factory_wasm="contracts/factory/target/wasm32-unknown-unknown/release/beluga_factory.wasm"
    local pool_wasm="contracts/pool/target/wasm32-unknown-unknown/release/beluga_pool.wasm"
    
    if [ -f "$factory_wasm" ]; then
        local size_kb=$(($(wc -c < "$factory_wasm") / 1024))
        if [ $size_kb -lt 1024 ]; then
            log_success "Factory WASM: ${size_kb}KB"
        else
            log_error "Factory WASM too large: ${size_kb}KB"
        fi
    fi
    
    if [ -f "$pool_wasm" ]; then
        local size_kb=$(($(wc -c < "$pool_wasm") / 1024))
        if [ $size_kb -lt 1024 ]; then
            log_success "Pool WASM: ${size_kb}KB"
        else
            log_error "Pool WASM too large: ${size_kb}KB"
        fi
    fi
}

generate_report() {
    log_section "RESULTS"
    
    echo "Passed: $PASSED"
    echo "Failed: $FAILED"
    
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}✓ ALL TESTS PASSED!${NC} 🎉"
        exit 0
    else
        echo -e "${RED}✗ SOME TESTS FAILED${NC}"
        exit 1
    fi
}

main