use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

pub const TLANTICAD_DATA_DIR: &str = "TlantiCADData";
pub const TLANTICAD_DATABASE_NAME: &str = "tlanticad.sqlite";

#[derive(Debug, Clone)]
pub struct TlantiCadDataLayout {
    pub root: PathBuf,
    pub database_path: PathBuf,
    pub cases_dir: PathBuf,
    pub patients_index_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub models_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub backups_dir: PathBuf,
}

pub fn data_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join(TLANTICAD_DATA_DIR))
        .map_err(|error| format!("Could not resolve app data dir: {error}"))
}

pub fn ensure_data_layout(app: &AppHandle) -> Result<TlantiCadDataLayout, String> {
    let root = data_root(app)?;
    let database_dir = root.join("database");
    let database_path = database_dir.join(TLANTICAD_DATABASE_NAME);
    let cases_dir = root.join("cases");
    let patients_index_dir = root.join("patients").join("index");
    let libraries_dir = root.join("libraries");
    let models_dir = root.join("models");
    let exports_dir = root.join("exports");
    let cache_dir = root.join("cache");
    let temp_dir = root.join("temp");
    let logs_dir = root.join("logs");
    let backups_dir = root.join("backups");

    for directory in [
        &root,
        &database_dir,
        &cases_dir,
        &patients_index_dir,
        &libraries_dir,
        &libraries_dir.join("implants"),
        &libraries_dir.join("teeth"),
        &libraries_dir.join("materials"),
        &libraries_dir.join("articulators"),
        &libraries_dir.join("components"),
        &models_dir,
        &models_dir.join("segmentation"),
        &models_dir.join("alignment"),
        &models_dir.join("assistant"),
        &exports_dir,
        &exports_dir.join("stl"),
        &exports_dir.join("obj"),
        &exports_dir.join("ply"),
        &exports_dir.join("pdf"),
        &exports_dir.join("reports"),
        &cache_dir,
        &temp_dir,
        &logs_dir,
        &backups_dir,
    ] {
        fs::create_dir_all(directory)
            .map_err(|error| format!("Could not create {}: {error}", directory.display()))?;
    }

    Ok(TlantiCadDataLayout {
        root,
        database_path,
        cases_dir,
        patients_index_dir,
        libraries_dir,
        models_dir,
        exports_dir,
        cache_dir,
        temp_dir,
        logs_dir,
        backups_dir,
    })
}

pub fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    ensure_data_layout(app).map(|layout| layout.database_path)
}

pub fn fallback_case_root(app: &AppHandle, case_id: &str) -> Result<PathBuf, String> {
    Ok(ensure_data_layout(app)?.cases_dir.join(case_id))
}
