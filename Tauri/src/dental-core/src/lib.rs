//! TlantiStudio Dental Core - Domain Models and Types
//! 
//! This crate contains all the core domain models for the dental clinic management system.

pub mod models;
pub mod error;
pub mod events;
pub mod enums;

pub use error::DentalError;
pub use models::*;
pub use enums::*;

// Re-export medical and dental notation crates
pub use medrs;
pub use dental_notation;
