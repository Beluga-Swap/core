# BelugaSwap Tick Package

Tick management and fee growth tracking for concentrated liquidity, implementing Uniswap V3's tick-based price organization and fee accounting.

## 📋 Table of Contents

- [Overview](#-overview)
- [Tick System Explained](#-tick-system-explained)
- [Fee Growth Tracking](#-fee-growth-tracking)
- [Tick Lifecycle](#-tick-lifecycle)
- [Functions Reference](#-functions-reference)
- [Usage Examples](#-usage-examples)
- [Advanced Topics](#-advanced-topics)

---

## 🌊 Overview

The Tick package manages the fundamental building blocks of concentrated liquidity: **ticks**. Each tick represents a discrete price point where liquidity can be added or removed.

### Key Responsibilities

- **Tick Initialization**: Set up new ticks when first liquidity is added
- **Liquidity Tracking**: Track gross and net liquidity at each tick
- **Fee Accounting**: Calculate and store fee growth on each side of a tick
- **Tick Crossing**: Update state when price crosses a tick boundary
- **Tick Traversal**: Find next initialized tick during swaps

### Architecture

The package consists of three core modules:
- `types.rs`: TickInfo data structure
- `update.rs`: Tick modification and traversal
- `fee_growth.rs`: Fee growth calculations

---

## 📊 Tick System Explained

### What is a Tick?

A **tick** is an index that represents a price point:

```
price = 1.0001^tick

Examples:
tick -100  → price = 0.9900
tick 0     → price = 1.0000
tick 100   → price = 1.0100
tick 6932  → price ≈ 2.0000
```

### Tick Spacing

Tick spacing determines which ticks can have liquidity:

```
Spacing 10:  ticks ..., -20, -10, 0, 10, 20, ...
Spacing 60:  ticks ..., -120, -60, 0, 60, 120, ...
Spacing 200: ticks ..., -400, -200, 0, 200, 400, ...
```

### Initialized vs Uninitialized Ticks

```
Initialized: Has liquidity (liquidity_gross > 0)
Uninitialized: No liquidity (liquidity_gross = 0)

Only initialized ticks are stored and processed!
```

### Tick Boundaries

```
MIN_TICK = -887272  (price ≈ 2.94 × 10^-39)
MAX_TICK = 887272   (price ≈ 3.40 × 10^38)
```

---

## 💰 Fee Growth Tracking

### The Core Concept

Fee growth tracking answers: **"How much fees were earned in a specific price range?"**

### Fee Growth Variables

Each tick stores **fee growth outside**:

```rust
pub struct TickInfo {
    pub fee_growth_outside_0: u128,  // Fees on "other side" of tick
    pub fee_growth_outside_1: u128,  // (token0 and token1)
    // ... other fields
}
```

### What is "Outside"?

```
Current Price = tick 100

For tick 50:
  "Outside" = fees below tick 50 (ticks < 50)
  "Inside"  = fees above tick 50 (ticks ≥ 50)

For tick 150:
  "Outside" = fees above tick 150 (ticks ≥ 150)
  "Inside"  = fees below tick 150 (ticks < 150)
```

### Fee Growth Inside a Range

To calculate fees in range `[lower_tick, upper_tick]`:

```
Case 1: Price BELOW range (current_tick < lower_tick)
  fee_inside = fee_outside[lower] - fee_outside[upper]

Case 2: Price INSIDE range (lower_tick ≤ current_tick < upper_tick)
  fee_inside = fee_global - fee_outside[lower] - fee_outside[upper]

Case 3: Price ABOVE range (current_tick ≥ upper_tick)
  fee_inside = fee_outside[upper] - fee_outside[lower]
```

### Why This Works

The **invariant**:
```
fee_global = fee_below[tick] + fee_inside[position] + fee_above[tick]
```

By tracking one piece (`fee_outside`), we can calculate the others!

---

## 🔄 Tick Lifecycle

### 1. Initialization (First Liquidity)

```rust
// When first liquidity is added at tick 100:

if liquidity_gross == 0 {  // Was uninitialized
    if current_tick >= tick {
        // Price is at or above this tick
        // Assume all past fees were earned BELOW
        fee_growth_outside_0 = fee_growth_global_0;
        fee_growth_outside_1 = fee_growth_global_1;
    } else {
        // Price is below this tick
        // Assume all past fees were earned ABOVE
        fee_growth_outside_0 = 0;
        fee_growth_outside_1 = 0;
    }
    initialized = true;
}
```

### 2. Liquidity Updates

```rust
// Adding liquidity:
liquidity_gross += liquidity_delta

// For lower tick: liquidity_net += delta
// For upper tick: liquidity_net -= delta

// Removing liquidity:
liquidity_gross -= liquidity_delta
```

### 3. Tick Crossing (During Swaps)

```rust
// When price crosses tick 100:

// Flip fee_growth_outside
fee_outside_0 = fee_global_0 - fee_outside_0
fee_outside_1 = fee_global_1 - fee_outside_1

// This "moves" the boundary of "outside"
```

### 4. Deinitialization (Last Liquidity Removed)

```rust
if liquidity_gross == 0 {
    initialized = false;
    // Tick can be deleted from storage
}
```

---

## 📚 Functions Reference

### Tick Update

#### `update_tick`

```rust
pub fn update_tick<F1, F2>(
    env: &Env,
    read_tick: F1,
    write_tick: F2,
    tick: i32,
    current_tick: i32,
    liquidity_delta: i128,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
    upper: bool,
) -> bool
```

Update a tick when liquidity is added or removed.

**Parameters:**
- `env`: Soroban environment
- `read_tick`: Callback to read tick info from storage
- `write_tick`: Callback to write tick info to storage
- `tick`: Tick index to update
- `current_tick`: Current pool tick
- `liquidity_delta`: Liquidity change (positive = add, negative = remove)
- `fee_growth_global_0`: Current global fee growth for token0
- `fee_growth_global_1`: Current global fee growth for token1
- `upper`: `true` if this is an upper tick boundary, `false` if lower

**Returns:** `true` if tick was flipped (initialized ↔ uninitialized)

**Process:**
1. Read current tick info
2. Update `liquidity_gross`
3. Initialize `fee_growth_outside` if first liquidity
4. Update `liquidity_net` based on upper/lower
5. Write updated tick info
6. Return whether tick flipped

**Example:**
```rust
use belugaswap_tick::update_tick;

// Add liquidity at lower tick -1000
let flipped = update_tick(
    &env,
    |e, t| storage::read_tick(e, t),
    |e, t, info| storage::write_tick(e, t, info),
    -1000,              // Tick to update
    0,                  // Current tick
    10_000_000,         // Add 10M liquidity
    fee_global_0,
    fee_global_1,
    false,              // This is a lower tick
);

if flipped {
    println!("Tick -1000 was initialized!");
}
```

---

### Tick Crossing

#### `cross_tick`

```rust
pub fn cross_tick<F1, F2>(
    env: &Env,
    read_tick: F1,
    write_tick: F2,
    tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> i128
```

Cross a tick boundary during a swap.

**Parameters:**
- `env`: Soroban environment
- `read_tick`: Callback to read tick info
- `write_tick`: Callback to write tick info
- `tick`: Tick being crossed
- `fee_growth_global_0`: Current global fee growth for token0
- `fee_growth_global_1`: Current global fee growth for token1

**Returns:** `liquidity_net` to add/subtract from active liquidity

**Process:**
1. Read tick info
2. Flip fee_growth_outside: `new = global - old`
3. Write updated tick info
4. Return liquidity_net

**Example:**
```rust
use belugaswap_tick::cross_tick;

// Price crosses tick -60 during swap
let liquidity_net = cross_tick(
    &env,
    |e, t| storage::read_tick(e, t),
    |e, t, info| storage::write_tick(e, t, info),
    -60,
    current_fee_growth_0,
    current_fee_growth_1,
);

// Update active liquidity based on direction
if zero_for_one {
    // Moving left (selling token0)
    active_liquidity -= liquidity_net;
} else {
    // Moving right (selling token1)
    active_liquidity += liquidity_net;
}
```

---

### Tick Traversal

#### `find_next_initialized_tick`

```rust
pub fn find_next_initialized_tick<F>(
    env: &Env,
    read_tick: F,
    current_tick: i32,
    tick_spacing: i32,
    zero_for_one: bool,
) -> i32
```

Find the next initialized tick in the given direction.

**Parameters:**
- `env`: Soroban environment
- `read_tick`: Callback to read tick info
- `current_tick`: Starting tick
- `tick_spacing`: Pool's tick spacing
- `zero_for_one`: Direction (`true` = search left, `false` = search right)

**Returns:** Next initialized tick index, or `current_tick` if none found

**Process:**
1. Align current_tick to spacing
2. Move one step in the direction
3. Search up to MAX_TICK_SEARCH_STEPS (2000)
4. Return first tick with `initialized = true` and `liquidity_gross > 0`

**Example:**
```rust
use belugaswap_tick::find_next_initialized_tick;

// Find next tick boundary when swapping token0 → token1
let next_tick = find_next_initialized_tick(
    &env,
    |e, t| storage::read_tick(e, t),
    0,      // Current tick
    60,     // Tick spacing
    true,   // zero_for_one (search left)
);

println!("Next tick boundary: {}", next_tick);
// Might return -60, -120, etc.
```

---

### Fee Growth Calculation

#### `get_fee_growth_inside`

```rust
pub fn get_fee_growth_inside(
    env: &Env,
    read_tick: impl Fn(&Env, i32) -> TickInfo,
    lower_tick: i32,
    upper_tick: i32,
    current_tick: i32,
    fee_growth_global_0: u128,
    fee_growth_global_1: u128,
) -> (u128, u128)
```

Calculate fee growth inside a tick range.

**Parameters:**
- `env`: Soroban environment
- `read_tick`: Callback to read tick info
- `lower_tick`: Lower boundary of range
- `upper_tick`: Upper boundary of range
- `current_tick`: Current pool tick
- `fee_growth_global_0`: Global fee growth for token0
- `fee_growth_global_1`: Global fee growth for token1

**Returns:** `(fee_growth_inside_0, fee_growth_inside_1)`

**Example:**
```rust
use belugaswap_tick::get_fee_growth_inside;

// Calculate fees earned in range [-1000, 1000]
let (fee_inside_0, fee_inside_1) = get_fee_growth_inside(
    &env,
    |e, t| storage::read_tick(e, t),
    -1000,  // Lower tick
    1000,   // Upper tick
    0,      // Current tick (inside range)
    fee_growth_global_0,
    fee_growth_global_1,
);

// Use for position fee calculation
let position_fees_0 = (liquidity * fee_inside_0) / (1u128 << 64);
let position_fees_1 = (liquidity * fee_inside_1) / (1u128 << 64);
```

---

### Tick Validation

#### `is_valid_tick`

```rust
pub fn is_valid_tick(tick: i32) -> bool
```

Check if a tick is within valid range `[MIN_TICK, MAX_TICK]`.

**Example:**
```rust
use belugaswap_tick::is_valid_tick;

assert!(is_valid_tick(0));
assert!(is_valid_tick(887272));
assert!(!is_valid_tick(1_000_000));  // Too high
```

---

#### `is_aligned_tick`

```rust
pub fn is_aligned_tick(tick: i32, tick_spacing: i32) -> bool
```

Check if a tick is properly aligned to spacing.

**Example:**
```rust
use belugaswap_tick::is_aligned_tick;

assert!(is_aligned_tick(60, 60));   // 60 % 60 = 0 ✓
assert!(is_aligned_tick(120, 60));  // 120 % 60 = 0 ✓
assert!(!is_aligned_tick(50, 60));  // 50 % 60 ≠ 0 ✗
```

---

#### `align_tick`

```rust
pub fn align_tick(tick: i32, tick_spacing: i32) -> i32
```

Align a tick to the nearest valid tick based on spacing.

**Example:**
```rust
use belugaswap_tick::align_tick;

assert_eq!(align_tick(125, 60), 120);   // Round down
assert_eq!(align_tick(-75, 60), -120);  // Round down
assert_eq!(align_tick(60, 60), 60);     // Already aligned
```

---

## 🔧 Usage Examples

### Example 1: Initialize Position Ticks

```rust
use belugaswap_tick::update_tick;

let lower_tick = -1000;
let upper_tick = 1000;
let liquidity = 10_000_000;

// Update lower tick
let lower_flipped = update_tick(
    &env,
    read_tick,
    write_tick,
    lower_tick,
    current_tick,
    liquidity,
    fee_growth_global_0,
    fee_growth_global_1,
    false,  // lower tick
);

// Update upper tick
let upper_flipped = update_tick(
    &env,
    read_tick,
    write_tick,
    upper_tick,
    current_tick,
    liquidity,
    fee_growth_global_0,
    fee_growth_global_1,
    true,   // upper tick
);

if lower_flipped {
    println!("Lower tick {} initialized", lower_tick);
}
if upper_flipped {
    println!("Upper tick {} initialized", upper_tick);
}
```

---

### Example 2: Remove Liquidity from Position

```rust
use belugaswap_tick::update_tick;

let liquidity_to_remove = 5_000_000;

// Update lower tick (remove liquidity)
update_tick(
    &env,
    read_tick,
    write_tick,
    lower_tick,
    current_tick,
    -liquidity_to_remove,  // Negative = remove
    fee_growth_global_0,
    fee_growth_global_1,
    false,
);

// Update upper tick (remove liquidity)
update_tick(
    &env,
    read_tick,
    write_tick,
    upper_tick,
    current_tick,
    -liquidity_to_remove,
    fee_growth_global_0,
    fee_growth_global_1,
    true,
);
```

---

### Example 3: Swap with Tick Crossing

```rust
use belugaswap_tick::{find_next_initialized_tick, cross_tick};

let mut current_tick = 0;
let mut active_liquidity = 10_000_000;
let zero_for_one = true;  // Selling token0

// Find next tick boundary
let next_tick = find_next_initialized_tick(
    &env,
    read_tick,
    current_tick,
    60,
    zero_for_one,
);

// Swap to next tick
// ... swap calculation ...

// Cross the tick
let liquidity_net = cross_tick(
    &env,
    read_tick,
    write_tick,
    next_tick,
    fee_growth_global_0,
    fee_growth_global_1,
);

// Update active liquidity
if zero_for_one {
    active_liquidity -= liquidity_net;
    current_tick = next_tick - 1;
} else {
    active_liquidity += liquidity_net;
    current_tick = next_tick;
}

println!("Crossed tick {}, new liquidity: {}", 
    next_tick, 
    active_liquidity
);
```

---

### Example 4: Calculate Position Fees

```rust
use belugaswap_tick::get_fee_growth_inside;

// Position from tick -1000 to 1000
let lower_tick = -1000;
let upper_tick = 1000;
let position_liquidity = 5_000_000;

// Get fee growth inside the position's range
let (fee_inside_0, fee_inside_1) = get_fee_growth_inside(
    &env,
    read_tick,
    lower_tick,
    upper_tick,
    current_tick,
    fee_growth_global_0,
    fee_growth_global_1,
);

// Calculate fees earned since last update
let last_checkpoint_0 = position.fee_growth_inside_last_0;
let last_checkpoint_1 = position.fee_growth_inside_last_1;

let delta_0 = fee_inside_0.wrapping_sub(last_checkpoint_0);
let delta_1 = fee_inside_1.wrapping_sub(last_checkpoint_1);

// Calculate actual fee amounts
let fees_0 = (position_liquidity as u128 * delta_0) >> 64;
let fees_1 = (position_liquidity as u128 * delta_1) >> 64;

println!("Fees earned: {} token0, {} token1", fees_0, fees_1);
```

---

### Example 5: Multi-Tick Swap Simulation

```rust
use belugaswap_tick::{find_next_initialized_tick, cross_tick};

let mut current_tick = 0;
let mut liquidity = 10_000_000;
let mut ticks_crossed = 0;

// Simulate swapping until amount exhausted or max iterations
for _ in 0..10 {
    let next_tick = find_next_initialized_tick(
        &env,
        read_tick,
        current_tick,
        60,
        true,  // zero_for_one
    );
    
    if next_tick == current_tick {
        println!("No more initialized ticks");
        break;
    }
    
    println!("Swapping from tick {} to {}", current_tick, next_tick);
    
    // Cross the tick
    let liquidity_net = cross_tick(
        &env,
        read_tick,
        write_tick,
        next_tick,
        fee_growth_global_0,
        fee_growth_global_1,
    );
    
    // Update state
    liquidity -= liquidity_net;
    current_tick = next_tick - 1;
    ticks_crossed += 1;
    
    println!("  New liquidity: {}", liquidity);
}

println!("Total ticks crossed: {}", ticks_crossed);
```

---

### Example 6: Validate Tick Parameters

```rust
use belugaswap_tick::{is_valid_tick, is_aligned_tick, align_tick};

fn validate_position(lower: i32, upper: i32, spacing: i32) -> Result<(), &'static str> {
    // Check valid range
    if !is_valid_tick(lower) || !is_valid_tick(upper) {
        return Err("Tick out of range");
    }
    
    // Check alignment
    if !is_aligned_tick(lower, spacing) {
        return Err("Lower tick not aligned");
    }
    
    if !is_aligned_tick(upper, spacing) {
        return Err("Upper tick not aligned");
    }
    
    // Check order
    if lower >= upper {
        return Err("Lower tick must be less than upper tick");
    }
    
    Ok(())
}

// Auto-align ticks
fn auto_align_position(lower: i32, upper: i32, spacing: i32) -> (i32, i32) {
    let aligned_lower = align_tick(lower, spacing);
    let aligned_upper = align_tick(upper, spacing);
    (aligned_lower, aligned_upper)
}

// Usage
let (lower, upper) = auto_align_position(-975, 1025, 60);
assert_eq!(lower, -1020);  // Aligned down
assert_eq!(upper, 1020);   // Aligned down
```

---

### Example 7: Track Tick State Changes

```rust
use belugaswap_tick::update_tick;

struct TickStateTracker {
    initialized_ticks: Vec<i32>,
}

impl TickStateTracker {
    fn track_update(
        &mut self,
        tick: i32,
        flipped: bool,
        liquidity_after: i128,
    ) {
        if flipped {
            if liquidity_after > 0 {
                // Tick was initialized
                self.initialized_ticks.push(tick);
                println!("✅ Tick {} initialized", tick);
            } else {
                // Tick was deinitialized
                self.initialized_ticks.retain(|&t| t != tick);
                println!("❌ Tick {} deinitialized", tick);
            }
        }
    }
}

// Usage
let mut tracker = TickStateTracker { 
    initialized_ticks: Vec::new() 
};

let flipped = update_tick(
    &env,
    read_tick,
    write_tick,
    -1000,
    0,
    10_000_000,
    fee_global_0,
    fee_global_1,
    false,
);

let tick_info = read_tick(&env, -1000);
tracker.track_update(-1000, flipped, tick_info.liquidity_gross);
```

---

## 🎓 Advanced Topics

### Wrapping Arithmetic for Fee Growth

Fee growth values use wrapping arithmetic to handle overflow:

```rust
// Fee growth can overflow u128::MAX
fee_growth_outside_0 = fee_growth_global_0
    .wrapping_sub(fee_growth_outside_0);

// Delta calculation also uses wrapping
delta = current_fee_growth.wrapping_sub(last_checkpoint);
```

**Why?** Fees accumulate indefinitely. Using wrapping arithmetic ensures:
- No panics on overflow
- Correct delta calculations via modular arithmetic

---

### Liquidity Net Explained

```rust
// liquidity_net represents the change in active liquidity
// when crossing this tick from left to right:

Position from tick A (lower) to tick B (upper):
  At tick A: liquidity_net = +L  (entering range)
  At tick B: liquidity_net = -L  (exiting range)

When swapping left-to-right (zero_for_one = false):
  Cross tick A → active_liquidity += liquidity_net
  Cross tick B → active_liquidity += liquidity_net

When swapping right-to-left (zero_for_one = true):
  Cross tick B → active_liquidity -= liquidity_net
  Cross tick A → active_liquidity -= liquidity_net
```

---

### Fee Growth Outside Flip

When crossing a tick, we "flip" the fee_growth_outside:

```
Before crossing tick 100:
  fee_outside = fees earned below tick 100

After crossing tick 100 (going right):
  fee_outside = fees earned above tick 100
  
The flip formula:
  new_outside = global - old_outside
```

This works because:
```
global = below + above

Before: outside = below
After:  outside = global - below = above ✓
```

---

### Tick Search Optimization

The `find_next_initialized_tick` function has optimizations:

```rust
const MAX_TICK_SEARCH_STEPS: i32 = 2000;

// Limit prevents:
// 1. Infinite loops
// 2. Excessive gas costs
// 3. DOS attacks

// If no tick found in 2000 steps:
//    Returns current_tick
//    Swap will stop at current position
```

---

### TickInfo Memory Layout

```rust
pub struct TickInfo {
    pub liquidity_gross: i128,         // 16 bytes
    pub liquidity_net: i128,           // 16 bytes
    pub fee_growth_outside_0: u128,    // 16 bytes
    pub fee_growth_outside_1: u128,    // 16 bytes
    pub initialized: bool,             // 1 byte
}
// Total: ~65 bytes per initialized tick
```

---

## 📊 Data Structure

### TickInfo

```rust
pub struct TickInfo {
    /// Total liquidity referencing this tick
    pub liquidity_gross: i128,
    
    /// Net liquidity change when crossing left-to-right
    pub liquidity_net: i128,
    
    /// Fee growth outside this tick for token0
    pub fee_growth_outside_0: u128,
    
    /// Fee growth outside this tick for token1
    pub fee_growth_outside_1: u128,
    
    /// Whether this tick is initialized
    pub initialized: bool,
}
```

**Field Explanations:**

- `liquidity_gross`: Total amount of liquidity using this tick as a boundary
  - Used to determine if tick should be initialized
  - Sum of all positions using this tick

- `liquidity_net`: Change in active liquidity when crossing this tick
  - Positive for lower ticks (liquidity enters)
  - Negative for upper ticks (liquidity exits)

- `fee_growth_outside_0/1`: Accumulated fees on the "other side" of this tick
  - In Q64.64 format (fee per unit of liquidity)
  - Flipped when tick is crossed

- `initialized`: Whether this tick has any liquidity
  - `true` when liquidity_gross > 0
  - `false` when liquidity_gross = 0

---

## 🔗 Links

- **Repository**: [github.com/Beluga-Swap/core](https://github.com/Beluga-Swap/core)
- **Tick Package**: [packages/tick](https://github.com/Beluga-Swap/core/tree/main/packages/tick)
- **Math Package**: [packages/math](https://github.com/Beluga-Swap/core/tree/main/packages/math)
- **Position Package**: [packages/position](https://github.com/Beluga-Swap/core/tree/main/packages/position)
- **Pool Contract**: [contracts/pool](https://github.com/Beluga-Swap/core/tree/main/contracts/pool)
- **Soroban Docs**: [soroban.stellar.org](https://soroban.stellar.org)

---

## 📄 License

MIT License - see LICENSE file for details