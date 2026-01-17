# BelugaSwap Swap Package

High-performance swap execution engine for concentrated liquidity AMM, implementing Uniswap V3-style multi-tick traversal and fee distribution.

## 📋 Table of Contents

- [Overview](#-overview)
- [How Swaps Work](#-how-swaps-work)
- [Swap Engine Architecture](#-swap-engine-architecture)
- [Fee Distribution](#-fee-distribution)
- [Functions Reference](#-functions-reference)
- [Usage Examples](#-usage-examples)
- [Advanced Topics](#-advanced-topics)

---

## 🌊 Overview

The Swap package provides the core swap execution engine that handles token exchanges in concentrated liquidity pools. It manages:

- **Multi-tick traversal**: Navigate through multiple price ranges
- **Fee calculation**: Calculate and distribute trading fees
- **Price impact**: Track price movement during swaps
- **Slippage protection**: Ensure swaps meet minimum output requirements
- **Dry-run simulation**: Preview swaps without executing

### Key Features

- **Zero-for-one / One-for-zero**: Bidirectional swap support
- **Price limits**: Optional maximum price slippage
- **Creator fees**: Automatic fee splitting between LPs and creators
- **Efficient iteration**: Optimized tick crossing algorithm
- **Read-only quotes**: Gas-free swap previews

---

## 🔄 How Swaps Work

### Basic Swap Flow

```
1. User wants to swap 100 token0 → token1
2. Engine calculates fee: 100 * 0.3% = 0.3 token0
3. Available for swap: 99.7 token0
4. Price moves along liquidity curve
5. Output calculated: ~99.4 token1 (example)
6. Fees distributed to LPs and creator
```

### Multi-Tick Traversal

When liquidity is concentrated, a swap may cross multiple ticks:

```
Initial state:
  Price = 1.0 (tick 0)
  Liquidity = 10M

Swap 1000 token0:

Step 1: Use liquidity at tick 0
  - Consume: 500 token0
  - Output: 499 token1
  - Price moves to tick -60 (next tick boundary)
  - Cross tick

Step 2: New liquidity at tick -60
  - Consume: 500 token0
  - Output: 497 token1
  - Price moves to tick -120
  
Total: 1000 token0 → 996 token1
```

### Price Impact

```
Price Impact = (amount_in - amount_out) / amount_in * 100%

Example:
  Swap 100 token0 → 98 token1
  Price impact = (100 - 98) / 100 = 2%
```

---

## 🏗️ Swap Engine Architecture

### SwapState

The swap engine operates on a minimal state structure:

```rust
pub struct SwapState {
    pub sqrt_price_x64: u128,       // Current price
    pub current_tick: i32,           // Current tick
    pub liquidity: i128,             // Active liquidity
    pub tick_spacing: i32,           // Tick spacing
    pub fee_growth_global_0: u128,   // Fee accumulator token0
    pub fee_growth_global_1: u128,   // Fee accumulator token1
}
```

### Swap Direction

```
zero_for_one = true:
  Selling token0 for token1
  Price decreases (moves left)
  
zero_for_one = false:
  Selling token1 for token0
  Price increases (moves right)
```

### Main Loop Algorithm

```rust
while iterations < MAX_ITERATIONS {
    // 1. Check exit conditions
    if amount_remaining <= 0 { break; }
    if price_limit_reached { break; }
    
    // 2. Find next initialized tick
    next_tick = find_next_tick(current_tick, direction);
    
    // 3. Calculate swap to next tick
    (new_price, amount_in, amount_out) = swap_step(
        current_price,
        next_tick_price,
        amount_remaining
    );
    
    // 4. Deduct fee
    fee = calculate_fee(amount_in);
    amount_remaining -= amount_in + fee;
    
    // 5. Cross tick if reached
    if price == next_tick_price {
        cross_tick(next_tick);
        update_liquidity();
    }
    
    // 6. Update state
    current_price = new_price;
    total_output += amount_out;
}
```

---

## 💰 Fee Distribution

### Fee Structure

```
Total Swap Cost = Input Amount + Fee

Fee = Input Amount * fee_bps / 10000

Example with 0.3% fee:
  Input: 100 tokens
  Fee: 100 * 30 / 10000 = 0.3 tokens
  Total: 100.3 tokens
```

### Fee Splitting

Fees are split between LPs and pool creator:

```
Total Fee = 0.3 tokens (0.3% of 100)

If creator_fee_bps = 100 (1% of total fee):
  Creator Fee = 0.3 * 100 / 10000 = 0.003 tokens
  LP Fee = 0.3 - 0.003 = 0.297 tokens

Distribution:
  99% → LPs (proportional to liquidity)
  1% → Pool Creator
```

### Fee Growth Accounting

Fees are tracked using growth variables in Q64.64 format:

```rust
if liquidity > 0 && lp_fee > 0 {
    growth_delta = (lp_fee * 2^64) / liquidity
    fee_growth_global_0 += growth_delta
}

// Each LP's share:
lp_fees = position.liquidity * growth_delta / 2^64
```

---

## 📚 Functions Reference

### Core Swap Functions

#### `engine_swap`

```rust
pub fn engine_swap<F1, F2, F3>(
    env: &Env,
    state: &mut SwapState,
    read_tick: F1,
    write_tick: F2,
    emit_sync: F3,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
    creator_fee_bps: i128,
) -> (i128, i128)
```

Execute a token swap with full state updates.

**Parameters:**
- `env`: Soroban environment
- `state`: Mutable swap state (will be modified)
- `read_tick`: Callback to read tick info from storage
- `write_tick`: Callback to write tick info to storage
- `emit_sync`: Callback to emit sync events
- `amount_specified`: Input amount to swap
- `zero_for_one`: Swap direction (true = token0→token1)
- `sqrt_price_limit_x64`: Maximum price movement (0 = no limit)
- `fee_bps`: Trading fee in basis points (e.g., 30 = 0.3%)
- `creator_fee_bps`: Creator fee share in basis points (e.g., 100 = 1%)

**Returns:** `(amount_in, amount_out)`
- `amount_in`: Actual input amount consumed
- `amount_out`: Output amount received

**Panics:**
- "swap amount too small": amount < MIN_SWAP_AMOUNT
- "no liquidity available": liquidity <= 0
- "output amount too small": output < MIN_OUTPUT_AMOUNT

**Example:**
```rust
use belugaswap_swap::{SwapState, engine_swap};

let mut state = SwapState {
    sqrt_price_x64: initial_price,
    current_tick: 0,
    liquidity: 10_000_000,
    tick_spacing: 60,
    fee_growth_global_0: 0,
    fee_growth_global_1: 0,
};

let (amount_in, amount_out) = engine_swap(
    &env,
    &mut state,
    |e, t| read_tick_info(e, t),
    |e, t, info| write_tick_info(e, t, info),
    |e, tick, price| emit_sync_event(e, tick, price),
    1_000_000,  // Swap 1 token
    true,       // token0 → token1
    0,          // No price limit
    30,         // 0.3% fee
    100,        // 1% creator fee
);

println!("Swapped {} for {}", amount_in, amount_out);
println!("New price: {}", state.sqrt_price_x64);
println!("New tick: {}", state.current_tick);
```

---

#### `quote_swap`

```rust
pub fn quote_swap<F>(
    env: &Env,
    state: &SwapState,
    read_tick: F,
    amount_in: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
) -> (i128, i128, u128)
```

Preview a swap without executing (read-only simulation).

**Parameters:**
- Same as `engine_swap` but with immutable `state`
- No write callbacks needed
- No creator fee (preview only)

**Returns:** `(amount_in_used, amount_out, final_sqrt_price)`

**Key Differences from engine_swap:**
- Does NOT modify state
- Does NOT write to storage
- Does NOT emit events
- Does NOT panic (returns 0 on error)
- Does NOT charge creator fees

**Example:**
```rust
use belugaswap_swap::quote_swap;

// Preview swap without executing
let (amount_in_used, amount_out, final_price) = quote_swap(
    &env,
    &state,
    |e, t| read_tick_info(e, t),
    1_000_000,  // Want to swap 1 token
    true,       // token0 → token1
    0,          // No price limit
    30,         // 0.3% fee
);

println!("Preview: {} in → {} out", amount_in_used, amount_out);
println!("Price would move to: {}", final_price);

// State is unchanged!
assert_eq!(state.sqrt_price_x64, original_price);
```

---

#### `validate_and_preview_swap`

```rust
pub fn validate_and_preview_swap<F>(
    env: &Env,
    state: &SwapState,
    read_tick: F,
    amount_in: i128,
    min_amount_out: i128,
    zero_for_one: bool,
    sqrt_price_limit_x64: u128,
    fee_bps: i128,
) -> Result<(i128, i128, i128, u128), Symbol>
```

Validate swap parameters and preview results.

**Parameters:**
- `min_amount_out`: Minimum acceptable output (slippage protection)

**Returns:**
- `Ok((amount_in_used, amount_out, fee_paid, final_price))` if valid
- `Err(Symbol)` if validation fails

**Error Symbols:**
- `"AMT_LOW"`: Input amount too small
- `"NO_LIQ"`: No liquidity available
- `"SLIP_HI"`: Output below min_amount_out
- `"OUT_DUST"`: Output amount too small (dust)
- `"SLIP_MAX"`: Price impact exceeds maximum (50%)

**Example:**
```rust
use belugaswap_swap::validate_and_preview_swap;

match validate_and_preview_swap(
    &env,
    &state,
    |e, t| read_tick_info(e, t),
    1_000_000,      // Input amount
    950_000,        // Min output (5% slippage tolerance)
    true,           // token0 → token1
    0,              // No price limit
    30,             // 0.3% fee
) {
    Ok((amount_in, amount_out, fee_paid, final_price)) => {
        println!("Valid swap:");
        println!("  Input: {}", amount_in);
        println!("  Output: {}", amount_out);
        println!("  Fee: {}", fee_paid);
        println!("  Final price: {}", final_price);
    }
    Err(error) => {
        println!("Invalid swap: {:?}", error);
    }
}
```

---

## 🎯 Usage Examples

### Example 1: Simple Swap Execution

```rust
use belugaswap_swap::{SwapState, engine_swap};

// Setup initial state
let mut state = SwapState {
    sqrt_price_x64: 1u128 << 64,  // Price = 1.0
    current_tick: 0,
    liquidity: 10_000_000,
    tick_spacing: 60,
    fee_growth_global_0: 0,
    fee_growth_global_1: 0,
};

// Execute swap: 100 token0 → token1
let (spent, received) = engine_swap(
    &env,
    &mut state,
    |e, t| storage::read_tick_info(e, t),
    |e, t, info| storage::write_tick_info(e, t, info),
    |e, tick, price| events::emit_sync(e, tick, price),
    100_000_000,    // 100 tokens (6 decimals)
    true,           // Sell token0
    0,              // No price limit
    30,             // 0.3% fee
    100,            // 1% creator fee
);

println!("Spent: {} token0", spent);
println!("Received: {} token1", received);
println!("Fee: {}", spent - received);
println!("New price: {}", state.sqrt_price_x64);
```

---

### Example 2: Swap with Price Limit

```rust
use belugaswap_swap::engine_swap;
use belugaswap_math::get_sqrt_ratio_at_tick;

// Don't let price drop below tick -1000 (~0.905)
let price_limit = get_sqrt_ratio_at_tick(-1000);

let (amount_in, amount_out) = engine_swap(
    &env,
    &mut state,
    read_tick,
    write_tick,
    emit_sync,
    1_000_000,
    true,           // Selling token0
    price_limit,    // Stop at this price
    30,
    100,
);

// Swap may use less than full amount if price limit reached
println!("Used {} of 1_000_000 input", amount_in);
println!("Price stopped at: {}", state.sqrt_price_x64);
```

---

### Example 3: Quote Before Swapping

```rust
use belugaswap_swap::{quote_swap, engine_swap};

// First, get a quote
let (quote_in, quote_out, quote_price) = quote_swap(
    &env,
    &state,  // Immutable reference
    read_tick,
    1_000_000,
    true,
    0,
    30,
);

println!("Quote: {} → {}", quote_in, quote_out);

// User approves the quote, execute actual swap
let (actual_in, actual_out) = engine_swap(
    &env,
    &mut state,  // Mutable reference
    read_tick,
    write_tick,
    emit_sync,
    1_000_000,
    true,
    0,
    30,
    100,
);

// Should match quote (minus creator fee)
assert_eq!(actual_in, quote_in);
// actual_out may be slightly less due to creator fee
```

---

### Example 4: Validate Slippage Tolerance

```rust
use belugaswap_swap::validate_and_preview_swap;

let input_amount = 1_000_000;
let max_slippage = 1;  // 1% = 100 bps
let min_output = input_amount * (10000 - max_slippage) / 10000;

match validate_and_preview_swap(
    &env,
    &state,
    read_tick,
    input_amount,
    min_output,     // 990,000 minimum
    true,
    0,
    30,
) {
    Ok((amount_in, amount_out, fee, final_price)) => {
        // Slippage acceptable, proceed with swap
        let (actual_in, actual_out) = engine_swap(
            &env,
            &mut state,
            read_tick,
            write_tick,
            emit_sync,
            amount_in,
            true,
            0,
            30,
            100,
        );
    }
    Err(symbol) => {
        panic!("Swap would exceed slippage tolerance: {:?}", symbol);
    }
}
```

---

### Example 5: Handle Multi-tick Swap

```rust
use belugaswap_swap::engine_swap;

// Large swap that crosses multiple ticks
let mut state = SwapState {
    sqrt_price_x64: 1u128 << 64,
    current_tick: 0,
    liquidity: 1_000_000,  // Low liquidity forces tick crossing
    tick_spacing: 60,
    fee_growth_global_0: 0,
    fee_growth_global_1: 0,
};

let initial_tick = state.current_tick;

let (amount_in, amount_out) = engine_swap(
    &env,
    &mut state,
    read_tick,
    write_tick,
    emit_sync,
    10_000_000,  // Large swap
    true,
    0,
    30,
    100,
);

let ticks_crossed = (initial_tick - state.current_tick).abs();
println!("Crossed {} ticks", ticks_crossed);
println!("Price moved from {} to {}", 
    1u128 << 64, 
    state.sqrt_price_x64
);
```

---

### Example 6: Bidirectional Swaps

```rust
use belugaswap_swap::engine_swap;

// Swap token0 → token1
let (in0, out1) = engine_swap(
    &env,
    &mut state,
    read_tick, write_tick, emit_sync,
    1_000_000,
    true,   // zero_for_one = true
    0, 30, 100,
);

println!("Sold {} token0, got {} token1", in0, out1);

// Swap token1 → token0 (reverse)
let (in1, out0) = engine_swap(
    &env,
    &mut state,
    read_tick, write_tick, emit_sync,
    out1,   // Use output from previous swap
    false,  // zero_for_one = false
    0, 30, 100,
);

println!("Sold {} token1, got {} token0 back", in1, out0);

// Due to fees, out0 < original in0
let fee_cost = in0 - out0;
println!("Total fee cost: {} token0", fee_cost);
```

---

### Example 7: Calculate Price Impact

```rust
use belugaswap_swap::quote_swap;

let initial_price = state.sqrt_price_x64;

// Get quote for swap
let (amount_in, amount_out, final_price) = quote_swap(
    &env,
    &state,
    read_tick,
    1_000_000,
    true,
    0,
    30,
);

// Calculate price impact
let price_change = if initial_price > final_price {
    initial_price - final_price
} else {
    final_price - initial_price
};

let price_impact_bps = (price_change * 10000) / initial_price;

println!("Price impact: {}.{}%", 
    price_impact_bps / 100, 
    price_impact_bps % 100
);

if price_impact_bps > 500 {  // 5%
    println!("WARNING: High price impact!");
}
```

---

## 🔬 Advanced Topics

### Fee Calculation Details

#### Step Fee Calculation

```rust
// For each swap step:

if amount_in == amount_available {
    // Used entire available amount
    fee = amount_remaining - amount_in
} else {
    // Calculate proportional fee
    fee = (amount_in * fee_bps) / (10000 - fee_bps)
    // Round up to favor pool
    if (amount_in * fee_bps) % (10000 - fee_bps) != 0 {
        fee += 1
    }
}
```

#### Creator Fee from LP Fee

```rust
// Creator fee is a percentage of LP fee (not total swap)

total_fee = step_fee
creator_fee = (total_fee * creator_fee_bps) / 10000
lp_fee = total_fee - creator_fee

// Example with 0.3% swap fee, 1% creator fee:
// Swap 100 tokens:
//   total_fee = 0.3 tokens
//   creator_fee = 0.3 * 0.01 = 0.003 tokens
//   lp_fee = 0.297 tokens
```

### Iteration Limits

The swap engine has safety limits to prevent infinite loops:

```rust
const MAX_SWAP_ITERATIONS: u32 = 1024;  // Max tick crossings
const MAX_TICK_SEARCH_STEPS: i32 = 2000; // Max tick lookups
```

If a swap requires more than these limits, it will stop early.

### Wrapping Arithmetic

Fee growth uses wrapping arithmetic to handle overflow:

```rust
// Fee growth can exceed u128::MAX
state.fee_growth_global_0 = state
    .fee_growth_global_0
    .wrapping_add(growth_delta);

// When calculating fees, wrapping subtraction handles overflow:
delta = current_growth.wrapping_sub(last_checkpoint);
```

### Price Limit Edge Cases

```rust
// Default limits when sqrt_price_limit_x64 = 0:

if zero_for_one {
    // Selling token0, price decreases
    limit = 100  // Near-zero price
} else {
    // Selling token1, price increases
    limit = u128::MAX - 1000  // Near-infinite price
}
```

### Dry Run vs Live Execution

```rust
// quote_swap uses dry_run = true:
let (in, out, price) = quote_swap(...);  // No storage writes

// engine_swap uses dry_run = false:
let (in, out) = engine_swap(...);  // Writes to storage
```

---

## 📊 Return Types

### SwapResult

```rust
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub current_tick: i32,
    pub sqrt_price_x64: u128,
}
```

Used by pool contract's `swap()` function to return complete swap info.

---

### PreviewResult

```rust
pub struct PreviewResult {
    pub amount_in_used: i128,
    pub amount_out_expected: i128,
    pub fee_paid: i128,
    pub price_impact_bps: i128,
    pub is_valid: bool,
    pub error_message: Option<Symbol>,
}
```

Used by pool contract's `preview_swap()` function.

**Helper constructors:**
```rust
// Valid result
PreviewResult::valid(amount_in, amount_out, fee, impact)

// Invalid result
PreviewResult::invalid(Symbol::new(&env, "SLIP_HI"))
```

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Swap Package**: [packages/swap](https://github.com/Beluga-Swap/core/tree/main/packages/swap)
- **Math Package**: [packages/math](https://github.com/Beluga-Swap/core/tree/main/packages/math)
- **Tick Package**: [packages/tick](https://github.com/Beluga-Swap/core/tree/main/packages/tick)
- **Pool Contract**: [contracts/pool](https://github.com/Beluga-Swap/core/tree/main/contracts/pool)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)

---

## 📄 License

MIT License - see LICENSE file for details