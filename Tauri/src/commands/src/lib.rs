//! Tauri Commands for TlantiStudio
//!
//! All Tauri commands are defined here and registered with the app.

pub mod database;
pub mod mesh;
pub mod ml;
pub mod project;
pub mod python;
pub mod system;

pub use database::*;
pub use mesh::*;
pub use ml::*;
pub use project::*;
pub use python::*;
pub use system::*;

use app_core::error::CoreError;

/// Command result type
pub type CommandResult<T> = Result<T, CommandError>;

/// Command error that can be serialized to frontend
#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<CoreError> for CommandError {
    fn from(err: CoreError) -> Self {
        Self {
            code: "CORE_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<::database::DbError> for CommandError {
    fn from(err: ::database::DbError) -> Self {
        Self {
            code: "DATABASE_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<meshlib_bridge::MeshError> for CommandError {
    fn from(err: meshlib_bridge::MeshError) -> Self {
        Self {
            code: "MESH_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<ml_runtime::MlError> for CommandError {
    fn from(err: ml_runtime::MlError) -> Self {
        Self {
            code: "ML_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<python_bridge::PythonError> for CommandError {
    fn from(err: python_bridge::PythonError) -> Self {
        Self {
            code: "PYTHON_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<sidecar_manager::SidecarError> for CommandError {
    fn from(err: sidecar_manager::SidecarError) -> Self {
        Self {
            code: "SIDECAR_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self {
            code: "IO_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}
