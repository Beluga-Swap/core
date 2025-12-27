# BelugaSwap 🐋

**Concentrated Liquidity Market Maker (CLMM) on Soroban**

This is the MVP core of an AMM with Concentrated Liquidity on Soroban. You can initialize pools, add liquidity, swap tokens, execute cross-tick swaps, and collect fees.


## 🏗️ Architecture

```
belugaswap-core/
├── src/
│   ├── lib.rs          # Main contract & position management
│   ├── pool.rs         # Pool state management
│   ├── tick.rs         # Tick operations & cross-tick logic
│   ├── math.rs         # Q64.64 fixed-point math library
│   ├── swap.rs         # Swap engine & routing
│   ├── position.rs     # Position data structures
│   └── twap.rs         # TWAP oracle implementation
├── Cargo.toml
└── README.md
```

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Stellar CLI
cargo install --locked stellar-cli --features opt

# Add wasm target
rustup target add wasm32-unknown-unknown
```

### Build

```bash
# Clone repository
git clone https://github.com/yourusername/belugaswap-core
cd belugaswap-core

# Clean build (recommended)
cargo clean
rm -rf target/

# Build contract
cargo build --release --target wasm32-unknown-unknown
```

### Deploy

```bash
# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/belugaswap.wasm \
  --source alice \
  --network testnet

# Save the contract ID
CONTRACT_ID="<your_contract_id>"
```

## 📚 Core Functions

### Pool Management

#### `initialize`
Initialize a new pool with token pair and fee configuration.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin alice \
  --token_a <TOKEN_A_ADDRESS> \
  --token_b <TOKEN_B_ADDRESS> \
  --fee_bps 30 \
  --protocol_fee_bps 1000 \
  --sqrt_price_x64 18446744073709551616 \
  --current_tick 0 \
  --tick_spacing 60
```

**Parameters:**
- `admin`: Pool administrator address
- `token_a`, `token_b`: Token addresses (sorted)
- `fee_bps`: Trading fee in basis points (30 = 0.3%)
- `protocol_fee_bps`: Protocol's share (1000 = 10% of trading fee)
- `sqrt_price_x64`: Initial price in Q64.64 format
- `current_tick`: Initial tick
- `tick_spacing`: Tick spacing (60 for volatile, 10 for stablecoins)

---

### Liquidity Management

#### `add_liquidity`
Add liquidity to a specific price range.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  add_liquidity \
  --owner alice \
  --lower_tick -60 \
  --upper_tick 60 \
  --amount0_desired 10000000 \
  --amount1_desired 10000000 \
  --amount0_min 0 \
  --amount1_min 0
```

**Returns:** `[liquidity, amount0, amount1]`

#### `remove_liquidity`
Remove liquidity from a position.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  remove_liquidity \
  --owner alice \
  --lower_tick -60 \
  --upper_tick 60 \
  --liquidity_delta 1000000
```

**Returns:** `[amount0, amount1]`

---

### Trading

#### `swap`
Execute a token swap.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source bob \
  --network testnet \
  -- \
  swap \
  --caller bob \
  --amount_specified 20000000 \
  --min_amount_out 18000000 \
  --zero_for_one true \
  --sqrt_price_limit_x64 0
```

**Parameters:**
- `amount_specified`: Input amount (in stroops)
- `min_amount_out`: Minimum output (slippage protection)
- `zero_for_one`: true = token0→token1, false = token1→token0
- `sqrt_price_limit_x64`: Price limit (0 = no limit)

**Returns:** `{amount_in, amount_out, current_tick, sqrt_price_x64}`

#### `preview_swap`
Preview swap output without executing.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source bob \
  --network testnet \
  -- \
  preview_swap \
  --amount_specified 20000000 \
  --min_amount_out 18000000 \
  --zero_for_one true \
  --sqrt_price_limit_x64 0
```

**Returns:** `{amount_in_used, amount_out_expected, fee_paid, price_impact_bps, is_valid}`

---

### Fee Collection

#### `collect`
Collect earned fees from a position.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  collect \
  --owner alice \
  --lower_tick -60 \
  --upper_tick 60
```

**Returns:** `[amount0_collected, amount1_collected]`

---

### View Functions

#### `get_position`
Get position information.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  get_position \
  --owner alice \
  --lower -60 \
  --upper 60
```

**Returns:** `{liquidity, amount0, amount1, fees_owed_0, fees_owed_1}`

#### `get_pool_state`
Get current pool state.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- \
  get_pool_state
```

**Returns:** Pool state including tick, price, liquidity, and fees.

---

## 🧪 Test Results

U can check manual test on folder tests/

### Test Scenarios

#### ✅ Test 1: Pool Initialization

#### ✅ Test 2: Multi-Range Liquidity

#### ✅ Test 3: Cross-Tick Swap (Down)

#### ✅ Test 4: Cross-Tick Swap (Up)

#### ✅ Test 5: Fee Collection


---

## 🛠️ Development

### Run Tests

```bash
cargo test
```

### Format Code

```bash
cargo fmt
```

### Check Lints

```bash
cargo clippy
```

---

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines.

---

## 📄 License

see LICENSE file for details

---

## 👥 Team

Built with 🐋 by the BelugaSwap team

---
