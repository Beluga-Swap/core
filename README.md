# BelugaSwap Core

Concentrated liquidity AMM on Stellar Soroban with creator fee incentives.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Soroban](https://img.shields.io/badge/Soroban-Stellar-blue)](https://soroban.stellar.org)

## Overview

BelugaSwap brings concentrated liquidity to Stellar, allowing LPs to provide liquidity within custom price ranges for up to 4000x capital efficiency. Pool creators earn a share of trading fees to incentivize new pair deployments.

## Architecture

```
Contracts/
├── Factory    → Permissionless pool deployment
└── Pool       → AMM core logic & swap execution

Packages/
├── Math       → Q64.64 arithmetic & price calculations  
├── Position   → Position tracking & fee accumulation
├── Swap       → Multi-tick swap engine
└── Tick       → Tick management & fee growth tracking
```

## Quick Start

```bash
# Install dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --locked soroban-cli
rustup target add wasm32-unknown-unknown

# Clone and build
git clone https://github.com/Beluga-Swap/core.git
cd core
cargo build --release --target wasm32-unknown-unknown

# Run tests
cargo test
```

## Documentation

Each component has detailed documentation:

### Contracts
- **[Factory](contracts/factory/README.md)** - Pool deployment, creator locks, fee tiers
- **[Pool](contracts/pool/README.md)** - Swaps, liquidity management, fee collection

### Packages  
- **[Math](packages/math/README.md)** - Q64.64 arithmetic, price/liquidity calculations
- **[Position](packages/position/README.md)** - Position tracking, fee accumulation
- **[Swap](packages/swap/README.md)** - Swap engine, multi-tick traversal
- **[Tick](packages/tick/README.md)** - Tick updates, crossing, fee growth

## Deployment

See individual contract READMEs for detailed deployment instructions:
- [Factory Deployment](contracts/factory/README.md#-local-setup)
- [Pool Deployment](contracts/pool/README.md#-local-setup)

## Key Concepts

- **Q64.64 Format**: Prices stored as `sqrt(price) * 2^64` for precision
- **Ticks**: Discrete price points where `price = 1.0001^tick`
- **Fee Tiers**: 0.05% (stablecoins), 0.30% (volatile), 1.00% (exotic)
- **Creator Fees**: 0.1%-10% of trading fees, requires locked LP

## License

Apache License 2.0 - see [LICENSE](LICENSE) file for details.


## Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)
- **Stellar Lab**: [laboratory.stellar.org](https://laboratory.stellar.org)

---

⚠️ **Disclaimer**: Experimental software. Not audited. Use at your own risk.
