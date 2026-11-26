//! Multiply-divide with a wide intermediate, so that no price or amount formula
//! silently overflows u128.

/// Limbs of the dividend: a 256-bit product shifted by at most 128 bits.
const LIMBS: usize = 6;

type Wide = [u64; LIMBS];

/// `floor` or `ceil` of `a * b * 2^shift / (d1 * d2)`, saturating at `u128::MAX`.
///
/// `shift` must be at most 128 and both divisors non-zero.
pub fn mul_div_wide(a: u128, b: u128, shift: u32, d1: u128, d2: u128, round_up: bool) -> u128 {
    assert!(d1 != 0 && d2 != 0, "mul_div_wide: divide by zero");
    assert!(shift <= 128, "mul_div_wide: shift too large");

    // floor(floor(x / d1) / d2) == floor(x / (d1 * d2))
    let mut value = shl(widening_mul(a, b), shift);
    let first = div_assign(&mut value, d1);
    let second = div_assign(&mut value, d2);

    if value[2..].iter().any(|limb| *limb != 0) {
        return u128::MAX;
    }
    let quotient = (value[0] as u128) | ((value[1] as u128) << 64);
    if round_up && (first || second) {
        quotient.saturating_add(1)
    } else {
        quotient
    }
}

fn widening_mul(a: u128, b: u128) -> Wide {
    let a = [a as u64, (a >> 64) as u64];
    let b = [b as u64, (b >> 64) as u64];
    let mut out: Wide = [0; LIMBS];
    for i in 0..2 {
        let mut carry: u128 = 0;
        for j in 0..2 {
            let t = (a[i] as u128) * (b[j] as u128) + (out[i + j] as u128) + carry;
            out[i + j] = t as u64;
            carry = t >> 64;
        }
        out[i + 2] = carry as u64;
    }
    out
}

fn shl(value: Wide, shift: u32) -> Wide {
    let limbs = (shift / 64) as usize;
    let bits = shift % 64;
    let mut out: Wide = [0; LIMBS];
    for i in limbs..LIMBS {
        out[i] = value[i - limbs] << bits;
        if bits != 0 && i > limbs {
            out[i] |= value[i - limbs - 1] >> (64 - bits);
        }
    }
    out
}

/// Divides in place and reports whether a remainder was left.
fn div_assign(value: &mut Wide, divisor: u128) -> bool {
    if divisor == 1 {
        return false;
    }
    let mut remainder: u128 = 0;
    if divisor <= u64::MAX as u128 {
        // Limb by limb: the running remainder fits in 64 bits.
        for i in (0..LIMBS).rev() {
            let current = (remainder << 64) | value[i] as u128;
            value[i] = (current / divisor) as u64;
            remainder = current % divisor;
        }
    } else {
        // Bit by bit long division.
        for i in (0..LIMBS).rev() {
            if remainder == 0 && value[i] == 0 {
                continue;
            }
            let limb = value[i];
            let mut quotient: u64 = 0;
            for bit in (0..64).rev() {
                let carry = remainder >> 127;
                remainder = (remainder << 1) | ((limb >> bit) & 1) as u128;
                if carry != 0 || remainder >= divisor {
                    remainder = remainder.wrapping_sub(divisor);
                    quotient |= 1 << bit;
                }
            }
            value[i] = quotient;
        }
    }
    remainder != 0
}
