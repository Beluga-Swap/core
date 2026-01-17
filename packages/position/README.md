# BelugaSwap Position Package

Position tracking and fee accounting for concentrated liquidity positions, implementing Uniswap V3-style fee growth mechanics.

## 📋 Table of Contents

- [Overview](#-overview)
- [Core Concepts](#-core-concepts)
- [Position Structure](#-position-structure)
- [Fee Accounting](#-fee-accounting)
- [Functions Reference](#-functions-reference)
- [Usage Examples](#-usage-examples)

---

## 🌊 Overview

The Position package manages individual liquidity positions and tracks accumulated fees. Each position represents a range of liquidity provided by an LP at specific price bounds (ticks).

### Key Features

- **Position Management**: Track liquidity, bounds, and ownership
- **Fee Accumulation**: Automatic fee tracking using growth accounting
- **Wrapping Arithmetic**: Handles fee overflow gracefully
- **Efficient Updates**: O(1) fee calculations regardless of time elapsed

### Architecture

The package consists of three core modules:
- `types.rs`: Position data structures
- `manager.rs`: Position modification and validation
- `fees.rs`: Fee calculation utilities

---

## 💡 Core Concepts

### What is a Position?

A **position** represents a liquidity provider's stake in a specific price range:

```
Position = {
  owner: Address,
  lower_tick: i32,      // Lower price bound
  upper_tick: i32,      // Upper price bound
  liquidity: i128,      // Amount of liquidity
  fees_owed: (u128, u128)  // Uncollected fees
}
```

### Position Key

Positions are uniquely identified by three parameters:
```
position_key = (owner_address, lower_tick, upper_tick)
```

Multiple positions can exist for:
- Same owner, different ranges ✅
- Different owners, same range ✅
- Same owner, same range ❌ (adds to existing position)

### Active vs Inactive Positions

A position is **active** when the current price is within its range:

```
Position is ACTIVE when:
  lower_tick ≤ current_tick < upper_tick

Position is INACTIVE when:
  current_tick < lower_tick  (price below range)
  OR
  current_tick ≥ upper_tick  (price above range)
```

**Important**: Only active positions earn fees!

---

## 📊 Position Structure

### Position (Storage Type)

```rust
pub struct Position {
    pub liquidity: i128,
    pub fee_growth_inside_last_0: u128,
    pub fee_growth_inside_last_1: u128,
    pub tokens_owed_0: u128,
    pub tokens_owed_1: u128,
}
```

**Fields:**
- `liquidity`: Current liquidity amount in the position
- `fee_growth_inside_last_0`: Last checkpoint for token0 fee growth
- `fee_growth_inside_last_1`: Last checkpoint for token1 fee growth
- `tokens_owed_0`: Accumulated uncollected fees in token0
- `tokens_owed_1`: Accumulated uncollected fees in token1

---

### PositionInfo (View Type)

```rust
pub struct PositionInfo {
    pub liquidity: i128,
    pub amount0: i128,
    pub amount1: i128,
    pub fees_owed_0: u128,
    pub fees_owed_1: u128,
}
```

**Fields:**
- `liquidity`: Position's liquidity
- `amount0`: Current token0 amount in position
- `amount1`: Current token1 amount in position
- `fees_owed_0`: Total fees owed (collected + pending)
- `fees_owed_1`: Total fees owed (collected + pending)

**Used by**: `get_position()` view function in pool contract

---

## 🧮 Fee Accounting

### Fee Growth Global

The pool tracks **global fee growth** per unit of liquidity:

```
fee_growth_global_0 = total_fees_0 / total_liquidity
fee_growth_global_1 = total_fees_1 / total_liquidity
```

This is a continuously increasing number stored in Q64.64 format.

### Fee Growth Inside

For a specific tick range, we calculate **fee growth inside** that range:

```
fee_growth_inside = fees earned per unit liquidity 
                    in the range [lower_tick, upper_tick]
```

### Fee Calculation Algorithm

When updating a position:

```rust
// 1. Calculate fee delta (wrapping arithmetic handles overflow)
delta_0 = current_fee_growth_inside_0 - last_checkpoint_0
delta_1 = current_fee_growth_inside_1 - last_checkpoint_1

// 2. Calculate fees earned
fees_0 = (liquidity * delta_0) / 2^64
fees_1 = (liquidity * delta_1) / 2^64

// 3. Accumulate owed tokens
tokens_owed_0 += fees_0
tokens_owed_1 += fees_1

// 4. Update checkpoints
last_checkpoint_0 = current_fee_growth_inside_0
last_checkpoint_1 = current_fee_growth_inside_1
```

### Why Wrapping Arithmetic?

Fee growth values are `u128` and can overflow. Using **wrapping subtraction** ensures correct delta calculation:

```
Example with overflow:
  last_checkpoint = u128::MAX - 100
  current_growth  = 50
  
  Wrapping delta = 50 - (u128::MAX - 100)
                 = 50 + 100 (wraps around)
                 = 150 ✅
  
  Regular delta  = 50 - (u128::MAX - 100)
                 = overflow panic! ❌
```

---

## 📚 Functions Reference

### Position Management

#### `modify_position`

```rust
pub fn modify_position(
    pos: &mut Position,
    liquidity_delta: i128,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
)
```

Modify a position's liquidity and update fees.

**Process:**
1. Update accumulated fees based on current fee growth
2. Adjust liquidity by `liquidity_delta`

**Parameters:**
- `pos`: Mutable position reference
- `liquidity_delta`: Amount to add (positive) or remove (negative)
- `fee_growth_inside_0`: Current fee growth inside range for token0
- `fee_growth_inside_1`: Current fee growth inside range for token1

**Example:**
```rust
use belugaswap_position::modify_position;

let mut position = Position::default();

// Add liquidity
modify_position(
    &mut position,
    1_000_000,      // Add 1M liquidity
    fee_growth_0,   // Current fee growth
    fee_growth_1,
);

// Later, remove half
modify_position(
    &mut position,
    -500_000,       // Remove 500k liquidity
    new_fee_growth_0,
    new_fee_growth_1,
);
```

---

#### `update_position`

```rust
pub fn update_position(
    pos: &mut Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
)
```

Update position's fee checkpoints without changing liquidity.

**Use case**: Collect fees without modifying liquidity

**Process:**
1. Calculate fee delta using wrapping arithmetic
2. Accumulate fees to `tokens_owed`
3. Update checkpoints to current values

**Example:**
```rust
use belugaswap_position::update_position;

let mut position = Position { 
    liquidity: 1_000_000,
    fee_growth_inside_last_0: 1000,
    fee_growth_inside_last_1: 2000,
    tokens_owed_0: 0,
    tokens_owed_1: 0,
};

// Update fees (current fee growth increased)
update_position(&mut position, 1500, 2500);

// Now tokens_owed contains accumulated fees
println!("Fees owed: {} / {}", 
    position.tokens_owed_0, 
    position.tokens_owed_1
);
```

---

### Fee Calculation

#### `calculate_pending_fees`

```rust
pub fn calculate_pending_fees(
    pos: &Position,
    fee_growth_inside_0: u128,
    fee_growth_inside_1: u128,
) -> (u128, u128)
```

Calculate pending fees WITHOUT modifying the position.

**Returns:** `(pending_fees_0, pending_fees_1)`

**Use case**: Preview fees before collection

**Example:**
```rust
use belugaswap_position::calculate_pending_fees;

let position = Position { 
    liquidity: 1_000_000,
    fee_growth_inside_last_0: 1000,
    fee_growth_inside_last_1: 2000,
    ..Default::default()
};

// Check pending fees
let (pending_0, pending_1) = calculate_pending_fees(
    &position,
    1500,  // Current fee growth
    2500,
);

println!("Pending fees: {} / {}", pending_0, pending_1);
// Position remains unchanged!
```

---

### Position Helpers

#### `has_liquidity`

```rust
pub fn has_liquidity(pos: &Position) -> bool
```

Check if position has any liquidity.

**Example:**
```rust
use belugaswap_position::has_liquidity;

if has_liquidity(&position) {
    println!("Position is active");
}
```

---

#### `has_uncollected_fees`

```rust
pub fn has_uncollected_fees(pos: &Position) -> bool
```

Check if position has uncollected fees.

**Example:**
```rust
use belugaswap_position::has_uncollected_fees;

if has_uncollected_fees(&position) {
    println!("Fees available to collect!");
}
```

---

#### `is_empty`

```rust
pub fn is_empty(pos: &Position) -> bool
```

Check if position is completely empty (no liquidity, no fees).

**Returns:** `true` if position can be deleted

**Example:**
```rust
use belugaswap_position::is_empty;

if is_empty(&position) {
    // Position can be safely deleted from storage
    delete_position(&owner, lower_tick, upper_tick);
}
```

---

#### `clear_fees`

```rust
pub fn clear_fees(pos: &mut Position, amount0: u128, amount1: u128)
```

Clear collected fees from position using saturating subtraction.

**Use case**: After transferring fees to user

**Example:**
```rust
use belugaswap_position::clear_fees;

// Transfer fees to user
transfer_tokens(&user, position.tokens_owed_0, position.tokens_owed_1);

// Clear from position
clear_fees(
    &mut position, 
    position.tokens_owed_0, 
    position.tokens_owed_1
);

// Now tokens_owed = 0
```

---

### Validation

#### `validate_position_params`

```rust
pub fn validate_position_params(
    lower: i32,
    upper: i32,
    tick_spacing: i32,
) -> Result<(), &'static str>
```

Validate position parameters before creation.

**Checks:**
- `lower < upper`
- `tick_spacing > 0`
- `lower` is aligned to tick spacing
- `upper` is aligned to tick spacing

**Example:**
```rust
use belugaswap_position::validate_position_params;

let lower = -1000;
let upper = 1000;
let spacing = 60;

match validate_position_params(lower, upper, spacing) {
    Ok(()) => println!("Valid position parameters"),
    Err(e) => panic!("Invalid: {}", e),
}

// Invalid example
let result = validate_position_params(500, -500, 60);
assert!(result.is_err());  // lower >= upper
```

---

## 🔧 Usage Examples

### Example 1: Create New Position

```rust
use belugaswap_position::{Position, modify_position, validate_position_params};

// Validate parameters first
let lower_tick = -1000;
let upper_tick = 1000;
let tick_spacing = 60;

validate_position_params(lower_tick, upper_tick, tick_spacing)
    .expect("Invalid position parameters");

// Create new position
let mut position = Position::default();

// Add initial liquidity
let fee_growth_inside_0 = get_fee_growth_inside_0();
let fee_growth_inside_1 = get_fee_growth_inside_1();

modify_position(
    &mut position,
    10_000_000,  // 10M liquidity
    fee_growth_inside_0,
    fee_growth_inside_1,
);

println!("Position created with {} liquidity", position.liquidity);
```

---

### Example 2: Update Position with Fees

```rust
use belugaswap_position::{modify_position, calculate_pending_fees};

// Check pending fees first (optional)
let (pending_0, pending_1) = calculate_pending_fees(
    &position,
    new_fee_growth_0,
    new_fee_growth_1,
);
println!("Will collect {} / {} in fees", pending_0, pending_1);

// Add more liquidity (also collects fees)
modify_position(
    &mut position,
    5_000_000,   // Add 5M more
    new_fee_growth_0,
    new_fee_growth_1,
);

// Fees are now in tokens_owed
println!("Tokens owed: {} / {}", 
    position.tokens_owed_0,
    position.tokens_owed_1
);
```

---

### Example 3: Collect Fees Only

```rust
use belugaswap_position::{update_position, clear_fees};

// Update position to collect fees (without changing liquidity)
update_position(
    &mut position,
    current_fee_growth_0,
    current_fee_growth_1,
);

// Get amounts to transfer
let amount0 = position.tokens_owed_0;
let amount1 = position.tokens_owed_1;

// Transfer tokens to user
if amount0 > 0 {
    token0.transfer(&pool_address, &user, amount0 as i128);
}
if amount1 > 0 {
    token1.transfer(&pool_address, &user, amount1 as i128);
}

// Clear collected fees
clear_fees(&mut position, amount0, amount1);

println!("Collected {} token0 and {} token1", amount0, amount1);
```

---

### Example 4: Remove Liquidity

```rust
use belugaswap_position::{modify_position, has_liquidity, is_empty};

// Remove all liquidity (fees are collected automatically)
modify_position(
    &mut position,
    -position.liquidity,  // Remove all
    current_fee_growth_0,
    current_fee_growth_1,
);

// Check status
assert!(!has_liquidity(&position));

// Collect accumulated fees
let fees_0 = position.tokens_owed_0;
let fees_1 = position.tokens_owed_1;

transfer_fees(&user, fees_0, fees_1);
clear_fees(&mut position, fees_0, fees_1);

// Check if completely empty
if is_empty(&position) {
    println!("Position is empty and can be deleted");
}
```

---

### Example 5: Fee Accumulation Over Time

```rust
use belugaswap_position::{Position, update_position, calculate_pending_fees};

let mut position = Position {
    liquidity: 1_000_000,
    fee_growth_inside_last_0: 100_000,
    fee_growth_inside_last_1: 200_000,
    tokens_owed_0: 0,
    tokens_owed_1: 0,
};

// After some swaps, fee growth increased
let new_fee_growth_0 = 150_000;  // +50,000
let new_fee_growth_1 = 300_000;  // +100,000

// Calculate what fees would be
let (pending_0, pending_1) = calculate_pending_fees(
    &position,
    new_fee_growth_0,
    new_fee_growth_1,
);

println!("Pending fees: {} / {}", pending_0, pending_1);
// Formula: pending = (liquidity * delta) / 2^64
// pending_0 ≈ (1_000_000 * 50_000) / 2^64

// Actually collect
update_position(&mut position, new_fee_growth_0, new_fee_growth_1);

assert_eq!(position.tokens_owed_0, pending_0);
assert_eq!(position.tokens_owed_1, pending_1);
```

---

### Example 6: Handle Position with Multiple Updates

```rust
use belugaswap_position::{Position, modify_position, has_uncollected_fees};

let mut position = Position::default();

// Day 1: Add liquidity
modify_position(&mut position, 1_000_000, 1000, 2000);

// Day 2: Swaps happen, fees accumulate
// Add more liquidity (fees collected automatically)
modify_position(&mut position, 500_000, 1500, 2500);

// Check if fees accumulated
if has_uncollected_fees(&position) {
    println!("Fees to collect: {} / {}", 
        position.tokens_owed_0,
        position.tokens_owed_1
    );
}

// Day 3: More swaps
modify_position(&mut position, 0, 2000, 3000);  // Just update fees

// Day 4: Remove half liquidity
modify_position(&mut position, -750_000, 2200, 3200);

// All fees from all periods are now in tokens_owed
println!("Total accumulated fees: {} / {}",
    position.tokens_owed_0,
    position.tokens_owed_1
);
```

---

### Example 7: Validation Before Position Creation

```rust
use belugaswap_position::validate_position_params;

fn create_position(lower: i32, upper: i32, spacing: i32) -> Result<(), &'static str> {
    // Validate first
    validate_position_params(lower, upper, spacing)?;
    
    // If validation passes, create position
    println!("Creating position [{}, {}]", lower, upper);
    Ok(())
}

// Valid positions
assert!(create_position(-1000, 1000, 60).is_ok());
assert!(create_position(-600, 600, 60).is_ok());

// Invalid positions
assert!(create_position(1000, -1000, 60).is_err());  // upper < lower
assert!(create_position(-1000, 1000, -60).is_err()); // negative spacing
assert!(create_position(-1050, 1000, 60).is_err());  // not aligned
assert!(create_position(-1000, 1050, 60).is_err());  // not aligned
```

---

## 📊 Fee Growth Mechanics

### Understanding Fee Growth Inside

Fee growth inside a tick range depends on the current price:

```
Case 1: Price BELOW range (current_tick < lower_tick)
  fee_growth_inside = fee_growth_below[lower] - fee_growth_below[upper]

Case 2: Price INSIDE range (lower_tick ≤ current_tick < upper_tick)
  fee_growth_inside = fee_growth_global 
                      - fee_growth_below[lower] 
                      - fee_growth_above[upper]

Case 3: Price ABOVE range (current_tick ≥ upper_tick)
  fee_growth_inside = fee_growth_above[upper] - fee_growth_above[lower]
```

### When Fees Accumulate

Positions earn fees when:
1. **Position is in range**: `lower_tick ≤ current_tick < upper_tick`
2. **Swaps occur**: Trading activity generates fees
3. **Proportional to liquidity**: More liquidity = more fees

Positions DON'T earn fees when:
- Price is outside the range
- No trading activity occurs
- Position has zero liquidity

---

## 🎯 Best Practices

### 1. Always Validate Parameters

```rust
// ✅ Good
validate_position_params(lower, upper, spacing)?;
modify_position(&mut pos, liquidity, growth_0, growth_1);

// ❌ Bad
modify_position(&mut pos, liquidity, growth_0, growth_1);
// Missing validation!
```

### 2. Update Fees Before Liquidity Changes

```rust
// ✅ Good - fees are updated automatically
modify_position(&mut pos, liquidity_delta, growth_0, growth_1);

// ❌ Bad - manual update might miss fees
pos.liquidity += liquidity_delta;  // Don't do this!
```

### 3. Check Empty Positions

```rust
// ✅ Good - clean up empty positions
if is_empty(&position) {
    delete_position_from_storage(&key);
}

// ❌ Bad - waste storage on empty positions
// Just leaving it...
```

### 4. Use Saturating Operations

```rust
// ✅ Good - built into clear_fees
clear_fees(&mut pos, amount0, amount1);

// ❌ Bad - potential underflow
pos.tokens_owed_0 -= amount0;  // Could panic!
```

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Position Package**: [packages/position](https://github.com/Beluga-Swap/core/tree/main/packages/position)
- **Math Package**: [packages/math](https://github.com/Beluga-Swap/core/tree/main/packages/math)
- **Pool Contract**: [contracts/pool](https://github.com/Beluga-Swap/core/tree/main/contracts/pool)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)

---

## 📄 License

MIT License - see LICENSE file for details