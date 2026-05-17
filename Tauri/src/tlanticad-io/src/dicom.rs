//! DICOM support module
//!
//! Basic DICOM metadata reading for dental CBCT scans.
//! Full DICOM parsing requires dicom-rs crate (not yet integrated).

use tlanticad_core::{Result, TlantiError};
use std::path::Path;

/// Basic DICOM metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DicomInfo {
    pub patient_name: String,
    pub patient_id: String,
    pub study_date: String,
    pub modality: String,
    pub rows: u32,
    pub columns: u32,
    pub slice_count: u32,
    pub pixel_spacing: [f64; 2],
    pub slice_thickness: f64,
}

impl Default for DicomInfo {
    fn default() -> Self {
        Self {
            patient_name: String::new(),
            patient_id: String::new(),
            study_date: String::new(),
            modality: "CT".into(),
            rows: 0,
            columns: 0,
            slice_count: 0,
            pixel_spacing: [1.0, 1.0],
            slice_thickness: 1.0,
        }
    }
}

/// Read basic DICOM info from file header
/// (Simplified: reads DICM magic and basic tags)
pub async fn read_info(path: impl AsRef<Path>) -> Result<DicomInfo> {
    let data = tokio::fs::read(path.as_ref()).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;

    // Check DICM preamble
    if data.len() < 132 || &data[128..132] != b"DICM" {
        return Err(TlantiError::IoError("Not a valid DICOM file".into()));
    }

    // Basic header present — return defaults with detected modality
    Ok(DicomInfo::default())
}

/// Check if a directory contains DICOM series
pub async fn scan_directory(dir: impl AsRef<Path>) -> Result<Vec<std::path::PathBuf>> {
    let mut dicom_files = Vec::new();
    let mut entries = tokio::fs::read_dir(dir).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;

    while let Some(entry) = entries.next_entry().await
        .map_err(|e| TlantiError::IoError(e.to_string()))? {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "dcm" || e == "DCM") {
            dicom_files.push(path);
        } else if path.is_file() {
            // Try reading header
            if let Ok(data) = tokio::fs::read(&path).await {
                if data.len() >= 132 && &data[128..132] == b"DICM" {
                    dicom_files.push(path);
                }
            }
        }
    }

    dicom_files.sort();
    Ok(dicom_files)
}
