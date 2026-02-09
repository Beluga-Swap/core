#!/bin/bash

# ============================================================
# COMMON HELPER FUNCTIONS
# ============================================================
# Location: tests/integration_factory/helpers/common.sh
#
# Shared utilities for integration tests.
# ============================================================

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN} $1${NC}"
    echo -e "${CYAN}============================================================${NC}"
    echo ""
}

# Check if command exists
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 not found. Please install it."
        return 1
    fi
    return 0
}

# Load state value
load_state() {
    local key="$1"
    local file="$STATE_DIR/${key}.txt"
    
    if [ -f "$file" ]; then
        cat "$file"
    else
        echo ""
    fi
}

# Save state value
save_state() {
    local key="$1"
    local value="$2"
    local file="$STATE_DIR/${key}.txt"
    
    echo "$value" > "$file"
}

# Get admin address
get_admin_address() {
    stellar keys address admin 2>/dev/null || echo ""
}

# Wait for ledger
wait_ledgers() {
    local count="${1:-1}"
    log_info "Waiting for $count ledger(s)..."
    sleep $((count * 5))
}

# Retry command
retry() {
    local max_attempts="$1"
    shift
    local cmd="$@"
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if eval "$cmd"; then
            return 0
        fi
        
        log_warn "Attempt $attempt/$max_attempts failed, retrying..."
        sleep 2
        ((attempt++))
    done
    
    log_error "Command failed after $max_attempts attempts"
    return 1
}

# Format amount (from raw to human readable)
format_amount() {
    local amount="$1"
    local decimals="${2:-7}"
    
    echo "scale=$decimals; $amount / 10^$decimals" | bc
}

# Parse JSON value
json_value() {
    local json="$1"
    local key="$2"
    
    echo "$json" | jq -r ".$key"
}