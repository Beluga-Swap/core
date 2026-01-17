# BelugaSwap Factory

Permissionless pool deployment factory contract with creator incentive system for BelugaSwap.
---

## 🏭 Factory Functions

The factory contract is responsible for:

### 1. **Pool Deployment (Atomic)**
Factory creates new pools in a single atomic transaction that includes:
- Deploy new pool contract
- Initialize pool with correct parameters
- Transfer initial tokens from creator
- Mint first liquidity position
- Lock creator liquidity

### 2. **Fee Tier Standardization**
Provides 3 standard fee tiers:
- **5 bps (0.05%)** - For stablecoins (tick spacing: 10)
- **30 bps (0.30%)** - For volatile tokens (tick spacing: 60)
- **100 bps (1.00%)** - For meme/exotic tokens (tick spacing: 200)

### 3. **Duplicate Prevention**
Prevents duplicate pool deployments for the same token pair with the same fee tier using deterministic addressing.

### 4. **Creator Lock Management**
Manages locked liquidity from pool creators with rules:
- Creator must lock initial LP to earn creator fees
- Lock can be temporary (min 7 days) or permanent
- Unlock LP = **REVOKE creator fee PERMANENTLY**
- LP out of range = temporarily no fees

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

- **Initial Liquidity**: Minimum 0.1 token (1_000_000 with 7 decimals) per token
- **Lock Duration**: 
  - 0 = permanent lock
  - Minimum 120,960 ledgers (~7 days at 5s/ledger)
- **Creator Fee**: 10-1000 bps (0.1% - 10% of total fees)
- **Tick Range**: Must be aligned with tick spacing

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

## ✅ Pool Verification

### 1. Check if Pool Exists

```rust
let exists = factory.is_pool_deployed(
    &token_a,
    &token_b,
    &30 // fee_bps
);
```

### 2. Get Pool Address

```rust
let pool_address = factory.get_pool_address(
    &token_a,
    &token_b,
    &30
);

match pool_address {
    Some(addr) => println!("Pool exists at: {}", addr),
    None => println!("Pool not found")
}
```

### 3. Verify Creator Lock

```rust
let lock_info = factory.get_creator_lock(&pool_address, &creator);

match lock_info {
    Some(lock) => {
        println!("Liquidity: {}", lock.liquidity);
        println!("Lock End: {}", lock.lock_end);
        println!("Is Permanent: {}", lock.is_permanent);
        println!("Fee Revoked: {}", lock.fee_revoked);
    }
    None => println!("No creator lock found")
}
```

### 4. List All Pools

```rust
let total = factory.get_total_pools();
let all_pools = factory.get_all_pool_addresses();

println!("Total pools: {}", total);
for pool in all_pools.iter() {
    println!("Pool: {}", pool);
}
```

---

## 📚 Functions

### Write Functions (3)

#### `initialize`
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    pool_wasm_hash: BytesN<32>,
) -> Result<(), FactoryError>
```
Initialize the factory contract. Can only be called once.

**Parameters:**
- `admin`: Factory admin address
- `pool_wasm_hash`: WASM hash of pool contract

**Errors:**
- `AlreadyInitialized`: Factory already initialized

---

#### `create_pool`
```rust
pub fn create_pool(
    env: Env,
    creator: Address,
    params: CreatePoolParams,
) -> Result<Address, FactoryError>
```
Deploy new pool (atomic: deploy + init + LP + lock).

**Parameters:**
- `creator`: Creator address providing initial liquidity
- `params`: Pool creation parameters (see `CreatePoolParams`)

**Returns:** Newly deployed pool address

**Errors:**
- `NotInitialized`: Factory not initialized
- `InvalidTokenPair`: Token A equals token B
- `PoolAlreadyExists`: Pool for this pair+fee already exists
- `InvalidFeeTier`: Fee tier invalid or disabled
- `InvalidTickSpacing`: Tick not aligned with spacing
- `InvalidTickRange`: Lower tick >= upper tick
- `InsufficientInitialLiquidity`: Amount below minimum
- `InvalidLockDuration`: Duration < 7 days (and not 0)
- `InvalidCreatorFee`: Creator fee outside 0.1%-10% range
- `InvalidInitialPrice`: sqrt_price_x64 = 0

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

**Parameters:**
- `pool_address`: Pool address
- `creator`: Creator address

**Returns:** Amount of liquidity unlocked

**Errors:**
- `CreatorLockNotFound`: Lock not found
- `NotPoolCreator`: Caller is not pool creator
- `CreatorFeeRevoked`: Fee already revoked
- `LiquidityStillLocked`: Lock period not ended yet

---

### Read Functions (6)

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
Check if pool is already deployed for specific pair+fee.

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

**Returns:**
```rust
struct FeeTier {
    fee_bps: u32,
    tick_spacing: i32,
    enabled: bool,
}
```

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

**Returns:**
```rust
struct CreatorLock {
    pool: Address,
    creator: Address,
    liquidity: i128,
    lower_tick: i32,
    upper_tick: i32,
    lock_start: u32,
    lock_end: u32,        // u32::MAX = permanent
    is_permanent: bool,
    is_unlocked: bool,
    fee_revoked: bool,    // true = PERMANENT
}
```

---

### Admin Functions (3)

#### `set_pool_wasm_hash`
```rust
pub fn set_pool_wasm_hash(
    env: Env,
    new_hash: BytesN<32>
) -> Result<(), FactoryError>
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

**Errors:**
- `InvalidTickSpacing`: tick_spacing <= 0

---

## ⚠️ Error Codes

### Initialization Errors
| Code | Error | Description |
|------|-------|-----------|
| 1 | `AlreadyInitialized` | Factory already initialized |
| 2 | `NotInitialized` | Factory not initialized yet |

### Pool Creation Errors
| Code | Error | Description |
|------|-------|-----------|
| 10 | `PoolAlreadyExists` | Pool for this pair+fee already exists |
| 11 | `InvalidTokenPair` | Token A equals token B |
| 12 | `InvalidFeeTier` | Fee tier invalid/disabled |
| 13 | `InvalidTickSpacing` | Tick not aligned with spacing |
| 14 | `InvalidTickRange` | Lower tick >= upper tick |
| 15 | `InvalidInitialPrice` | sqrt_price_x64 = 0 |
| 16 | `InvalidCreatorFee` | Creator fee outside 0.1%-10% |

### Liquidity Errors
| Code | Error | Description |
|------|-------|-----------|
| 20 | `InsufficientInitialLiquidity` | Amount < 0.1 token |
| 21 | `InvalidLockDuration` | Duration < 7 days (and != 0) |
| 22 | `LiquidityStillLocked` | Lock period not ended yet |

### Creator Errors
| Code | Error | Description |
|------|-------|-----------|
| 30 | `NotPoolCreator` | Caller is not pool creator |
| 31 | `CreatorFeeRevoked` | Creator fee already revoked |
| 32 | `CreatorLockNotFound` | Lock not found |

### Admin Errors
| Code | Error | Description |
|------|-------|-----------|
| 50 | `Unauthorized` | Caller is not admin |

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
cd core/contracts/factory

# Folder structure:
# factory/
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

# Output: target/wasm32-unknown-unknown/release/belugaswap_factory.wasm
```

### Local Testing

```bash
# Run tests (if available)
cargo test

# Optimize WASM
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/belugaswap_factory.wasm
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

# Deploy factory
FACTORY_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/belugaswap_factory.wasm \
  --source alice \
  --network testnet)

echo "Factory deployed at: $FACTORY_ID"

# Deploy pool contract (to be installed in factory)
POOL_ID=$(soroban contract deploy \
  --wasm path/to/pool.wasm \
  --source alice \
  --network testnet)

# Install pool WASM to factory
POOL_WASM_HASH=$(soroban contract install \
  --wasm path/to/pool.wasm \
  --source alice \
  --network testnet)

echo "Pool WASM hash: $POOL_WASM_HASH"

# Initialize factory
soroban contract invoke \
  --id $FACTORY_ID \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin GBXXX... \
  --pool_wasm_hash $POOL_WASM_HASH
```

### Interact with Factory

```bash
# Create pool
soroban contract invoke \
  --id $FACTORY_ID \
  --source alice \
  --network testnet \
  -- \
  create_pool \
  --creator GBXXX... \
  --params '{
    "token_a": "CDXXX...",
    "token_b": "CDYYY...",
    "fee_bps": 30,
    "creator_fee_bps": 100,
    "initial_sqrt_price_x64": "18446744073709551616",
    "amount0_desired": "1000000000",
    "amount1_desired": "500000000",
    "lower_tick": -887220,
    "upper_tick": 887220,
    "lock_duration": 0
  }'

# Get pool address
soroban contract invoke \
  --id $FACTORY_ID \
  --network testnet \
  -- \
  get_pool_address \
  --token_a CDXXX... \
  --token_b CDYYY... \
  --fee_bps 30

# Check creator lock
soroban contract invoke \
  --id $FACTORY_ID \
  --network testnet \
  -- \
  get_creator_lock \
  --pool_address POOL_ADDRESS \
  --creator CREATOR_ADDRESS
```

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Factory Contract**: [contracts/factory](https://github.com/Beluga-Swap/core/tree/main/contracts/factory)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)
- **Stellar Laboratory**: [laboratory.stellar.org](https://laboratory.stellar.org)

---