//! Configuration management for TlantiStudio

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub ml: MlConfig,
    pub python: PythonConfig,
    pub ui: UiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            ml: MlConfig::default(),
            python: PythonConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub sqlite_path: PathBuf,
    pub postgres_url: Option<String>,
    pub pool_size: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            sqlite_path: PathBuf::from("data/tlanti.db"),
            postgres_url: None,
            pool_size: 10,
        }
    }
}

/// ML Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlConfig {
    pub models_dir: PathBuf,
    pub use_gpu: bool,
    pub onnx_threads: u32,
}

impl Default for MlConfig {
    fn default() -> Self {
        Self {
            models_dir: PathBuf::from("models"),
            use_gpu: true,
            onnx_threads: 4,
        }
    }
}

/// Python sidecar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    pub python_path: Option<PathBuf>,
    pub scripts_dir: PathBuf,
    pub venv_path: Option<PathBuf>,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            python_path: None,
            scripts_dir: PathBuf::from("python"),
            venv_path: None,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub language: String,
    pub show_fps: bool,
    pub auto_save: bool,
    pub auto_save_interval_secs: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            show_fps: false,
            auto_save: true,
            auto_save_interval_secs: 300,
        }
    }
}
