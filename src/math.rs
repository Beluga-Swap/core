#![allow(dead_code)]
use soroban_sdk::{Env, U256};

// ============================================================
// CONSTANTS
// ============================================================
pub const ONE_X64: u128 = 1u128 << 64;
pub const MIN_TICK: i32 = -887_272;
pub const MAX_TICK: i32 = 887_272;

pub const MIN_LIQUIDITY: i128 = 0; 
pub const MIN_PRICE_DELTA: u128 = 1; 

// ============================================================
// TICK SPACING
// ============================================================
pub fn snap_tick_to_spacing(tick: i32, spacing: i32) -> i32 {
    if spacing <= 0 {
        panic!("tick_spacing must be > 0");
    }
    let spacing_abs = spacing.abs();
    let rem = tick.rem_euclid(spacing_abs);
    tick - rem
}

// ============================================================
// LOW LEVEL MATH HELPER
// ============================================================

fn i128_to_u128_safe(x: i128) -> u128 {
    if x <= 0 { 0 } else { x as u128 }
}

fn u128_to_i128_saturating(x: u128) -> i128 {
    if x > i128::MAX as u128 { i128::MAX } else { x as i128 }
}

#[inline]
pub fn mul_q64(a: u128, b: u128) -> u128 {
    let a_hi = a >> 64;
    let a_lo = a & 0xFFFFFFFFFFFFFFFF;
    let b_hi = b >> 64;
    let b_lo = b & 0xFFFFFFFFFFFFFFFF;
    
    let term_hh = a_hi * b_hi;
    let term_hl = a_hi * b_lo;
    let term_lh = a_lo * b_hi;
    let term_ll = a_lo * b_lo;
    
    (term_hh << 64) + term_hl + term_lh + (term_ll >> 64)
}

#[inline]
pub fn div_q64(a: u128, b: u128) -> u128 {
    if b == 0 { return u128::MAX; }
    
    if a <= (u128::MAX >> 64) {
        return (a << 64) / b;
    }
    
    let q = a / b;
    let r = a % b;
    
    let q_part = q << 64;
    let r_part = if r <= (u128::MAX >> 64) {
        (r << 64) / b
    } else {
        ((r >> 32) << 32) / (b >> 32).max(1)
    };
    
    q_part.saturating_add(r_part)
}

/// FIX: Menggunakan U256 untuk intermediate math
/// Menghitung (a * b) / denominator tanpa overflow di 128 bit
pub fn mul_div(env: &Env, a: u128, b: u128, denominator: u128) -> u128 {
    if denominator == 0 { panic!("mul_div divide by zero"); }

    // Convert ke U256 (Host Type)
    let a_256 = U256::from_u128(env, a);
    let b_256 = U256::from_u128(env, b);
    let den_256 = U256::from_u128(env, denominator);

    // Operasi matematika aman di 256 bit
    // product = a * b
    let product = a_256.mul(&b_256);
    
    // result = product / denominator
    let result = product.div(&den_256);

    // Kembalikan ke u128
    // Jika hasil > u128::MAX, to_u128() akan panic (ini perilaku yg benar utk safety)
    // Tapi di logika DEX normal, result (harga/amount) harusnya muat di u128
    result.to_u128().unwrap_or(u128::MAX)
}

fn div_round_up(numerator: u128, denominator: u128) -> u128 {
    if denominator == 0 { return 0; }
    let result = numerator / denominator;
    if !numerator.is_multiple_of(denominator) {
        result.saturating_add(1)
    } else {
        result
    }
}

// ============================================================
// SQRT PRICE MATH (FIXED WITH U256)
// ============================================================

pub fn get_next_sqrt_price_from_input(
    env: &Env, // Added Env parameter
    sqrt_price: u128,
    liquidity: u128,
    amount_in: u128,
    zero_for_one: bool,
) -> u128 {
    if amount_in == 0 || liquidity == 0 {
        return sqrt_price;
    }

    if zero_for_one {
        // Token0 in -> Price decrease
        
        // Numerator = L * P (Keep Q64 precision)
        // Ini berpotensi besar, tapi kita simpan di u128 dulu jika muat,
        // atau kita gunakan mul_div langsung nanti.
        
        let product = amount_in.saturating_mul(sqrt_price); // Delta L
        let numerator = liquidity.saturating_mul(sqrt_price); // L * P
        
        // Denominator: (L * 2^64) + (amount * P)
        // Kita hitung denominator di u128. Jika L sangat besar, shift left bisa overflow.
        // Tapi untuk sekarang kita asumsikan L masuk akal.
        let liq_shifted = liquidity << 64; 
        let denominator = liq_shifted.saturating_add(product);
        
        if denominator == 0 { return sqrt_price; }

        // Calculation: (numerator * ONE_X64) / denominator
        // Di sinilah kita butuh mul_div U256 karena numerator * ONE_X64 pasti overflow u128
        mul_div(env, numerator, ONE_X64, denominator)

    } else {
        // Token1 in -> Price increase
        let quotient = div_q64(amount_in, liquidity);
        sqrt_price.saturating_add(quotient)
    }
}

pub fn get_next_sqrt_price_from_output(
    env: &Env, // Added Env parameter
    sqrt_price: u128,
    liquidity: u128,
    amount_out: u128,
    zero_for_one: bool,
) -> u128 {
    if amount_out == 0 || liquidity == 0 {
        return sqrt_price;
    }

    if zero_for_one {
        // Token1 out -> Price decrease
        let quotient = div_q64(amount_out, liquidity); 
        sqrt_price.saturating_sub(quotient)
    } else {
        // Token0 out -> Price increase
        let product = amount_out.saturating_mul(sqrt_price);
        let numerator = liquidity.saturating_mul(sqrt_price);
        
        let liq_shifted = liquidity << 64;
        let denominator = liq_shifted.saturating_sub(product);
        
        if denominator == 0 { return u128::MAX; } 

        // Calculation: (numerator * ONE_X64) / denominator
        mul_div(env, numerator, ONE_X64, denominator)
    }
}

// ============================================================
// SWAP MATH (DELTA CALCULATION)
// ============================================================

pub fn get_amount_0_delta(
    sqrt_price_a: u128,
    sqrt_price_b: u128,
    liquidity: u128,
    round_up: bool,
) -> u128 {
    let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_a < sqrt_price_b {
        (sqrt_price_a, sqrt_price_b)
    } else {
        (sqrt_price_b, sqrt_price_a)
    };

    let delta_price = sqrt_price_upper.saturating_sub(sqrt_price_lower);
    let product_prices = mul_q64(sqrt_price_upper, sqrt_price_lower);
    
    if product_prices == 0 { return 0; }
    
    // Formula: L * (upper - lower) / (upper * lower)
    // Numerator is L * delta (Q64 scaled)
    let numerator = liquidity.saturating_mul(delta_price); 
    
    // Denominator is product_prices (Q64 scaled)
    // Result is Int.
    
    if round_up {
        div_round_up(numerator, product_prices)
    } else {
        numerator / product_prices
    }
}

pub fn get_amount_1_delta(
    sqrt_price_a: u128,
    sqrt_price_b: u128,
    liquidity: u128,
    round_up: bool,
) -> u128 {
    let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_a < sqrt_price_b {
        (sqrt_price_a, sqrt_price_b)
    } else {
        (sqrt_price_b, sqrt_price_a)
    };

    let delta = sqrt_price_upper.saturating_sub(sqrt_price_lower);
    let product = liquidity.saturating_mul(delta); 
    
    if round_up {
        if product & 0xFFFFFFFFFFFFFFFF != 0 {
            (product >> 64) + 1
        } else {
            product >> 64
        }
    } else {
        product >> 64
    }
}

// ============================================================
// MAIN COMPUTE SWAP STEP
// ============================================================

pub fn compute_swap_step(
    env: &Env,
    sqrt_price_current: u128,
    liquidity: i128,
    amount_remaining: i128,
    zero_for_one: bool,
) -> (u128, i128, i128) {
    if liquidity <= 0 || amount_remaining <= 0 {
        return (sqrt_price_current, 0, 0);
    }
    
    let liq_u = i128_to_u128_safe(liquidity);
    let amt_in_remaining = i128_to_u128_safe(amount_remaining);
    
    // PASS ENV HERE
    let next_sqrt_price = get_next_sqrt_price_from_input(
        env,
        sqrt_price_current,
        liq_u,
        amt_in_remaining,
        zero_for_one
    );
    
    let sqrt_price_next = next_sqrt_price;
    
    let price_delta = sqrt_price_next.abs_diff(sqrt_price_current);
    
    if price_delta < MIN_PRICE_DELTA {
        return (sqrt_price_current, 0, 0);
    }

    let amount_0 = get_amount_0_delta(
        sqrt_price_current,
        sqrt_price_next,
        liq_u,
        true 
    );
    
    let amount_1 = get_amount_1_delta(
        sqrt_price_current,
        sqrt_price_next,
        liq_u,
        true 
    );
    
    let (amount_in, amount_out) = if zero_for_one {
        (amount_0, amount_1)
    } else {
        (amount_1, amount_0)
    };
    
    let final_amount_in = if amount_in > amt_in_remaining {
        amt_in_remaining
    } else {
        amount_in
    };

    (
        sqrt_price_next,
        u128_to_i128_saturating(final_amount_in),
        u128_to_i128_saturating(amount_out)
    )
}

pub fn compute_swap_step_with_target(
    env: &Env,
    sqrt_price_current: u128,
    liquidity: i128,
    amount_specified: i128,
    zero_for_one: bool,
    sqrt_price_target: u128,
) -> (u128, i128, i128) {
    let liq_u = i128_to_u128_safe(liquidity);
    let amount_rem_u = i128_to_u128_safe(amount_specified);

    // PASS ENV HERE
    let next_price_input = get_next_sqrt_price_from_input(
        env,
        sqrt_price_current,
        liq_u,
        amount_rem_u,
        zero_for_one
    );

    let target_reached = if zero_for_one {
        next_price_input <= sqrt_price_target
    } else {
        next_price_input >= sqrt_price_target
    };

    let sqrt_price_next = if target_reached {
        sqrt_price_target
    } else {
        next_price_input
    };

    let amount_in: u128;
    let amount_out: u128;

    if zero_for_one {
        amount_in = get_amount_0_delta(sqrt_price_current, sqrt_price_next, liq_u, true);
        amount_out = get_amount_1_delta(sqrt_price_current, sqrt_price_next, liq_u, false);
    } else {
        amount_in = get_amount_1_delta(sqrt_price_current, sqrt_price_next, liq_u, true);
        amount_out = get_amount_0_delta(sqrt_price_current, sqrt_price_next, liq_u, false);
    }

    let final_amount_in = if !target_reached && amount_in > amount_rem_u {
        amount_rem_u
    } else {
        amount_in
    };

    (
        sqrt_price_next,
        u128_to_i128_saturating(final_amount_in),
        u128_to_i128_saturating(amount_out)
    )
}
// ============================================================
// CONVERSION HELPER
// ============================================================

pub fn get_sqrt_ratio_at_tick(tick: i32) -> u128 {
    if !(MIN_TICK..=MAX_TICK).contains(&tick) { panic!("tick out of range"); }
    if tick == 0 { return ONE_X64; }
    let abs_tick = tick.unsigned_abs();
    let mut ratio: u128 = ONE_X64;
    
    if abs_tick & 0x1 != 0 { ratio = mul_q64(ratio, 18447666387855958016); }
    if abs_tick & 0x2 != 0 { ratio = mul_q64(ratio, 18448588748116922368); }
    if abs_tick & 0x4 != 0 { ratio = mul_q64(ratio, 18450433606991732736); }
    if abs_tick & 0x8 != 0 { ratio = mul_q64(ratio, 18454123878217469952); }
    if abs_tick & 0x10 != 0 { ratio = mul_q64(ratio, 18461506635090006016); }
    if abs_tick & 0x20 != 0 { ratio = mul_q64(ratio, 18476281010653908992); }
    if abs_tick & 0x40 != 0 { ratio = mul_q64(ratio, 18505849059060717568); }
    if abs_tick & 0x80 != 0 { ratio = mul_q64(ratio, 18565033932859791360); }
    if abs_tick & 0x100 != 0 { ratio = mul_q64(ratio, 18683636815981789184); }
    if abs_tick & 0x200 != 0 { ratio = mul_q64(ratio, 18922376066158198784); }
    if abs_tick & 0x400 != 0 { ratio = mul_q64(ratio, 19403906064415539200); }
    if abs_tick & 0x800 != 0 { ratio = mul_q64(ratio, 20388321338895749120); }
    if abs_tick & 0x1000 != 0 { ratio = mul_q64(ratio, 22486086334269071360); }
    if abs_tick & 0x2000 != 0 { ratio = mul_q64(ratio, 27241267204663885824); }
    if abs_tick & 0x4000 != 0 { ratio = mul_q64(ratio, 40198444615281172480); }
    if abs_tick & 0x8000 != 0 { ratio = mul_q64(ratio, 87150709742682460160); }
    if abs_tick & 0x10000 != 0 { ratio = mul_q64(ratio, 409916713094318874624); }
    
    if tick < 0 {
        if ratio == 0 { return u128::MAX; }
        let numerator = ONE_X64.saturating_mul(ONE_X64); 
        ratio = numerator / ratio;
    }
    ratio
}

pub fn tick_to_sqrt_price_x64(_env: &Env, tick: i32) -> u128 { get_sqrt_ratio_at_tick(tick) }

pub fn get_liquidity_for_amount0(
    _env: &Env, amount0: i128, sqrt_price_lower: u128, sqrt_price_upper: u128
) -> i128 {
    if amount0 <= 0 || sqrt_price_lower >= sqrt_price_upper { return 0; }
    let amt0_u = i128_to_u128_safe(amount0);
    let product = mul_q64(sqrt_price_upper, sqrt_price_lower);
    let numerator = amt0_u.saturating_mul(product); 
    let denominator = sqrt_price_upper.saturating_sub(sqrt_price_lower);
    if denominator == 0 { return 0; }
    u128_to_i128_saturating(numerator / denominator)
}

pub fn get_liquidity_for_amount1(
    _env: &Env, amount1: i128, sqrt_price_lower: u128, sqrt_price_upper: u128
) -> i128 {
    if amount1 <= 0 || sqrt_price_lower >= sqrt_price_upper { return 0; }
    let amt1_u = i128_to_u128_safe(amount1);
    let diff = sqrt_price_upper.saturating_sub(sqrt_price_lower);
    if diff == 0 { return 0; }
    let liq_u = div_q64(amt1_u, diff);
    u128_to_i128_saturating(liq_u)
}

pub fn get_liquidity_for_amounts(
    env: &Env,
    amount0_desired: i128, amount1_desired: i128,
    sqrt_price_lower: u128, sqrt_price_upper: u128, current_sqrt_price: u128,
) -> i128 {
    if sqrt_price_lower >= sqrt_price_upper { return 0; }
    if current_sqrt_price <= sqrt_price_lower {
        get_liquidity_for_amount0(env, amount0_desired, sqrt_price_lower, sqrt_price_upper)
    } else if current_sqrt_price >= sqrt_price_upper {
        get_liquidity_for_amount1(env, amount1_desired, sqrt_price_lower, sqrt_price_upper)
    } else {
        let liq0 = get_liquidity_for_amount0(env, amount0_desired, current_sqrt_price, sqrt_price_upper);
        let liq1 = get_liquidity_for_amount1(env, amount1_desired, sqrt_price_lower, current_sqrt_price);
        liq0.min(liq1)
    }
}

pub fn get_amounts_for_liquidity(
    _env: &Env, liquidity: i128, sqrt_price_lower: u128, sqrt_price_upper: u128, current_sqrt_price: u128,
) -> (i128, i128) {
    if liquidity <= 0 { return (0, 0); }
    let liq_u = i128_to_u128_safe(liquidity);
    let mut sp = current_sqrt_price;
    if sp < sqrt_price_lower { sp = sqrt_price_lower; }
    if sp > sqrt_price_upper { sp = sqrt_price_upper; }
    let mut amount0_u: u128 = 0;
    let mut amount1_u: u128 = 0;
    if sp < sqrt_price_upper { amount0_u = get_amount_0_delta(sp, sqrt_price_upper, liq_u, false); }
    if sp > sqrt_price_lower { amount1_u = get_amount_1_delta(sqrt_price_lower, sp, liq_u, false); }
    (u128_to_i128_saturating(amount0_u), u128_to_i128_saturating(amount1_u))
}