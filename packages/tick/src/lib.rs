#![no_std]

pub mod types;
pub mod update;
pub mod fee_growth;
pub mod bitmap;

// Re-export types
pub use types::TickInfo;

// Re-export update functions
pub use update::{
    update_tick, 
    cross_tick, 
    is_valid_tick,
    is_aligned_tick,  // <-- ADDED: was missing
    align_tick,       // <-- ADDED: was missing
};

pub use bitmap::{flip_tick, next_initialized_tick};

// Re-export fee growth functions
pub use fee_growth::get_fee_growth_inside;

// Re-export from math for convenience
pub use belugaswap_math::snap_tick_to_spacing;