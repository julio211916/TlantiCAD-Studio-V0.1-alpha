// V145 — Custom abutment preset save/load.
//
// The exocad workflow lets the lab tech "Save custom design…" from the
// Abutment Top tab. We persist each preset as a JSON file under the OS
// app-data directory so the user can share folders between machines.
//
// Path convention:
//   <app_data>/TlantiCAD/abutment-presets/<slug>.json

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentPreset {
    pub id: String,
    pub label: String,
    pub style: String,
    pub shoulder_size_mm: f64,
    pub roundness: f64,
    pub minimum_angle_deg: f64,
    /// Free-form metadata (e.g. originating dentist, tooth-number bias).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PresetError {
    #[error("could not resolve app-data directory")]
    AppDataUnavailable,
    #[error("io: {message}")]
    Io { message: String },
    #[error("invalid preset id: must be slug-like (letters, digits, dash, underscore)")]
    InvalidId,
    #[error("preset not found: {id}")]
    NotFound { id: String },
}

fn presets_dir(app: &AppHandle) -> Result<PathBuf, PresetError> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|_| PresetError::AppDataUnavailable)?;
    let dir = base.join("TlantiCAD").join("abutment-presets");
    fs::create_dir_all(&dir).map_err(|e| PresetError::Io {
        message: e.to_string(),
    })?;
    Ok(dir)
}

fn validate_id(id: &str) -> Result<(), PresetError> {
    if id.is_empty() {
        return Err(PresetError::InvalidId);
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(PresetError::InvalidId);
    }
    Ok(())
}

#[tauri::command]
pub fn abutment_preset_save(
    app: AppHandle,
    preset: AbutmentPreset,
) -> Result<PathBuf, PresetError> {
    validate_id(&preset.id)?;
    let dir = presets_dir(&app)?;
    let path = dir.join(format!("{}.json", preset.id));
    let json = serde_json::to_string_pretty(&preset).map_err(|e| PresetError::Io {
        message: e.to_string(),
    })?;
    fs::write(&path, json).map_err(|e| PresetError::Io {
        message: e.to_string(),
    })?;
    Ok(path)
}

#[tauri::command]
pub fn abutment_preset_list(app: AppHandle) -> Result<Vec<AbutmentPreset>, PresetError> {
    let dir = presets_dir(&app)?;
    let mut out: Vec<AbutmentPreset> = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| PresetError::Io {
        message: e.to_string(),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(preset) = serde_json::from_str::<AbutmentPreset>(&text) {
                out.push(preset);
            }
        }
    }
    out.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    Ok(out)
}

#[tauri::command]
pub fn abutment_preset_delete(app: AppHandle, id: String) -> Result<(), PresetError> {
    validate_id(&id)?;
    let dir = presets_dir(&app)?;
    let path = dir.join(format!("{}.json", id));
    if !path.exists() {
        return Err(PresetError::NotFound { id });
    }
    fs::remove_file(&path).map_err(|e| PresetError::Io {
        message: e.to_string(),
    })?;
    Ok(())
}

#[tauri::command]
pub fn abutment_preset_open_folder(app: AppHandle) -> Result<PathBuf, PresetError> {
    let dir = presets_dir(&app)?;
    Ok(dir)
}
