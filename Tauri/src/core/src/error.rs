//! Error types for TlantiStudio core

use thiserror::Error;

/// Core error type
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("MeshLib error: {0}")]
    MeshLib(String),

    #[error("ML Runtime error: {0}")]
    MlRuntime(String),

    #[error("Python bridge error: {0}")]
    PythonBridge(String),

    #[error("Sidecar error: {0}")]
    Sidecar(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias using CoreError
pub type Result<T> = std::result::Result<T, CoreError>;

impl serde::Serialize for CoreError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
