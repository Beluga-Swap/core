# BelugaSwap Factory

Factory contract for deploying and managing BelugaSwap liquidity pools.

---

## 🏭 Factory Functions

The factory contract provides:

### 1. **Pool Deployment**
- Atomic pool creation (deploy + init + LP + lock)
- Automatic token ordering
- Creator fee configuration

### 2. **Fee Tier Management**
- Configure fee tiers with tick spacing
- Enable/disable fee tiers
- Multiple fee tier support (0.05%, 0.30%, 1.00%)

### 3. **Creator Liquidity Lock**
- Lock initial LP tokens
- Creator fee while locked
- Unlock revokes creator fee permanently

### 4. **Pool Registry**
- Track all deployed pools
- Lookup by token pair + fee
- Enumerate all pools

---

## 🔧 Factory Configuration

Factory is initialized with:

```rust
pub struct FactoryConfig {
    pub admin: Address,
    pub pool_wasm_hash: BytesN<32>,
    pub router: Option<Address>,
}
```

---

## 📚 Functions

### Pool Creation

#### `create_pool`
```rust
pub fn create_pool(
    env: Env,
    creator: Address,
    params: CreatePoolParams,
) -> Result<Address, FactoryError>
```
Create pool atomically: deploy + initialize + add initial LP + lock.

**Parameters:**
```rust
pub struct CreatePoolParams {
    pub token_a: Address,
    pub token_b: Address,
    pub fee_bps: u32,
    pub creator_fee_bps: u32,
    pub initial_sqrt_price_x64: u128,
    pub amount0_desired: i128,
    pub amount1_desired: i128,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub lock_duration: u32,
}
```

**Returns:** Pool contract address

---

### Fee Tier Management

#### `set_fee_tier`
```rust
pub fn set_fee_tier(
    env: Env,
    fee_bps: u32,
    tick_spacing: i32,
    enabled: bool,
) -> Result<(), FactoryError>
```
Add or update fee tier configuration (admin only).

#### `get_fee_tier`
```rust
pub fn get_fee_tier(env: Env, fee_bps: u32) -> Result<FeeTier, FactoryError>
```

**Returns:**
```rust
pub struct FeeTier {
    pub fee_bps: u32,
    pub tick_spacing: i32,
    pub enabled: bool,
}
```

---

### Pool Registry

#### `get_pool_address`
```rust
pub fn get_pool_address(
    env: Env,
    token_a: Address,
    token_b: Address,
    fee_bps: u32,
) -> Result<Address, FactoryError>
```

#### `is_pool_deployed`
```rust
pub fn is_pool_deployed(
    env: Env,
    token_a: Address,
    token_b: Address,
    fee_bps: u32,
) -> bool
```

#### `get_all_pool_addresses`
```rust
pub fn get_all_pool_addresses(env: Env) -> Vec<Address>
```

#### `get_total_pools`
```rust
pub fn get_total_pools(env: Env) -> u32
```

---

### Creator Lock Management

#### `get_creator_lock`
```rust
pub fn get_creator_lock(env: Env, pool: Address) -> Result<CreatorLock, FactoryError>
```

**Returns:**
```rust
pub struct CreatorLock {
    pub creator: Address,
    pub liquidity: i128,
    pub unlock_time: u32,
    pub fee_active: bool,
}
```

#### `is_liquidity_locked`
```rust
pub fn is_liquidity_locked(env: Env, pool: Address) -> bool
```

#### `is_creator_fee_active`
```rust
pub fn is_creator_fee_active(env: Env, pool: Address) -> bool
```

#### `unlock_creator_liquidity`
```rust
pub fn unlock_creator_liquidity(env: Env, pool: Address) -> Result<(), FactoryError>
```
**⚠️ REVOKES CREATOR FEE PERMANENTLY!**

---

### Admin Functions

#### `initialize`
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    pool_wasm_hash: BytesN<32>,
) -> Result<(), FactoryError>
```

#### `set_router`
```rust
pub fn set_router(env: Env, router: Address) -> Result<(), FactoryError>
```

#### `set_admin`
```rust
pub fn set_admin(env: Env, new_admin: Address) -> Result<(), FactoryError>
```

#### `set_pool_wasm_hash`
```rust
pub fn set_pool_wasm_hash(env: Env, wasm_hash: BytesN<32>) -> Result<(), FactoryError>
```

---

### View Functions

#### `is_ready`
```rust
pub fn is_ready(env: Env) -> bool
```

#### `get_router`
```rust
pub fn get_router(env: Env) -> Result<Address, FactoryError>
```

---

## ⚠️ Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 1 | `AlreadyInitialized` | Factory already initialized |
| 2 | `NotInitialized` | Factory not initialized |
| 9 | `ExpirationTooHigh` | Approval expiration ledger too high |
| 10 | `Unauthorized` | Not authorized |
| 11 | `PoolAlreadyExists` | Pool for pair+fee already deployed |
| 12 | `InvalidFeeTier` | Fee tier not enabled |
| 13 | `TrustlineMissing` | Token trustline missing |
| 20 | `InsufficientInitialLiquidity` | Initial LP amount too low |
| 21 | `InvalidTickRange` | Ticks not divisible by spacing |
| 30 | `RouterAlreadySet` | Router already configured |
| 31 | `RouterNotSet` | Router not yet configured |

---

## 💻 Local Setup

### Prerequisites

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install stellar-cli
rustup target add wasm32v1-none
```

### Build

```bash
stellar contract build
```

### Deploy

```bash
# Setup
stellar keys generate alice --network testnet
stellar keys fund alice --network testnet

export NETWORK="testnet"
export SOURCE="alice"
export ADMIN=$(stellar keys address alice)

# Upload pool WASM
export POOL_WASM_HASH=$(stellar contract upload \
  --wasm target/wasm32v1-none/release/belugaswap_pool.wasm \
  --source $SOURCE --network $NETWORK)

# Deploy factory
export FACTORY=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/beluga_factory.wasm \
  --source $SOURCE --network $NETWORK)

# Initialize
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- initialize --admin $ADMIN --pool_wasm_hash $POOL_WASM_HASH

# Set router (after router deployment)
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- set_router --router $ROUTER

# Set fee tier
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- set_fee_tier --fee_bps 30 --tick_spacing 60 --enabled true

# Verify
stellar contract invoke --id $FACTORY --network $NETWORK -- is_ready
```

### Create Pool

```bash
export TOKEN_A="<TOKEN_A_ADDRESS>"
export TOKEN_B="<TOKEN_B_ADDRESS>"

# Approve tokens
stellar contract invoke --id $TOKEN_A --source $SOURCE --network $NETWORK \
  -- approve --from $SOURCE --spender $FACTORY --amount 1000000000 --expiration_ledger 3110400

stellar contract invoke --id $TOKEN_B --source $SOURCE --network $NETWORK \
  -- approve --from $SOURCE --spender $FACTORY --amount 1000000000 --expiration_ledger 3110400

# Create pool
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- create_pool \
  --creator $SOURCE \
  --params '{ "amount0_desired": "1000000000", "amount1_desired": "1000000000", "creator_fee_bps": 100, "fee_bps": 30, "initial_sqrt_price_x64": "18446744073709551616", "lock_duration": 0, "lower_tick": -600, "token_a": "<TOKEN_A_ADDRESS>", "token_b": "<TOKEN_B_ADDRESS>", "upper_tick": 600 }'
```

---

## 📋 CreatePoolParams Reference

| Field | Type | Description |
|-------|------|-------------|
| `token_a` | Address | First token contract address |
| `token_b` | Address | Second token contract address |
| `fee_bps` | u32 | Pool fee (must match enabled tier) |
| `creator_fee_bps` | u32 | Creator fee in basis points |
| `initial_sqrt_price_x64` | u128 (string) | Initial sqrt price × 2^64 |
| `amount0_desired` | i128 (string) | Initial liquidity for token0 |
| `amount1_desired` | i128 (string) | Initial liquidity for token1 |
| `lower_tick` | i32 | Lower tick (divisible by tick_spacing) |
| `upper_tick` | i32 | Upper tick (divisible by tick_spacing) |
| `lock_duration` | u32 | Lock duration in seconds (0 = no lock) |

> **Note:** `i128` and `u128` types must be passed as strings in JSON.

---

## 🔗 Links

- **Router Contract**: [contracts/router](../router/README.md)
- **Pool Contract**: [contracts/pool](../pool/README.md)