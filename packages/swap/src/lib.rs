#![no_std]

pub mod types;
pub mod engine;

// Re-export types
pub use types::{PoolSwapResult, PreviewResult};

// Re-export engine functions and types
pub use engine::{
    SwapState, 
    engine_swap,
    SwapError,
    quote_swap, 
    validate_and_preview_swap,
};