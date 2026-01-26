# BelugaSwap Core

Concentrated liquidity AMM on Stellar Soroban with creator fee incentives.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Soroban](https://img.shields.io/badge/Soroban-Stellar-blue)](https://soroban.stellar.org)

## Overview

BelugaSwap brings concentrated liquidity to Stellar, allowing LPs to provide liquidity within custom price ranges for up to 4000x capital efficiency. Pool creators earn a share of trading fees to incentivize new pair deployments.

## Architecture

```
Contracts/
├── Factory    → Pool deployment & registry
├── Pool       → AMM core logic & swap execution
└── Router     → Smart routing & multi-hop swaps

Packages/
├── Math       → Q64.64 arithmetic & price calculations  
├── Position   → Position tracking & fee accumulation
├── Swap       → Multi-tick swap engine
└── Tick       → Tick management & fee growth tracking
```

## Quick Start

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
# Clone repository
git clone https://github.com/Beluga-Swap/core.git
cd core

# Build all contracts
stellar contract build
```

Output: `target/wasm32v1-none/release/*.wasm`

### Test

```bash
cargo test
```

## Deployment

### 1. Setup Environment

```bash
export NETWORK="testnet"
export SOURCE="your-key-identity"
export ADMIN=$(stellar keys address $SOURCE)
```

### 2. Deploy Contracts

```bash
# Upload Pool WASM
export POOL_WASM_HASH=$(stellar contract upload \
  --wasm target/wasm32v1-none/release/belugaswap_pool.wasm \
  --source $SOURCE --network $NETWORK)

# Deploy Factory
export FACTORY=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/beluga_factory.wasm \
  --source $SOURCE --network $NETWORK)

# Initialize Factory
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- initialize --admin $ADMIN --pool_wasm_hash $POOL_WASM_HASH

# Deploy Router
export ROUTER=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/belugaswap_router.wasm \
  --source $SOURCE --network $NETWORK)

# Initialize Router
stellar contract invoke --id $ROUTER --source $SOURCE --network $NETWORK \
  -- initialize --factory $FACTORY --admin $ADMIN

# Link Router to Factory
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- set_router --router $ROUTER

# Verify
stellar contract invoke --id $FACTORY --network $NETWORK -- is_ready
# Expected: true
```

### 3. Create Pool

```bash
stellar contract invoke --id $FACTORY --source $SOURCE --network $NETWORK \
  -- create_pool --creator $ADMIN --params '{
    "token_a": "<TOKEN_A_ADDRESS>",
    "token_b": "<TOKEN_B_ADDRESS>",
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

## Documentation

### Contracts
| Contract | Description | Docs |
|----------|-------------|------|
| **Factory** | Pool deployment, creator locks, fee tiers | [README](contracts/factory/README.md) |
| **Pool** | Swaps, liquidity management, fee collection | [README](contracts/pool/README.md) |
| **Router** | Smart routing, multi-hop, split swaps | [README](contracts/router/README.md) |

### Packages  
| Package | Description | Docs |
|---------|-------------|------|
| **Math** | Q64.64 arithmetic, price/liquidity calculations | [README](packages/math/README.md) |
| **Position** | Position tracking, fee accumulation | [README](packages/position/README.md) |
| **Swap** | Swap engine, multi-tick traversal | [README](packages/swap/README.md) |
| **Tick** | Tick updates, crossing, fee growth | [README](packages/tick/README.md) |

## Key Concepts

| Concept | Description |
|---------|-------------|
| **Q64.64 Format** | Prices stored as `sqrt(price) * 2^64` for precision |
| **Ticks** | Discrete price points where `price = 1.0001^tick` |
| **Fee Tiers** | 0.05% (stablecoins), 0.30% (volatile), 1.00% (exotic) |
| **Creator Fees** | 0.1%-10% of trading fees, requires locked LP |

## Contract Roles

| Role | Permissions |
|------|-------------|
| **Admin** | Set fee tiers, update router, transfer admin |
| **Pool Creator** | Earn creator fees (if LP locked) |
| **LP Provider** | Add/remove liquidity, collect fees |
| **Trader** | Swap via Router or direct Pool |

## Testnet Deployment

```
Factory: CDESPZU35UBOL5WVTMPUOKSMYVLNVMJRUOQUQGLCOOU5POWTOBSTCM7N
Router:  CC363XX4IXCC57KC5LMYOXRCC6L7VWFXAGBX7C2573XP36BTYRCQGM54
```

## License

Apache License 2.0 - see [LICENSE](LICENSE) file for details.

## Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)
- **Stellar Lab**: [laboratory.stellar.org](https://laboratory.stellar.org)

---

⚠️ **Disclaimer**: Experimental software. Not audited. Use at your own risk.
