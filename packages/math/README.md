# BelugaSwap Math Package

Core mathematical operations and utilities for concentrated liquidity AMM calculations using Q64.64 fixed-point arithmetic.

## 📋 Table of Contents

- [Overview](#-overview)
- [Q64.64 Fixed-Point Format](#-q6464-fixed-point-format)
- [Modules](#-modules)
- [Key Concepts](#-key-concepts)
- [Functions Reference](#-functions-reference)
- [Usage Examples](#-usage-examples)
- [Constants](#-constants)

---

## 🌊 Overview

The BelugaSwap Math package provides low-level mathematical primitives for concentrated liquidity calculations. It handles:

- **Fixed-point arithmetic** using Q64.64 format for price precision
- **Sqrt price calculations** for tick-to-price conversions
- **Liquidity calculations** for position management
- **Swap computations** for trade execution

### Why Q64.64?

Traditional floating-point arithmetic is not available in Soroban smart contracts. Q64.64 provides:
- **Deterministic precision**: Same results across all nodes
- **No rounding errors**: Exact arithmetic operations
- **Sufficient range**: Handles prices from ~10^-39 to ~10^38

---

## 🔢 Q64.64 Fixed-Point Format

### What is Q64.64?

Q64.64 is a fixed-point number format with:
- **64 bits** for the integer part
- **64 bits** for the fractional part
- Total: 128 bits (u128 in Rust)

### Representation

```
Value in Q64.64 = actual_value * 2^64

Example:
1.0     → 18446744073709551616  (1 * 2^64)
2.0     → 36893488147419103232  (2 * 2^64)
0.5     → 9223372036854775808   (0.5 * 2^64)
1.5     → 27670116110564327424  (1.5 * 2^64)
```

### Why sqrt(price)?

Prices are stored as `sqrt(price) * 2^64` because:
1. **Symmetry**: sqrt makes price relationships symmetric (1/x ↔ x)
2. **Efficiency**: Simplifies liquidity calculations
3. **Precision**: Better precision for small price movements

```
If price = 4.0 (4 token1 per 1 token0):
  sqrt_price = sqrt(4) = 2.0
  sqrt_price_x64 = 2.0 * 2^64 = 36893488147419103232
```

---

## 📦 Modules

### 1. `constants.rs`

Defines all mathematical and protocol constants.

**Categories:**
- Tick bounds (MIN_TICK, MAX_TICK)
- Sqrt price limits
- Liquidity constraints
- Swap parameters
- Fee limits
- Math constants (Q64, Q128)

### 2. `q64.rs`

Q64.64 fixed-point arithmetic operations.

**Core Operations:**
- `mul_q64()` - Multiply two Q64 numbers
- `div_q64()` - Divide two Q64 numbers
- `mul_div()` - Multiply and divide with U256 precision
- `div_round_up()` - Division with ceiling rounding

### 3. `sqrt_price.rs`

Price and tick conversion functions.

**Core Functions:**
- `get_sqrt_ratio_at_tick()` - Convert tick to sqrt price
- `get_next_sqrt_price_from_input()` - Calculate price after swap input
- `get_next_sqrt_price_from_output()` - Calculate price after swap output
- `compute_swap_step_with_target()` - Compute swap with price limit

### 4. `liquidity.rs`

Liquidity and amount calculations.

**Core Functions:**
- `get_liquidity_for_amounts()` - Calculate liquidity from token amounts
- `get_amounts_for_liquidity()` - Calculate token amounts from liquidity
- `get_amount_0_delta()` - Calculate token0 amount for price range
- `get_amount_1_delta()` - Calculate token1 amount for price range

---

## 💡 Key Concepts

### Ticks and Prices

A **tick** is a discrete price point:

```
price = 1.0001^tick

Tick Examples:
tick = 0      → price = 1.0
tick = 1      → price = 1.0001
tick = 100    → price = 1.0100
tick = -100   → price = 0.9900
tick = 69080  → price ≈ 1000.0
```

### Price Range Bounds

```
MIN_TICK = -887272  → price ≈ 2.94 × 10^-39
MAX_TICK = 887272   → price ≈ 3.40 × 10^38
```

### Liquidity Formula

Liquidity (L) represents the relationship between price and token amounts:

```
For price range [P_lower, P_upper]:

token0 = L * (1/√P - 1/√P_upper)  when P < P_upper
token1 = L * (√P - √P_lower)      when P > P_lower

Where P is current price in Q64.64 format
```

### Swap Calculations

During a swap, price moves along the liquidity curve:

```
Zero-for-one swap (selling token0):
  new_sqrt_price = (L * √P) / (L + Δx * √P)

One-for-zero swap (selling token1):
  new_sqrt_price = √P + (Δy / L)

Where:
  L = liquidity
  Δx = token0 amount in
  Δy = token1 amount in
```

---

## 📚 Functions Reference

### Q64.64 Arithmetic

#### `mul_q64`

```rust
pub fn mul_q64(a: u128, b: u128) -> u128
```

Multiply two Q64.64 numbers, returning Q64.64 result.

**Formula:** `(a * b) / 2^64`

**Example:**
```rust
use belugaswap_math::mul_q64;

// 1.5 * 2.0 = 3.0
let a = (3u128 << 64) / 2;  // 1.5 in Q64
let b = 2u128 << 64;         // 2.0 in Q64
let result = mul_q64(a, b);  // 3.0 in Q64

assert_eq!(result, 3u128 << 64);
```

---

#### `div_q64`

```rust
pub fn div_q64(a: u128, b: u128) -> u128
```

Divide two Q64.64 numbers, returning Q64.64 result.

**Formula:** `(a * 2^64) / b`

**Example:**
```rust
use belugaswap_math::div_q64;

// 6.0 / 2.0 = 3.0
let a = 6u128 << 64;         // 6.0 in Q64
let b = 2u128 << 64;         // 2.0 in Q64
let result = div_q64(a, b);  // 3.0 in Q64

assert_eq!(result, 3u128 << 64);
```

---

#### `mul_div`

```rust
pub fn mul_div(env: &Env, a: u128, b: u128, denominator: u128) -> u128
```

Safely multiply and divide using U256 intermediate to prevent overflow.

**Formula:** `(a * b) / denominator`

**Example:**
```rust
use belugaswap_math::mul_div;

// (large_num1 * large_num2) / divisor
let result = mul_div(&env, 1_000_000_000, 2_000_000_000, 1_000_000);
// result = 2_000_000_000_000
```

---

### Sqrt Price Functions

#### `get_sqrt_ratio_at_tick`

```rust
pub fn get_sqrt_ratio_at_tick(tick: i32) -> u128
```

Convert tick to sqrt price in Q64.64 format.

**Formula:** `sqrt(1.0001^tick) * 2^64`

**Example:**
```rust
use belugaswap_math::get_sqrt_ratio_at_tick;

// Get sqrt price at tick 0 (price = 1.0)
let sqrt_price = get_sqrt_ratio_at_tick(0);
assert_eq!(sqrt_price, 1u128 << 64);  // 2^64

// Get sqrt price at tick 6932 (price ≈ 2.0)
let sqrt_price = get_sqrt_ratio_at_tick(6932);
// sqrt_price ≈ sqrt(2) * 2^64 ≈ 1.414 * 2^64
```

**Panics:** If tick is outside [MIN_TICK, MAX_TICK]

---

#### `get_next_sqrt_price_from_input`

```rust
pub fn get_next_sqrt_price_from_input(
    env: &Env,
    sqrt_price: u128,
    liquidity: u128,
    amount_in: u128,
    zero_for_one: bool,
) -> u128
```

Calculate the next sqrt price after adding input amount.

**Parameters:**
- `sqrt_price`: Current sqrt price in Q64.64
- `liquidity`: Active liquidity
- `amount_in`: Input token amount
- `zero_for_one`: true if selling token0, false if selling token1

**Formula (zero_for_one = true):**
```
new_sqrt_price = (L * √P) / (L + Δx * √P)
```

**Formula (zero_for_one = false):**
```
new_sqrt_price = √P + (Δy / L)
```

**Example:**
```rust
use belugaswap_math::get_next_sqrt_price_from_input;

let current_sqrt_price = 1u128 << 64;  // Price = 1.0
let liquidity = 1_000_000u128;
let amount_in = 100u128;
let zero_for_one = true;

let new_sqrt_price = get_next_sqrt_price_from_input(
    &env,
    current_sqrt_price,
    liquidity,
    amount_in,
    zero_for_one
);
// Price decreases when selling token0
```

---

#### `get_next_sqrt_price_from_output`

```rust
pub fn get_next_sqrt_price_from_output(
    env: &Env,
    sqrt_price: u128,
    liquidity: u128,
    amount_out: u128,
    zero_for_one: bool,
) -> u128
```

Calculate the next sqrt price after removing output amount.

Similar to `get_next_sqrt_price_from_input` but works backwards from desired output.

---

### Liquidity Functions

#### `get_liquidity_for_amounts`

```rust
pub fn get_liquidity_for_amounts(
    env: &Env,
    amount0_desired: i128,
    amount1_desired: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    current_sqrt_price: u128,
) -> i128
```

Calculate liquidity from desired token amounts.

**Logic:**
- If `current_price < lower_price`: Only token0 needed
- If `current_price > upper_price`: Only token1 needed
- If `lower_price ≤ current_price ≤ upper_price`: Both tokens needed

**Example:**
```rust
use belugaswap_math::{get_liquidity_for_amounts, get_sqrt_ratio_at_tick};

let amount0 = 1_000_000;  // 1 token0
let amount1 = 1_000_000;  // 1 token1

let sqrt_lower = get_sqrt_ratio_at_tick(-1000);  // ~0.905
let sqrt_upper = get_sqrt_ratio_at_tick(1000);   // ~1.105
let sqrt_current = get_sqrt_ratio_at_tick(0);    // 1.0

let liquidity = get_liquidity_for_amounts(
    &env,
    amount0,
    amount1,
    sqrt_lower,
    sqrt_upper,
    sqrt_current,
);
```

---

#### `get_amounts_for_liquidity`

```rust
pub fn get_amounts_for_liquidity(
    env: &Env,
    liquidity: i128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    current_sqrt_price: u128,
) -> (i128, i128)
```

Calculate token amounts from liquidity.

**Returns:** `(amount0, amount1)`

**Example:**
```rust
use belugaswap_math::{get_amounts_for_liquidity, get_sqrt_ratio_at_tick};

let liquidity = 10_000_000;
let sqrt_lower = get_sqrt_ratio_at_tick(-1000);
let sqrt_upper = get_sqrt_ratio_at_tick(1000);
let sqrt_current = get_sqrt_ratio_at_tick(0);

let (amount0, amount1) = get_amounts_for_liquidity(
    &env,
    liquidity,
    sqrt_lower,
    sqrt_upper,
    sqrt_current,
);

println!("Token0: {}, Token1: {}", amount0, amount1);
```

---

#### `get_amount_0_delta`

```rust
pub fn get_amount_0_delta(
    sqrt_price_a: u128,
    sqrt_price_b: u128,
    liquidity: u128,
    round_up: bool,
) -> u128
```

Calculate token0 amount for a price range and liquidity.

**Formula:**
```
amount0 = L * (1/√P_a - 1/√P_b)
        = L * (√P_b - √P_a) / (√P_a * √P_b)
```

**Parameters:**
- `sqrt_price_a`, `sqrt_price_b`: Price range bounds
- `liquidity`: Liquidity amount
- `round_up`: true to round up (for safety), false to round down

---

#### `get_amount_1_delta`

```rust
pub fn get_amount_1_delta(
    sqrt_price_a: u128,
    sqrt_price_b: u128,
    liquidity: u128,
    round_up: bool,
) -> u128
```

Calculate token1 amount for a price range and liquidity.

**Formula:**
```
amount1 = L * (√P_b - √P_a)
```

---

### Utility Functions

#### `snap_tick_to_spacing`

```rust
pub fn snap_tick_to_spacing(tick: i32, spacing: i32) -> i32
```

Snap a tick to the nearest valid tick based on spacing.

**Example:**
```rust
use belugaswap_math::snap_tick_to_spacing;

// Tick spacing = 60
let tick = 125;
let snapped = snap_tick_to_spacing(tick, 60);
assert_eq!(snapped, 120);  // Rounds down to nearest multiple

let tick = -75;
let snapped = snap_tick_to_spacing(tick, 60);
assert_eq!(snapped, -120);  // Rounds down to nearest multiple
```

---

#### `compute_swap_step`

```rust
pub fn compute_swap_step(
    env: &Env,
    sqrt_price_current: u128,
    liquidity: i128,
    amount_remaining: i128,
    zero_for_one: bool,
) -> (u128, i128, i128)
```

Compute a single swap step without target price.

**Returns:** `(new_sqrt_price, amount_in, amount_out)`

---

## 🔧 Usage Examples

### Example 1: Price Conversion

```rust
use belugaswap_math::{get_sqrt_ratio_at_tick, ONE_X64};

// Convert tick to price
let tick = 6932;  // Approximately price = 2.0
let sqrt_price = get_sqrt_ratio_at_tick(tick);

// Calculate actual price from sqrt_price
// price = (sqrt_price / 2^64)^2
let sqrt_price_f64 = sqrt_price as f64 / (ONE_X64 as f64);
let price = sqrt_price_f64 * sqrt_price_f64;
println!("Price at tick {}: {}", tick, price);  // ~2.0
```

---

### Example 2: Calculate Required Amounts

```rust
use belugaswap_math::{get_amounts_for_liquidity, get_sqrt_ratio_at_tick};

// Position: 10M liquidity from tick -1000 to 1000
let liquidity = 10_000_000;
let lower_tick = -1000;
let upper_tick = 1000;
let current_tick = 0;

let sqrt_lower = get_sqrt_ratio_at_tick(lower_tick);
let sqrt_upper = get_sqrt_ratio_at_tick(upper_tick);
let sqrt_current = get_sqrt_ratio_at_tick(current_tick);

let (amount0, amount1) = get_amounts_for_liquidity(
    &env,
    liquidity,
    sqrt_lower,
    sqrt_upper,
    sqrt_current,
);

println!("Need {} token0 and {} token1", amount0, amount1);
```

---

### Example 3: Simulate Swap

```rust
use belugaswap_math::{
    get_sqrt_ratio_at_tick,
    get_next_sqrt_price_from_input,
    get_amount_1_delta,
};

// Current state: price = 1.0, liquidity = 1M
let current_sqrt_price = get_sqrt_ratio_at_tick(0);
let liquidity = 1_000_000u128;

// Swap 100 token0 for token1
let amount_in = 100u128;
let zero_for_one = true;

// Calculate new price
let new_sqrt_price = get_next_sqrt_price_from_input(
    &env,
    current_sqrt_price,
    liquidity,
    amount_in,
    zero_for_one,
);

// Calculate output amount
let amount_out = get_amount_1_delta(
    current_sqrt_price,
    new_sqrt_price,
    liquidity,
    false,  // round down for output
);

println!("Swap {} token0 → {} token1", amount_in, amount_out);
println!("Price moved from {} to {}", current_sqrt_price, new_sqrt_price);
```

---

### Example 4: Add Liquidity Calculation

```rust
use belugaswap_math::{
    get_liquidity_for_amounts,
    get_amounts_for_liquidity,
    get_sqrt_ratio_at_tick,
};

// Want to add liquidity with 1000 of each token
let amount0_desired = 1_000_000_000;  // 1000 tokens (6 decimals)
let amount1_desired = 1_000_000_000;

// Price range: 0.9 to 1.1 (approximately)
let sqrt_lower = get_sqrt_ratio_at_tick(-1000);
let sqrt_upper = get_sqrt_ratio_at_tick(1000);
let sqrt_current = get_sqrt_ratio_at_tick(0);  // Current price = 1.0

// Calculate liquidity
let liquidity = get_liquidity_for_amounts(
    &env,
    amount0_desired,
    amount1_desired,
    sqrt_lower,
    sqrt_upper,
    sqrt_current,
);

// Calculate actual amounts that will be used
let (amount0_actual, amount1_actual) = get_amounts_for_liquidity(
    &env,
    liquidity,
    sqrt_lower,
    sqrt_upper,
    sqrt_current,
);

println!("Liquidity: {}", liquidity);
println!("Will use: {} token0, {} token1", amount0_actual, amount1_actual);
```

---

## 📊 Constants

### Tick Constants

```rust
pub const MIN_TICK: i32 = -887272;      // Min price ≈ 2.94e-39
pub const MAX_TICK: i32 = 887272;       // Max price ≈ 3.40e+38
```

### Math Constants

```rust
pub const Q64: u128 = 1u128 << 64;      // 2^64 = 18446744073709551616
pub const ONE_X64: u128 = Q64;           // 1.0 in Q64.64 format
```

### Liquidity Constants

```rust
pub const MIN_LIQUIDITY: i128 = 1000;   // Minimum position liquidity
```

### Swap Constants

```rust
pub const MIN_SWAP_AMOUNT: i128 = 1;           // Minimum swap amount
pub const MIN_OUTPUT_AMOUNT: i128 = 1;         // Dust threshold
pub const MAX_SLIPPAGE_BPS: i128 = 5000;       // 50% max slippage
pub const MAX_SWAP_ITERATIONS: u32 = 1024;     // Max swap loops
pub const MAX_TICK_SEARCH_STEPS: i32 = 2000;   // Max tick searches
```

### Fee Constants

```rust
pub const MAX_FEE_BPS: u32 = 10000;            // 100% = 10000 bps
pub const MIN_CREATOR_FEE_BPS: u32 = 1;        // 0.01% minimum
pub const MAX_CREATOR_FEE_BPS: u32 = 1000;     // 10% maximum
```

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Math Package**: [packages/math](https://github.com/Beluga-Swap/core/tree/main/packages/math)
- **Pool Contract**: [contracts/pool](https://github.com/Beluga-Swap/core/tree/main/contracts/pool)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)

---

## 📄 License

MIT License - see LICENSE file for details