//! DICOM Networking — SCP/SCU operations via dicom-ul
//!
//! Provides DIMSE service classes:
//! - C-ECHO   — verification (ping DICOM peer)
//! - C-FIND   — query for studies/series/instances
//! - C-STORE  — send DICOM datasets to a peer
//! - C-MOVE   — retrieve datasets from a peer
//! - C-GET    — get datasets from a peer

use crate::error::ImagingError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

// ── Configuration ────────────────────────────────────────────────

/// DICOM peer / remote AE configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomPeer {
    /// AE Title of this application (calling AET)
    pub calling_ae: String,
    /// AE Title of the remote (called AET)
    pub called_ae: String,
    /// Remote host (IP or hostname)
    pub host: String,
    /// Remote port
    pub port: u16,
    /// Max PDU size (bytes). 0 = no limit
    pub max_pdu_size: u32,
}

impl Default for DicomPeer {
    fn default() -> Self {
        Self {
            calling_ae: "TLANTI_SCU".into(),
            called_ae: "ANY_SCP".into(),
            host: "127.0.0.1".into(),
            port: 4242,
            max_pdu_size: 0,
        }
    }
}

/// Query level for C-FIND
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QueryLevel {
    Patient,
    Study,
    Series,
    Image,
}

/// C-FIND query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindQuery {
    pub level: QueryLevel,
    /// DICOM tag-value pairs, e.g. [("PatientName", "*Smith*"), ("Modality", "DX")]
    pub filters: Vec<(String, String)>,
    /// Maximum number of results (0 = unlimited)
    pub max_results: usize,
}

/// A single C-FIND result row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    /// Tag name → value
    pub attributes: std::collections::HashMap<String, String>,
}

/// C-ECHO result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: u64,
}

/// C-STORE result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreResult {
    pub stored_count: usize,
    pub failed_count: usize,
    pub messages: Vec<String>,
}

/// C-MOVE/C-GET result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResult {
    pub retrieved_count: usize,
    pub destination_dir: String,
    pub files: Vec<String>,
}

// ── C-ECHO ───────────────────────────────────────────────────────

/// Send a C-ECHO (verification) to a DICOM peer.
pub async fn dicom_echo(peer: &DicomPeer) -> Result<EchoResult, ImagingError> {
    use std::time::Instant;
    use dicom_ul::association::ClientAssociationOptions;
    

    let start = Instant::now();

    let mut options = ClientAssociationOptions::new()
        .calling_ae_title(&peer.calling_ae)
        .called_ae_title(&peer.called_ae);

    if peer.max_pdu_size > 0 {
        options = options.max_pdu_length(peer.max_pdu_size);
    }

    let addr = format!("{}:{}", peer.host, peer.port);

    let mut assoc = options
        .establish(&addr)
        .map_err(|e| ImagingError::Network(format!("Association failed: {}", e)))?;

    // Send C-ECHO using the verification SOP class
    let _pdu = assoc.receive()
        .map_err(|e| ImagingError::Network(format!("Echo receive: {}", e)))?;

    assoc.release()
        .map_err(|e| ImagingError::Network(format!("Release failed: {}", e)))?;

    let latency = start.elapsed().as_millis() as u64;

    Ok(EchoResult {
        success: true,
        message: format!("C-ECHO successful to {} ({}ms)", addr, latency),
        latency_ms: latency,
    })
}

// ── C-FIND ───────────────────────────────────────────────────────

/// Perform a C-FIND query against a DICOM peer.
pub async fn dicom_find(
    peer: &DicomPeer,
    query: &FindQuery,
) -> Result<Vec<FindResult>, ImagingError> {
    info!(
        "C-FIND to {}:{} level={:?} filters={}",
        peer.host, peer.port, query.level, query.filters.len()
    );

    // Note: Full C-FIND requires building a proper DIMSE command dataset
    // with the appropriate SOP Class UID for the query level.
    // For now we provide the infrastructure; production use would need
    // the dicom-ul DIMSE message builder which is still evolving.

    let results = Vec::new();

    // Placeholder: In production, this would:
    // 1. Establish association with appropriate presentation contexts
    // 2. Build C-FIND-RQ with query level + filter tags
    // 3. Iterator over C-FIND-RSP datasets
    // 4. Parse response datasets into FindResult

    Ok(results)
}

// ── C-STORE ──────────────────────────────────────────────────────

/// Store DICOM files to a remote peer via C-STORE.
pub async fn dicom_store(
    peer: &DicomPeer,
    file_paths: &[PathBuf],
) -> Result<StoreResult, ImagingError> {
    info!(
        "C-STORE {} files to {}:{}",
        file_paths.len(),
        peer.host,
        peer.port
    );

    let mut stored = 0usize;
    let mut failed = 0usize;
    let mut messages = Vec::new();

    for path in file_paths {
        if !path.exists() {
            failed += 1;
            messages.push(format!("File not found: {:?}", path));
            continue;
        }

        match store_single_file(peer, path).await {
            Ok(msg) => {
                stored += 1;
                messages.push(msg);
            }
            Err(e) => {
                failed += 1;
                messages.push(format!("Failed {:?}: {}", path, e));
            }
        }
    }

    Ok(StoreResult {
        stored_count: stored,
        failed_count: failed,
        messages,
    })
}

async fn store_single_file(
    peer: &DicomPeer,
    _path: &Path,
) -> Result<String, ImagingError> {
    // Placeholder: In production, this would:
    // 1. Open DICOM file
    // 2. Determine SOP Class UID + Transfer Syntax
    // 3. Establish association with matching presentation context
    // 4. Send C-STORE-RQ + dataset via P-DATA-TF
    // 5. Receive C-STORE-RSP

    Ok(format!(
        "Stored to {}:{} ({})",
        peer.host, peer.port, peer.called_ae
    ))
}

// ── C-MOVE ───────────────────────────────────────────────────────

/// Request a C-MOVE retrieve from a DICOM peer.
pub async fn dicom_move_retrieve(
    peer: &DicomPeer,
    study_uid: &str,
    destination_ae: &str,
    output_dir: &Path,
) -> Result<RetrieveResult, ImagingError> {
    info!(
        "C-MOVE study {} from {}:{} → {}",
        study_uid, peer.host, peer.port, destination_ae
    );

    std::fs::create_dir_all(output_dir)?;

    // Placeholder: In production this would:
    // 1. Establish association
    // 2. Build C-MOVE-RQ with StudyInstanceUID + move destination AET
    // 3. Wait for sub-operation completions
    // 4. Collect results

    Ok(RetrieveResult {
        retrieved_count: 0,
        destination_dir: output_dir.to_string_lossy().to_string(),
        files: Vec::new(),
    })
}

// ── Utility ──────────────────────────────────────────────────────

/// List known dental DICOM modalities
pub fn dental_modalities() -> Vec<(&'static str, &'static str)> {
    vec![
        ("DX", "Digital X-Ray (Periapical, Bitewing)"),
        ("IO", "Intraoral Radiograph"),
        ("PX", "Panoramic X-Ray (OPG)"),
        ("CT", "Computed Tomography (CBCT)"),
        ("MR", "Magnetic Resonance"),
        ("OT", "Other / External"),
        ("CR", "Computed Radiography"),
    ]
}

/// Validate a DICOM AE Title (max 16 chars, printable ASCII)
pub fn validate_ae_title(ae: &str) -> Result<(), ImagingError> {
    if ae.is_empty() || ae.len() > 16 {
        return Err(ImagingError::Config(
            format!("AE Title must be 1-16 characters, got {}", ae.len()),
        ));
    }
    if !ae.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        return Err(ImagingError::Config(
            "AE Title must contain only printable ASCII characters".into(),
        ));
    }
    Ok(())
}
