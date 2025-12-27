# Post-Stellar Q4 Testnet Reset Deployment

## 1. Environment & Build

Initiating contract redeployment following the Stellar Testnet Q4 reset.

```bash
# Build optimized WASM 
cargo build --release --target wasm32-unknown-unknown
```

**Status:** Finished [optimized] with 4 warnings  
*(Unused imports and assignments to be cleaned in v0.1.1).*

---

## 2. Deployment

Uploaded the WASM and deployed the contract with the alias `belugaswap`.

```bash
# Install/Upload WASM
stellar contract install \
  --network testnet \
  --source alice \
  --wasm target/wasm32-unknown-unknown/release/belugaswap.optimized.wasm
```

**WASM Hash:** `b86dafbe59d84ada8b853983d5f476bc8d18d5aac00a40dd0ce410210830175f`

```bash
# Deploy Contract
stellar contract deploy \
  --network testnet \
  --source alice \
  --wasm-hash b86dafbe59d84ada8b853983d5f476bc8d18d5aac00a40dd0ce410210830175f \
  --alias belugaswap
```

**Contract ID:** `CBEFI3PZYWOMSIUIH5HCXJHTBF3ZBIRFKNIANHC7ONUT3LE4ZMAYUQVT`

---

## 3. Initialization & Liquidity Provision

Initialized the pool with Token A (XLM) and Token B (USDC).

**Price Setup:** `sqrt_price_x64` set to `18446744073709551616` (Initial 1:1 peg) for testing simplification.

```bash
# Initialize Pool
stellar contract invoke --id belugaswap --network testnet --source alice -- \
  initialize --admin alice \
  --token_a CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
  --token_b CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA \
  --fee_bps 30 --sqrt_price_x64 18446744073709551616 --current_tick 0 --tick_spacing 60
```

```bash
# Add Initial Liquidity (1M stroops each)
stellar contract invoke --id belugaswap --network testnet --source alice -- \
  add_liquidity --owner alice --lower -60 --upper 60 \
  --liquidity 1000000 --amt_a 1000000 --amt_b 1000000
```

**Warning:** Currently, the liquidity input is dummy-based (follows user input). This presents a risk where the provided `amt_a`/`amt_b` might not mathematically align with the `L` value. A verification formula is required in the next update.

---

## 4. Swap Execution & Precision Analysis

### Test 1: Low Volume (Dust Issue)

Attempted to swap 1,000 units.

```json
{
  "amount_in": "0",
  "amount_out": "0",
  "current_tick": 0,
  "sqrt_price_x64": "18446744073000000000"
}
```

**Observation:** `amount_in` and `amount_out` resulted in 0. Small amounts are currently being rounded down to zero due to integer division (Dust Issue).

### Test 2: High Volume (Successful Swap)

Attempted to swap 1,000,000 units with `zero_for_one: true`.

```json
{
  "amount_in": "525783",
  "amount_out": "511573",
  "current_tick": -61,
  "sqrt_price_x64": "18000000000000000000"
}
```

**Result:** Success. The `current_tick` shifted to -61, confirming that selling XLM for USDC correctly lowered the XLM price.

---

## 5. Accounting & State Verification (Identified Bugs)

Post-swap position checks reveal discrepancies in internal accounting.

```bash
# Query Position for Range -600 to 600
stellar contract invoke --id belugaswap --network testnet --source alice -- \
  get_position --owner alice --lower -600 --upper 600
```

**Result:**
```json
{
  "liquidity": "10000000",
  "token_a_amount": "10000000",
  "token_b_amount": "10000000"
}
```

### Critical Bugs Identified:

1. **Critical Bug:** While the physical token balances changed on-chain (verified via StellarExpert), the `token_a_amount` and `token_b_amount` within the Position struct remain static.

2. **Fee Bug:** `tokens_owed` and `fee_growth` are not being updated/accumulated post-swap.

---

## Next Improvements

### 1. Automated Liquidity Math
Implement the $L$ formula to automatically calculate and verify the required `amt_a` and `amt_b`.

**Formula:**
$$L = \sqrt{x \cdot y}$$

Where:
- $x$ = amount of token A
- $y$ = amount of token B
- $L$ = liquidity

### 2. Dust Mitigation
Optimize precision arithmetic to ensure small-volume swaps are processed correctly instead of rounding to zero.

**Approaches:**
- Use fixed-point arithmetic with higher precision (Q128 instead of Q64)
- Implement minimum swap thresholds
- Add liquidity scaling factors

### 3. Dynamic Accounting
Fix the `get_position` logic to reflect real-time balances and fee accumulation based on the current `sqrt_price`.

**Required Changes:**
- Update position tracking to recalculate amounts based on current price
- Implement fee accrual tracking per position
- Add event emissions for fee collection


Updated.

1. Liquidity Calculation Improvement Current implementation refactors how liquidity (L) is handled. Users no longer need to calculate liquidity manually. Instead, L is now derived automatically based on the provided inputs: amount_a, amount_b, lower_tick, and upper_tick.

2. ⚠️ Critical Bug Report: Potential Pool Drain Despite the improvements, there is a critical bug in the swap execution logic. When a user performs a swap (e.g., Bob swapping 1,000,000 units), the contract fails to cap the execution correctly, resulting in an amount_in that far exceeds the user's amount_specified.

In the reproduction log below, the user specified 1,000,000, but the contract executed a transfer of 20,230,446 (approx. 20x the intended amount). This discrepancy represents a severe pool draining vulnerability.



