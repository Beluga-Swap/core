# Manual Test Report: Core AMM Logic (Legacy Monolith)

This directory contains artifacts and documentation regarding the manual testing performed on the **Legacy Monolith** version of BelugaSwap (Refer to Git Tag: `legacy-monolith`). 

Before transitioning to the current Modular/Factory architecture, we successfully verified the core AMM primitives to ensure mathematical and functional correctness.

## Tested Features & Workflow
We have conducted end-to-end manual testing for the basic AMM flow, covering the following steps:
1. **Pool Contract Deployment:** Successful initialization on the Soroban network.
2. **Liquidity Provision (Concentrated):** Adding liquidity within specific tick ranges.
3. **Token Swapping:** Executing trades against the liquidity pool.
4. **Fee Generation:** Verifying that fees are correctly calculated and accrued during swaps.
5. **Fee Collection:** Ensuring Liquidity Providers (LPs) can successfully claim their earned fees.

## Implementation Details
The legacy version successfully implemented and validated the following logic:
* **Swap Mathematics:** Price calculations based on $x \cdot y = k$.
* **Concentrated Liquidity:** Tick-based liquidity management.
* **Price Dynamics:** Correct handling of $\Delta \sqrt{P}$ (Delta Sqrt Price).
* **Fee Accounting:** Accurate $\Delta Fee$ distribution.
* **Tick Management:** Transition logic between active and inactive ticks.

---

## Current Development: Factory & Incentives
We are currently refactoring the architecture to introduce the **Beluga Factory**. This transition moves us toward a permissionless ecosystem where:

* **Permissionless Deployment:** Any user can act as a **Pool Creator** to deploy new trading pairs.
* **Creator Incentives:** To encourage ecosystem growth, Pool Creators will receive a dedicated share of the protocol incentives from every swap occurring in their created pools.

For the latest progress on the Factory implementation, please refer to the active development branches.