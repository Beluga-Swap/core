#![no_std]

pub mod types;
pub mod manager;
pub mod fees;

// Re-export types
pub use types::{Position, PositionInfo};

// Re-export manager functions
pub use manager::{
    modify_position, 
    update_position,
    has_liquidity,           // <-- ADDED: was not exported
    has_uncollected_fees,    // <-- ADDED: was missing
    is_empty,                // <-- ADDED: was missing
    clear_fees,              // <-- ADDED: was missing
};

// Re-export fee functions
pub use fees::calculate_pending_fees;

// Re-export validation
pub use manager::validate_position_params;  // <-- ADDED: was missing