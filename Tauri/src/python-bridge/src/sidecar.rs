//! Python sidecar process management
//!
//! For heavy Python workloads, we run Python as a separate process
//! and communicate via IPC (JSON over stdin/stdout).

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::{PythonError, Result};

/// Python sidecar request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarRequest {
    pub id: String,
    pub action: String,
    pub payload: serde_json::Value,
}

/// Python sidecar response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarResponse {
    pub id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Python sidecar manager
pub struct PythonSidecar {
    python_path: PathBuf,
    scripts_dir: PathBuf,
    process: Arc<RwLock<Option<Child>>>,
}

impl PythonSidecar {
    pub fn new(python_path: Option<PathBuf>, scripts_dir: PathBuf) -> Self {
        let python_path = python_path
            .or_else(resolve_embedded_python)
            .or_else(resolve_env_python)
            .unwrap_or_else(|| PathBuf::from("python3"));
        
        Self {
            python_path,
            scripts_dir,
            process: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the Python sidecar process
    pub async fn start(&self) -> Result<()> {
        let sidecar_script = self.scripts_dir.join("sidecar_main.py");

        if !sidecar_script.exists() {
            // Create default sidecar script
            self.create_default_sidecar_script(&sidecar_script)?;
        }

        let child = Command::new(&self.python_path)
            .arg(&sidecar_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PythonError::SidecarError(e.to_string()))?;

        *self.process.write().await = Some(child);
        info!("Python sidecar started");

        Ok(())
    }

    /// Stop the Python sidecar process
    pub async fn stop(&self) -> Result<()> {
        if let Some(mut process) = self.process.write().await.take() {
            process.kill().ok();
            process.wait().ok();
            info!("Python sidecar stopped");
        }
        Ok(())
    }

    /// Send a request to the sidecar
    pub async fn send_request(&self, request: SidecarRequest) -> Result<SidecarResponse> {
        let mut process_guard = self.process.write().await;
        let process = process_guard
            .as_mut()
            .ok_or_else(|| PythonError::SidecarError("Sidecar not running".to_string()))?;

        // Serialize request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| PythonError::SerializationError(e.to_string()))?;

        // Send to stdin
        if let Some(stdin) = process.stdin.as_mut() {
            writeln!(stdin, "{}", request_json)
                .map_err(|e| PythonError::SidecarError(e.to_string()))?;
            stdin.flush()
                .map_err(|e| PythonError::SidecarError(e.to_string()))?;
        }

        // Read response from stdout
        if let Some(stdout) = process.stdout.as_mut() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.map_err(|e| PythonError::SidecarError(e.to_string()))?;
                if !line.is_empty() {
                    let response: SidecarResponse = serde_json::from_str(&line)
                        .map_err(|e| PythonError::SerializationError(e.to_string()))?;
                    return Ok(response);
                }
            }
        }

        Err(PythonError::SidecarError("No response from sidecar".to_string()))
    }

    /// Execute a Python script via the sidecar
    pub async fn execute_script(
        &self,
        script_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let request = SidecarRequest {
            id: uuid::Uuid::new_v4().to_string(),
            action: "execute_script".to_string(),
            payload: serde_json::json!({
                "script": script_name,
                "args": args,
            }),
        };

        let response = self.send_request(request).await?;

        if response.success {
            Ok(response.result.unwrap_or(serde_json::Value::Null))
        } else {
            Err(PythonError::ExecutionError(
                response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Execute raw Python code via the sidecar
    pub async fn execute_code(
        &self,
        code: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let request = SidecarRequest {
            id: uuid::Uuid::new_v4().to_string(),
            action: "execute_code".to_string(),
            payload: serde_json::json!({
                "code": code,
                "args": args,
            }),
        };

        let response = self.send_request(request).await?;

        if response.success {
            Ok(response.result.unwrap_or(serde_json::Value::Null))
        } else {
            Err(PythonError::ExecutionError(
                response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Create the default sidecar script
    fn create_default_sidecar_script(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path.parent().unwrap())?;

        let script = r#"#!/usr/bin/env python3
"""
TlantiStudio Python Sidecar
Handles IPC requests from Rust via stdin/stdout
"""

import json
import sys
import traceback
from pathlib import Path

# Add scripts directory to path
SCRIPTS_DIR = Path(__file__).parent
sys.path.insert(0, str(SCRIPTS_DIR))


def handle_request(request: dict) -> dict:
    """Handle incoming request"""
    request_id = request.get("id", "unknown")
    action = request.get("action", "")
    payload = request.get("payload", {})
    
    try:
        if action == "execute_script":
            script_name = payload.get("script", "")
            args = payload.get("args", {})
            
            # Import and execute script
            script_path = SCRIPTS_DIR / script_name
            if script_path.exists():
                exec_globals = {"args": args, "result": None}
                exec(script_path.read_text(), exec_globals)
                return {
                    "id": request_id,
                    "success": True,
                    "result": exec_globals.get("result"),
                    "error": None,
                }
            else:
                return {
                    "id": request_id,
                    "success": False,
                    "result": None,
                    "error": f"Script not found: {script_name}",
                }

        elif action == "execute_code":
            code = payload.get("code", "")
            args = payload.get("args", {})

            exec_globals = {"args": args, "result": None}
            exec(code, exec_globals)

            return {
                "id": request_id,
                "success": True,
                "result": exec_globals.get("result"),
                "error": None,
            }
        
        elif action == "call_function":
            module_name = payload.get("module", "")
            function_name = payload.get("function", "")
            args = payload.get("args", [])
            
            module = __import__(module_name)
            func = getattr(module, function_name)
            result = func(*args)
            
            return {
                "id": request_id,
                "success": True,
                "result": result,
                "error": None,
            }
        
        elif action == "ping":
            return {
                "id": request_id,
                "success": True,
                "result": "pong",
                "error": None,
            }
        
        else:
            return {
                "id": request_id,
                "success": False,
                "result": None,
                "error": f"Unknown action: {action}",
            }
    
    except Exception as e:
        return {
            "id": request_id,
            "success": False,
            "result": None,
            "error": traceback.format_exc(),
        }


def main():
    """Main sidecar loop"""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        
        try:
            request = json.loads(line)
            response = handle_request(request)
            print(json.dumps(response), flush=True)
        except json.JSONDecodeError as e:
            error_response = {
                "id": "unknown",
                "success": False,
                "result": None,
                "error": f"Invalid JSON: {e}",
            }
            print(json.dumps(error_response), flush=True)


if __name__ == "__main__":
    main()
"#;

        std::fs::write(path, script)?;
        info!("Created default sidecar script at {:?}", path);

        Ok(())
    }

    /// Check if sidecar is running
    pub async fn is_running(&self) -> bool {
        if let Some(_process) = self.process.read().await.as_ref() {
            // Try to get exit status without blocking
            // If we can't, it's still running
            true
        } else {
            false
        }
    }
}

fn resolve_env_python() -> Option<PathBuf> {
    std::env::var("TLANTI_PYTHON_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.exists())
}

fn resolve_embedded_python() -> Option<PathBuf> {
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
