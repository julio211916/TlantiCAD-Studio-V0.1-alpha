//! Core library for TlantiStudio
//! 
//! This crate contains the core logic, types, and utilities used across
//! the entire TlantiStudio application.

pub mod config;
pub mod error;
pub mod events;
pub mod types;
pub mod utils;

pub use error::{CoreError, Result};
pub use types::*;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "TlantiStudio";
