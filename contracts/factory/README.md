# BelugaSwap Factory

Permissionless pool deployment factory contract with creator incentive system and router integration.

---

## 🏭 Factory Functions

The factory contract is responsible for:

### 1. **Pool Deployment (Atomic)**
Factory creates new pools in a single atomic transaction that includes:
- Deploy new pool contract
- Initialize pool with factory + router addresses
- Transfer initial tokens from creator
- Mint first liquidity position
- Lock creator liquidity

### 2. **Router Registry**
Links to Router contract for smart swap routing:
- Must set router before creating pools
- Router address passed to every pool on creation
- `is_ready()` returns true only when router is set

### 3. **Fee Tier Standardization**
Provides 3 standard fee tiers:
| Fee | Bps | Tick Spacing | Use Case |
|-----|-----|--------------|----------|
| 0.05% | 5 | 10 | Stablecoins |
| 0.30% | 30 | 60 | Volatile tokens |
| 1.00% | 100 | 200 | Meme/Exotic tokens |

### 4. **Duplicate Prevention**
Prevents duplicate pool deployments for the same token pair with the same fee tier using deterministic addressing.

### 5. **Creator Lock Management**
Manages locked liquidity from pool creators with rules:
- Creator must lock initial LP to earn creator fees
- Lock can be temporary (min 7 days) or permanent
- Unlock LP = **REVOKE creator fee PERMANENTLY**
- LP out of range = temporarily no fees

---

## 🚀 Deployment Flow

```
1. Deploy Factory    → factory.initialize(admin, pool_wasm_hash)
2. Deploy Router     → router.initialize(factory, admin)
3. Link Router       → factory.set_router(router)
4. Verify Ready      → factory.is_ready() == true
5. Create Pools      → factory.create_pool(...)
```

---

## 👨‍💻 As a Creator

### Pool Creation Flow

```rust
// 1. Prepare parameters
let params = CreatePoolParams {
    token_a: token_usdc_address,
    token_b: token_xlm_address,
    fee_bps: 30,                    // 0.30% fee tier
    creator_fee_bps: 100,           // 1% of total fees
    initial_sqrt_price_x64: sqrt_price,
    amount0_desired: 1_000_000_000, // 100 USDC (7 decimals)
    amount1_desired: 500_000_000,   // 50 XLM (7 decimals)
    lower_tick: -887220,            // Full range
    upper_tick: 887220,             // Full range
    lock_duration: 0,               // 0 = permanent lock
};

// 2. Call create_pool
let pool_address = factory.create_pool(&creator, &params);
```

### Minimum Requirements

| Parameter | Requirement |
|-----------|-------------|
| Initial Liquidity | Min 0.1 token (1_000_000 with 7 decimals) per token |
| Lock Duration | 0 = permanent, or min 120,960 ledgers (~7 days) |
| Creator Fee | 10-1000 bps (0.1% - 10% of total fees) |
| Tick Range | Must be aligned with tick spacing |

### Creator Fee Rules

✅ **Earns Creator Fee:**
- Liquidity is still locked
- Position is in-range

❌ **No Creator Fee:**
- Liquidity has been unlocked (PERMANENT)
- Position is out of range (temporary)
- Creator fee has been revoked

⚠️ **IMPORTANT**: Unlocking liquidity = **REVOKE creator fee FOREVER!**

---

## 📚 Functions

### Write Functions

#### `initialize`
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    pool_wasm_hash: BytesN<32>,
) -> Result<(), FactoryError>
```
Initialize the factory contract. Router is set separately after deployment.

---

#### `set_router`
```rust
pub fn set_router(
    env: Env,
    router: Address,
) -> Result<(), FactoryError>
```
Set router address. **Admin only.** Must be called before creating pools.

---

#### `create_pool`
```rust
pub fn create_pool(
    env: Env,
    creator: Address,
    params: CreatePoolParams,
) -> Result<Address, FactoryError>
```
Deploy new pool (atomic: deploy + init + LP + lock). Requires router to be set.

**Errors:**
- `RouterNotSet`: Router not configured yet

---

#### `unlock_creator_liquidity`
```rust
pub fn unlock_creator_liquidity(
    env: Env,
    pool_address: Address,
    creator: Address,
) -> Result<i128, FactoryError>
```
Unlock creator liquidity. **REVOKES creator fee PERMANENTLY!**

---

### Read Functions

#### `is_ready`
```rust
pub fn is_ready(env: Env) -> bool
```
Check if factory is ready (initialized + router set).

---

#### `get_router`
```rust
pub fn get_router(env: Env) -> Option<Address>
```
Get router address.

---

#### `get_pool_address`
```rust
pub fn get_pool_address(
    env: Env,
    token_a: Address,
    token_b: Address,
    fee_bps: u32
) -> Option<Address>
```
Get pool address by token pair and fee tier.

---

#### `is_pool_deployed`
```rust
pub fn is_pool_deployed(
    env: Env,
    token_a: Address,
    token_b: Address,
    fee_bps: u32
) -> bool
```
Check if pool exists for specific pair+fee.

---

#### `get_total_pools`
```rust
pub fn get_total_pools(env: Env) -> u32
```
Get total number of deployed pools.

---

#### `get_all_pool_addresses`
```rust
pub fn get_all_pool_addresses(env: Env) -> Vec<Address>
```
Get all deployed pool addresses.

---

#### `get_fee_tier`
```rust
pub fn get_fee_tier(env: Env, fee_bps: u32) -> Option<FeeTier>
```
Get specific fee tier configuration.

---

#### `get_creator_lock`
```rust
pub fn get_creator_lock(
    env: Env,
    pool_address: Address,
    creator: Address
) -> Option<CreatorLock>
```
Get creator lock information for a specific pool.

---

#### `is_liquidity_locked`
```rust
pub fn is_liquidity_locked(
    env: Env,
    pool_address: Address,
    creator: Address,
    lower_tick: i32,
    upper_tick: i32,
) -> bool
```
Check if creator's position is still locked. Called by Pool contract.

---

#### `is_creator_fee_active`
```rust
pub fn is_creator_fee_active(
    env: Env,
    pool_address: Address,
    creator: Address,
) -> bool
```
Check if creator fee is still active. Called by Pool contract during swaps.

---

### Admin Functions

#### `set_pool_wasm_hash`
```rust
pub fn set_pool_wasm_hash(env: Env, new_hash: BytesN<32>) -> Result<(), FactoryError>
```
Update WASM hash for future pool deployments. **Admin only.**

---

#### `set_admin`
```rust
pub fn set_admin(env: Env, new_admin: Address) -> Result<(), FactoryError>
```
Transfer admin role to new address. **Admin only.**

---

#### `set_fee_tier`
```rust
pub fn set_fee_tier(
    env: Env,
    fee_bps: u32,
    tick_spacing: i32,
    enabled: bool,
) -> Result<(), FactoryError>
```
Add/update fee tier configuration. **Admin only.**

---

## ⚠️ Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 1 | `AlreadyInitialized` | Factory already initialized |
| 2 | `NotInitialized` | Factory not initialized yet |
| 5 | `RouterNotSet` | Router not configured |
| 6 | `RouterAlreadySet` | Router already configured |
| 10 | `PoolAlreadyExists` | Pool for this pair+fee exists |
| 11 | `InvalidTokenPair` | Token A equals token B |
| 12 | `InvalidFeeTier` | Fee tier invalid/disabled |
| 13 | `InvalidTickSpacing` | Tick not aligned |
| 14 | `InvalidTickRange` | Lower tick >= upper tick |
| 15 | `InvalidInitialPrice` | sqrt_price_x64 = 0 |
| 16 | `InvalidCreatorFee` | Outside 0.1%-10% range |
| 20 | `InsufficientInitialLiquidity` | Amount < 0.1 token |
| 21 | `InvalidLockDuration` | Duration < 7 days |
| 22 | `LiquidityStillLocked` | Lock not expired |
| 30 | `NotPoolCreator` | Caller not creator |
| 31 | `CreatorFeeRevoked` | Fee already revoked |
| 32 | `CreatorLockNotFound` | Lock not found |
| 50 | `Unauthorized` | Caller not admin |

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

# Output: target/wasm32v1-none/release/beluga_factory.wasm
```

### Deploy

```bash
# Setup
export NETWORK="testnet"
export SOURCE="admin"
export ADMIN=$(stellar keys address $SOURCE)

# Upload Pool WASM first
export POOL_WASM_HASH=$(stellar contract upload \
  --wasm target/wasm32v1-none/release/belugaswap_pool.wasm \
  --source $SOURCE --network $NETWORK)

# Deploy Factory
export FACTORY=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/beluga_factory.wasm \
  --source $SOURCE --network $NETWORK)

# Initialize
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- initialize --admin $ADMIN --pool_wasm_hash $POOL_WASM_HASH

# Check (should be false - no router yet)
stellar contract invoke --id $FACTORY --network $NETWORK -- is_ready

# After deploying router, link it:
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- set_router --router $ROUTER

# Verify (should be true now)
stellar contract invoke --id $FACTORY --network $NETWORK -- is_ready
```

### Create Pool

```bash
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- create_pool --creator $ADMIN --params '{
    "token_a": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
    "token_b": "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
    "fee_bps": 30,
    "creator_fee_bps": 100,
    "initial_sqrt_price_x64": 18446744073709551616,
    "amount0_desired": 10000000000,
    "amount1_desired": 10000000000,
    "lower_tick": -887220,
    "upper_tick": 887220,
    "lock_duration": 0
  }'
```

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Pool Contract**: [contracts/pool](../pool/README.md)
- **Router Contract**: [contracts/router](../router/README.md)