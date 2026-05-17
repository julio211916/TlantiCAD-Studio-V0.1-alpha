//! Sidecar Manager for TlantiStudio
//!
//! Manages external processes like PostgreSQL and Python services.

pub mod manager;
pub mod postgres;

pub use manager::*;
pub use postgres::*;

use app_core::error::CoreError;
use thiserror::Error;

/// Sidecar errors
#[derive(Error, Debug)]
pub enum SidecarError {
    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Failed to start process: {0}")]
    StartError(String),

    #[error("Failed to stop process: {0}")]
    StopError(String),

    #[error("Process crashed: {0}")]
    Crashed(String),

    #[error("Port already in use: {0}")]
    PortInUse(u16),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<SidecarError> for CoreError {
    fn from(err: SidecarError) -> Self {
        CoreError::Sidecar(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SidecarError>;
