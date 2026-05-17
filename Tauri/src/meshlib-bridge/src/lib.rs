//! MeshLib Bridge for TlantiStudio
//!
//! This crate provides the bridge between Rust/Tauri and MeshLib WebAssembly.
//! MeshLib is compiled to WebAssembly using Emscripten and loaded in the frontend.
//! This crate handles mesh data serialization and provides native Rust mesh operations.

pub mod formats;
pub mod mesh_ops;
pub mod wasm_bridge;

pub use formats::*;
pub use mesh_ops::*;
pub use wasm_bridge::*;

use app_core::error::CoreError;
use thiserror::Error;

/// MeshLib-specific errors
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Failed to load mesh: {0}")]
    LoadError(String),

    #[error("Failed to save mesh: {0}")]
    SaveError(String),

    #[error("Invalid mesh format: {0}")]
    InvalidFormat(String),

    #[error("Mesh operation failed: {0}")]
    OperationFailed(String),

    #[error("WASM bridge error: {0}")]
    WasmError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<MeshError> for CoreError {
    fn from(err: MeshError) -> Self {
        CoreError::MeshLib(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MeshError>;
