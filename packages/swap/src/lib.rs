#![no_std]

pub mod types;
pub mod engine;

// Re-export types
pub use types::{SwapResult, PreviewResult};

// Re-export engine functions and types
pub use engine::{
    SwapState, 
    engine_swap, 
    quote_swap, 
    validate_and_preview_swap,
    // Note: engine_swap_safe is intentionally not exported (internal use only)
};