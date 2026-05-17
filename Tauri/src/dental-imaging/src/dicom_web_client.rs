//! DICOMweb REST client — WADO-RS, STOW-RS, QIDO-RS
//!
//! Implements the DICOMweb standard for browser-friendly DICOM access:
//! - **QIDO-RS**: Query based on ID for DICOM Objects (search studies/series/instances)
//! - **WADO-RS**: Web Access to DICOM Objects — Retrieve (download instances & metadata)
//! - **STOW-RS**: Store Over the Web — sending DICOM datasets via HTTP multipart

use crate::error::ImagingError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Configuration ────────────────────────────────────────────────

/// DICOMweb server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomWebConfig {
    /// Base URL, e.g. "http://localhost:8042/dicom-web"
    pub base_url: String,
    /// Optional Bearer token or Basic auth
    pub auth: Option<DicomWebAuth>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DicomWebAuth {
    Bearer { token: String },
    Basic { username: String, password: String },
}

impl Default for DicomWebConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8042/dicom-web".into(),
            auth: None,
            timeout_secs: 30,
        }
    }
}

// ── Types ────────────────────────────────────────────────────────

/// QIDO-RS study result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomWebStudy {
    pub study_instance_uid: String,
    pub patient_name: Option<String>,
    pub patient_id: Option<String>,
    pub study_date: Option<String>,
    pub study_description: Option<String>,
    pub modalities: Vec<String>,
    pub number_of_series: Option<u32>,
    pub number_of_instances: Option<u32>,
}

/// QIDO-RS series result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomWebSeries {
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub modality: Option<String>,
    pub series_description: Option<String>,
    pub series_number: Option<u32>,
    pub number_of_instances: Option<u32>,
}

/// QIDO-RS instance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomWebInstance {
    pub sop_instance_uid: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub sop_class_uid: Option<String>,
    pub instance_number: Option<u32>,
}

/// STOW-RS result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StowResult {
    pub stored_count: usize,
    pub failed_count: usize,
    pub referenced_uids: Vec<String>,
}

// ── Client ───────────────────────────────────────────────────────

/// DICOMweb REST client
pub struct DicomWebClient {
    config: DicomWebConfig,
    http: reqwest::Client,
}

impl DicomWebClient {
    /// Create a new DICOMweb client
    pub fn new(config: DicomWebConfig) -> Result<Self, ImagingError> {
        let builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs));

        let http = builder
            .build()
            .map_err(|e| ImagingError::Network(format!("HTTP client init: {}", e)))?;

        Ok(Self { config, http })
    }

    /// Build a request with auth headers
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http.request(method, &url);

        if let Some(ref auth) = self.config.auth {
            req = match auth {
                DicomWebAuth::Bearer { token } => req.bearer_auth(token),
                DicomWebAuth::Basic { username, password } => {
                    req.basic_auth(username, Some(password))
                }
            };
        }

        req
    }

    // ── QIDO-RS: Search ──────────────────────────────────────────

    /// Search for studies matching given parameters
    pub async fn search_studies(
        &self,
        params: &[(&str, &str)],
    ) -> Result<Vec<DicomWebStudy>, ImagingError> {
        let resp = self
            .request(reqwest::Method::GET, "/studies")
            .query(params)
            .header("Accept", "application/dicom+json")
            .send()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("QIDO studies: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::DicomWebError(format!(
                "QIDO studies returned {}",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("Parse studies: {}", e)))?;

        let studies = parse_studies_response(&body);
        Ok(studies)
    }

    /// Search for series within a study
    pub async fn search_series(
        &self,
        study_uid: &str,
        params: &[(&str, &str)],
    ) -> Result<Vec<DicomWebSeries>, ImagingError> {
        let path = format!("/studies/{}/series", study_uid);
        let resp = self
            .request(reqwest::Method::GET, &path)
            .query(params)
            .header("Accept", "application/dicom+json")
            .send()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("QIDO series: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::DicomWebError(format!(
                "QIDO series returned {}",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("Parse series: {}", e)))?;

        let series = parse_series_response(&body);
        Ok(series)
    }

    // ── WADO-RS: Retrieve ────────────────────────────────────────

    /// Download a DICOM instance as bytes
    pub async fn retrieve_instance(
        &self,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Result<Vec<u8>, ImagingError> {
        let path = format!(
            "/studies/{}/series/{}/instances/{}",
            study_uid, series_uid, instance_uid
        );

        let resp = self
            .request(reqwest::Method::GET, &path)
            .header("Accept", "application/dicom")
            .send()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("WADO retrieve: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::DicomWebError(format!(
                "WADO retrieve returned {}",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("WADO bytes: {}", e)))?;

        Ok(bytes.to_vec())
    }

    /// Download an instance and save it to a file
    pub async fn retrieve_instance_to_file(
        &self,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
        output_path: &Path,
    ) -> Result<PathBuf, ImagingError> {
        let bytes = self
            .retrieve_instance(study_uid, series_uid, instance_uid)
            .await?;

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, &bytes)?;
        Ok(output_path.to_path_buf())
    }

    /// Retrieve instance metadata (DICOM JSON)
    pub async fn retrieve_metadata(
        &self,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Result<serde_json::Value, ImagingError> {
        let path = format!(
            "/studies/{}/series/{}/instances/{}/metadata",
            study_uid, series_uid, instance_uid
        );

        let resp = self
            .request(reqwest::Method::GET, &path)
            .header("Accept", "application/dicom+json")
            .send()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("WADO metadata: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::DicomWebError(format!(
                "WADO metadata returned {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("Parse metadata: {}", e)))
    }

    /// Retrieve a rendered frame as PNG (WADO-RS rendered)
    pub async fn retrieve_rendered_frame(
        &self,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
        frame: u32,
    ) -> Result<Vec<u8>, ImagingError> {
        let path = format!(
            "/studies/{}/series/{}/instances/{}/frames/{}/rendered",
            study_uid, series_uid, instance_uid, frame
        );

        let resp = self
            .request(reqwest::Method::GET, &path)
            .header("Accept", "image/png")
            .send()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("WADO rendered: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::DicomWebError(format!(
                "WADO rendered returned {}",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ImagingError::DicomWebError(format!("WADO rendered bytes: {}", e)))?;

        Ok(bytes.to_vec())
    }

    // ── STOW-RS: Store ───────────────────────────────────────────

    /// Upload DICOM files via STOW-RS
    pub async fn store_instances(
        &self,
        file_paths: &[PathBuf],
    ) -> Result<StowResult, ImagingError> {
        let mut stored = 0usize;
        let mut failed = 0usize;
        let mut uids = Vec::new();

        for path in file_paths {
            if !path.exists() {
                failed += 1;
                continue;
            }

            let data = std::fs::read(path)?;
            let boundary = format!("----dicomweb-{}", uuid::Uuid::new_v4());

            let body = build_multipart_body(&boundary, &data);

            let resp = self
                .request(reqwest::Method::POST, "/studies")
                .header(
                    "Content-Type",
                    format!("multipart/related; type=\"application/dicom\"; boundary={}", boundary),
                )
                .body(body)
                .send()
                .await
                .map_err(|e| ImagingError::DicomWebError(format!("STOW: {}", e)))?;

            if resp.status().is_success() {
                stored += 1;
                uids.push(path.to_string_lossy().to_string());
            } else {
                failed += 1;
            }
        }

        Ok(StowResult {
            stored_count: stored,
            failed_count: failed,
            referenced_uids: uids,
        })
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn build_multipart_body(boundary: &str, dicom_data: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/dicom\r\n\r\n");
    body.extend_from_slice(dicom_data);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
    body
}

fn parse_studies_response(json: &serde_json::Value) -> Vec<DicomWebStudy> {
    let arr = match json.as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    arr.iter()
        .map(|item| DicomWebStudy {
            study_instance_uid: dicom_json_string(item, "0020000D"),
            patient_name: dicom_json_opt_string(item, "00100010"),
            patient_id: dicom_json_opt_string(item, "00100020"),
            study_date: dicom_json_opt_string(item, "00080020"),
            study_description: dicom_json_opt_string(item, "00081030"),
            modalities: dicom_json_strings(item, "00080061"),
            number_of_series: dicom_json_opt_u32(item, "00201206"),
            number_of_instances: dicom_json_opt_u32(item, "00201208"),
        })
        .collect()
}

fn parse_series_response(json: &serde_json::Value) -> Vec<DicomWebSeries> {
    let arr = match json.as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    arr.iter()
        .map(|item| DicomWebSeries {
            series_instance_uid: dicom_json_string(item, "0020000E"),
            study_instance_uid: dicom_json_string(item, "0020000D"),
            modality: dicom_json_opt_string(item, "00080060"),
            series_description: dicom_json_opt_string(item, "0008103E"),
            series_number: dicom_json_opt_u32(item, "00200011"),
            number_of_instances: dicom_json_opt_u32(item, "00201209"),
        })
        .collect()
}

/// Extract a string value from DICOM JSON format: { "TAG": { "vr": "XX", "Value": ["val"] } }
fn dicom_json_string(item: &serde_json::Value, tag: &str) -> String {
    item.get(tag)
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| {
            // Can be a string or an object with Alphabetic
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else {
                v.get("Alphabetic").and_then(|a| a.as_str()).map(|s| s.to_string())
            }
        })
        .unwrap_or_default()
}

fn dicom_json_opt_string(item: &serde_json::Value, tag: &str) -> Option<String> {
    let s = dicom_json_string(item, tag);
    if s.is_empty() { None } else { Some(s) }
}

fn dicom_json_strings(item: &serde_json::Value, tag: &str) -> Vec<String> {
    item.get(tag)
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn dicom_json_opt_u32(item: &serde_json::Value, tag: &str) -> Option<u32> {
    item.get(tag)
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
}
