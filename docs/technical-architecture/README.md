# BelugaSwap Technical Architecture

## Table of Contents
1. [Introduction](#1-introduction)
2. [Development](#2-development)
   - [High Level Architecture](#21-high-level-architecture)
   - [Smart Contracts](#22-smart-contracts)
   - [Backend Services](#23-backend-services)
   - [Frontend Application](#24-frontend-application)
3. [Operation Flows](#3-operation-flows)
   - [Deploy Pool](#31-deploy-pool)
   - [Add Liquidity](#32-add-liquidity)
   - [Swap (Single Hop)](#33-swap-single-hop)
   - [Swap Multi-Hop](#34-swap-multi-hop)
   - [Claim Fees](#35-claim-fees)
   - [Remove Liquidity](#36-remove-liquidity)

---

## 1. Introduction

### High Level Overview

BelugaSwap is a **Concentrated Liquidity Market Maker (CLMM)** built on Stellar Soroban, bringing capital-efficient trading to the Stellar ecosystem with unique creator incentive mechanisms.

### AMM V2 vs AMM V3 (CLMM)

#### Traditional AMM (V2) - Constant Product

```
x * y = k

┌─────────────────────────────────────┐
│ Liquidity Distribution              │
│                                     │
│ ████████████████████████████████████│ ← Liquidity spread across
│ ████████████████████████████████████│   entire price range (0 → ∞)
│ ████████████████████████████████████│
│                                     │
│ $0 ─────────── Price ─────────── ∞ │
└─────────────────────────────────────┘

Problems:
• Capital inefficient - most liquidity unused
• High slippage for large trades
• Low fee earnings for LPs
```

#### Concentrated Liquidity AMM (V3/CLMM) - BelugaSwap

```
┌─────────────────────────────────────┐
│ Liquidity Distribution              │
│                                     │
│              ████████               │
│            ████████████             │ ← Liquidity concentrated
│          ████████████████           │   in active price range
│        ████████████████████         │
│                 ▲                   │
│ $0 ──────── Current ────────── ∞   │
│              Price                  │
└─────────────────────────────────────┘

Benefits:
• Up to 4000x capital efficiency
• Lower slippage
• Higher fee earnings for active LPs
```

### Unique Feature: Creator Pool Fee

BelugaSwap introduces **Creator Pool Fees** - an incentive mechanism for pool creators:

```
┌─────────────────────────────────────────────────────┐
│                    SWAP FEE FLOW                    │
├─────────────────────────────────────────────────────┤
│                                                     │
│   User Swaps 1000 USDC → XLM                       │
│                    │                                │
│                    ▼                                │
│            ┌──────────────┐                        │
│            │  Total Fee   │                        │
│            │   0.30%      │                        │
│            │   (3 USDC)   │                        │
│            └──────┬───────┘                        │
│                   │                                 │
│         ┌─────────┴─────────┐                      │
│         ▼                   ▼                      │
│   ┌───────────┐      ┌───────────┐                │
│   │ LP Fee    │      │ Creator   │                │
│   │ 99%       │      │ Fee 1%    │                │
│   │ (2.97)    │      │ (0.03)    │                │
│   └───────────┘      └───────────┘                │
│         │                   │                      │
│         ▼                   ▼                      │
│   All Liquidity       Pool Creator                │
│   Providers           (if locked)                  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

#### Creator Fee Rules

| Condition | Creator Fee Status |
|-----------|-------------------|
| LP locked + in-range | ✅ Active |
| LP locked + out-of-range | ⏸️ Paused (temporary) |
| LP unlocked | ❌ Revoked (permanent) |

---

## 2. Development

### 2.1 High Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           BELUGASWAP ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                         FRONTEND LAYER                              │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │ │
│  │  │    React     │  │   Stellar    │  │      UI Components       │ │ │
│  │  │  + Vite      │  │  Wallet SDK  │  │  Swap, Pool, Liquidity   │ │ │
│  │  │  + TypeScript│  │  (Freighter) │  │  Charts, Analytics       │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘ │ │
│  └─────────────────────────────┬──────────────────────────────────────┘ │
│                                │                                         │
│                                ▼                                         │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                         BACKEND LAYER                               │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │ │
│  │  │   Node.js    │  │   Indexer    │  │        D3.js             │ │ │
│  │  │   REST API   │  │  (Mercury/   │  │   Liquidity Heatmaps     │ │ │
│  │  │   WebSocket  │  │   Custom)    │  │   Price Charts           │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘ │ │
│  └─────────────────────────────┬──────────────────────────────────────┘ │
│                                │                                         │
│                                ▼                                         │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                      SMART CONTRACT LAYER                           │ │
│  │                        (Stellar Soroban)                            │ │
│  │                                                                      │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │ │
│  │  │   FACTORY    │  │     POOL     │  │         ROUTER           │ │ │
│  │  │              │  │              │  │                          │ │ │
│  │  │ • Deploy     │  │ • Swap       │  │ • Best Route Finding     │ │ │
│  │  │ • Registry   │  │ • Liquidity  │  │ • Multi-hop              │ │ │
│  │  │ • Fee Tiers  │  │ • Fees       │  │ • Split Routing          │ │ │
│  │  │ • Creator    │  │ • Positions  │  │ • Quotes                 │ │ │
│  │  │   Locks      │  │              │  │                          │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘ │ │
│  │                                                                      │ │
│  │  ┌───────────────────────────────────────────────────────────────┐ │ │
│  │  │                    SHARED PACKAGES                             │ │ │
│  │  │  ┌────────┐  ┌──────────┐  ┌────────┐  ┌────────────────┐   │ │ │
│  │  │  │  Math  │  │ Position │  │  Swap  │  │      Tick      │   │ │ │
│  │  │  └────────┘  └──────────┘  └────────┘  └────────────────┘   │ │ │
│  │  └───────────────────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

#### Frontend Layer (React + Wallet SDK)

| Component | Purpose |
|-----------|---------|
| **React + Vite** | Fast, modern UI framework with hot reload |
| **TypeScript** | Type-safe development, auto-generated contract types |
| **Stellar Wallet SDK** | Freighter wallet integration for signing transactions |
| **UI Components** | Swap interface, pool management, position tracking |

#### Backend Layer (Node.js + Indexer)

| Component | Purpose |
|-----------|---------|
| **Node.js API** | REST endpoints for aggregated data, WebSocket for real-time updates |
| **Indexer** | Index blockchain events, track swaps/positions/fees |
| **D3.js** | Generate liquidity heatmaps, price charts, analytics visualizations |

#### Smart Contract Layer (Soroban)

| Contract | Purpose |
|----------|---------|
| **Factory** | Deploy pools, manage registry, enforce fee tiers, handle creator locks |
| **Pool** | Execute swaps, manage liquidity, track positions, distribute fees |
| **Router** | Find optimal routes, execute multi-hop swaps, aggregate liquidity |

---

### 2.2 Smart Contracts
### 2.2 Smart Contracts

#### Contract Architecture

The protocol uses 3 specialized contracts for security, upgradeability, and clean separation of concerns:

```
┌─────────────────────────────────────────────────────────────────┐
│                     CONTRACT OVERVIEW                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   FACTORY                                                        │
│   └── Pool deployment, registry, creator lock management        │
│                                                                  │
│   POOL                                                           │
│   └── AMM core logic, swaps, liquidity, fee distribution        │
│                                                                  │
│   ROUTER                                                         │
│   └── Smart routing, multi-hop swaps, quote aggregation         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Contract Descriptions

**Factory** — Manages pool deployment and creator incentive system. Handles permissionless pool creation with atomic initialization (deploy + init + mint LP + lock in single tx). Validates fee tiers, prevents duplicate pools, and tracks creator locks. This contract acts as the registry and rule engine for the entire protocol.

Key Methods:
| Method | Description |
|--------|-------------|
| `initialize(admin, pool_wasm_hash)` | Initialize factory with admin and pool template |
| `set_router(router)` | Link router contract (admin only) |
| `create_pool(creator, params)` | Deploy new pool with initial liquidity |
| `unlock_creator_liquidity(pool, creator)` | Unlock LP, revokes creator fee permanently |
| `get_pool_address(token_a, token_b, fee)` | Get pool address by pair + fee tier |
| `is_creator_fee_active(pool, creator)` | Check if creator fee still active |
| `is_liquidity_locked(pool, creator, ticks)` | Check if position is locked |

**Pool** — Core AMM contract holding all liquidity. Executes swaps using concentrated liquidity math with multi-tick traversal. Manages LP positions with per-tick fee accumulation. Distributes fees between LPs (99%) and pool creator (1%). Queries Factory for creator lock status on removals.

Key Methods:
| Method | Description |
|--------|-------------|
| `initialize(factory, router, creator, ...)` | Initialize pool with config and initial price |
| `swap(sender, token_in, amount_in, min_out, limit)` | Execute token swap |
| `preview_swap(token_in, amount_in, ...)` | Simulate swap without execution |
| `add_liquidity(owner, lower, upper, amounts, mins)` | Add liquidity to position |
| `remove_liquidity(owner, lower, upper, liquidity, mins)` | Remove liquidity (checks lock) |
| `collect_fees(owner, lower, upper)` | Collect accumulated LP fees |
| `claim_creator_fees()` | Claim creator fees (creator only) |
| `get_pool_state()` | Get current price, tick, liquidity |
| `get_position(owner, lower, upper)` | Get position details |

**Router** — Execution layer for optimal swap routing. Finds best pool across fee tiers, executes multi-hop swaps (up to 4 hops), and provides split routing for large orders. All operations are atomic with slippage protection and deadline checks.

Key Methods:
| Method | Description |
|--------|-------------|
| `initialize(factory, admin)` | Initialize with factory reference |
| `swap_exact_input(sender, params)` | Swap with auto best pool selection |
| `swap_multihop(sender, params)` | Execute multi-hop swap (A→B→C) |
| `swap_split(sender, ...)` | Split order across multiple pools |
| `get_best_quote(token_in, token_out, amount)` | Get best quote across pools |
| `get_all_quotes(token_in, token_out, amount)` | Get quotes from all pools |
| `quote_multihop(token_in, amount, path)` | Simulate multi-hop output |

#### Why This Architecture

| Benefit | Description |
|---------|-------------|
| **Security** | Fund-holding contract (Pool) isolated from routing logic |
| **Upgradeability** | Can upgrade Router without touching Pool or Factory |
| **Permissionless** | Anyone can create pools, no gatekeeping |
| **Auditing** | Smaller, focused contracts easier to audit |
| **Risk Isolation** | Bug in Router doesn't affect Pool funds |

#### Shared Packages

The contracts share common libraries for complex calculations:

**Math** — Q64.64 fixed-point arithmetic for precise price calculations. Handles sqrt price conversions, tick-to-price mappings, and liquidity math without floating point errors.

Key Functions:
- `mul_div(a, b, c)` → Overflow-safe a × b ÷ c
- `get_sqrt_ratio_at_tick(tick)` → Convert tick to sqrt_price_x64
- `get_tick_at_sqrt_ratio(sqrt_price)` → Convert sqrt_price to tick
- `get_liquidity_for_amounts(...)` → Calculate liquidity from token amounts
- `get_amounts_for_liquidity(...)` → Calculate amounts from liquidity

**Position** — LP position tracking with fee accumulation. Stores liquidity amount and fee growth checkpoints per position. Calculates pending fees on collection.

Key Functions:
- `update_position(pos, liquidity_delta, fee_growth_0, fee_growth_1)`
- `calculate_pending_fees(pos, fee_growth_inside_0, fee_growth_inside_1)`

**Swap** — Swap execution engine with multi-tick traversal. Computes output amounts within tick ranges and handles tick boundary crossings for large swaps.

Key Functions:
- `compute_swap_step(sqrt_price, target, liquidity, amount_remaining)`
- `execute_swap(state, amount_in, zero_for_one, sqrt_price_limit)`

**Tick** — Tick state management for concentrated liquidity. Tracks liquidity changes at each tick boundary and fee growth outside the tick range.

Key Functions:
- `update_tick(tick, liquidity_delta, fee_growth_global, upper)`
- `cross_tick(tick, fee_growth_global_0, fee_growth_global_1)`
- `get_fee_growth_inside(lower, upper, current_tick, fee_growth_global)`

#### Data Storage Overview

**Factory Storage:**
- Instance: admin, router, pool_wasm_hash, fee_tiers
- Persistent: pools registry (token pair → address), creator_locks (pool + creator → lock info)

**Pool Storage:**
- Instance: config (factory, router, creator, tokens, fees), state (sqrt_price, tick, liquidity, fee_growth_global)
- Persistent: positions (owner + ticks → position), ticks (tick_index → tick_info), creator_fees

**Router Storage:**
- Instance: factory address, admin

#### Fee Configuration

| Fee Tier | Bps | Tick Spacing | Typical Use |
|----------|-----|--------------|-------------|
| 0.05% | 5 | 10 | Stablecoin pairs (USDC/USDT) |
| 0.30% | 30 | 60 | Standard volatile pairs (XLM/USDC) |
| 1.00% | 100 | 200 | Exotic/Meme tokens |

#### Creator Fee Mechanism

```
Swap Fee Distribution:
├── 99% → LP Fee (distributed to all LPs proportional to liquidity)
└── 1%  → Creator Fee (only if creator LP still locked)

Creator Fee Rules:
├── ✅ Active: LP locked + position in range
├── ⏸️ Paused: LP locked + position out of range (temporary)
└── ❌ Revoked: LP unlocked (permanent, cannot be restored)
```

---


### 2.3 Backend Services

```
┌─────────────────────────────────────────────────────────────────┐
│                      BACKEND SERVICES                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                       INDEXER                               │ │
│  ├────────────────────────────────────────────────────────────┤ │
│  │                                                             │ │
│  │  Purpose: Index blockchain events for fast queries          │ │
│  │                                                             │ │
│  │  Data Indexed:                                              │ │
│  │  ├── Swap Events                                            │ │
│  │  │   ├── pool, sender, token_in, token_out                 │ │
│  │  │   ├── amount_in, amount_out, fee                        │ │
│  │  │   └── timestamp, tx_hash                                 │ │
│  │  │                                                          │ │
│  │  ├── Liquidity Events                                       │ │
│  │  │   ├── pool, owner, lower_tick, upper_tick               │ │
│  │  │   ├── liquidity, amount0, amount1                       │ │
│  │  │   └── event_type (add/remove)                           │ │
│  │  │                                                          │ │
│  │  ├── Pool Events                                            │ │
│  │  │   ├── pool_address, creator, tokens, fee_tier           │ │
│  │  │   └── creation_timestamp                                 │ │
│  │  │                                                          │ │
│  │  └── Position Snapshots                                     │ │
│  │      ├── owner, pool, ticks, liquidity                     │ │
│  │      └── fees_earned, current_value                        │ │
│  │                                                             │ │
│  │  Options:                                                   │ │
│  │  • Mercury (Stellar native indexer)                        │ │
│  │  • Custom indexer (Node.js + PostgreSQL)                   │ │
│  │  • SubQuery / The Graph (if supported)                     │ │
│  │                                                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                      REST API                               │ │
│  ├────────────────────────────────────────────────────────────┤ │
│  │                                                             │ │
│  │  Framework: Node.js + Express/Fastify                       │ │
│  │                                                             │ │
│  │  Endpoints:                                                 │ │
│  │                                                             │ │
│  │  GET /pools                                                 │ │
│  │  └── List all pools with TVL, volume, APR                  │ │
│  │                                                             │ │
│  │  GET /pools/:address                                        │ │
│  │  └── Pool details, current price, liquidity distribution   │ │
│  │                                                             │ │
│  │  GET /pools/:address/ticks                                  │ │
│  │  └── Tick data for liquidity visualization                 │ │
│  │                                                             │ │
│  │  GET /tokens                                                │ │
│  │  └── Token list with prices, volumes                       │ │
│  │                                                             │ │
│  │  GET /positions/:owner                                      │ │
│  │  └── User's LP positions across all pools                  │ │
│  │                                                             │ │
│  │  GET /stats                                                 │ │
│  │  └── Protocol TVL, volume, fees, transactions              │ │
│  │                                                             │ │
│  │  GET /history/swaps                                         │ │
│  │  └── Recent swap history with pagination                   │ │
│  │                                                             │ │
│  │  WebSocket /ws                                              │ │
│  │  └── Real-time price updates, new swaps, TVL changes       │ │
│  │                                                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                       D3.js                                 │ │
│  ├────────────────────────────────────────────────────────────┤ │
│  │                                                             │ │
│  │  Purpose: Generate complex visualizations server-side       │ │
│  │                                                             │ │
│  │  Visualizations:                                            │ │
│  │                                                             │ │
│  │  1. Liquidity Heatmap                                       │ │
│  │     ┌─────────────────────────────────────┐                │ │
│  │     │ ░░░░░░░████████████░░░░░░░░░░░░░░░│                │ │
│  │     │ ░░░░████████████████████░░░░░░░░░░│                │ │
│  │     │ ██████████████████████████████░░░░│                │ │
│  │     │ Price Range ──────────────────────│                │ │
│  │     └─────────────────────────────────────┘                │ │
│  │     Shows liquidity concentration across price range       │ │
│  │                                                             │ │
│  │  2. Price Charts (OHLCV)                                    │ │
│  │     • Candlestick charts                                   │ │
│  │     • Volume bars                                          │ │
│  │     • Moving averages                                      │ │
│  │                                                             │ │
│  │  3. TVL/Volume Over Time                                    │ │
│  │     • Area charts                                          │ │
│  │     • Bar charts                                           │ │
│  │                                                             │ │
│  │  4. Fee Tier Distribution                                   │ │
│  │     • Pie/donut charts                                     │ │
│  │     • Volume by fee tier                                   │ │
│  │                                                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                      Node.js                                │ │
│  ├────────────────────────────────────────────────────────────┤ │
│  │                                                             │ │
│  │  Core Services:                                             │ │
│  │                                                             │ │
│  │  1. Event Listener                                          │ │
│  │     • Subscribe to Soroban contract events                 │ │
│  │     • Parse and store in database                          │ │
│  │     • Trigger WebSocket broadcasts                         │ │
│  │                                                             │ │
│  │  2. Price Oracle                                            │ │
│  │     • Aggregate prices from pools                          │ │
│  │     • Calculate TWAP (Time-Weighted Average Price)         │ │
│  │     • Provide price feeds for frontend                     │ │
│  │                                                             │ │
│  │  3. Analytics Engine                                        │ │
│  │     • Calculate APR/APY for pools                          │ │
│  │     • Track impermanent loss                               │ │
│  │     • Generate leaderboards                                │ │
│  │                                                             │ │
│  │  4. Cache Layer                                             │ │
│  │     • Redis for hot data                                   │ │
│  │     • Reduce RPC calls                                     │ │
│  │     • Sub-second response times                            │ │
│  │                                                             │ │
│  │  Tech Stack:                                                │ │
│  │  • Runtime: Node.js 20+                                    │ │
│  │  • Framework: Express / Fastify                            │ │
│  │  • Database: PostgreSQL + TimescaleDB                      │ │
│  │  • Cache: Redis                                            │ │
│  │  • Queue: Bull (for background jobs)                       │ │
│  │                                                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### 2.4 Frontend Application

```
┌─────────────────────────────────────────────────────────────────┐
│                     FRONTEND APPLICATION                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Tech Stack:                                                     │
│  • React 18+ with Vite                                          │
│  • TypeScript                                                    │
│  • TailwindCSS / Stellar Design System                          │ 
│  • Stellar Wallet SDK (Freighter)                               │
│  • Auto-generated contract clients                              │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Pages & Components:                                             │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                      SWAP PAGE                           │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │  ┌─────────────────────────────────────┐                │   │
│  │  │  From: [USDC      ▼] [1000     ]   │                │   │
│  │  │        Balance: 5,000 USDC         │                │   │
│  │  │                  ⇅                  │                │   │
│  │  │  To:   [XLM       ▼] [9,850    ]   │                │   │
│  │  │        Balance: 100 XLM            │                │   │
│  │  │                                     │                │   │
│  │  │  Rate: 1 USDC = 9.85 XLM           │                │   │
│  │  │  Price Impact: 0.12%               │                │   │
│  │  │  Fee: 0.30% (3 USDC)               │                │   │
│  │  │  Route: USDC → XLM (direct)        │                │   │
│  │  │                                     │                │   │
│  │  │  [        SWAP         ]           │                │   │
│  │  └─────────────────────────────────────┘                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                      POOL PAGE                           │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │                                                          │   │
│  │  Pool List:                                              │   │
│  │  ┌──────────────────────────────────────────────────┐  │   │
│  │  │ Pair       │ Fee  │ TVL      │ Volume  │ APR    │  │   │
│  │  ├──────────────────────────────────────────────────┤  │   │
│  │  │ XLM/USDC   │ 0.3% │ $1.2M    │ $500K   │ 45%    │  │   │
│  │  │ USDC/USDB  │ 0.05%│ $800K    │ $1.2M   │ 12%    │  │   │
│  │  │ XLM/EURC   │ 0.3% │ $300K    │ $50K    │ 25%    │  │   │
│  │  └──────────────────────────────────────────────────┘  │   │
│  │                                                          │   │
│  │  [+ Create New Pool]                                    │   │
│  │                                                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   LIQUIDITY PAGE                         │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │                                                          │   │
│  │  Add Liquidity to XLM/USDC (0.3%)                       │   │
│  │                                                          │   │
│  │  Price Range:                                            │   │
│  │  ┌─────────────────────────────────────┐                │   │
│  │  │ Min: [0.08    ] Max: [0.12    ]     │                │   │
│  │  │                                      │                │   │
│  │  │     ████████████                    │  ← Current     │   │
│  │  │   ██████████████████                │    liquidity   │   │
│  │  │ ████████████████████████            │                │   │
│  │  │ ──────────▲─────────────            │                │   │
│  │  │      Current Price                  │                │   │
│  │  └─────────────────────────────────────┘                │   │
│  │                                                          │   │
│  │  Deposit Amounts:                                        │   │
│  │  XLM:  [1000     ]  USDC: [100      ]                   │   │
│  │                                                          │   │
│  │  [      Add Liquidity      ]                            │   │
│  │                                                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  POSITIONS PAGE                          │   │
│  ├─────────────────────────────────────────────────────────┤   │
│  │                                                          │   │
│  │  Your Positions:                                         │   │
│  │  ┌──────────────────────────────────────────────────┐  │   │
│  │  │ XLM/USDC 0.3%                                    │  │   │
│  │  │ Range: $0.08 - $0.12  [IN RANGE ✓]              │  │   │
│  │  │ Liquidity: $5,000                                │  │   │
│  │  │ Unclaimed Fees: $45.20                           │  │   │
│  │  │ [Collect Fees] [Remove] [Increase]              │  │   │
│  │  └──────────────────────────────────────────────────┘  │   │
│  │                                                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Operation Flows

### 3.1 Deploy Pool

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          DEPLOY POOL FLOW                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   CREATOR                                                                │
│      │                                                                   │
│      │ 1. Call create_pool(params)                                      │
│      │    params: {                                                      │
│      │      token_a, token_b,                                           │
│      │      fee_bps: 30,                                                │
│      │      creator_fee_bps: 100,                                       │
│      │      initial_sqrt_price_x64,                                     │
│      │      amount0_desired, amount1_desired,                           │
│      │      lower_tick, upper_tick,                                     │
│      │      lock_duration: 0 (permanent)                                │
│      │    }                                                              │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                         FACTORY                                  │   │
│   │                                                                  │   │
│   │  2. Validate Parameters                                          │   │
│   │     ├── Check fee_bps is valid tier (5, 30, 100)                │   │
│   │     ├── Check tick alignment with spacing                        │   │
│   │     ├── Check pool doesn't exist (token_a/token_b/fee)          │   │
│   │     ├── Check creator_fee_bps in range (10-1000)                │   │
│   │     └── Check initial liquidity >= minimum                       │   │
│   │                                                                  │   │
│   │  3. Deploy Pool Contract                                         │   │
│   │     └── deployer.deploy(pool_wasm_hash, salt)                   │   │
│   │                                                                  │   │
│   │  4. Initialize Pool                                              │   │
│   │     └── pool.initialize(                                         │   │
│   │           factory, router, creator,                              │   │
│   │           token0, token1, fee_bps,                               │   │
│   │           creator_fee_bps, tick_spacing,                         │   │
│   │           initial_sqrt_price_x64                                 │   │
│   │         )                                                        │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                │                                         │
│                                ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                           POOL                                   │   │
│   │                                                                  │   │
│   │  5. Store Config & Initial State                                 │   │
│   │     ├── config: factory, router, creator, tokens, fees          │   │
│   │     └── state: sqrt_price, tick, liquidity=0                    │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                │                                         │
│                                ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                    BACK TO FACTORY                               │   │
│   │                                                                  │   │
│   │  6. Transfer Initial Tokens                                      │   │
│   │     ├── token0.transfer(creator → pool, amount0)                │   │
│   │     └── token1.transfer(creator → pool, amount1)                │   │
│   │                                                                  │   │
│   │  7. Mint Initial LP Position                                     │   │
│   │     └── pool.mint(creator, lower_tick, upper_tick, amounts)     │   │
│   │                                                                  │   │
│   │  8. Create Creator Lock                                          │   │
│   │     └── store CreatorLock {                                      │   │
│   │           pool, creator, liquidity,                              │   │
│   │           lower_tick, upper_tick,                                │   │
│   │           lock_end: permanent or timestamp,                      │   │
│   │           fee_revoked: false                                     │   │
│   │         }                                                        │   │
│   │                                                                  │   │
│   │  9. Register Pool                                                │   │
│   │     ├── pools[(token0, token1, fee)] = pool_address             │   │
│   │     └── total_pools += 1                                        │   │
│   │                                                                  │   │
│   │  10. Emit Event: PoolCreated                                     │   │
│   │                                                                  │   │
│   │  11. Return pool_address                                         │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   RESULT:                                                                │
│   ├── New pool deployed and initialized                                 │
│   ├── Creator has locked LP position                                    │
│   ├── Creator eligible for creator fees                                 │
│   └── Pool registered in factory                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### 3.2 Add Liquidity

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        ADD LIQUIDITY FLOW                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   LP PROVIDER                                                            │
│      │                                                                   │
│      │ 1. Call pool.add_liquidity(                                      │
│      │      owner,                                                       │
│      │      lower_tick: -887220,  // Full range example                 │
│      │      upper_tick: 887220,                                         │
│      │      amount0_desired: 1000 USDC,                                 │
│      │      amount1_desired: 10000 XLM,                                 │
│      │      amount0_min: 990 USDC,   // 1% slippage                     │
│      │      amount1_min: 9900 XLM                                       │
│      │    )                                                              │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                           POOL                                   │   │
│   │                                                                  │   │
│   │  2. Validate Parameters                                          │   │
│   │     ├── Align ticks to tick_spacing                             │   │
│   │     ├── Check lower_tick < upper_tick                           │   │
│   │     └── Check amounts > 0                                        │   │
│   │                                                                  │   │
│   │  3. Calculate Liquidity                                          │   │
│   │     │                                                            │   │
│   │     │  Current State:                                            │   │
│   │     │  ├── sqrt_price_x64 = current price                       │   │
│   │     │  ├── sqrt_price_lower = price at lower_tick               │   │
│   │     │  └── sqrt_price_upper = price at upper_tick               │   │
│   │     │                                                            │   │
│   │     │  Three scenarios:                                          │   │
│   │     │                                                            │   │
│   │     │  A) Current price BELOW range (price < lower)             │   │
│   │     │     └── Only token0 needed                                │   │
│   │     │         liquidity = amount0 / (1/sqrt_lower - 1/sqrt_upper)│   │
│   │     │                                                            │   │
│   │     │  B) Current price IN range (lower ≤ price ≤ upper)        │   │
│   │     │     └── Both tokens needed                                │   │
│   │     │         liquidity = min(L0, L1) where:                    │   │
│   │     │         L0 = amount0 / (1/sqrt_current - 1/sqrt_upper)    │   │
│   │     │         L1 = amount1 / (sqrt_current - sqrt_lower)        │   │
│   │     │                                                            │   │
│   │     │  C) Current price ABOVE range (price > upper)             │   │
│   │     │     └── Only token1 needed                                │   │
│   │     │         liquidity = amount1 / (sqrt_upper - sqrt_lower)   │   │
│   │     │                                                            │   │
│   │  4. Calculate Actual Amounts                                     │   │
│   │     └── Based on liquidity, get exact amount0 & amount1         │   │
│   │                                                                  │   │
│   │  5. Slippage Check                                               │   │
│   │     ├── amount0 >= amount0_min ? ✓                              │   │
│   │     └── amount1 >= amount1_min ? ✓                              │   │
│   │                                                                  │   │
│   │  6. Update Ticks                                                 │   │
│   │     ├── lower_tick: liquidity_net += liquidity                  │   │
│   │     └── upper_tick: liquidity_net -= liquidity                  │   │
│   │                                                                  │   │
│   │  7. Update Position                                              │   │
│   │     └── positions[(owner, lower, upper)] = Position {           │   │
│   │           liquidity: existing + new,                             │   │
│   │           fee_growth_inside_last_0,                              │   │
│   │           fee_growth_inside_last_1,                              │   │
│   │           tokens_owed_0, tokens_owed_1                           │   │
│   │         }                                                        │   │
│   │                                                                  │   │
│   │  8. Update Pool State (if in range)                              │   │
│   │     └── state.liquidity += liquidity                            │   │
│   │                                                                  │   │
│   │  9. Transfer Tokens                                              │   │
│   │     ├── token0.transfer(owner → pool, amount0)                  │   │
│   │     └── token1.transfer(owner → pool, amount1)                  │   │
│   │                                                                  │   │
│   │  10. Emit Event: LiquidityAdded                                  │   │
│   │                                                                  │   │
│   │  11. Return (liquidity, amount0, amount1)                        │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   RESULT:                                                                │
│   ├── LP has new/increased position                                     │
│   ├── Tokens locked in pool                                             │
│   ├── Will earn fees when price in range                                │
│   └── Position tracked: (owner, lower_tick, upper_tick)                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Liquidity Visualization:**

```
                    Current Price
                         │
    ┌────────────────────┼────────────────────┐
    │                    ▼                    │
    │    ░░░░░░░░░░░████████████░░░░░░░░░░░  │
    │    ░░░░░░░████████████████████░░░░░░░  │
    │    ░░░░████████████████████████████░░  │
    │    ████████████████████████████████░░  │
    │                                         │
    │    ◄─────── Your Position ───────►     │
    │    lower_tick            upper_tick     │
    └─────────────────────────────────────────┘
    
    In Range = Earning Fees ✓
    Out of Range = Not Earning (but no IL either)
```

---

### 3.3 Swap (Single Hop)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           SWAP FLOW                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   USER                                                                   │
│      │                                                                   │
│      │ 1. Call router.swap_exact_input(                                 │
│      │      sender,                                                      │
│      │      params: {                                                    │
│      │        token_in: USDC,                                           │
│      │        token_out: XLM,                                           │
│      │        amount_in: 1000,                                          │
│      │        amount_out_min: 9800,  // slippage protection             │
│      │        fee_tiers: [],         // empty = try all                 │
│      │        recipient,                                                 │
│      │        deadline                                                   │
│      │      }                                                            │
│      │    )                                                              │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                          ROUTER                                  │   │
│   │                                                                  │   │
│   │  2. Check Deadline                                               │   │
│   │     └── current_ledger <= deadline ? ✓                          │   │
│   │                                                                  │   │
│   │  3. Find Best Pool                                               │   │
│   │     │                                                            │   │
│   │     │  Query Factory for pools:                                  │   │
│   │     │  ├── USDC/XLM @ 0.05% → quote: 9,750 XLM                  │   │
│   │     │  ├── USDC/XLM @ 0.30% → quote: 9,850 XLM  ← BEST         │   │
│   │     │  └── USDC/XLM @ 1.00% → quote: 9,700 XLM                  │   │
│   │     │                                                            │   │
│   │     └── Select: 0.30% pool                                       │   │
│   │                                                                  │   │
│   │  4. Transfer token_in from User                                  │   │
│   │     └── USDC.transfer(user → router, 1000)                      │   │
│   │                                                                  │   │
│   │  5. Approve Pool to spend                                        │   │
│   │     └── USDC.approve(router → pool, 1000)                       │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                │                                         │
│                                ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                           POOL                                   │   │
│   │                                                                  │   │
│   │  6. Execute Swap                                                 │   │
│   │     │                                                            │   │
│   │     │  Determine Direction:                                      │   │
│   │     │  └── USDC = token0, XLM = token1                          │   │
│   │     │  └── Direction: 0 → 1 (zero_for_one = true)               │   │
│   │     │                                                            │   │
│   │     │  Calculate Fee:                                            │   │
│   │     │  ├── Total Fee: 1000 × 0.30% = 3 USDC                     │   │
│   │     │  ├── LP Fee (99%): 2.97 USDC                              │   │
│   │     │  └── Creator Fee (1%): 0.03 USDC                          │   │
│   │     │                                                            │   │
│   │     │  Swap Engine Loop:                                         │   │
│   │     │  ┌─────────────────────────────────────────────────────┐  │   │
│   │     │  │ remaining = 997 USDC (after fee)                    │  │   │
│   │     │  │                                                      │  │   │
│   │     │  │ STEP 1: Current tick = 1000                         │  │   │
│   │     │  │ ├── Liquidity at tick: 50,000                       │  │   │
│   │     │  │ ├── Can swap: 500 USDC → 4,925 XLM                  │  │   │
│   │     │  │ ├── remaining = 497 USDC                            │  │   │
│   │     │  │ └── Cross tick boundary → tick = 999                │  │   │
│   │     │  │                                                      │  │   │
│   │     │  │ STEP 2: Current tick = 999                          │  │   │
│   │     │  │ ├── Liquidity at tick: 45,000                       │  │   │
│   │     │  │ ├── Can swap: 497 USDC → 4,925 XLM                  │  │   │
│   │     │  │ ├── remaining = 0                                   │  │   │
│   │     │  │ └── Done!                                           │  │   │
│   │     │  │                                                      │  │   │
│   │     │  │ Total Output: 9,850 XLM                             │  │   │
│   │     │  └─────────────────────────────────────────────────────┘  │   │
│   │     │                                                            │   │
│   │  7. Update State                                                 │   │
│   │     ├── sqrt_price_x64 = new_price                              │   │
│   │     ├── current_tick = 999                                      │   │
│   │     ├── fee_growth_global_0 += 2.97 / liquidity                │   │
│   │     └── creator_fees_0 += 0.03                                  │   │
│   │                                                                  │   │
│   │  8. Transfer Tokens                                              │   │
│   │     ├── USDC.transfer(router → pool, 1000)                      │   │
│   │     └── XLM.transfer(pool → router, 9850)                       │   │
│   │                                                                  │   │
│   │  9. Emit Event: Swap                                             │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                │                                         │
│                                ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                     BACK TO ROUTER                               │   │
│   │                                                                  │   │
│   │  10. Slippage Check                                              │   │
│   │      └── 9,850 >= 9,800 (amount_out_min) ? ✓                    │   │
│   │                                                                  │   │
│   │  11. Transfer to Recipient                                       │   │
│   │      └── XLM.transfer(router → recipient, 9850)                 │   │
│   │                                                                  │   │
│   │  12. Return SwapResult {                                         │   │
│   │        amount_in: 1000,                                          │   │
│   │        amount_out: 9850,                                         │   │
│   │        pools_used: [pool_address],                               │   │
│   │        fee_tiers_used: [30]                                      │   │
│   │      }                                                           │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   RESULT:                                                                │
│   ├── User received 9,850 XLM                                           │
│   ├── LPs earned 2.97 USDC in fees                                      │
│   ├── Creator earned 0.03 USDC in fees                                  │
│   └── Pool price updated                                                │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### 3.4 Swap Multi-Hop

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        MULTI-HOP SWAP FLOW                               │
│                        XLM → USDC → USDB                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   USER                                                                   │
│      │                                                                   │
│      │ 1. Call router.swap_multihop(                                    │
│      │      sender,                                                      │
│      │      params: {                                                    │
│      │        token_in: XLM,                                            │
│      │        amount_in: 10000,                                         │
│      │        path: [                                                    │
│      │          { token: USDC, fee_bps: 30 },   // Hop 1: XLM→USDC     │
│      │          { token: USDB, fee_bps: 5 }     // Hop 2: USDC→USDB    │
│      │        ],                                                         │
│      │        amount_out_min: 990,                                      │
│      │        recipient,                                                 │
│      │        deadline                                                   │
│      │      }                                                            │
│      │    )                                                              │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                          ROUTER                                  │   │
│   │                                                                  │   │
│   │  2. Validate Path                                                │   │
│   │     ├── path.length <= 4 (max hops) ? ✓                         │   │
│   │     └── All pools exist ? ✓                                      │   │
│   │                                                                  │   │
│   │  3. Transfer Initial Token                                       │   │
│   │     └── XLM.transfer(user → router, 10000)                      │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                │                                         │
│   ═══════════════════════════════════════════════════════════════════   │
│                            HOP 1: XLM → USDC                             │
│   ═══════════════════════════════════════════════════════════════════   │
│                                │                                         │
│                                ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                      POOL 1 (XLM/USDC 0.3%)                      │   │
│   │                                                                  │   │
│   │  4. Execute Swap                                                 │   │
│   │     ├── Input: 10,000 XLM                                       │   │
│   │     ├── Fee: 30 XLM (0.30%)                                     │   │
│   │     ├── Output: 1,000 USDC                                      │   │
│   │     └── Transfer: XLM in, USDC out to Router                    │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                │                                         │
│                                │  Router holds: 1,000 USDC              │
│                                │                                         │
│   ═══════════════════════════════════════════════════════════════════   │
│                           HOP 2: USDC → USDB                             │
│   ═══════════════════════════════════════════════════════════════════   │
│                                │                                         │
│                                ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                     POOL 2 (USDC/USDB 0.05%)                     │   │
│   │                                                                  │   │
│   │  5. Execute Swap                                                 │   │
│   │     ├── Input: 1,000 USDC (from Hop 1)                          │   │
│   │     ├── Fee: 0.5 USDC (0.05%)                                   │   │
│   │     ├── Output: 999.50 USDB                                     │   │
│   │     └── Transfer: USDC in, USDB out to Router                   │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                │                                         │
│                                ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                     BACK TO ROUTER                               │   │
│   │                                                                  │   │
│   │  6. Slippage Check                                               │   │
│   │     └── 999.50 >= 990 (amount_out_min) ? ✓                      │   │
│   │                                                                  │   │
│   │  7. Transfer Final Output                                        │   │
│   │     └── USDB.transfer(router → recipient, 999.50)               │   │
│   │                                                                  │   │
│   │  8. Return SwapResult {                                          │   │
│   │       amount_in: 10000,                                          │   │
│   │       amount_out: 999.50,                                        │   │
│   │       pools_used: [pool1, pool2],                                │   │
│   │       fee_tiers_used: [30, 5]                                    │   │
│   │     }                                                            │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   FLOW SUMMARY:                                                          │
│   ┌─────────┐      ┌─────────────┐      ┌─────────┐      ┌─────────┐   │
│   │   XLM   │─────►│   Pool 1    │─────►│  USDC   │─────►│ Pool 2  │   │
│   │ 10,000  │      │   0.30%     │      │  1,000  │      │  0.05%  │   │
│   └─────────┘      └─────────────┘      └─────────┘      └────┬────┘   │
│                                                               │        │
│                                                               ▼        │
│                                                          ┌─────────┐   │
│                                                          │  USDB   │   │
│                                                          │ 999.50  │   │
│                                                          └─────────┘   │
│                                                                          │
│   TOTAL FEES: 0.30% + 0.05% = 0.35%                                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### 3.5 Claim Fees

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          CLAIM FEES FLOW                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   There are TWO types of fees to claim:                                  │
│   1. LP Fees (for liquidity providers)                                  │
│   2. Creator Fees (for pool creator)                                    │
│                                                                          │
│   ═══════════════════════════════════════════════════════════════════   │
│                         LP COLLECT FEES                                  │
│   ═══════════════════════════════════════════════════════════════════   │
│                                                                          │
│   LP PROVIDER                                                            │
│      │                                                                   │
│      │ 1. Call pool.collect_fees(                                       │
│      │      owner,                                                       │
│      │      lower_tick,                                                  │
│      │      upper_tick                                                   │
│      │    )                                                              │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                           POOL                                   │   │
│   │                                                                  │   │
│   │  2. Get Position                                                 │   │
│   │     └── pos = positions[(owner, lower, upper)]                  │   │
│   │                                                                  │   │
│   │  3. Calculate Fee Growth Inside Range                            │   │
│   │     │                                                            │   │
│   │     │  fee_growth_inside_0 = calculate based on:                │   │
│   │     │  ├── fee_growth_global_0                                  │   │
│   │     │  ├── tick_lower.fee_growth_outside_0                      │   │
│   │     │  └── tick_upper.fee_growth_outside_0                      │   │
│   │     │                                                            │   │
│   │     │  Same for fee_growth_inside_1                             │   │
│   │                                                                  │   │
│   │  4. Calculate Pending Fees                                       │   │
│   │     │                                                            │   │
│   │     │  pending_0 = (fee_growth_inside_0 - pos.fee_growth_last_0)│   │
│   │     │              × pos.liquidity                               │   │
│   │     │                                                            │   │
│   │     │  pending_1 = (fee_growth_inside_1 - pos.fee_growth_last_1)│   │
│   │     │              × pos.liquidity                               │   │
│   │     │                                                            │   │
│   │     │  total_fees_0 = pos.tokens_owed_0 + pending_0             │   │
│   │     │  total_fees_1 = pos.tokens_owed_1 + pending_1             │   │
│   │                                                                  │   │
│   │  5. Update Position                                              │   │
│   │     ├── pos.tokens_owed_0 = 0                                   │   │
│   │     ├── pos.tokens_owed_1 = 0                                   │   │
│   │     ├── pos.fee_growth_inside_last_0 = fee_growth_inside_0     │   │
│   │     └── pos.fee_growth_inside_last_1 = fee_growth_inside_1     │   │
│   │                                                                  │   │
│   │  6. Transfer Fees                                                │   │
│   │     ├── token0.transfer(pool → owner, total_fees_0)            │   │
│   │     └── token1.transfer(pool → owner, total_fees_1)            │   │
│   │                                                                  │   │
│   │  7. Emit Event: FeesCollected                                    │   │
│   │                                                                  │   │
│   │  8. Return (total_fees_0, total_fees_1)                          │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   ═══════════════════════════════════════════════════════════════════   │
│                       CREATOR CLAIM FEES                                 │
│   ═══════════════════════════════════════════════════════════════════   │
│                                                                          │
│   POOL CREATOR                                                           │
│      │                                                                   │
│      │ 1. Call pool.claim_creator_fees()                                │
│      │                                                                   │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                           POOL                                   │   │
│   │                                                                  │   │
│   │  2. Verify Caller                                                │   │
│   │     └── caller == config.creator ? ✓                            │   │
│   │                                                                  │   │
│   │  3. Check Creator Fee Status (via Factory)                       │   │
│   │     └── factory.is_creator_fee_active(pool, creator)            │   │
│   │         ├── LP still locked? ✓                                  │   │
│   │         └── Fee not revoked? ✓                                  │   │
│   │                                                                  │   │
│   │  4. Get Accumulated Creator Fees                                 │   │
│   │     ├── fees_0 = state.creator_fees_0                           │   │
│   │     └── fees_1 = state.creator_fees_1                           │   │
│   │                                                                  │   │
│   │  5. Reset Creator Fees                                           │   │
│   │     ├── state.creator_fees_0 = 0                                │   │
│   │     └── state.creator_fees_1 = 0                                │   │
│   │                                                                  │   │
│   │  6. Transfer Fees                                                │   │
│   │     ├── token0.transfer(pool → creator, fees_0)                 │   │
│   │     └── token1.transfer(pool → creator, fees_1)                 │   │
│   │                                                                  │   │
│   │  7. Emit Event: CreatorFeesClaimed                               │   │
│   │                                                                  │   │
│   │  8. Return (fees_0, fees_1)                                      │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   FEE DISTRIBUTION RECAP:                                                │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                                                                  │   │
│   │   Total Swap Fee (e.g., 0.30%)                                  │   │
│   │          │                                                       │   │
│   │          ├──── 99% ────► LP Fees (fee_growth_global)            │   │
│   │          │                    │                                  │   │
│   │          │                    └──► Distributed to all LPs       │   │
│   │          │                         proportional to liquidity    │   │
│   │          │                                                       │   │
│   │          └──── 1% ─────► Creator Fees (creator_fees)            │   │
│   │                               │                                  │   │
│   │                               └──► Only pool creator            │   │
│   │                                    (if LP locked)               │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### 3.6 Remove Liquidity

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       REMOVE LIQUIDITY FLOW                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   LP PROVIDER                                                            │
│      │                                                                   │
│      │ 1. Call pool.remove_liquidity(                                   │
│      │      owner,                                                       │
│      │      lower_tick,                                                  │
│      │      upper_tick,                                                  │
│      │      liquidity: 5000,        // amount to remove                 │
│      │      amount0_min: 450,       // slippage protection              │
│      │      amount1_min: 4500                                           │
│      │    )                                                              │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                           POOL                                   │   │
│   │                                                                  │   │
│   │  2. Verify Authorization                                         │   │
│   │     └── owner.require_auth()                                    │   │
│   │                                                                  │   │
│   │  3. Check Creator Lock (if owner is creator)                     │   │
│   │     │                                                            │   │
│   │     │  ┌───────────────────────────────────────────────────┐    │   │
│   │     │  │           FACTORY CHECK                            │    │   │
│   │     │  │                                                    │    │   │
│   │     │  │  factory.is_liquidity_locked(                     │    │   │
│   │     │  │    pool, owner, lower_tick, upper_tick            │    │   │
│   │     │  │  )                                                 │    │   │
│   │     │  │                                                    │    │   │
│   │     │  │  Returns TRUE if:                                  │    │   │
│   │     │  │  ├── Owner is pool creator                        │    │   │
│   │     │  │  ├── Position matches creator lock                │    │   │
│   │     │  │  ├── Lock not expired (or permanent)              │    │   │
│   │     │  │  └── Fee not yet revoked                          │    │   │
│   │     │  │                                                    │    │   │
│   │     │  │  If TRUE → REJECT removal ❌                       │    │   │
│   │     │  │  If FALSE → Allow removal ✓                       │    │   │
│   │     │  │                                                    │    │   │
│   │     │  └───────────────────────────────────────────────────┘    │   │
│   │     │                                                            │   │
│   │     └── Position NOT locked ? ✓ (proceed)                       │   │
│   │                                                                  │   │
│   │  4. Get Position                                                 │   │
│   │     └── pos = positions[(owner, lower, upper)]                  │   │
│   │                                                                  │   │
│   │  5. Validate Liquidity                                           │   │
│   │     └── pos.liquidity >= liquidity ? ✓                          │   │
│   │                                                                  │   │
│   │  6. Calculate Amounts to Withdraw                                │   │
│   │     │                                                            │   │
│   │     │  Based on current price vs position range:                │   │
│   │     │                                                            │   │
│   │     │  amount0 = liquidity × (1/sqrt_current - 1/sqrt_upper)    │   │
│   │     │  amount1 = liquidity × (sqrt_current - sqrt_lower)        │   │
│   │     │                                                            │   │
│   │     │  Example: liquidity = 5000                                │   │
│   │     │  └── amount0 = 500 USDC, amount1 = 5000 XLM              │   │
│   │                                                                  │   │
│   │  7. Slippage Check                                               │   │
│   │     ├── 500 >= 450 (amount0_min) ? ✓                            │   │
│   │     └── 5000 >= 4500 (amount1_min) ? ✓                          │   │
│   │                                                                  │   │
│   │  8. Update Fee Growth & Collect Pending                          │   │
│   │     ├── Calculate pending fees                                  │   │
│   │     └── Add to tokens_owed                                      │   │
│   │                                                                  │   │
│   │  9. Update Position                                              │   │
│   │     └── pos.liquidity -= 5000                                   │   │
│   │                                                                  │   │
│   │  10. Update Ticks                                                │   │
│   │      ├── lower_tick: liquidity_net -= 5000                      │   │
│   │      └── upper_tick: liquidity_net += 5000                      │   │
│   │                                                                  │   │
│   │  11. Update Pool State (if position was in range)                │   │
│   │      └── state.liquidity -= 5000                                │   │
│   │                                                                  │   │
│   │  12. Transfer Tokens to Owner                                    │   │
│   │      ├── token0.transfer(pool → owner, 500)                     │   │
│   │      └── token1.transfer(pool → owner, 5000)                    │   │
│   │                                                                  │   │
│   │  13. Emit Event: LiquidityRemoved                                │   │
│   │                                                                  │   │
│   │  14. Return (amount0, amount1)                                   │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   ═══════════════════════════════════════════════════════════════════   │
│                    CREATOR UNLOCK FLOW (Special Case)                    │
│   ═══════════════════════════════════════════════════════════════════   │
│                                                                          │
│   If CREATOR wants to unlock their locked position:                      │
│                                                                          │
│   CREATOR                                                                │
│      │                                                                   │
│      │ 1. Call factory.unlock_creator_liquidity(pool, creator)          │
│      │                                                                   │
│      ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                         FACTORY                                  │   │
│   │                                                                  │   │
│   │  2. Get Creator Lock                                             │   │
│   │     └── lock = creator_locks[(pool, creator)]                   │   │
│   │                                                                  │   │
│   │  3. Validate                                                     │   │
│   │     ├── Lock exists ? ✓                                         │   │
│   │     ├── Caller is creator ? ✓                                   │   │
│   │     ├── Fee not already revoked ? ✓                             │   │
│   │     └── Lock period expired OR permanent with consent ? ✓       │   │
│   │                                                                  │   │
│   │  ⚠️  WARNING DISPLAYED:                                          │   │
│   │  ┌─────────────────────────────────────────────────────────┐    │   │
│   │  │  UNLOCKING WILL PERMANENTLY REVOKE YOUR CREATOR FEE!    │    │   │
│   │  │  You will no longer earn the 1% creator fee from swaps. │    │   │
│   │  │  This action CANNOT be undone.                          │    │   │
│   │  └─────────────────────────────────────────────────────────┘    │   │
│   │                                                                  │   │
│   │  4. Update Lock Status                                           │   │
│   │     ├── lock.is_unlocked = true                                 │   │
│   │     └── lock.fee_revoked = true  // PERMANENT!                  │   │
│   │                                                                  │   │
│   │  5. Emit Event: CreatorLiquidityUnlocked                         │   │
│   │                                                                  │   │
│   │  6. Return liquidity amount                                      │   │
│   │                                                                  │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   Now creator can call pool.remove_liquidity() normally                 │
│                                                                          │
│   RESULT:                                                                │
│   ├── Creator's LP is now unlocked                                      │
│   ├── Creator fee is PERMANENTLY revoked                                │
│   ├── Creator can remove liquidity anytime                              │
│   └── Creator will NOT earn creator fees anymore                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---


## Deployment Addresses (Testnet)

```
Factory: CDESPZU35UBOL5WVTMPUOKSMYVLNVMJRUOQUQGLCOOU5POWTOBSTCM7N
Router:  CC363XX4IXCC57KC5LMYOXRCC6L7VWFXAGBX7C2573XP36BTYRCQGM54
```

---

*Document Version: 1.0*
*Last Updated: January 2026*