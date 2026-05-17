//! System-related Tauri commands

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{CommandError, CommandResult};

/// System information
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub app_version: String,
    pub data_dir: String,
    pub models_dir: String,
    pub python_available: bool,
}

/// Get system information
#[tauri::command]
pub async fn get_system_info() -> CommandResult<SystemInfo> {
    Ok(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: get_data_dir().to_string_lossy().to_string(),
        models_dir: get_models_dir().to_string_lossy().to_string(),
        python_available: check_python_available(),
    })
}

/// Get application data directory
pub fn get_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TlantiStudio")
}

/// Get models directory
pub fn get_models_dir() -> PathBuf {
    get_data_dir().join("models")
}

/// Get database path
pub fn get_database_path() -> PathBuf {
    get_data_dir().join("data").join("tlanti.db")
}

/// Get Python scripts directory
pub fn get_python_scripts_dir() -> PathBuf {
    get_data_dir().join("python")
}

/// Check if Python is available
fn check_python_available() -> bool {
    let candidate = find_embedded_python().or_else(find_env_python);

    let mut command = match candidate {
        Some(path) => std::process::Command::new(path),
        None => std::process::Command::new("python3"),
    };

    command
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn find_env_python() -> Option<PathBuf> {
    std::env::var("TLANTI_PYTHON_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.exists())
}

fn find_embedded_python() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let mut candidates = vec![
        exe_dir.join("resources").join("python").join("python"),
        exe_dir.join("resources").join("python").join("python.exe"),
        exe_dir.join("python").join("python"),
        exe_dir.join("python").join("python.exe"),
    ];

    if let Some(parent) = exe_dir.parent() {
        candidates.extend([
            parent.join("Resources").join("python").join("python"),
            parent.join("Resources").join("python").join("python.exe"),
        ]);
    }

    candidates.into_iter().find(|path| path.exists())
}

/// Get available disk space
#[tauri::command]
pub async fn get_disk_space() -> CommandResult<DiskSpace> {
    let _data_dir = get_data_dir();
    
    // This is a simplified version - in production you'd use sysinfo
    Ok(DiskSpace {
        total_bytes: 0,
        free_bytes: 0,
        used_bytes: 0,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSpace {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
}

/// Open file in system file manager
#[tauri::command]
pub async fn show_in_folder(path: String) -> CommandResult<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| CommandError {
                code: "SYSTEM_ERROR".to_string(),
                message: e.to_string(),
            })?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| CommandError {
                code: "SYSTEM_ERROR".to_string(),
                message: e.to_string(),
            })?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(std::path::Path::new(&path).parent().unwrap_or_else(|| std::path::Path::new(".")))
            .spawn()
            .map_err(|e| CommandError {
                code: "SYSTEM_ERROR".to_string(),
                message: e.to_string(),
            })?;
    }

    Ok(())
}

/// Initialize application directories
#[tauri::command]
pub async fn init_app_directories() -> CommandResult<()> {
    let dirs = [
        get_data_dir(),
        get_models_dir(),
        get_data_dir().join("data"),
        get_python_scripts_dir(),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    Ok(())
}
