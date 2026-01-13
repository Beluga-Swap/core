# 🐋 BelugaSwap

**A Concentrated Liquidity AMM for Soroban (Stellar)**

BelugaSwap is a next-generation decentralized exchange built on the Stellar network using Soroban smart contracts. Inspired by Uniswap V3, it implements concentrated liquidity to maximize capital efficiency for liquidity providers.

---

## ✨ Key Features

### 1. Concentrated Liquidity — Capital Efficient
Unlike traditional AMMs that spread liquidity across the entire price curve (0 to ∞), BelugaSwap allows LPs to concentrate their capital within specific price ranges. This means:

- **Up to 4000x more capital efficiency** compared to traditional AMMs
- Earn more fees with less capital
- Better rates for traders due to deeper liquidity at active price ranges

### 2. Custom Price Ranges — Tick Architecture
Liquidity Providers have full control over where they deploy their liquidity:

- Set exact lower and upper price boundaries using tick system
- Position liquidity precisely where trading activity occurs
- Multiple positions at different ranges for advanced strategies
- Tick spacing configuration for different pool types

### 3. Creator Fee — Earn Revenue from Your Pool
Deploy a new trading pair and earn passive income from every swap:

- **Pool creators receive a percentage of LP fees** (configurable 0.01% - 10%)
- Example: Create an AQUA/USDC pool → every time users swap, you earn revenue from LP fees
- Claim accumulated fees anytime

### 4. Configurable Fees — Flexible Pool Parameters
Customize pool fees based on asset characteristics:

| Pool Type | Recommended Fee | Use Case |
|-----------|-----------------|----------|
| Stablecoin-Stablecoin | 0.01%, 0.05%, 0.1% | USDC/EURC pairs |
| Stable-Volatile | 0.30%, 0.5%, 1% | XLM/USDC pairs |

### 5. Multi-hop Routing *(Coming Soon)*
Swap between any tokens seamlessly:

- Automatic route finding: A → B → C
- Gas-optimized multi-step swaps
- Best price discovery across multiple pools

---

## 🏗️ Architecture

### Module Structure

```
belugaswap/
├── lib.rs          # Contract entry point & public functions
├── constants.rs    # Protocol constants (fees, ticks, limits)
├── error.rs        # Error definitions
├── events.rs       # On-chain event emissions
├── math.rs         # Q64.64 fixed-point arithmetic
├── position.rs     # LP position management
├── storage.rs      # Persistent storage handlers
├── swap.rs         # Swap execution engine
├── tick.rs         # Tick management & traversal
└── types.rs        # Data structures & types
```

## 📦 Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)
- [Stellar CLI](https://github.com/stellar/stellar-cli)

### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/belugaswap.git
   cd belugaswap
   ```

2. **Install Soroban toolchain**
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install --locked soroban-cli
   ```

3. **Build the contract**
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

4. **Optimize WASM (optional but recommended)**
   ```bash
   soroban contract optimize \
     --wasm target/wasm32-unknown-unknown/release/belugaswap.wasm
   ```

---

## 🚀 Deployment

### 1. Configure Network

**Testnet:**
```bash
soroban network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

**Mainnet:**
```bash
soroban network add --global mainnet \
  --rpc-url https://soroban.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

### 2. Create Identity

```bash
soroban keys generate --global deployer --network testnet
soroban keys address deployer
```

Fund your testnet account at [Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test).

### 3. Deploy Contract

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/belugaswap.wasm \
  --source deployer \
  --network testnet
```

Save the returned contract ID.

### 4. Initialize Pool

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --creator <CREATOR_ADDRESS> \
  --token_a <TOKEN_A_ADDRESS> \
  --token_b <TOKEN_B_ADDRESS> \
  --fee_bps 30 \
  --creator_fee_bps 100 \
  --sqrt_price_x64 18446744073709551616 \
  --current_tick 0 \
  --tick_spacing 60
```

**Parameters explained:**
- `fee_bps`: LP fee in basis points (30 = 0.30%)
- `creator_fee_bps`: Creator's share of LP fees (100 = 1% of LP fees)
- `sqrt_price_x64`: Initial price (18446744073709551616 = 1:1 ratio)
- `tick_spacing`: Tick granularity (60 for 0.30% fee pools)

---

## 📖 Usage Examples

### Add Liquidity

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source lp_wallet \
  --network testnet \
  -- \
  add_liquidity \
  --owner <LP_ADDRESS> \
  --lower_tick -1000 \
  --upper_tick 1000 \
  --amount0_desired 1000000000 \
  --amount1_desired 1000000000 \
  --amount0_min 990000000 \
  --amount1_min 990000000
```

### Swap Tokens

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source trader \
  --network testnet \
  -- \
  swap \
  --caller <TRADER_ADDRESS> \
  --token_in <TOKEN_A_ADDRESS> \
  --token_out <TOKEN_B_ADDRESS> \
  --amount_in 100000000 \
  --min_amount_out 99000000 \
  --sqrt_price_limit_x64 0
```

### Collect LP Fees

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source lp_wallet \
  --network testnet \
  -- \
  collect \
  --owner <LP_ADDRESS> \
  --lower_tick -1000 \
  --upper_tick 1000
```

### Claim Creator Fees

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source creator_wallet \
  --network testnet \
  -- \
  claim_creator_fees \
  --claimer <CREATOR_ADDRESS>
```

### Query Functions

```bash
# Get pool state
soroban contract invoke --id <CONTRACT_ID> --network testnet -- get_pool_state

# Get position info
soroban contract invoke --id <CONTRACT_ID> --network testnet -- \
  get_position --owner <ADDRESS> --lower -1000 --upper 1000

# Preview swap
soroban contract invoke --id <CONTRACT_ID> --network testnet -- \
  preview_swap \
  --token_in <TOKEN_A> \
  --token_out <TOKEN_B> \
  --amount_in 1000000 \
  --min_amount_out 0 \
  --sqrt_price_limit_x64 0
```

---

## 📊 Contract Functions

### Initialization
| Function | Description |
|----------|-------------|
| `initialize` | Initialize pool with tokens, fees, and initial price |

### Swap Functions
| Function | Description |
|----------|-------------|
| `swap` | Execute token swap with auto direction detection |
| `swap_advanced` | Execute swap with explicit direction |
| `preview_swap` | Simulate swap without execution |

### Liquidity Functions
| Function | Description |
|----------|-------------|
| `add_liquidity` | Add liquidity to a price range |
| `remove_liquidity` | Remove liquidity from a position |

### Fee Collection
| Function | Description |
|----------|-------------|
| `collect` | Collect LP fees from a position |
| `claim_creator_fees` | Claim accumulated creator fees |

### View Functions
| Function | Description |
|----------|-------------|
| `get_pool_state` | Get current pool state |
| `get_pool_config` | Get pool configuration |
| `get_position` | Get position with pending fees |
| `get_tick_info` | Get tick data |
| `get_creator_fees` | Get accumulated creator fees |
| `get_swap_direction` | Get swap direction for a token |

---

## 📄 License

see [LICENSE](LICENSE) for details.

---

*Built with ❤️ on Stellar/Soroban*