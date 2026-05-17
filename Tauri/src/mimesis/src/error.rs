use thiserror::Error;

#[derive(Error, Debug)]
pub enum MimesisError {
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No contours found in image")]
    NoContours,

    #[error("Polygon too small: {0} vertices (minimum: 3)")]
    PolygonTooSmall(usize),

    #[error("Triangulation failed: {0}")]
    Triangulation(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MimesisError>;
