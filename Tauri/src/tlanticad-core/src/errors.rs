//! Error types for TlantiCAD

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TlantiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Mesh processing error: {0}")]
    Mesh(String),

    #[error("Geometry error: {0}")]
    Geometry(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("License error: {0}")]
    License(String),

    #[error("Workflow error: {0}")]
    Workflow(String),

    #[error("AI inference error: {0}")]
    AiInference(String),

    #[error("Import/Export error: {0}")]
    IoError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, TlantiError>;

impl From<sqlx::Error> for TlantiError {
    fn from(e: sqlx::Error) -> Self {
        TlantiError::Database(e.to_string())
    }
}
