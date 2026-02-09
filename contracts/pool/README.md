# BelugaSwap Pool

Concentrated liquidity AMM pool contract with creator fees and factory/router integration.

---

## 🦈 Pool Functions

The pool contract handles:

### 1. **Swap Execution**
- Execute token swaps within concentrated liquidity ranges
- Multi-tick traversal for large swaps
- Price impact calculation
- Creator fee distribution

### 2. **Liquidity Management**
- Add/remove liquidity in custom price ranges
- Position tracking with fee accumulation
- Creator lock enforcement (via Factory)

### 3. **Fee Collection**
- LP fee accumulation per position
- Creator fee accumulation (if active)
- Separate collection functions

### 4. **Factory & Router Integration**
- Initialized by Factory with router address
- Queries Factory for creator lock status
- Works with Router for multi-hop swaps

---

## 🔧 Pool Configuration

Pool is initialized with:

```rust
pub struct PoolConfig {
    pub factory: Address,      // Factory that deployed this pool
    pub router: Address,       // Router for swap routing
    pub creator: Address,      // Pool creator
    pub token_a: Address,      // First token (original order)
    pub token_b: Address,      // Second token (original order)
    pub fee_bps: u32,          // Trading fee (5, 30, or 100)
    pub creator_fee_bps: u32,  // Creator's share of fees
}
```

---

## ⚙️ Quick Setup

### Step 1: Deploy Pool via Factory

Pools are created through the Factory contract, not deployed directly. After deploying Factory and Router, create a pool:

```bash
# Set your environment
export NETWORK="testnet"
export FACTORY="YOUR_FACTORY_ADDRESS"
export SOURCE="alice"  # your account

# Create pool
stellar contract invoke \
  --id $FACTORY \
  --source $SOURCE \
  --network $NETWORK \
  -- create_pool \
  --creator alice \
  --params '{
    "token_a": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
    "token_b": "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
    "fee_bps": "30",
    "creator_fee_bps": "100",
    "initial_sqrt_price_x64": "18446744073709551616",
    "amount0_desired": "1000000000",
    "amount1_desired": "1000000000",
    "lower_tick": -600,
    "upper_tick": 600,
    "lock_duration": 0
  }'
```

**Output will show the pool address:**
```bash
"CAYNHCAVRMSI3HXKOS7HZ6OUWBA2WQKELIKQ3T532F44QKSOCLDJZ62H"
```

---

### Step 2: Save Pool Address

**IMMEDIATELY save the returned pool address:**

```bash
# Export for current session
export POOL="CAYNHCAVRMSI3HXKOS7HZ6OUWBA2WQKELIKQ3T532F44QKSOCLDJZ62H"

# Verify it's set
echo $POOL
# Should output: CAYNHCAVRMSI3HXKOS7HZ6OUWBA2WQKELIKQ3T532F44QKSOCLDJZ62H
```

---

### Step 3: Create Environment File (Recommended)

Save addresses permanently for easy reuse:

```bash
# Create .env.testnet file
cat > .env.testnet << 'EOF'
# BelugaSwap Testnet Environment
export NETWORK="testnet"

# Core Contracts
export FACTORY="CAYNNWB3GCC3WIIL7J2HS6QJTXGIL4E3INWBA6UF2OEIUTO2ZOVJFM7V"
export ROUTER="CC363XX4IXCC57KC5LMYOXRCC6L7VWFXAGBX7C2573XP36BTYRCQGM54"

# Your Pool (from create_pool output)
export POOL="CAYNHCAVRMSI3HXKOS7HZ6OUWBA2WQKELIKQ3T532F44QKSOCLDJZ62H"

# Tokens
export TOKEN_XLM="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
export TOKEN_USDC="CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"

# Accounts
export SOURCE="alice"
export USER="GBRKBE3LLJG2GH3MDLKYRKFFIZANSLEODQ4CO7USVSVMLQRJLYWQUIGO"
EOF

# Load environment
source .env.testnet

# Verify all variables
echo "Pool: $POOL"
echo "Network: $NETWORK"
echo "Factory: $FACTORY"
```

---

### Step 4: Test Pool Connection

Verify the pool is accessible and initialized:

```bash
# Should return pool state with liquidity, price, etc.
stellar contract invoke --id $POOL --network $NETWORK -- get_pool_state

# Should return configuration with factory, router, tokens, fees
stellar contract invoke --id $POOL --network $NETWORK -- get_pool_config

# Should return true
stellar contract invoke --id $POOL --network $NETWORK -- is_initialized
```

**If you get `error: a value is required for '--id <CONTRACT_ID>'`:**
- Your `$POOL` variable is not set
- Run `export POOL="YOUR_POOL_ADDRESS"` first
- Or load environment: `source .env.testnet`

---

### Step 5: Ready to Interact!

Now you can use all pool functions. Jump to:
- [View Functions](#view-functions-no-transaction-required) - Read pool data
- [Transaction Functions](#transaction-functions-require-signing) - Add/remove liquidity, collect fees
- [Common Use Cases](#-common-use-cases) - Complete examples

---

## 📚 Functions

### Swap Functions

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
- `sender`: Address executing the swap (must authorize)
- `token_in`: Input token address
- `amount_in`: Amount to swap
- `amount_out_min`: Minimum output (slippage protection)
- `sqrt_price_limit_x64`: Price limit (0 for no limit)

**Returns:**
```rust
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub current_tick: i32,
    pub sqrt_price_x64: u128,
}
```

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
Simulate swap without execution.

**Returns:**
```rust
pub struct PreviewResult {
    pub is_valid: bool,
    pub amount_in: i128,
    pub amount_out: i128,
    pub fee_amount: i128,
    pub price_impact_bps: i128,
    pub error_code: Symbol,
}
```

---

### Liquidity Functions

#### `add_liquidity`
```rust
pub fn add_liquidity(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
    amount0_desired: i128,
    amount1_desired: i128,
    amount0_min: i128,
    amount1_min: i128,
) -> (i128, i128, i128)
```
Add liquidity to a position.

**Returns:** `(liquidity, amount0, amount1)`

---

#### `remove_liquidity`
```rust
pub fn remove_liquidity(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
    liquidity: i128,
    amount0_min: i128,
    amount1_min: i128,
) -> (i128, i128)
```
Remove liquidity from a position. Checks creator lock via Factory.

**Returns:** `(amount0, amount1)`

---

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
Mint liquidity (called by Factory during pool creation). Tokens must already be transferred.

**Returns:** `liquidity`

---

### Fee Functions

#### `collect_fees`
```rust
pub fn collect_fees(
    env: Env,
    owner: Address,
    lower_tick: i32,
    upper_tick: i32,
) -> (u128, u128)
```
Collect accumulated LP fees from a position.

**Returns:** `(fees0, fees1)`

---

#### `claim_creator_fees`
```rust
pub fn claim_creator_fees(env: Env) -> (u128, u128)
```
Claim accumulated creator fees. **Creator only.**

**Returns:** `(fees0, fees1)`

---

### View Functions

#### `get_pool_state`
```rust
pub fn get_pool_state(env: Env) -> PoolState
```

**Returns:**
```rust
pub struct PoolState {
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub liquidity: i128,
    pub tick_spacing: i32,
    pub token0: Address,
    pub token1: Address,
    pub fee_growth_global_0: u128,
    pub fee_growth_global_1: u128,
    pub creator_fees_0: u128,
    pub creator_fees_1: u128,
}
```

---

#### `get_pool_config`
```rust
pub fn get_pool_config(env: Env) -> PoolConfig
```
Get pool configuration (factory, router, creator, tokens, fees).

---

#### `get_router`
```rust
pub fn get_router(env: Env) -> Address
```
Get router address.

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

**Returns:**
```rust
pub struct PositionInfo {
    pub liquidity: i128,
    pub amount0: i128,
    pub amount1: i128,
    pub fees_owed_0: u128,
    pub fees_owed_1: u128,
}
```

---

#### `get_creator_fees`
```rust
pub fn get_creator_fees(env: Env) -> CreatorFeesInfo
```

**Returns:**
```rust
pub struct CreatorFeesInfo {
    pub fees_token0: u128,
    pub fees_token1: u128,
}
```

---

#### `get_swap_direction`
```rust
pub fn get_swap_direction(env: Env, token_in: Address) -> bool
```
Get swap direction. Returns `true` for token0→token1, `false` for token1→token0.

---

#### `is_initialized`
```rust
pub fn is_initialized(env: Env) -> bool
```
Check if pool is initialized.

---

## 🔒 Creator Lock Integration

Pool queries Factory to check creator lock status:

```rust
// Called internally during remove_liquidity
fn is_position_locked(
    env: &Env,
    config: &PoolConfig,
    owner: &Address,
    lower_tick: i32,
    upper_tick: i32,
) -> bool {
    // Only affects creator's locked position
    // Queries factory.is_liquidity_locked()
}
```

Creator fee is checked during swaps:
```rust
// Called internally during swap
fn get_active_creator_fee_bps_safe(env: &Env, config: &PoolConfig) -> i128 {
    // Queries factory.is_creator_fee_active()
    // Returns 0 if revoked or factory call fails
}
```

---

## ⚠️ Error Messages

| Error | Description |
|-------|-------------|
| `pool already initialized` | Cannot reinitialize |
| `invalid fee` | Fee outside valid range |
| `invalid creator fee` | Creator fee outside 0.1%-10% |
| `invalid tick spacing` | Tick spacing must be positive |
| `invalid tick range` | Lower tick >= upper tick |
| `invalid token` | Token not in this pool |
| `slippage exceeded` | Output below minimum |
| `liquidity too low` | Below minimum liquidity |
| `insufficient liquidity` | Not enough in position |
| `position is locked` | Creator lock still active |

---

## 💻 Local Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Stellar CLI (v25+)
cargo install stellar-cli

# Add WASM target
rustup target add wasm32v1-none
```

### Build

```bash
# From project root
stellar contract build

# Output: target/wasm32v1-none/release/belugaswap_pool.wasm
```

---

## 🔧 Pool Interaction

**⚠️ IMPORTANT:** Before using these commands, complete the [Quick Setup](#%EF%B8%8F-quick-setup) above to:
1. Create your pool via Factory
2. Get the pool address from the output
3. Export the pool address as `$POOL`

If you've already created a pool and have the address, set it now:

```bash
export POOL="YOUR_POOL_ADDRESS"  # From create_pool output
export NETWORK="testnet"
```

To avoid setting variables repeatedly, create a `.env.testnet` file (see [Step 3 in Quick Setup](#step-3-create-environment-file-recommended)).

---

### View Functions (No Transaction Required)

These functions are read-only and don't require transaction signing:

```bash
# Get pool state
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- get_pool_state

# Get pool config
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- get_pool_config

# Preview swap (simulation)
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- preview_swap \
  --token_in "$TOKEN_A" \
  --amount_in "10000000" \
  --min_amount_out "0" \
  --sqrt_price_limit_x64 "0"

# Get position info
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- get_position \
  --owner "$USER" \
  --lower_tick "-887220" \
  --upper_tick "887220"

# Get creator fees
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- get_creator_fees

# Get swap direction
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- get_swap_direction \
  --token_in "$TOKEN_A"

# Check if initialized
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- is_initialized

# Get router address
stellar contract invoke \
  --id $POOL \
  --network $NETWORK \
  -- get_router
```

---

### Transaction Functions (Require Signing)

These functions modify state and require `--source` for transaction signing:

```bash
# Direct swap (usually done via Router)
stellar contract invoke \
  --id $POOL \
  --source $SOURCE \
  --network $NETWORK \
  -- swap \
  --sender "$USER" \
  --token_in "$TOKEN_A" \
  --amount_in "10000000" \
  --amount_out_min "9000000" \
  --sqrt_price_limit_x64 "0"

# Add liquidity
stellar contract invoke \
  --id $POOL \
  --source $SOURCE \
  --network $NETWORK \
  -- add_liquidity \
  --owner "$USER" \
  --lower_tick "-887220" \
  --upper_tick "887220" \
  --amount0_desired "10000000000" \
  --amount1_desired "10000000000" \
  --amount0_min "0" \
  --amount1_min "0"

# Remove liquidity
stellar contract invoke \
  --id $POOL \
  --source $SOURCE \
  --network $NETWORK \
  -- remove_liquidity \
  --owner "$USER" \
  --lower_tick "-887220" \
  --upper_tick "887220" \
  --liquidity "1000000" \
  --amount0_min "0" \
  --amount1_min "0"

# Collect LP fees
stellar contract invoke \
  --id $POOL \
  --source $SOURCE \
  --network $NETWORK \
  -- collect_fees \
  --owner "$USER" \
  --lower_tick "-887220" \
  --upper_tick "887220"

# Claim creator fees (creator only)
stellar contract invoke \
  --id $POOL \
  --source $SOURCE \
  --network $NETWORK \
  -- claim_creator_fees
```


## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Factory Contract**: [contracts/factory](../factory/README.md)
- **Router Contract**: [contracts/router](../router/README.md)

---

**Last Updated:** February 2026  
**Soroban Version:** v25+  