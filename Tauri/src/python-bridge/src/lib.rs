//! Python Bridge for TlantiStudio
//!
//! Provides PyO3 integration for running Python scripts and using Python libraries.
//! This enables ML inference with PyTorch, custom Python scripts, and more.

pub mod executor;
pub mod sidecar;
pub mod pydicom;

pub use executor::*;
pub use sidecar::*;

use app_core::error::CoreError;
use thiserror::Error;

/// Python bridge errors
#[derive(Error, Debug)]
pub enum PythonError {
    #[error("Python interpreter not found")]
    InterpreterNotFound,

    #[error("Failed to execute script: {0}")]
    ExecutionError(String),

    #[error("Python exception: {0}")]
    PythonException(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Sidecar error: {0}")]
    SidecarError(String),

    #[error("Timeout: script did not complete in time")]
    Timeout,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<PythonError> for CoreError {
    fn from(err: PythonError) -> Self {
        CoreError::PythonBridge(err.to_string())
    }
}

#[cfg(feature = "pyo3")]
impl From<pyo3::PyErr> for PythonError {
    fn from(err: pyo3::PyErr) -> Self {
        PythonError::PythonException(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PythonError>;
