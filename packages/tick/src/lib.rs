#![no_std]

pub mod types;
pub mod update;
pub mod fee_growth;

// Re-export types
pub use types::TickInfo;

// Re-export update functions
pub use update::{
    update_tick, 
    cross_tick, 
    find_next_initialized_tick, 
    is_valid_tick,
    is_aligned_tick,  // <-- ADDED: was missing
    align_tick,       // <-- ADDED: was missing
};

// Re-export fee growth functions
pub use fee_growth::get_fee_growth_inside;

// Re-export from math for convenience
pub use belugaswap_math::snap_tick_to_spacing;