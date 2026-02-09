#!/bin/bash

# ============================================================
# ASSERTION HELPER FUNCTIONS
# ============================================================
# Location: tests/integration_factory/helpers/assert.sh
# ============================================================

# Assert equals
assert_eq() {
    local actual="$1"
    local expected="$2"
    local message="${3:-Assertion failed}"
    
    if [ "$actual" == "$expected" ]; then
        log_success "$message: '$actual' == '$expected'"
        return 0
    else
        log_error "$message: Expected '$expected', got '$actual'"
        return 1
    fi
}

# Assert not equals
assert_ne() {
    local actual="$1"
    local expected="$2"
    local message="${3:-Assertion failed}"
    
    if [ "$actual" != "$expected" ]; then
        log_success "$message"
        return 0
    else
        log_error "$message: Values should not be equal: '$actual'"
        return 1
    fi
}

# Assert not empty
assert_not_empty() {
    local value="$1"
    local message="${2:-Value should not be empty}"
    
    if [ -n "$value" ]; then
        log_success "$message"
        return 0
    else
        log_error "$message: Value is empty"
        return 1
    fi
}

# Assert empty
assert_empty() {
    local value="$1"
    local message="${2:-Value should be empty}"
    
    if [ -z "$value" ]; then
        log_success "$message"
        return 0
    else
        log_error "$message: Value is not empty: '$value'"
        return 1
    fi
}

# Assert success (command exits with 0)
assert_success() {
    local message="${1:-Command should succeed}"
    shift
    local cmd="$@"
    
    if eval "$cmd" > /dev/null 2>&1; then
        log_success "$message"
        return 0
    else
        log_error "$message: Command failed"
        return 1
    fi
}

# Assert failure (command exits with non-zero)
assert_fails() {
    local message="${1:-Command should fail}"
    shift
    local cmd="$@"
    
    if eval "$cmd" > /dev/null 2>&1; then
        log_error "$message: Command succeeded when it should fail"
        return 1
    else
        log_success "$message"
        return 0
    fi
}

# Assert contains
assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-String should contain substring}"
    
    if echo "$haystack" | grep -q "$needle"; then
        log_success "$message"
        return 0
    else
        log_error "$message: '$haystack' does not contain '$needle'"
        return 1
    fi
}

# Assert greater than
assert_gt() {
    local actual="$1"
    local expected="$2"
    local message="${3:-Value should be greater than}"
    
    if [ "$actual" -gt "$expected" ]; then
        log_success "$message: $actual > $expected"
        return 0
    else
        log_error "$message: $actual is not > $expected"
        return 1
    fi
}

# Assert less than
assert_lt() {
    local actual="$1"
    local expected="$2"
    local message="${3:-Value should be less than}"
    
    if [ "$actual" -lt "$expected" ]; then
        log_success "$message: $actual < $expected"
        return 0
    else
        log_error "$message: $actual is not < $expected"
        return 1
    fi
}

# Assert file exists
assert_file_exists() {
    local file="$1"
    local message="${2:-File should exist}"
    
    if [ -f "$file" ]; then
        log_success "$message: $file"
        return 0
    else
        log_error "$message: $file not found"
        return 1
    fi
}