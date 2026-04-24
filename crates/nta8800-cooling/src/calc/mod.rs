//! Rekenmodule — twee entry-points:
//!
//! - [`calculate_cooling`] — actieve koeling, pad 1 (H.10)
//! - [`calculate_simplified_cooling`] — vereenvoudigd, pad 2 (bijlage AA)

pub mod monthly_use;
pub mod simplified;

pub use monthly_use::calculate_cooling;
pub use simplified::{calculate_simplified_cooling, SimplifiedAreaInput, SimplifiedLoadInput};
