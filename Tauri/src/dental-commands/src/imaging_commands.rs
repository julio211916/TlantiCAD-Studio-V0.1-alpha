//! Tauri commands for medical imaging (DICOM, STL, patient files)

use crate::DentalCommandError;
use dental_imaging::{dicom_viewer, gallery, stl_viewer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

// ── DICOM Commands ──────────────────────────────────────────────

#[tauri::command]
pub async fn dicom_parse(
    path: String,
) -> Result<dental_imaging::dicom_viewer::DicomStudy, DentalCommandError> {
    let p = PathBuf::from(&path);
    dicom_viewer::parse_dicom_metadata(&p).map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn dicom_get_image(
    path: String,
) -> Result<dental_imaging::dicom_viewer::DicomImage, DentalCommandError> {
    let p = PathBuf::from(&path);
    dicom_viewer::parse_dicom_image(&p).map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn dicom_list(
    directory: String,
) -> Result<Vec<dental_imaging::dicom_viewer::DicomStudy>, DentalCommandError> {
    let p = PathBuf::from(&directory);
    dicom_viewer::list_dicom_files(&p).map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── STL Commands ────────────────────────────────────────────────

#[tauri::command]
pub async fn stl_parse(
    path: String,
) -> Result<dental_imaging::stl_viewer::StlMesh, DentalCommandError> {
    let p = PathBuf::from(&path);
    stl_viewer::parse_stl(&p).map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn stl_get_info(
    path: String,
) -> Result<dental_imaging::stl_viewer::StlInfo, DentalCommandError> {
    let p = PathBuf::from(&path);
    stl_viewer::parse_stl_info(&p).map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn stl_list(
    directory: String,
) -> Result<Vec<dental_imaging::stl_viewer::StlInfo>, DentalCommandError> {
    let p = PathBuf::from(&directory);
    stl_viewer::list_stl_files(&p).map_err(|e| DentalCommandError::Internal(e.to_string()))
}

// ── Patient File Gallery Commands ───────────────────────────────

#[tauri::command]
pub async fn patient_files_list(
    patient_id: String,
    app: tauri::AppHandle,
) -> Result<Vec<gallery::PatientFile>, DentalCommandError> {
    let base_dir = app
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| DentalCommandError::Internal(e.to_string()))?;
    gallery::list_patient_files(&base_dir, &patient_id)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn patient_file_save(
    patient_id: String,
    file_name: String,
    data: Vec<u8>,
    app: tauri::AppHandle,
) -> Result<String, DentalCommandError> {
    let base_dir = app
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| DentalCommandError::Internal(e.to_string()))?;
    let path = gallery::save_patient_file(&base_dir, &patient_id, &file_name, &data)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;
    Ok(path.to_string_lossy().to_string())
}

// ── Odontogram Notation Commands ────────────────────────────────

use dental_core::models::odontogram::{
    universal_to_fdi, ToothId, ToothNumbers,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ToothNotationResult {
    pub fdi: i32,
    pub universal: String,
    pub palmer: String,
    pub name: String,
    pub name_es: String,
    pub quadrant: i32,
    pub quadrant_name: String,
    pub quadrant_name_es: String,
    pub is_deciduous: bool,
}

#[tauri::command]
pub async fn tooth_get_notation(
    fdi_number: i32,
) -> Result<ToothNotationResult, DentalCommandError> {
    let tooth = ToothId::from_fdi(fdi_number)
        .ok_or_else(|| DentalCommandError::Validation(format!("Invalid FDI number: {}", fdi_number)))?;

    Ok(ToothNotationResult {
        fdi: tooth.fdi,
        universal: tooth.universal,
        palmer: tooth.palmer,
        name: tooth.name,
        name_es: ToothNumbers::nombre(fdi_number).to_string(),
        quadrant: tooth.quadrant,
        quadrant_name: ToothNumbers::quadrant_name(tooth.quadrant).to_string(),
        quadrant_name_es: ToothNumbers::nombre_cuadrante(tooth.quadrant).to_string(),
        is_deciduous: tooth.is_deciduous,
    })
}

#[tauri::command]
pub async fn tooth_convert_notation(
    value: String,
    from_system: String,
) -> Result<ToothNotationResult, DentalCommandError> {
    let fdi = match from_system.as_str() {
        "universal" => universal_to_fdi(&value)
            .ok_or_else(|| DentalCommandError::Validation(format!("Invalid Universal number: {}", value)))?,
        "fdi" => value
            .parse::<i32>()
            .map_err(|_| DentalCommandError::Validation(format!("Invalid FDI: {}", value)))?,
        _ => return Err(DentalCommandError::Validation("Use 'fdi' or 'universal'".into())),
    };

    let tooth = ToothId::from_fdi(fdi)
        .ok_or_else(|| DentalCommandError::Validation(format!("Invalid tooth: {}", fdi)))?;

    Ok(ToothNotationResult {
        fdi: tooth.fdi,
        universal: tooth.universal,
        palmer: tooth.palmer,
        name: tooth.name,
        name_es: ToothNumbers::nombre(fdi).to_string(),
        quadrant: tooth.quadrant,
        quadrant_name: ToothNumbers::quadrant_name(tooth.quadrant).to_string(),
        quadrant_name_es: ToothNumbers::nombre_cuadrante(tooth.quadrant).to_string(),
        is_deciduous: tooth.is_deciduous,
    })
}

#[tauri::command]
pub async fn tooth_list_all(
    include_deciduous: bool,
) -> Result<Vec<ToothNotationResult>, DentalCommandError> {
    let mut results: Vec<ToothNotationResult> = ToothNumbers::all_permanent()
        .into_iter()
        .map(|t| ToothNotationResult {
            fdi: t.fdi,
            universal: t.universal,
            palmer: t.palmer,
            name: t.name.clone(),
            name_es: ToothNumbers::nombre(t.fdi).to_string(),
            quadrant: t.quadrant,
            quadrant_name: ToothNumbers::quadrant_name(t.quadrant).to_string(),
            quadrant_name_es: ToothNumbers::nombre_cuadrante(t.quadrant).to_string(),
            is_deciduous: false,
        })
        .collect();

    if include_deciduous {
        let deciduous: Vec<ToothNotationResult> = ToothNumbers::all_deciduous()
            .into_iter()
            .map(|t| ToothNotationResult {
                fdi: t.fdi,
                universal: t.universal,
                palmer: t.palmer,
                name: t.name.clone(),
                name_es: ToothNumbers::nombre(t.fdi).to_string(),
                quadrant: t.quadrant,
                quadrant_name: ToothNumbers::quadrant_name(t.quadrant).to_string(),
                quadrant_name_es: ToothNumbers::nombre_cuadrante(t.quadrant).to_string(),
                is_deciduous: true,
            })
            .collect();
        results.extend(deciduous);
    }

    Ok(results)
}
