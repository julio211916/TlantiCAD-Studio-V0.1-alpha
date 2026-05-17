//! Python-related Tauri commands

use python_bridge::{PythonExecutor, PythonSidecar};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::CommandResult;

/// Python state managed by Tauri
pub struct PythonState {
    pub executor: Arc<RwLock<PythonExecutor>>,
    pub sidecar: Arc<RwLock<PythonSidecar>>,
}

/// Python execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct PythonResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

/// Execute Python code
#[tauri::command]
pub async fn execute_python_code(
    state: State<'_, PythonState>,
    code: String,
) -> CommandResult<PythonResult> {
    let executor = state.executor.read().await;
    
    match executor.execute_code(&code) {
        Ok(result) => {
            let output = serde_json::from_str(&result)
                .unwrap_or(serde_json::Value::String(result));
            Ok(PythonResult {
                success: true,
                output,
                error: None,
            })
        }
        Err(e) => {
            if matches!(e, python_bridge::PythonError::InterpreterNotFound) {
                let sidecar = state.sidecar.read().await;
                if !sidecar.is_running().await {
                    sidecar.start().await?;
                }

                let output = sidecar.execute_code(&code, serde_json::Value::Null).await?;
                return Ok(PythonResult {
                    success: true,
                    output,
                    error: None,
                });
            }

            Ok(PythonResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Execute Python script file
#[tauri::command]
pub async fn execute_python_script(
    state: State<'_, PythonState>,
    script_path: String,
    args: Option<String>,
) -> CommandResult<PythonResult> {
    let executor = state.executor.read().await;
    let path = PathBuf::from(&script_path);
    
    match executor.execute_script(&path, args.as_deref()) {
        Ok(result) => {
            let output = serde_json::from_str(&result)
                .unwrap_or(serde_json::Value::String(result));
            Ok(PythonResult {
                success: true,
                output,
                error: None,
            })
        }
        Err(e) => {
            if matches!(e, python_bridge::PythonError::InterpreterNotFound) {
                let sidecar = state.sidecar.read().await;
                if !sidecar.is_running().await {
                    sidecar.start().await?;
                }

                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default();

                if filename.is_empty() {
                    return Ok(PythonResult {
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some("Script path is invalid".to_string()),
                    });
                }

                let args_json = args
                    .and_then(|raw| serde_json::from_str(&raw).ok())
                    .unwrap_or(serde_json::Value::Null);

                let output = sidecar.execute_script(filename, args_json).await?;
                return Ok(PythonResult {
                    success: true,
                    output,
                    error: None,
                });
            }

            Ok(PythonResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Check if a Python package is installed
#[tauri::command]
#[allow(unused)]
pub async fn check_python_package(
    state: State<'_, PythonState>,
    package_name: String,
) -> CommandResult<bool> {
    let executor = state.executor.read().await;
    Ok(executor.check_package(&package_name))
}

/// Install a Python package
#[tauri::command]
#[allow(unused)]
pub async fn install_python_package(
    state: State<'_, PythonState>,
    package_name: String,
) -> CommandResult<()> {
    let executor = state.executor.read().await;
    executor.install_package(&package_name)?;
    Ok(())
}

/// Start Python sidecar process
#[tauri::command]
#[allow(unused)]
pub async fn start_python_sidecar(
    state: State<'_, PythonState>,
) -> CommandResult<()> {
    let sidecar = state.sidecar.read().await;
    sidecar.start().await?;
    Ok(())
}

/// Stop Python sidecar process
#[tauri::command]
#[allow(unused)]
pub async fn stop_python_sidecar(
    state: State<'_, PythonState>,
) -> CommandResult<()> {
    let sidecar = state.sidecar.read().await;
    sidecar.stop().await?;
    Ok(())
}

/// Execute script via sidecar
#[tauri::command]
#[allow(unused)]
pub async fn sidecar_execute_script(
    state: State<'_, PythonState>,
    script_name: String,
    args: serde_json::Value,
) -> CommandResult<serde_json::Value> {
    let sidecar = state.sidecar.read().await;
    let result = sidecar.execute_script(&script_name, args).await?;
    Ok(result)
}

/// Check if sidecar is running
#[tauri::command]
#[allow(unused)]
pub async fn is_sidecar_running(
    state: State<'_, PythonState>,
) -> CommandResult<bool> {
    let sidecar = state.sidecar.read().await;
    Ok(sidecar.is_running().await)
}
