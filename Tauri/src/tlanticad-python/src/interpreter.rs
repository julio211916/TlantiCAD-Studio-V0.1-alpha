//! Python interpreter bridge for executing dental analysis scripts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Execution method for Python scripts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMethod {
    /// Execute via system Python subprocess
    Subprocess,
    /// Embedded via PyO3 (requires pyo3-bridge feature)
    Embedded,
}

/// Script execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    pub method: ExecutionMethod,
    pub python_path: Option<String>,
    pub timeout_secs: u64,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<String>,
}

impl Default for ScriptContext {
    fn default() -> Self {
        Self {
            method: ExecutionMethod::Subprocess,
            python_path: None,
            timeout_secs: 30,
            env_vars: HashMap::new(),
            working_dir: None,
        }
    }
}

/// Result of Python script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub output_data: Option<serde_json::Value>,
    pub duration_ms: u64,
}

/// Python script definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonScript {
    pub name: String,
    pub code: String,
    pub input_data: Option<serde_json::Value>,
    pub requirements: Vec<String>,
}

impl PythonScript {
    pub fn new(name: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
            input_data: None,
            requirements: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.input_data = Some(data);
        self
    }

    pub fn require(mut self, package: impl Into<String>) -> Self {
        self.requirements.push(package.into());
        self
    }
}

/// Execute a Python script via subprocess
pub fn execute_script(script: &PythonScript, ctx: &ScriptContext) -> ScriptResult {
    use std::process::Command;
    use std::time::Instant;

    let start = Instant::now();

    // Build Python code: inject input data if present
    let full_code = if let Some(ref data) = script.input_data {
        format!(
            "import json\n_input = json.loads('{}')\n{}",
            serde_json::to_string(data).unwrap_or_default().replace('\'', "\\'"),
            script.code
        )
    } else {
        script.code.clone()
    };

    let python_bin = ctx.python_path.as_deref().unwrap_or("python3");

    let output = Command::new(python_bin)
        .arg("-c")
        .arg(&full_code)
        .envs(&ctx.env_vars)
        .output();

    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let exit_code = out.status.code().unwrap_or(-1);
            let output_data = serde_json::from_str(&stdout).ok();
            ScriptResult { success: out.status.success(), stdout, stderr, exit_code, output_data, duration_ms }
        }
        Err(e) => ScriptResult {
            success: false,
            stdout: String::new(),
            stderr: e.to_string(),
            exit_code: -1,
            output_data: None,
            duration_ms,
        },
    }
}

/// Check if Python is available on system
pub fn python_available() -> bool {
    std::process::Command::new("python3").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// Get Python version string
pub fn python_version() -> Option<String> {
    let out = std::process::Command::new("python3").arg("--version").output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
