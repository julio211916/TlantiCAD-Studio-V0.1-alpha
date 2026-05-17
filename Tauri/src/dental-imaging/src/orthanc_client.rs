//! Orthanc REST API client
//!
//! Provides a high-level Rust client for interacting with an Orthanc DICOM PACS:
//! - System info & stats
//! - Patient / Study / Series / Instance CRUD
//! - Upload DICOM
//! - Download DICOM / archives
//! - Modality management (peers, C-ECHO, C-STORE, C-FIND, C-MOVE)
//! - Tools: anonymize, modify, find

use crate::error::ImagingError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Configuration ────────────────────────────────────────────────

/// Orthanc server connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancConfig {
    /// Base URL, e.g. "http://localhost:8042"
    pub base_url: String,
    /// Username for HTTP auth
    pub username: Option<String>,
    /// Password for HTTP auth
    pub password: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for OrthancConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8042".into(),
            username: Some("orthanc".into()),
            password: Some("orthanc".into()),
            timeout_secs: 30,
        }
    }
}

// ── Types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancSystemInfo {
    pub api_version: String,
    pub database_backend_plugin: Option<String>,
    pub dicom_aet: String,
    pub dicom_port: u16,
    pub is_http_server_secure: bool,
    pub name: String,
    pub plugins_enabled: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancStatistics {
    pub count_instances: u64,
    pub count_patients: u64,
    pub count_series: u64,
    pub count_studies: u64,
    pub total_disk_size_mb: f64,
    pub total_uncompressed_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancPatient {
    pub id: String,
    pub main_dicom_tags: PatientDicomTags,
    pub studies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PatientDicomTags {
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub patient_birth_date: Option<String>,
    pub patient_sex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancStudy {
    pub id: String,
    pub patient_id: String,
    pub main_dicom_tags: StudyDicomTags,
    pub series: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StudyDicomTags {
    pub study_instance_uid: Option<String>,
    pub study_date: Option<String>,
    pub study_description: Option<String>,
    pub accession_number: Option<String>,
    pub referring_physician_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancSeries {
    pub id: String,
    pub study_id: String,
    pub main_dicom_tags: SeriesDicomTags,
    pub instances: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SeriesDicomTags {
    pub series_instance_uid: Option<String>,
    pub modality: Option<String>,
    pub series_description: Option<String>,
    pub series_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancInstance {
    pub id: String,
    pub series_id: String,
    pub main_dicom_tags: InstanceDicomTags,
    pub file_size: Option<u64>,
    pub file_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InstanceDicomTags {
    pub sop_instance_uid: Option<String>,
    pub sop_class_uid: Option<String>,
    pub instance_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthancModality {
    pub name: String,
    pub aet: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub id: String,
    pub path: String,
    pub status: String,
    pub parent_patient: Option<String>,
    pub parent_study: Option<String>,
    pub parent_series: Option<String>,
}

// ── Client ───────────────────────────────────────────────────────

/// Orthanc REST API client
pub struct OrthancClient {
    config: OrthancConfig,
    http: reqwest::Client,
}

impl OrthancClient {
    /// Create a new Orthanc client from config
    pub fn new(config: OrthancConfig) -> Result<Self, ImagingError> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| ImagingError::OrthancError(format!("HTTP client: {}", e)))?;

        Ok(Self { config, http })
    }

    fn req(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http.request(method, &url);
        if let (Some(u), Some(p)) = (&self.config.username, &self.config.password) {
            req = req.basic_auth(u, Some(p));
        }
        req
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, ImagingError> {
        let resp = self
            .req(reqwest::Method::GET, path)
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("GET {}: {}", path, e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "GET {} returned {}",
                path,
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Parse {}: {}", path, e)))
    }

    // ── System ───────────────────────────────────────────────────

    /// Get Orthanc system information
    pub async fn system_info(&self) -> Result<OrthancSystemInfo, ImagingError> {
        self.get_json("/system").await
    }

    /// Get Orthanc statistics
    pub async fn statistics(&self) -> Result<OrthancStatistics, ImagingError> {
        self.get_json("/statistics").await
    }

    // ── Patients ─────────────────────────────────────────────────

    /// List all patient IDs
    pub async fn list_patients(&self) -> Result<Vec<String>, ImagingError> {
        self.get_json("/patients").await
    }

    /// Get patient details
    pub async fn get_patient(&self, id: &str) -> Result<OrthancPatient, ImagingError> {
        self.get_json(&format!("/patients/{}", id)).await
    }

    // ── Studies ──────────────────────────────────────────────────

    /// List all study IDs
    pub async fn list_studies(&self) -> Result<Vec<String>, ImagingError> {
        self.get_json("/studies").await
    }

    /// Get study details
    pub async fn get_study(&self, id: &str) -> Result<OrthancStudy, ImagingError> {
        self.get_json(&format!("/studies/{}", id)).await
    }

    // ── Series ───────────────────────────────────────────────────

    /// List all series IDs
    pub async fn list_series(&self) -> Result<Vec<String>, ImagingError> {
        self.get_json("/series").await
    }

    /// Get series details
    pub async fn get_series(&self, id: &str) -> Result<OrthancSeries, ImagingError> {
        self.get_json(&format!("/series/{}", id)).await
    }

    // ── Instances ────────────────────────────────────────────────

    /// List all instance IDs
    pub async fn list_instances(&self) -> Result<Vec<String>, ImagingError> {
        self.get_json("/instances").await
    }

    /// Get instance details
    pub async fn get_instance(&self, id: &str) -> Result<OrthancInstance, ImagingError> {
        self.get_json(&format!("/instances/{}", id)).await
    }

    /// Download a DICOM instance as bytes
    pub async fn download_instance(&self, id: &str) -> Result<Vec<u8>, ImagingError> {
        let resp = self
            .req(reqwest::Method::GET, &format!("/instances/{}/file", id))
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Download instance: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Download instance {} returned {}",
                id,
                resp.status()
            )));
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| ImagingError::OrthancError(format!("Download bytes: {}", e)))
    }

    /// Download instance to a local file
    pub async fn download_instance_to_file(
        &self,
        id: &str,
        output_path: &Path,
    ) -> Result<PathBuf, ImagingError> {
        let bytes = self.download_instance(id).await?;
        if let Some(p) = output_path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(output_path, &bytes)?;
        Ok(output_path.to_path_buf())
    }

    /// Download a study archive (ZIP)
    pub async fn download_study_archive(
        &self,
        study_id: &str,
        output_path: &Path,
    ) -> Result<PathBuf, ImagingError> {
        let resp = self
            .req(
                reqwest::Method::GET,
                &format!("/studies/{}/archive", study_id),
            )
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Download archive: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Download archive {} returned {}",
                study_id,
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Archive bytes: {}", e)))?;

        if let Some(p) = output_path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(output_path, &bytes)?;
        Ok(output_path.to_path_buf())
    }

    // ── Upload ───────────────────────────────────────────────────

    /// Upload a DICOM file to Orthanc
    pub async fn upload_dicom(&self, file_path: &Path) -> Result<UploadResult, ImagingError> {
        let data = std::fs::read(file_path)?;

        let resp = self
            .req(reqwest::Method::POST, "/instances")
            .header("Content-Type", "application/dicom")
            .body(data)
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Upload: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Upload failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Upload parse: {}", e)))
    }

    /// Upload multiple DICOM files
    pub async fn upload_dicom_batch(
        &self,
        paths: &[PathBuf],
    ) -> Result<Vec<UploadResult>, ImagingError> {
        let mut results = Vec::new();
        for p in paths {
            results.push(self.upload_dicom(p).await?);
        }
        Ok(results)
    }

    // ── Modalities ───────────────────────────────────────────────

    /// List configured DICOM modalities
    pub async fn list_modalities(
        &self,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, ImagingError> {
        self.get_json("/modalities").await
    }

    /// Send a C-ECHO to a configured modality
    pub async fn modality_echo(&self, modality_name: &str) -> Result<bool, ImagingError> {
        let resp = self
            .req(
                reqwest::Method::POST,
                &format!("/modalities/{}/echo", modality_name),
            )
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Modality echo: {}", e)))?;

        Ok(resp.status().is_success())
    }

    /// Send a C-STORE to a configured modality
    pub async fn modality_store(
        &self,
        modality_name: &str,
        resource_ids: &[String],
    ) -> Result<serde_json::Value, ImagingError> {
        let body = serde_json::json!({
            "Resources": resource_ids
        });

        let resp = self
            .req(
                reqwest::Method::POST,
                &format!("/modalities/{}/store", modality_name),
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Modality store: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Modality store to {} returned {}",
                modality_name,
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Modality store parse: {}", e)))
    }

    /// Perform a C-FIND query on a configured modality
    pub async fn modality_find(
        &self,
        modality_name: &str,
        level: &str,
        query: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<serde_json::Value>, ImagingError> {
        let body = serde_json::json!({
            "Level": level,
            "Query": query
        });

        let resp = self
            .req(
                reqwest::Method::POST,
                &format!("/modalities/{}/query", modality_name),
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Modality find: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Modality find on {} returned {}",
                modality_name,
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Modality find parse: {}", e)))
    }

    // ── Tools ────────────────────────────────────────────────────

    /// Anonymize a study
    pub async fn anonymize_study(
        &self,
        study_id: &str,
    ) -> Result<serde_json::Value, ImagingError> {
        let resp = self
            .req(
                reqwest::Method::POST,
                &format!("/studies/{}/anonymize", study_id),
            )
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Anonymize: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Anonymize {} returned {}",
                study_id,
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Anonymize parse: {}", e)))
    }

    /// Delete a resource (patient, study, series, or instance)
    pub async fn delete_resource(
        &self,
        resource_type: &str,
        id: &str,
    ) -> Result<(), ImagingError> {
        let resp = self
            .req(
                reqwest::Method::DELETE,
                &format!("/{}/{}", resource_type, id),
            )
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Delete: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Delete {}/{} returned {}",
                resource_type,
                id,
                resp.status()
            )));
        }

        Ok(())
    }

    /// Search for patients/studies by filter
    pub async fn tools_find(
        &self,
        level: &str,
        query: &std::collections::HashMap<String, String>,
        expand: bool,
    ) -> Result<Vec<serde_json::Value>, ImagingError> {
        let body = serde_json::json!({
            "Level": level,
            "Query": query,
            "Expand": expand,
        });

        let resp = self
            .req(reqwest::Method::POST, "/tools/find")
            .json(&body)
            .send()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Tools find: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ImagingError::OrthancError(format!(
                "Tools find returned {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ImagingError::OrthancError(format!("Tools find parse: {}", e)))
    }
}
