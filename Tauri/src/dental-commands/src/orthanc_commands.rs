//! Tauri commands for Orthanc PACS server

use crate::DentalCommandError;
use dental_imaging::orthanc_client::{
    OrthancClient, OrthancConfig, OrthancInstance, OrthancPatient,
    OrthancSeries, OrthancStatistics, OrthancStudy, OrthancSystemInfo,
    UploadResult,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Helper to build client from config params
fn make_client(
    base_url: &str,
    username: Option<String>,
    password: Option<String>,
) -> Result<OrthancClient, DentalCommandError> {
    let config = OrthancConfig {
        base_url: base_url.to_string(),
        username,
        password,
        timeout_secs: 30,
    };
    OrthancClient::new(config).map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── System ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_system_info(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<OrthancSystemInfo, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .system_info()
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_statistics(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<OrthancStatistics, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .statistics()
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── Patients ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_list_patients(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<Vec<String>, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .list_patients()
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_get_patient(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    patient_id: String,
) -> Result<OrthancPatient, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .get_patient(&patient_id)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── Studies ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_list_studies(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<Vec<String>, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .list_studies()
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_get_study(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    study_id: String,
) -> Result<OrthancStudy, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .get_study(&study_id)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── Series ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_list_series(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<Vec<String>, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .list_series()
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_get_series(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    series_id: String,
) -> Result<OrthancSeries, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .get_series(&series_id)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── Instances ────────────────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_list_instances(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<Vec<String>, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .list_instances()
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_get_instance(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    instance_id: String,
) -> Result<OrthancInstance, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .get_instance(&instance_id)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_download_instance(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    instance_id: String,
    output_path: String,
) -> Result<String, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    let path = client
        .download_instance_to_file(&instance_id, &PathBuf::from(&output_path))
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;
    Ok(path.to_string_lossy().to_string())
}

// ── Upload ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_upload_dicom(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    file_path: String,
) -> Result<UploadResult, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .upload_dicom(&PathBuf::from(&file_path))
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_upload_dicom_batch(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    file_paths: Vec<String>,
) -> Result<Vec<UploadResult>, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    let paths: Vec<PathBuf> = file_paths.into_iter().map(PathBuf::from).collect();
    client
        .upload_dicom_batch(&paths)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── Modality Operations ─────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_modality_echo(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    modality_name: String,
) -> Result<bool, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .modality_echo(&modality_name)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_modality_store(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    modality_name: String,
    resource_ids: Vec<String>,
) -> Result<serde_json::Value, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .modality_store(&modality_name, &resource_ids)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── Tools ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn orthanc_find(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    level: String,
    query: HashMap<String, String>,
    expand: bool,
) -> Result<Vec<serde_json::Value>, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .tools_find(&level, &query, expand)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_anonymize_study(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    study_id: String,
) -> Result<serde_json::Value, DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .anonymize_study(&study_id)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn orthanc_delete_resource(
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    resource_type: String,
    resource_id: String,
) -> Result<(), DentalCommandError> {
    let client = make_client(&base_url, username, password)?;
    client
        .delete_resource(&resource_type, &resource_id)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}
