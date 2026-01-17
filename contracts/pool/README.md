# BelugaSwap Pool

Concentrated liquidity AMM pool contract implementing Uniswap v3-style concentrated liquidity with creator fee incentives.

## 📋 Table of Contents

- [Overview](#-overview)
- [Core Concepts](#-core-concepts)
- [Main Functions](#-main-functions)
- [Usage Examples](#-usage-examples)
- [Error Codes](#-error-codes)
- [Local Setup](#-local-setup)

---

## 🌊 Overview

BelugaSwap Pool is a concentrated liquidity AMM that allows liquidity providers (LPs) to concentrate their capital within custom price ranges, improving capital efficiency compared to traditional AMMs.

### Key Features

- **Concentrated Liquidity**: LPs choose specific price ranges for their liquidity
- **Multiple Fee Tiers**: 0.05%, 0.30%, 1.00% fee options
- **Creator Incentives**: Pool creators earn a share of trading fees
- **Tick-based Pricing**: Granular price control using tick system
- **Fee Accumulation**: Automatic fee tracking per position

### Architecture

The pool contract uses several specialized packages:
- `belugaswap-math`: Core mathematical operations and price calculations
- `belugaswap-tick`: Tick management and liquidity transitions
- `belugaswap-position`: Position tracking and fee calculation
- `belugaswap-swap`: Swap execution engine

---

## 💡 Core Concepts

### Ticks and Price Ranges

Liquidity is organized into **ticks**, which represent discrete price points:

```
Price = 1.0001^tick

Example ticks:
tick -10 = price 0.999
tick 0   = price 1.000
tick 10  = price 1.001
```

LPs provide liquidity between two ticks (lower and upper), creating a **position**.

### Tick Spacing

Tick spacing determines granularity based on fee tier:
- **5 bps (0.05%)**: spacing 10 → ticks ..., -20, -10, 0, 10, 20, ...
- **30 bps (0.30%)**: spacing 60 → ticks ..., -120, -60, 0, 60, 120, ...
- **100 bps (1.00%)**: spacing 200 → ticks ..., -400, -200, 0, 200, 400, ...

### sqrt_price_x64

Prices are stored as `sqrt(price) * 2^64` for precision:

```rust
// Price = 1.0 (1:1 ratio)
let sqrt_price_x64 = 1u128 << 64; // 18446744073709551616

// Price = 4.0 (4 token1 per 1 token0)
let sqrt_price_x64 = 2u128 << 64; // sqrt(4) * 2^64
```

### Fee Distribution

Trading fees are split:
1. **LP Fees**: (100% - creator_fee%) → Distributed to LPs proportionally
2. **Creator Fees**: (creator_fee%) → Accumulated for pool creator

---

## 📚 Main Functions

### Initialization (1 Function)

#### `initialize`

```rust
pub fn initialize(
    env: Env,
    creator: Address,
    token_a: Address,
    token_b: Address,
    fee_bps: u32,
    creator_fee_bps: u32,
    sqrt_price_x64: u128,
    current_tick: i32,
    tick_spacing: i32,
)
```

Initialize the pool contract. **Can only be called once.**

**Parameters:**
- `creator`: Pool creator address (earns creator fees)
- `token_a`: First token address
- `token_b`: Second token address
- `fee_bps`: Trading fee in basis points (5, 30, or 100)
- `creator_fee_bps`: Creator's fee share (10-1000 bps = 0.1%-10%)
- `sqrt_price_x64`: Initial price as sqrt(price) * 2^64
- `current_tick`: Initial tick corresponding to sqrt_price_x64
- `tick_spacing`: Tick spacing (10, 60, or 200)

**Errors:**
- `AlreadyInitialized`: Pool already initialized
- `InvalidFee`: fee_bps not in valid range
- `InvalidCreatorFee`: creator_fee_bps outside 0.1%-10%
- `InvalidTickSpacing`: tick_spacing <= 0

---

### View Functions (7 Functions)

#### `is_initialized`

```rust
pub fn is_initialized(env: Env) -> bool
```

Check if pool has been initialized.

---

#### `get_pool_state`

```rust
pub fn get_pool_state(env: Env) -> PoolState
```

Get current pool state including price, tick, and liquidity.

**Returns:**
```rust
struct PoolState {
    sqrt_price_x64: u128,      // Current price
    current_tick: i32,          // Current tick
    liquidity: i128,            // Active liquidity
    tick_spacing: i32,          // Tick spacing
    token0: Address,            // Token 0 (sorted)
    token1: Address,            // Token 1 (sorted)
    fee_growth_global_0: u128,  // Accumulated fees token0
    fee_growth_global_1: u128,  // Accumulated fees token1
    creator_fees_0: u128,       // Creator fees token0
    creator_fees_1: u128,       // Creator fees token1
}
```

---

#### `get_pool_config`

```rust
pub fn get_pool_config(env: Env) -> PoolConfig
```

Get pool configuration.

**Returns:**
```rust
struct PoolConfig {
    factory: Address,        // Factory address
    creator: Address,        // Pool creator
    token_a: Address,        // Token A (original order)
    token_b: Address,        // Token B (original order)
    fee_bps: u32,           // Trading fee (bps)
    creator_fee_bps: u32,   // Creator fee share (bps)
}
```

---

#### `get_position`

```rust
pub fn get_position(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
) -> PositionInfo
```

Get detailed position information for an LP.

**Returns:**
```rust
struct PositionInfo {
    liquidity: i128,      // Position liquidity
    amount0: i128,        // Current token0 amount
    amount1: i128,        // Current token1 amount
    fees_owed_0: u128,    // Uncollected fees token0
    fees_owed_1: u128,    // Uncollected fees token1
}
```

---

#### `get_creator_fees`

```rust
pub fn get_creator_fees(env: Env) -> CreatorFeesInfo
```

Get accumulated creator fees.

**Returns:**
```rust
struct CreatorFeesInfo {
    fees_token0: u128,
    fees_token1: u128,
}
```

---

#### `get_swap_direction`

```rust
pub fn get_swap_direction(env: Env, token_in: Address) -> bool
```

Get swap direction based on input token.

**Returns:** 
- `true` if swapping token0 → token1 (zero_for_one)
- `false` if swapping token1 → token0

**Errors:**
- `InvalidToken`: token_in is not in this pool

---

#### `preview_swap`

```rust
pub fn preview_swap(
    env: Env,
    token_in: Address,
    amount_in: i128,
    min_amount_out: i128,
    sqrt_price_limit_x64: u128,
) -> PreviewResult
```

Preview swap output without executing the swap.

**Parameters:**
- `token_in`: Input token address
- `amount_in`: Input amount
- `min_amount_out`: Minimum output (for slippage check)
- `sqrt_price_limit_x64`: Price limit (0 = no limit)

**Returns:**
```rust
struct PreviewResult {
    is_valid: bool,           // Swap is valid
    amount_in: i128,          // Input used
    amount_out: i128,         // Output received
    fee_paid: i128,           // Fee amount
    price_impact_bps: i128,   // Price impact in bps
    error: Symbol,            // Error symbol if invalid
}
```

---

### Trading Functions (1 Function)

#### `swap`

```rust
pub fn swap(
    env: Env,
    sender: Address,
    token_in: Address,
    amount_in: i128,
    amount_out_min: i128,
    sqrt_price_limit_x64: u128,
) -> SwapResult
```

Execute a token swap.

**Parameters:**
- `sender`: Sender address (must authorize)
- `token_in`: Input token address
- `amount_in`: Amount to swap
- `amount_out_min`: Minimum output (slippage protection)
- `sqrt_price_limit_x64`: Price limit (0 = no limit)

**Returns:**
```rust
struct SwapResult {
    amount_in: i128,       // Amount spent
    amount_out: i128,      // Amount received
    fee_paid: i128,        // Fee paid
    sqrt_price_x64: u128,  // Final price
    current_tick: i32,     // Final tick
}
```

**Errors:**
- `InvalidToken`: token_in not in pool
- `SwapAmountTooSmall`: amount_in too small
- `SlippageExceeded`: amount_out < amount_out_min
- `NoLiquidity`: No liquidity available

---

### Liquidity Functions (3 Functions)

#### `mint`

```rust
pub fn mint(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
    amount0_desired: i128,
    amount1_desired: i128,
) -> i128
```

Add liquidity to create or increase a position.

**Parameters:**
- `owner`: LP owner address (must authorize)
- `lower_tick`: Lower price tick
- `upper_tick`: Upper price tick
- `amount0_desired`: Desired token0 amount
- `amount1_desired`: Desired token1 amount

**Returns:** Liquidity amount minted

**Errors:**
- `InvalidTickRange`: lower_tick >= upper_tick
- `InvalidTickSpacing`: Ticks not aligned to spacing
- `LiquidityTooLow`: Below minimum liquidity

**Note:** Automatically transfers tokens from owner to pool.

---

#### `burn`

```rust
pub fn burn(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
    liquidity_delta: i128,
) -> (i128, i128)
```

Remove liquidity from a position (fees stay in position until collected).

**Parameters:**
- `owner`: LP owner address (must authorize)
- `lower_tick`: Lower price tick
- `upper_tick`: Upper price tick
- `liquidity_delta`: Amount of liquidity to remove

**Returns:** `(amount0, amount1)` - tokens removed from position

**Errors:**
- `InvalidLiquidityAmount`: liquidity_delta <= 0
- `InsufficientLiquidity`: Position doesn't have enough liquidity

**Note:** Fees remain in position. Use `collect()` to withdraw fees.

---

#### `remove_liquidity`

```rust
pub fn remove_liquidity(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
    liquidity_delta: i128,
) -> (i128, i128)
```

Remove liquidity AND transfer tokens back to owner.

**Parameters:** Same as `burn()`

**Returns:** `(amount0, amount1)` - tokens transferred to owner

**Errors:** Same as `burn()`

**Note:** Unlike `burn()`, this function transfers tokens immediately.

---

### Fee Collection Functions (2 Functions)

#### `collect`

```rust
pub fn collect(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
) -> (u128, u128)
```

Collect accumulated LP fees from a position.

**Parameters:**
- `owner`: LP owner address (must authorize)
- `lower_tick`: Lower price tick
- `upper_tick`: Upper price tick

**Returns:** `(amount0, amount1)` - fees collected

**Note:** Updates position and transfers fees to owner.

---

#### `claim_creator_fees`

```rust
pub fn claim_creator_fees(env: Env, claimer: Address) -> (u128, u128)
```

Claim accumulated creator fees. **Only pool creator can call.**

**Parameters:**
- `claimer`: Must be pool creator address

**Returns:** `(amount0, amount1)` - creator fees claimed

**Errors:**
- `Unauthorized`: Caller is not pool creator

---

## 🔧 Usage Examples

### 1. Initialize Pool

```rust
// Initialize USDC/XLM pool with 0.30% fee
pool.initialize(
    &creator,
    &token_usdc,
    &token_xlm,
    30,                    // 0.30% trading fee
    100,                   // 1% creator fee
    1u128 << 64,          // Price = 1.0
    0,                    // Current tick = 0
    60,                   // Tick spacing = 60
);
```

---

### 2. Add Liquidity (Full Range)

```rust
// Add liquidity across full price range
let liquidity = pool.mint(
    &lp_address,
    -887220,              // Min tick (full range)
    887220,               // Max tick (full range)
    1_000_000_000,        // 100 USDC (7 decimals)
    1_000_000_000,        // 100 XLM (7 decimals)
);
```

---

### 3. Add Liquidity (Concentrated Range)

```rust
// Add liquidity in price range [0.95, 1.05]
// Assuming tick spacing = 60

let liquidity = pool.mint(
    &lp_address,
    -5160,                // ~0.95 price
    4920,                 // ~1.05 price
    1_000_000_000,        // 100 USDC
    1_000_000_000,        // 100 XLM
);
```

---

### 4. Preview Swap

```rust
// Preview swapping 10 USDC for XLM
let preview = pool.preview_swap(
    &token_usdc,          // Input token
    10_000_000,           // 10 USDC
    0,                    // Min output (0 = just preview)
    0,                    // No price limit
);

if preview.is_valid {
    println!("Output: {} XLM", preview.amount_out);
    println!("Fee: {} USDC", preview.fee_paid);
    println!("Price impact: {}%", preview.price_impact_bps / 100);
}
```

---

### 5. Execute Swap

```rust
// Swap 10 USDC for XLM with 1% slippage tolerance
let result = pool.swap(
    &trader,
    &token_usdc,          // Selling USDC
    10_000_000,           // 10 USDC
    9_700_000,            // Min 9.7 XLM (3% slippage)
    0,                    // No price limit
);

println!("Spent: {} USDC", result.amount_in);
println!("Received: {} XLM", result.amount_out);
println!("Fee: {} USDC", result.fee_paid);
```

---

### 6. Check Position

```rust
// Get position info
let position = pool.get_position(
    &lp_address,
    -5160,
    4920,
);

println!("Liquidity: {}", position.liquidity);
println!("Token0: {}", position.amount0);
println!("Token1: {}", position.amount1);
println!("Fees owed token0: {}", position.fees_owed_0);
println!("Fees owed token1: {}", position.fees_owed_1);
```

---

### 7. Collect Fees

```rust
// Collect accumulated fees from position
let (fee0, fee1) = pool.collect(
    &lp_address,
    -5160,
    4920,
);

println!("Collected token0 fees: {}", fee0);
println!("Collected token1 fees: {}", fee1);
```

---

### 8. Remove Liquidity

```rust
// Remove 50% of liquidity from position
let current_position = pool.get_position(&lp_address, -5160, 4920);
let liquidity_to_remove = current_position.liquidity / 2;

let (amount0, amount1) = pool.remove_liquidity(
    &lp_address,
    -5160,
    4920,
    liquidity_to_remove,
);

println!("Removed token0: {}", amount0);
println!("Removed token1: {}", amount1);
```

---

### 9. Claim Creator Fees

```rust
// Creator claims accumulated fees
let (fee0, fee1) = pool.claim_creator_fees(&creator);

println!("Creator claimed token0: {}", fee0);
println!("Creator claimed token1: {}", fee1);
```

---

## ⚠️ Error Codes

### Initialization Errors (100-199)

| Code | Error | Description |
|------|-------|-------------|
| 100 | `AlreadyInitialized` | Pool has already been initialized |
| 101 | `NotInitialized` | Pool has not been initialized |

### Configuration Errors (200-299)

| Code | Error | Description |
|------|-------|-------------|
| 200 | `InvalidFee` | Fee must be 1-10000 bps |
| 201 | `InvalidCreatorFee` | Creator fee must be 1-1000 bps (0.01%-10%) |
| 202 | `InvalidTickSpacing` | Tick spacing must be positive |
| 203 | `InvalidTickRange` | Lower tick must be < upper tick |
| 204 | `InvalidTick` | Tick is out of valid range |

### Token Errors (300-399)

| Code | Error | Description |
|------|-------|-------------|
| 300 | `InvalidToken` | Token is not part of this pool |
| 301 | `SameToken` | Input and output tokens are the same |

### Liquidity Errors (400-499)

| Code | Error | Description |
|------|-------|-------------|
| 400 | `LiquidityTooLow` | Liquidity below minimum required |
| 401 | `InsufficientLiquidity` | Position doesn't have enough liquidity |
| 402 | `InvalidLiquidityAmount` | Liquidity amount must be positive |

### Swap Errors (500-599)

| Code | Error | Description |
|------|-------|-------------|
| 500 | `SwapAmountTooSmall` | Swap amount too small |
| 501 | `SlippageExceeded` | Output below min_amount_out |
| 502 | `OutputDust` | Output amount too small (dust) |
| 503 | `NoLiquidity` | No liquidity available for swap |
| 504 | `MaxSlippageExceeded` | Maximum slippage tolerance exceeded |

### Authorization Errors (600-699)

| Code | Error | Description |
|------|-------|-------------|
| 600 | `Unauthorized` | Only pool creator can perform this action |

### Math Errors (700-799)

| Code | Error | Description |
|------|-------|-------------|
| 700 | `DivisionByZero` | Division by zero |
| 701 | `Overflow` | Arithmetic overflow |

---

## 💻 Local Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked soroban-cli

# Add wasm32 target
rustup target add wasm32-unknown-unknown
```

### Clone Repository

```bash
# Clone project
git clone https://github.com/Beluga-Swap/core.git
cd core/contracts/pool

# Folder structure:
# pool/
# ├── src/
# │   ├── lib.rs
# │   ├── error.rs
# │   ├── events.rs
# │   ├── storage.rs
# │   └── types.rs
# ├── Cargo.toml
# └── README.md
```

### Build Contract

```bash
# Build contract
soroban contract build

# Output: target/wasm32-unknown-unknown/release/belugaswap_pool.wasm
```

### Local Testing

```bash
# Run tests (if available)
cargo test

# Optimize WASM
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/belugaswap_pool.wasm
```

### Deploy to Testnet

```bash
# Configure network
soroban network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate keypair
soroban keys generate alice --network testnet

# Get testnet XLM
# Visit: https://laboratory.stellar.org/#account-creator

# Deploy pool
POOL_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/belugaswap_pool.wasm \
  --source alice \
  --network testnet)

echo "Pool deployed at: $POOL_ID"

# Initialize pool
soroban contract invoke \
  --id $POOL_ID \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --creator GBXXX... \
  --token_a CDXXX... \
  --token_b CDYYY... \
  --fee_bps 30 \
  --creator_fee_bps 100 \
  --sqrt_price_x64 18446744073709551616 \
  --current_tick 0 \
  --tick_spacing 60
```

### Interact with Pool

```bash
# Add liquidity
soroban contract invoke \
  --id $POOL_ID \
  --source alice \
  --network testnet \
  -- \
  mint \
  --owner GBXXX... \
  --lower_tick -887220 \
  --upper_tick 887220 \
  --amount0_desired 100000000 \
  --amount1_desired 100000000

# Execute swap
soroban contract invoke \
  --id $POOL_ID \
  --source alice \
  --network testnet \
  -- \
  swap \
  --sender GBXXX... \
  --token_in CDXXX... \
  --amount_in 10000000 \
  --amount_out_min 9700000 \
  --sqrt_price_limit_x64 0

# Check pool state
soroban contract invoke \
  --id $POOL_ID \
  --network testnet \
  -- \
  get_pool_state

# Check position
soroban contract invoke \
  --id $POOL_ID \
  --network testnet \
  -- \
  get_position \
  --owner GBXXX... \
  --lower_tick -887220 \
  --upper_tick 887220
```

---

## 📝 Important Notes

### Tick Alignment

All ticks must be aligned to tick spacing:

```rust
// Valid ticks for spacing 60:
-120, -60, 0, 60, 120, 180, ...

// Invalid ticks:
-50, -30, 25, 75, ... // Not multiples of 60
```

### Fee Accumulation

- LP fees accumulate automatically during swaps
- Fees are tracked per position
- Call `collect()` to withdraw accumulated fees
- Creator fees accumulate separately

### Position Management

- Positions are identified by: `(owner, lower_tick, upper_tick)`
- Multiple calls to `mint()` with same parameters increase the position
- `burn()` decreases liquidity but keeps fees in position
- `remove_liquidity()` decreases liquidity AND transfers tokens

### Price Impact

Large swaps can have significant price impact:

```rust
// Check price impact before swapping
let preview = pool.preview_swap(...);
if preview.price_impact_bps > 100 { // > 1%
    // Consider splitting into smaller swaps
}
```

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Pool Contract**: [contracts/pool](https://github.com/Beluga-Swap/core/tree/main/contracts/pool)
- **Factory Contract**: [contracts/factory](https://github.com/Beluga-Swap/core/tree/main/contracts/factory)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)
- **Stellar Laboratory**: [laboratory.stellar.org](https://laboratory.stellar.org)

