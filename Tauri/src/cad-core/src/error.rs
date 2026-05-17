use thiserror::Error;

#[derive(Debug, Error)]
pub enum CadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Mesh error: {0}")]
    Mesh(String),

    #[error("Geometry error: {0}")]
    Geometry(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid tooth number {0}: must be 11–48 (FDI) or 1–32 (Universal)")]
    InvalidToothNumber(u8),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CadError>;
