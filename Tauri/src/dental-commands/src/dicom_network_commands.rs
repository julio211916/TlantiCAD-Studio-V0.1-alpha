//! Tauri commands for DICOM networking (SCP/SCU) and DICOMweb

use crate::DentalCommandError;
use dental_imaging::dicom_network::{
    DicomPeer, EchoResult, FindQuery, FindResult, QueryLevel,
    RetrieveResult, StoreResult,
};
use dental_imaging::dicom_web_client::{
    DicomWebClient, DicomWebConfig, DicomWebStudy, DicomWebSeries, StowResult,
};
use std::path::PathBuf;

// ── DICOM Network (DIMSE) Commands ──────────────────────────────

/// Ping a DICOM peer with C-ECHO
#[tauri::command]
pub async fn dicom_echo(
    calling_ae: String,
    called_ae: String,
    host: String,
    port: u16,
) -> Result<EchoResult, DentalCommandError> {
    let peer = DicomPeer {
        calling_ae,
        called_ae,
        host,
        port,
        max_pdu_size: 0,
    };
    dental_imaging::dicom_network::dicom_echo(&peer)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

/// C-FIND query on a DICOM peer
#[tauri::command]
pub async fn dicom_find(
    calling_ae: String,
    called_ae: String,
    host: String,
    port: u16,
    level: String,
    filters: Vec<(String, String)>,
) -> Result<Vec<FindResult>, DentalCommandError> {
    let peer = DicomPeer {
        calling_ae,
        called_ae,
        host,
        port,
        max_pdu_size: 0,
    };
    let ql = match level.to_uppercase().as_str() {
        "PATIENT" => QueryLevel::Patient,
        "STUDY" => QueryLevel::Study,
        "SERIES" => QueryLevel::Series,
        "IMAGE" => QueryLevel::Image,
        _ => return Err(DentalCommandError::Validation("level must be PATIENT|STUDY|SERIES|IMAGE".into())),
    };
    let query = FindQuery {
        level: ql,
        filters,
        max_results: 0,
    };
    dental_imaging::dicom_network::dicom_find(&peer, &query)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

/// C-STORE DICOM files to a remote peer
#[tauri::command]
pub async fn dicom_store_to_peer(
    calling_ae: String,
    called_ae: String,
    host: String,
    port: u16,
    file_paths: Vec<String>,
) -> Result<StoreResult, DentalCommandError> {
    let peer = DicomPeer {
        calling_ae,
        called_ae,
        host,
        port,
        max_pdu_size: 0,
    };
    let paths: Vec<PathBuf> = file_paths.into_iter().map(PathBuf::from).collect();
    dental_imaging::dicom_network::dicom_store(&peer, &paths)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

/// C-MOVE retrieve from a DICOM peer
#[tauri::command]
pub async fn dicom_move_retrieve(
    calling_ae: String,
    called_ae: String,
    host: String,
    port: u16,
    study_uid: String,
    destination_ae: String,
    output_dir: String,
) -> Result<RetrieveResult, DentalCommandError> {
    let peer = DicomPeer {
        calling_ae,
        called_ae,
        host,
        port,
        max_pdu_size: 0,
    };
    dental_imaging::dicom_network::dicom_move_retrieve(
        &peer,
        &study_uid,
        &destination_ae,
        &PathBuf::from(output_dir),
    )
    .await
    .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

/// List dental DICOM modalities
#[tauri::command]
pub async fn dicom_dental_modalities() -> Result<Vec<(String, String)>, DentalCommandError> {
    Ok(dental_imaging::dicom_network::dental_modalities()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

// ── DICOMweb REST Commands ──────────────────────────────────────

/// Search DICOM studies via DICOMweb QIDO-RS
#[tauri::command]
pub async fn dicomweb_search_studies(
    base_url: String,
    patient_name: Option<String>,
    patient_id: Option<String>,
    study_date: Option<String>,
    modality: Option<String>,
) -> Result<Vec<DicomWebStudy>, DentalCommandError> {
    let config = DicomWebConfig {
        base_url,
        auth: None,
        timeout_secs: 30,
    };
    let client = DicomWebClient::new(config)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

    let mut params: Vec<(&str, &str)> = Vec::new();
    let pn_str;
    if let Some(ref pn) = patient_name {
        pn_str = pn.clone();
        params.push(("PatientName", &pn_str));
    }
    let pid_str;
    if let Some(ref pid) = patient_id {
        pid_str = pid.clone();
        params.push(("PatientID", &pid_str));
    }
    let sd_str;
    if let Some(ref sd) = study_date {
        sd_str = sd.clone();
        params.push(("StudyDate", &sd_str));
    }
    let mod_str;
    if let Some(ref m) = modality {
        mod_str = m.clone();
        params.push(("ModalitiesInStudy", &mod_str));
    }

    client
        .search_studies(&params)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

/// Search series within a study via DICOMweb
#[tauri::command]
pub async fn dicomweb_search_series(
    base_url: String,
    study_uid: String,
    modality: Option<String>,
) -> Result<Vec<DicomWebSeries>, DentalCommandError> {
    let config = DicomWebConfig {
        base_url,
        auth: None,
        timeout_secs: 30,
    };
    let client = DicomWebClient::new(config)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

    let mut params: Vec<(&str, &str)> = Vec::new();
    let mod_str;
    if let Some(ref m) = modality {
        mod_str = m.clone();
        params.push(("Modality", &mod_str));
    }

    client
        .search_series(&study_uid, &params)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

/// Download a DICOM instance via DICOMweb WADO-RS
#[tauri::command]
pub async fn dicomweb_download_instance(
    base_url: String,
    study_uid: String,
    series_uid: String,
    instance_uid: String,
    output_path: String,
) -> Result<String, DentalCommandError> {
    let config = DicomWebConfig {
        base_url,
        auth: None,
        timeout_secs: 60,
    };
    let client = DicomWebClient::new(config)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

    let path = client
        .retrieve_instance_to_file(
            &study_uid,
            &series_uid,
            &instance_uid,
            &PathBuf::from(&output_path),
        )
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

    Ok(path.to_string_lossy().to_string())
}

/// Upload DICOM files via DICOMweb STOW-RS
#[tauri::command]
pub async fn dicomweb_store_instances(
    base_url: String,
    file_paths: Vec<String>,
) -> Result<StowResult, DentalCommandError> {
    let config = DicomWebConfig {
        base_url,
        auth: None,
        timeout_secs: 120,
    };
    let client = DicomWebClient::new(config)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

    let paths: Vec<PathBuf> = file_paths.into_iter().map(PathBuf::from).collect();

    client
        .store_instances(&paths)
        .await
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}
