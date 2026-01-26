# BelugaSwap Router

Smart routing contract for optimal swap execution with multi-hop and split routing support.

---

## 🛣️ Router Functions

The router contract provides:

### 1. **Best Route Finding**
- Auto-select pool with best output across all fee tiers
- Compare quotes from 0.05%, 0.30%, and 1.00% pools
- Return best quote with price impact

### 2. **Multi-Hop Swaps**
- Execute A → B → C in single transaction
- Up to 4 hops supported
- Intermediate tokens held by router

### 3. **Split Routing**
- Split large orders across multiple pools
- Minimize price impact
- Aggregate outputs

### 4. **Quote/Preview**
- Simulate swaps without execution
- Compare all available pools
- Get price impact estimates

---

## 🔧 Router Configuration

Router is initialized with:

```rust
pub struct RouterConfig {
    pub factory: Address,  // Factory for pool lookup
    pub admin: Address,    // Admin address
}
```

---

## 📚 Functions

### Swap Functions

#### `swap_exact_input`
```rust
pub fn swap_exact_input(
    env: Env,
    sender: Address,
    params: ExactInputParams,
) -> Result<SwapResult, RouterError>
```
Swap with exact input amount, auto-selecting best pool.

**Parameters:**
```rust
pub struct ExactInputParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out_min: i128,    // Slippage protection
    pub fee_tiers: Vec<u32>,     // Empty = try all
    pub recipient: Address,
    pub deadline: u32,           // Ledger sequence
}
```

**Returns:**
```rust
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub pools_used: Vec<Address>,
    pub fee_tiers_used: Vec<u32>,
}
```

---

#### `swap_multihop`
```rust
pub fn swap_multihop(
    env: Env,
    sender: Address,
    params: MultihopExactInputParams,
) -> Result<SwapResult, RouterError>
```
Execute multi-hop swap (A → B → C → ...).

**Parameters:**
```rust
pub struct MultihopExactInputParams {
    pub token_in: Address,
    pub amount_in: i128,
    pub path: Vec<Hop>,          // Intermediate tokens + fees
    pub amount_out_min: i128,
    pub recipient: Address,
    pub deadline: u32,
}

pub struct Hop {
    pub token: Address,  // Next token in path
    pub fee_bps: u32,    // Fee tier for this hop
}
```

**Example Path (XLM → USDC → USDB):**
```rust
path: [
    Hop { token: USDC, fee_bps: 30 },   // XLM→USDC via 0.3% pool
    Hop { token: USDB, fee_bps: 5 },    // USDC→USDB via 0.05% pool
]
```

---

#### `swap_split`
```rust
pub fn swap_split(
    env: Env,
    sender: Address,
    token_in: Address,
    token_out: Address,
    amount_in: i128,
    amount_out_min: i128,
    splits: Vec<SplitQuote>,
    recipient: Address,
    deadline: u32,
) -> Result<SwapResult, RouterError>
```
Split swap across multiple pools.

**Parameters:**
```rust
pub struct SplitQuote {
    pub pool: Address,
    pub fee_bps: u32,
    pub amount_in: i128,   // Amount for this pool
    pub amount_out: i128,  // Expected output
}
```

---

### Quote Functions

#### `get_best_quote`
```rust
pub fn get_best_quote(
    env: Env,
    token_in: Address,
    token_out: Address,
    amount_in: i128,
    fee_tiers: Vec<u32>,  // Empty = try all
) -> Result<BestQuote, RouterError>
```

**Returns:**
```rust
pub struct BestQuote {
    pub pool: Address,
    pub fee_bps: u32,
    pub amount_out: i128,
    pub price_impact_bps: i128,
    pub all_quotes: Vec<PoolQuote>,  // All pool comparisons
}
```

---

#### `get_all_quotes`
```rust
pub fn get_all_quotes(
    env: Env,
    token_in: Address,
    token_out: Address,
    amount_in: i128,
    fee_tiers: Vec<u32>,
) -> Result<Vec<PoolQuote>, RouterError>
```

**Returns:**
```rust
pub struct PoolQuote {
    pub pool: Address,
    pub fee_bps: u32,
    pub amount_out: i128,
    pub price_impact_bps: i128,
}
```

---

#### `quote_multihop`
```rust
pub fn quote_multihop(
    env: Env,
    token_in: Address,
    amount_in: i128,
    path: Vec<Hop>,
) -> Result<i128, RouterError>
```
Get expected output for multi-hop path.

**Returns:** Final output amount

---

#### `get_split_quote`
```rust
pub fn get_split_quote(
    env: Env,
    token_in: Address,
    token_out: Address,
    amount_in: i128,
    fee_tiers: Vec<u32>,
) -> Result<AggregatedQuote, RouterError>
```

**Returns:**
```rust
pub struct AggregatedQuote {
    pub total_amount_in: i128,
    pub total_amount_out: i128,
    pub splits: Vec<SplitQuote>,
    pub is_split_recommended: bool,
}
```

---

### View Functions

#### `get_config`
```rust
pub fn get_config(env: Env) -> Result<RouterConfig, RouterError>
```
Get router configuration.

---

#### `get_factory`
```rust
pub fn get_factory(env: Env) -> Result<Address, RouterError>
```
Get factory address.

---

#### `is_initialized`
```rust
pub fn is_initialized(env: Env) -> bool
```
Check if router is initialized.

---

## ⚠️ Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 1 | `AlreadyInitialized` | Router already initialized |
| 2 | `NotInitialized` | Router not initialized |
| 10 | `InvalidPath` | Empty path |
| 11 | `PathTooLong` | Path exceeds 4 hops |
| 12 | `NoPoolsFound` | No pools for token pair |
| 13 | `InsufficientOutput` | Output below minimum |
| 14 | `SlippageExceeded` | Slippage check failed |
| 15 | `DeadlineExpired` | Transaction too late |
| 20 | `PoolNotFound` | Pool doesn't exist |
| 21 | `InvalidTokenPair` | Invalid token pair |
| 22 | `NoLiquidityAvailable` | Pool has no liquidity |
| 30 | `QuoteFailed` | Quote simulation failed |
| 31 | `InvalidAmount` | Amount <= 0 |
| 40 | `EmptySplits` | No valid splits |
| 41 | `SplitAmountMismatch` | Split amounts don't sum |
| 50 | `Unauthorized` | Not authorized |

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

# Output: target/wasm32v1-none/release/belugaswap_router.wasm
```

### Deploy

```bash
# Setup
export NETWORK="testnet"
export SOURCE="admin"
export ADMIN=$(stellar keys address $SOURCE)

# Deploy Router
export ROUTER=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/belugaswap_router.wasm \
  --source $SOURCE --network $NETWORK)

# Initialize (after Factory is deployed)
stellar contract invoke --id $ROUTER --source $SOURCE --network $NETWORK \
  -- initialize --factory $FACTORY --admin $ADMIN

# Link to Factory
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- set_router --router $ROUTER
```

### Usage Examples

```bash
# Token addresses
export XLM="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
export USDC="CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
export USDB="CAN6LSTLRS2NNPAQAABUTP7HXR22P5WEUZKQWCEYELK4IH5VHYYC4LQE"

# Get best quote
stellar contract invoke --id $ROUTER --network $NETWORK \
  -- get_best_quote \
  --token_in $XLM \
  --token_out $USDC \
  --amount_in 10000000 \
  --fee_tiers '[]'

# Single swap
stellar contract invoke --id $ROUTER --source $SOURCE --network $NETWORK \
  -- swap_exact_input \
  --sender $ADMIN \
  --params '{
    "token_in": "'$XLM'",
    "token_out": "'$USDC'",
    "amount_in": 10000000,
    "amount_out_min": 9000000,
    "fee_tiers": [],
    "recipient": "'$ADMIN'",
    "deadline": 999999999
  }'

# Multihop quote (XLM → USDC → USDB)
stellar contract invoke --id $ROUTER --network $NETWORK \
  -- quote_multihop \
  --token_in $XLM \
  --amount_in 10000000 \
  --path '[
    {"token": "'$USDC'", "fee_bps": 30},
    {"token": "'$USDB'", "fee_bps": 5}
  ]'

# Multihop swap
stellar contract invoke --id $ROUTER --source $SOURCE --network $NETWORK \
  -- swap_multihop \
  --sender $ADMIN \
  --params '{
    "token_in": "'$XLM'",
    "amount_in": 10000000,
    "path": [
      {"token": "'$USDC'", "fee_bps": 30},
      {"token": "'$USDB'", "fee_bps": 5}
    ],
    "amount_out_min": 1,
    "recipient": "'$ADMIN'",
    "deadline": 999999999
  }'

# Get all quotes for comparison
stellar contract invoke --id $ROUTER --network $NETWORK \
  -- get_all_quotes \
  --token_in $XLM \
  --token_out $USDC \
  --amount_in 10000000 \
  --fee_tiers '[]'
```

---

## 🔄 Swap Flow Diagrams

### Single Swap
```
User → Router.swap_exact_input()
         ↓
       Query Factory for best pool
         ↓
       Transfer token_in: User → Router
         ↓
       Approve & call Pool.swap()
         ↓
       Transfer token_out: Router → Recipient
```

### Multi-Hop Swap
```
User → Router.swap_multihop()
         ↓
       Transfer token_in: User → Router
         ↓
       Hop 1: Router → Pool1.swap() → Router holds intermediate
         ↓
       Hop 2: Router → Pool2.swap() → Router holds intermediate
         ↓
       Final: Transfer token_out → Recipient
```

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Factory Contract**: [contracts/factory](../factory/README.md)
- **Pool Contract**: [contracts/pool](../pool/README.md)