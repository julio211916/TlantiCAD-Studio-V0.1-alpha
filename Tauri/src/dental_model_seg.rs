use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;
use tauri::State;

use crate::python_runtime::resolve_default_python_path;

const PYTHON_PATH_ENV: &str = "TLANTI_PYTHON_PATH";
const REPO_PATH_ENV: &str = "TLANTI_DENTALMODELSEG_REPO";
const CLI_WRAPPER_ENV: &str = "TLANTI_DENTALMODELSEG_CLI_WRAPPER";
const EXECUTABLE_ENV: &str = "TLANTI_DENTALMODELSEG_EXECUTABLE";
const MODEL_PATH_ENV: &str = "TLANTI_DENTALMODELSEG_MODEL";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalModelSegConfig {
    pub python_path: Option<String>,
    pub repo_path: Option<String>,
    pub cli_wrapper_path: Option<String>,
    pub dental_model_seg_executable_path: Option<String>,
    pub model_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedDentalModelSegConfig {
    python_path: PathBuf,
    repo_path: Option<PathBuf>,
    cli_wrapper_path: Option<PathBuf>,
    dental_model_seg_executable_path: Option<PathBuf>,
    model_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalModelSegStatus {
    pub ready: bool,
    pub sidecar_running: bool,
    pub python_available: bool,
    pub python_path: String,
    pub repo_path: Option<String>,
    pub cli_wrapper_path: Option<String>,
    pub dental_model_seg_executable_path: Option<String>,
    pub model_path: Option<String>,
    pub missing: Vec<String>,
    pub notes: Vec<String>,
    pub supported_input_formats: Vec<String>,
    pub numbering_systems: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDentalModelSegRequest {
    pub input_path: String,
    pub output_path: Option<String>,
    pub numbering_system: String,
    pub array_name: Option<String>,
    pub overwrite: Option<bool>,
    pub suffix: Option<String>,
    pub vtk_folder: Option<String>,
    pub input_csv: Option<String>,
    pub crown_segmentation: Option<bool>,
    pub config: Option<DentalModelSegConfig>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDentalModelSegResponse {
    pub success: bool,
    pub input_path: String,
    pub output_path: String,
    pub numbering_system: String,
    pub logs: String,
    pub model_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SidecarRequest {
    action: String,
    payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SidecarResponse {
    success: bool,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

struct RunningDentalModelSegSidecar {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    config: ResolvedDentalModelSegConfig,
}

#[derive(Default)]
pub struct DentalModelSegSidecarState {
    process: Mutex<Option<RunningDentalModelSegSidecar>>,
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key)
        .ok()
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

fn default_cli_wrapper_from_repo(repo_path: &Path) -> PathBuf {
    repo_path
        .join("CrownSegmentationcli")
        .join("CrownSegmentationcli.py")
}

fn resolve_config(config: Option<DentalModelSegConfig>) -> ResolvedDentalModelSegConfig {
    let config = config.unwrap_or_default();

    let python_path = config
        .python_path
        .map(PathBuf::from)
        .or_else(|| env_path(PYTHON_PATH_ENV))
        .or_else(resolve_default_python_path)
        .unwrap_or_else(|| PathBuf::from("python3"));

    let repo_path = config
        .repo_path
        .map(PathBuf::from)
        .or_else(|| env_path(REPO_PATH_ENV));
    let cli_wrapper_path = config
        .cli_wrapper_path
        .map(PathBuf::from)
        .or_else(|| env_path(CLI_WRAPPER_ENV))
        .or_else(|| repo_path.as_deref().map(default_cli_wrapper_from_repo));
    let dental_model_seg_executable_path = config
        .dental_model_seg_executable_path
        .map(PathBuf::from)
        .or_else(|| env_path(EXECUTABLE_ENV));
    let model_path = config
        .model_path
        .map(PathBuf::from)
        .or_else(|| env_path(MODEL_PATH_ENV));

    ResolvedDentalModelSegConfig {
        python_path,
        repo_path,
        cli_wrapper_path,
        dental_model_seg_executable_path,
        model_path,
    }
}

fn path_to_string(path: Option<&PathBuf>) -> Option<String> {
    path.map(|value| value.to_string_lossy().to_string())
}

fn check_python_available(path: &Path) -> bool {
    Command::new(path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn build_status(
    config: &ResolvedDentalModelSegConfig,
    sidecar_running: bool,
) -> DentalModelSegStatus {
    let python_available = check_python_available(&config.python_path);
    let mut missing = Vec::new();
    let mut notes = vec![
        "This integration adapts the external SlicerDentalModelSeg CLI into TlantiCAD via Python."
            .to_string(),
        "Desktop runtime is required. Browser fallback exposes status only.".to_string(),
    ];

    if !python_available {
        missing.push(format!(
            "Python interpreter is not available: {}",
            config.python_path.to_string_lossy()
        ));
    }

    match &config.cli_wrapper_path {
        Some(path) if path.exists() => {}
        Some(path) => missing.push(format!("CLI wrapper not found: {}", path.to_string_lossy())),
        None => missing.push("CLI wrapper path is not configured.".to_string()),
    }

    match &config.dental_model_seg_executable_path {
        Some(path) if path.exists() => {}
        Some(path) => missing.push(format!(
            "DentalModelSeg executable not found: {}",
            path.to_string_lossy()
        )),
        None => missing.push("DentalModelSeg executable path is not configured.".to_string()),
    }

    if let Some(model_path) = &config.model_path {
        if !model_path.exists() {
            missing.push(format!(
                "Model path not found: {}",
                model_path.to_string_lossy()
            ));
        }
    } else {
        notes.push(
            "No model path configured. The external CLI will be called with `latest`.".to_string(),
        );
    }

    notes.push(
        "Supported first-cut inputs: .stl and .vtk models imported from disk with sourcePath."
            .to_string(),
    );
    notes.push(
        "Output visualization of .vtk results inside the viewport is planned in a later phase."
            .to_string(),
    );

    DentalModelSegStatus {
        ready: missing.is_empty(),
        sidecar_running,
        python_available,
        python_path: config.python_path.to_string_lossy().to_string(),
        repo_path: path_to_string(config.repo_path.as_ref()),
        cli_wrapper_path: path_to_string(config.cli_wrapper_path.as_ref()),
        dental_model_seg_executable_path: path_to_string(
            config.dental_model_seg_executable_path.as_ref(),
        ),
        model_path: path_to_string(config.model_path.as_ref()),
        missing,
        notes,
        supported_input_formats: vec![".stl".to_string(), ".vtk".to_string()],
        numbering_systems: vec!["FDI".to_string(), "UNIVERSAL".to_string()],
    }
}

fn normalize_output_path(request: &RunDentalModelSegRequest) -> Result<PathBuf, String> {
    if let Some(output_path) = &request.output_path {
        return Ok(PathBuf::from(output_path));
    }

    let input_path = PathBuf::from(&request.input_path);
    let parent = input_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Input path does not have a parent directory.".to_string())?;
    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Input path does not have a valid file name.".to_string())?;
    let suffix = request.suffix.clone().unwrap_or_else(|| "_seg".to_string());
    Ok(parent.join(format!("{}{}.vtk", stem, suffix)))
}

fn ensure_sidecar_script() -> Result<PathBuf, String> {
    let root = std::env::temp_dir().join("tlanticad-ai");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let script_path = root.join("dental_model_seg_sidecar.py");
    fs::write(&script_path, DENTAL_MODEL_SEG_SIDECAR).map_err(|error| error.to_string())?;
    Ok(script_path)
}

fn start_sidecar(
    state: &DentalModelSegSidecarState,
    config: ResolvedDentalModelSegConfig,
) -> Result<(), String> {
    let mut guard = state
        .process
        .lock()
        .map_err(|_| "Failed to lock dental model segmentation sidecar state".to_string())?;

    let should_restart = guard
        .as_ref()
        .map(|running| running.config != config)
        .unwrap_or(true);

    if !should_restart {
        return Ok(());
    }

    if let Some(mut running) = guard.take() {
        let _ = running.child.kill();
        let _ = running.child.wait();
    }

    let script_path = ensure_sidecar_script()?;
    let mut child = Command::new(&config.python_path)
        .arg(&script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start dental model segmentation sidecar: {error}"))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to open sidecar stdin".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to open sidecar stdout".to_string())?;

    *guard = Some(RunningDentalModelSegSidecar {
        child,
        stdin,
        stdout: BufReader::new(stdout),
        config,
    });

    Ok(())
}

fn send_request(
    state: &DentalModelSegSidecarState,
    config: ResolvedDentalModelSegConfig,
    action: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    start_sidecar(state, config.clone())?;

    let mut guard = state
        .process
        .lock()
        .map_err(|_| "Failed to lock sidecar process".to_string())?;
    let running = guard
        .as_mut()
        .ok_or_else(|| "Sidecar is not running".to_string())?;

    let request = serde_json::to_string(&SidecarRequest {
        action: action.to_string(),
        payload,
    })
    .map_err(|error| error.to_string())?;

    writeln!(running.stdin, "{request}").map_err(|error| error.to_string())?;
    running.stdin.flush().map_err(|error| error.to_string())?;

    let mut line = String::new();
    running
        .stdout
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    if line.trim().is_empty() {
        return Err("No response from dental model segmentation sidecar".to_string());
    }

    let response: SidecarResponse =
        serde_json::from_str(line.trim()).map_err(|error| error.to_string())?;
    if response.success {
        Ok(response.result.unwrap_or(serde_json::Value::Null))
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Unknown sidecar error".to_string()))
    }
}

#[tauri::command]
pub fn get_dental_model_seg_status(
    state: State<'_, DentalModelSegSidecarState>,
    config: Option<DentalModelSegConfig>,
) -> Result<DentalModelSegStatus, String> {
    let resolved = resolve_config(config);
    let sidecar_running = state
        .process
        .lock()
        .map_err(|_| "Failed to inspect sidecar state".to_string())?
        .is_some();
    Ok(build_status(&resolved, sidecar_running))
}

#[tauri::command]
pub fn run_dental_model_segmentation(
    state: State<'_, DentalModelSegSidecarState>,
    request: RunDentalModelSegRequest,
) -> Result<RunDentalModelSegResponse, String> {
    let resolved = resolve_config(request.config.clone());
    let status = build_status(&resolved, true);
    if !status.ready {
        return Err(status.missing.join(" | "));
    }

    let input_path = PathBuf::from(&request.input_path);
    if !input_path.exists() {
        return Err(format!(
            "Input model does not exist: {}",
            input_path.to_string_lossy()
        ));
    }

    let extension = input_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "Input model does not have a valid extension".to_string())?;
    if extension != "stl" && extension != "vtk" {
        return Err(
            "AI segmentation currently supports .stl or .vtk source files only".to_string(),
        );
    }

    let output_path = normalize_output_path(&request)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let payload = serde_json::json!({
      "config": {
        "cliWrapperPath": resolved.cli_wrapper_path.as_ref().map(|path| path.to_string_lossy().to_string()),
        "dentalModelSegExecutablePath": resolved.dental_model_seg_executable_path.as_ref().map(|path| path.to_string_lossy().to_string()),
        "modelPath": resolved.model_path.as_ref().map(|path| path.to_string_lossy().to_string()),
      },
      "request": {
        "inputPath": input_path.to_string_lossy().to_string(),
        "outputPath": output_path.to_string_lossy().to_string(),
        "numberingSystem": request.numbering_system,
        "arrayName": request.array_name.unwrap_or_else(|| "PredictedID".to_string()),
        "overwrite": request.overwrite.unwrap_or(true),
        "suffix": request.suffix.unwrap_or_else(|| "_seg".to_string()),
        "vtkFolder": request.vtk_folder.unwrap_or_else(|| "None".to_string()),
        "inputCsv": request.input_csv.unwrap_or_else(|| "None".to_string()),
        "crownSegmentation": request.crown_segmentation.unwrap_or(true),
      }
    });

    let result = send_request(&state, resolved.clone(), "segment", payload)?;
    let logs = result
        .get("logs")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    Ok(RunDentalModelSegResponse {
        success: true,
        input_path: input_path.to_string_lossy().to_string(),
        output_path: output_path.to_string_lossy().to_string(),
        numbering_system: if request.numbering_system.eq_ignore_ascii_case("FDI") {
            "FDI".to_string()
        } else {
            "UNIVERSAL".to_string()
        },
        logs,
        model_path: path_to_string(resolved.model_path.as_ref()),
    })
}

#[tauri::command]
pub fn stop_dental_model_seg_sidecar(
    state: State<'_, DentalModelSegSidecarState>,
) -> Result<(), String> {
    let mut guard = state
        .process
        .lock()
        .map_err(|_| "Failed to lock sidecar state".to_string())?;
    if let Some(mut running) = guard.take() {
        let _ = running.child.kill();
        let _ = running.child.wait();
    }
    Ok(())
}

const DENTAL_MODEL_SEG_SIDECAR: &str = r#"#!/usr/bin/env python3
import json
import subprocess
import sys
import traceback


def bool_string(value):
    return "true" if value else "false"


def handle_segment(payload):
    config = payload.get("config") or {}
    request = payload.get("request") or {}

    cli_wrapper = config.get("cliWrapperPath")
    executable = config.get("dentalModelSegExecutablePath")
    model_path = config.get("modelPath") or "latest"
    if not cli_wrapper:
        raise RuntimeError("cliWrapperPath is required")
    if not executable:
        raise RuntimeError("dentalModelSegExecutablePath is required")

    command = [
        sys.executable,
        cli_wrapper,
        request.get("inputPath"),
        request.get("inputCsv", "None"),
        request.get("outputPath"),
        bool_string(request.get("overwrite", True)),
        model_path,
        bool_string(request.get("crownSegmentation", True)),
        request.get("arrayName", "PredictedID"),
        bool_string(str(request.get("numberingSystem", "FDI")).upper() == "FDI"),
        request.get("suffix", "_seg"),
        request.get("vtkFolder", "None"),
        executable,
    ]

    completed = subprocess.run(command, capture_output=True, text=True)
    logs = "\n".join([completed.stdout or "", completed.stderr or ""]).strip()
    if completed.returncode != 0:
        raise RuntimeError(logs or f"DentalModelSeg CLI exited with code {completed.returncode}")

    return {
        "outputPath": request.get("outputPath"),
        "logs": logs,
        "command": command,
    }


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
            action = request.get("action")
            payload = request.get("payload") or {}
            if action == "segment":
                result = handle_segment(payload)
                response = {"success": True, "result": result, "error": None}
            elif action == "ping":
                response = {"success": True, "result": {"message": "pong"}, "error": None}
            else:
                response = {"success": False, "result": None, "error": f"Unknown action: {action}"}
        except Exception:
            response = {"success": False, "result": None, "error": traceback.format_exc()}
        print(json.dumps(response), flush=True)


if __name__ == "__main__":
    main()
"#;
