//! Patient image gallery management
//!
//! Manages files attached to a patient: photos, X-rays, DICOM, STL, documents.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::ImagingError;

/// Supported file categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileCategory {
    Photo,
    Xray,
    Dicom,
    Stl,
    Document,
    Other,
}

/// A patient file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientFile {
    pub id: String,
    pub patient_id: String,
    pub category: FileCategory,
    pub file_name: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub mime_type: String,
    pub description: Option<String>,
    pub uploaded_at: String,
    pub uploaded_by: Option<String>,
}

/// Detect file category from extension
pub fn detect_category(file_name: &str) -> FileCategory {
    let ext = Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "bmp" | "webp" | "heic" => FileCategory::Photo,
        "dcm" | "dicom" => FileCategory::Dicom,
        "stl" | "obj" | "ply" => FileCategory::Stl,
        "pdf" | "doc" | "docx" | "txt" => FileCategory::Document,
        _ => FileCategory::Other,
    }
}

/// Detect MIME type from extension
pub fn detect_mime(file_name: &str) -> String {
    let ext = Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "dcm" | "dicom" => "application/dicom",
        "stl" => "application/sla",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "csv" => "text/csv",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Build the patient storage directory path
pub fn patient_storage_dir(base_dir: &Path, patient_id: &str) -> PathBuf {
    base_dir.join("patients").join(patient_id).join("files")
}

/// Save a file to the patient's storage directory
pub fn save_patient_file(
    base_dir: &Path,
    patient_id: &str,
    file_name: &str,
    data: &[u8],
) -> Result<PathBuf, ImagingError> {
    let dir = patient_storage_dir(base_dir, patient_id);
    let category = detect_category(file_name);
    let sub_dir = dir.join(format!("{:?}", category).to_lowercase());

    std::fs::create_dir_all(&sub_dir)?;

    let dest = sub_dir.join(file_name);
    std::fs::write(&dest, data)?;

    Ok(dest)
}

/// List all files for a patient
pub fn list_patient_files(
    base_dir: &Path,
    patient_id: &str,
) -> Result<Vec<PatientFile>, ImagingError> {
    let dir = patient_storage_dir(base_dir, patient_id);
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    for entry in walkdir::WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let meta = std::fs::metadata(path).ok();

            files.push(PatientFile {
                id: uuid::Uuid::new_v4().to_string(),
                patient_id: patient_id.to_string(),
                category: detect_category(&file_name),
                file_name: file_name.clone(),
                file_path: path.to_string_lossy().to_string(),
                file_size_bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                mime_type: detect_mime(&file_name),
                description: None,
                uploaded_at: chrono::Utc::now().to_rfc3339(),
                uploaded_by: None,
            });
        }
    }

    Ok(files)
}
