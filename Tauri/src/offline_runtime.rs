use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

const PYTHON_RUNTIME_MANIFEST_ENV: &str = "TLANTI_PYTHON_RUNTIME_MANIFEST";
const PYTHON_HOME_ENV: &str = "TLANTI_PYTHON_HOME";
const PYTHON_PATH_ENV: &str = "TLANTI_PYTHON_PATH";
const BUNDLED_RUNTIME_MANIFEST_RESOURCE: &str = "python-runtime/runtime.json";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PythonRuntimeManifest {
    bundled_python_home: Option<String>,
    bundled_python_path: Option<String>,
    bundled_python_relative_home: Option<String>,
    bundled_python_relative_path: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineRuntimeStatus {
    pub offline_first: bool,
    pub remote_import_map_allowed: bool,
    pub cloud_ai_runtime_allowed: bool,
    pub python_manifest_configured: bool,
    pub python_home_configured: bool,
    pub python_path_configured: bool,
    pub route: &'static str,
    pub notes: Vec<&'static str>,
}

fn env_path_exists(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .filter(|value| !value.is_empty())
        .map(|value| Path::new(&value).exists())
        .unwrap_or(false)
}

pub fn hydrate_bundled_python_env(app: &AppHandle) -> Result<(), String> {
    let Some(manifest_path) = app
        .path()
        .resolve(BUNDLED_RUNTIME_MANIFEST_RESOURCE, BaseDirectory::Resource)
        .ok()
        .filter(|path| path.exists())
    else {
        return Ok(());
    };

    let raw = std::fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;
    let manifest =
        serde_json::from_str::<PythonRuntimeManifest>(&raw).map_err(|error| error.to_string())?;
    let manifest_root = manifest_path
        .parent()
        .ok_or_else(|| "Bundled Python manifest has no parent directory".to_string())?;

    std::env::set_var(
        PYTHON_RUNTIME_MANIFEST_ENV,
        manifest_path.to_string_lossy().to_string(),
    );

    if let Some(relative_home) = manifest.bundled_python_relative_home.as_deref() {
        std::env::set_var(
            PYTHON_HOME_ENV,
            manifest_root
                .join(relative_home)
                .to_string_lossy()
                .to_string(),
        );
    } else if let Some(home) = manifest.bundled_python_home.as_deref() {
        std::env::set_var(PYTHON_HOME_ENV, home);
    }

    if let Some(relative_python) = manifest.bundled_python_relative_path.as_deref() {
        std::env::set_var(
            PYTHON_PATH_ENV,
            manifest_root
                .join(relative_python)
                .to_string_lossy()
                .to_string(),
        );
    } else if let Some(python_path) = manifest.bundled_python_path.as_deref() {
        std::env::set_var(PYTHON_PATH_ENV, python_path);
    }

    Ok(())
}

#[tauri::command]
pub fn get_offline_runtime_status() -> OfflineRuntimeStatus {
    OfflineRuntimeStatus {
        offline_first: true,
        remote_import_map_allowed: false,
        cloud_ai_runtime_allowed: false,
        python_manifest_configured: env_path_exists(PYTHON_RUNTIME_MANIFEST_ENV),
        python_home_configured: env_path_exists(PYTHON_HOME_ENV),
        python_path_configured: env_path_exists(PYTHON_PATH_ENV),
        route: "workspace/backend/offline-runtime/status",
        notes: vec![
            "Runtime dependencies must be bundled by Vite or Tauri resources, not remote CDN import maps.",
            "Cloud AI SDKs are not allowed in the desktop CAD runtime; segmentation and inference must use local Python or ONNX/TorchScript paths.",
            "Bundled Python is hydrated from the Tauri resource manifest when available.",
        ],
    }
}
