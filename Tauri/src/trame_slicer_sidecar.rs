use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::python_runtime;

const TRAME_SIDECAR_URL_ENV: &str = "TLANTI_TRAME_SLICER_URL";
const DEFAULT_TRAME_SIDECAR_URL: &str = "http://127.0.0.1:17494";
const HEALTH_TIMEOUT_MS: u64 = 1_500;
const CLINICAL_TIMEOUT_MS: u64 = 900_000;

static TRAME_CHILD: Mutex<Option<Child>> = Mutex::new(None);
static TRAME_CACHE: Mutex<TrameSlicerSidecarCache> = Mutex::new(TrameSlicerSidecarCache {
    first_seen_alive: None,
    last_health_check: None,
    last_health_check_latency: None,
    last_error: None,
});

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrameSlicerSidecarStatus {
    pub name: &'static str,
    pub url: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub last_health_check: Option<u64>,
    pub last_health_check_latency_ms: Option<u64>,
    pub uptime_secs: Option<u64>,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlicerModelsDownloadRequest {
    #[serde(default = "default_true")]
    pub include_optional: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlicerFixtureDownloadRequest {
    pub fixture_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlicerClinicalJobRequest {
    pub case_id: String,
    pub workflow_id: String,
    pub source_path: String,
    pub output_dir: Option<String>,
    pub model_id: Option<String>,
    #[serde(default)]
    pub options: Value,
}

#[derive(Debug)]
struct TrameSlicerSidecarCache {
    first_seen_alive: Option<Instant>,
    last_health_check: Option<SystemTime>,
    last_health_check_latency: Option<Duration>,
    last_error: Option<String>,
}

fn default_true() -> bool {
    true
}

fn base_url() -> String {
    std::env::var(TRAME_SIDECAR_URL_ENV)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_TRAME_SIDECAR_URL.to_string())
}

fn backend_python_dir() -> Option<PathBuf> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|root| root.join("backend").join("python"))
}

fn probe_health(url: &str) -> (bool, Duration, Option<String>) {
    let started = Instant::now();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
    {
        Ok(rt) => rt,
        Err(error) => return (false, started.elapsed(), Some(error.to_string())),
    };
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(HEALTH_TIMEOUT_MS))
        .build()
    {
        Ok(client) => client,
        Err(error) => return (false, started.elapsed(), Some(error.to_string())),
    };
    let health_url = format!("{}/health", url.trim_end_matches('/'));
    runtime.block_on(async move {
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => (true, started.elapsed(), None),
            Ok(response) => (
                false,
                started.elapsed(),
                Some(format!("HTTP {} from {health_url}", response.status())),
            ),
            Err(error) => (false, started.elapsed(), Some(error.to_string())),
        }
    })
}

fn status_from_probe(pid: Option<u32>) -> Result<TrameSlicerSidecarStatus, String> {
    let url = base_url();
    let (healthy, latency, error) = probe_health(&url);
    let mut cache = TRAME_CACHE
        .lock()
        .map_err(|error| format!("trame sidecar cache poisoned: {error}"))?;

    if healthy {
        if cache.first_seen_alive.is_none() {
            cache.first_seen_alive = Some(Instant::now());
        }
        cache.last_health_check = Some(SystemTime::now());
        cache.last_health_check_latency = Some(latency);
        cache.last_error = None;
    } else {
        cache.last_health_check_latency = Some(latency);
        cache.last_error = error.clone();
    }

    Ok(TrameSlicerSidecarStatus {
        name: "tlanticad-trame-slicer-sidecar",
        url,
        running: healthy,
        pid,
        last_health_check: cache
            .last_health_check
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_secs()),
        last_health_check_latency_ms: cache
            .last_health_check_latency
            .map(|value| value.as_millis() as u64),
        uptime_secs: if healthy {
            cache.first_seen_alive.map(|value| value.elapsed().as_secs())
        } else {
            None
        },
        restart_count: 0,
        last_error: cache.last_error.clone(),
    })
}

fn ensure_running() -> Result<(), String> {
    if status_from_probe(child_pid()?)?.running {
        return Ok(());
    }
    let status = trame_slicer_sidecar_start()?;
    if status.running {
        Ok(())
    } else {
        Err(status
            .last_error
            .unwrap_or_else(|| "trame-slicer sidecar did not become healthy".to_string()))
    }
}

fn sidecar_get(path: &str, timeout_ms: u64) -> Result<Value, String> {
    ensure_running()?;
    let url = format!("{}{}", base_url().trim_end_matches('/'), path);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|error| error.to_string())?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(async move {
        let response = client.get(&url).send().await.map_err(|error| error.to_string())?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP {status} from {url}"));
        }
        response.json::<Value>().await.map_err(|error| error.to_string())
    })
}

fn sidecar_post<T: Serialize>(path: &str, body: &T, timeout_ms: u64) -> Result<Value, String> {
    ensure_running()?;
    let url = format!("{}{}", base_url().trim_end_matches('/'), path);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|error| error.to_string())?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(async move {
        let response = client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|error| error.to_string())?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP {status} from {url}"));
        }
        response.json::<Value>().await.map_err(|error| error.to_string())
    })
}

fn child_pid() -> Result<Option<u32>, String> {
    let mut guard = TRAME_CHILD
        .lock()
        .map_err(|error| format!("trame sidecar child lock poisoned: {error}"))?;
    if let Some(child) = guard.as_mut() {
        match child.try_wait().map_err(|error| error.to_string())? {
            Some(_) => {
                *guard = None;
                Ok(None)
            }
            None => Ok(Some(child.id())),
        }
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn trame_slicer_sidecar_status() -> Result<TrameSlicerSidecarStatus, String> {
    status_from_probe(child_pid()?)
}

#[tauri::command]
pub fn trame_slicer_sidecar_start() -> Result<TrameSlicerSidecarStatus, String> {
    if let Some(pid) = child_pid()? {
        return status_from_probe(Some(pid));
    }

    let python = python_runtime::resolve_default_python_path()
        .ok_or_else(|| "No Python runtime found. Set TLANTI_PYTHON_PATH or create Tauri/backend/.venv.".to_string())?;
    let python_dir = backend_python_dir()
        .ok_or_else(|| "Could not resolve Tauri/backend/python directory.".to_string())?;
    let url = base_url();
    let port = url
        .rsplit(':')
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(17494)
        .to_string();

    let child = Command::new(python)
        .arg("-m")
        .arg("uvicorn")
        .arg("trame_slicer_sidecar:app")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(&port)
        .current_dir(&python_dir)
        .env("PYTHONPATH", &python_dir)
        .env("TLANTI_TRAME_SLICER_PORT", &port)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start trame-slicer sidecar: {error}"))?;

    let pid = child.id();
    {
        let mut guard = TRAME_CHILD
            .lock()
            .map_err(|error| format!("trame sidecar child lock poisoned: {error}"))?;
        *guard = Some(child);
    }

    std::thread::sleep(Duration::from_millis(500));
    status_from_probe(Some(pid))
}

#[tauri::command]
pub fn trame_slicer_sidecar_stop() -> Result<TrameSlicerSidecarStatus, String> {
    {
        let mut guard = TRAME_CHILD
            .lock()
            .map_err(|error| format!("trame sidecar child lock poisoned: {error}"))?;
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *guard = None;
    }
    status_from_probe(None)
}

#[tauri::command]
pub fn slicer_runtime_status() -> Result<Value, String> {
    sidecar_get("/slicer/runtime/status", HEALTH_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_runtime_download() -> Result<Value, String> {
    sidecar_post("/slicer/runtime/download", &serde_json::json!({}), CLINICAL_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_models_status() -> Result<Value, String> {
    sidecar_get("/slicer/models/status", HEALTH_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_fixtures_status() -> Result<Value, String> {
    sidecar_get("/slicer/fixtures/status", HEALTH_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_fixtures_download(request: SlicerFixtureDownloadRequest) -> Result<Value, String> {
    sidecar_post("/slicer/fixtures/download", &request, CLINICAL_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_models_download_all(request: SlicerModelsDownloadRequest) -> Result<Value, String> {
    sidecar_post("/slicer/models/download-all", &request, CLINICAL_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_clinical_job_start(request: SlicerClinicalJobRequest) -> Result<Value, String> {
    sidecar_post("/slicer/jobs/start", &request, CLINICAL_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_clinical_job_status(job_id: String) -> Result<Value, String> {
    sidecar_get(&format!("/slicer/jobs/{job_id}"), HEALTH_TIMEOUT_MS)
}

#[tauri::command]
pub fn slicer_clinical_job_cancel(job_id: String) -> Result<Value, String> {
    sidecar_post(&format!("/slicer/jobs/{job_id}/cancel"), &serde_json::json!({}), HEALTH_TIMEOUT_MS)
}
