//! ML Runtime for TlantiStudio
//!
//! Provides ONNX Runtime and CNN inference capabilities.
//! PyTorch models should be converted to ONNX for optimal performance.

pub mod inference;
pub mod models;
pub mod preprocessing;

pub use inference::*;
pub use models::*;

use app_core::error::CoreError;
use thiserror::Error;

/// ML Runtime errors
#[derive(Error, Debug)]
pub enum MlError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Failed to load model: {0}")]
    LoadError(String),

    #[error("Inference error: {0}")]
    InferenceError(String),

    #[error("Invalid input shape: expected {expected:?}, got {actual:?}")]
    InvalidInputShape { expected: Vec<i64>, actual: Vec<i64> },

    #[error("Preprocessing error: {0}")]
    PreprocessingError(String),

    #[error("ONNX Runtime error: {0}")]
    OnnxError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<MlError> for CoreError {
    fn from(err: MlError) -> Self {
        CoreError::MlRuntime(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MlError>;
