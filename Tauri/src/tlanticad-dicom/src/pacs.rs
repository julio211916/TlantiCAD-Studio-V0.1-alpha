//! PACS (Picture Archiving and Communication System) connectivity

use serde::{Deserialize, Serialize};

/// PACS server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacsConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub ae_title: String,
    pub calling_ae: String,
    pub protocol: PacsProtocol,
    pub use_tls: bool,
}

impl Default for PacsConfig {
    fn default() -> Self {
        Self {
            name: "Local PACS".to_string(),
            host: "127.0.0.1".to_string(),
            port: 4242,
            ae_title: "DCMSERVER".to_string(),
            calling_ae: "TLANTICAD".to_string(),
            protocol: PacsProtocol::Dimse,
            use_tls: false,
        }
    }
}

/// PACS communication protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacsProtocol {
    /// Traditional DICOM network protocol
    Dimse,
    /// WADO-RS REST API (DICOMweb)
    WadoRs,
    /// STOW-RS for storing images
    StowRs,
}

/// WADO-RS query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WadoQuery {
    pub study_uid: Option<String>,
    pub series_uid: Option<String>,
    pub instance_uid: Option<String>,
    pub patient_id: Option<String>,
    pub modality: Option<String>,
}

impl WadoQuery {
    pub fn for_study(uid: impl Into<String>) -> Self {
        Self { study_uid: Some(uid.into()), series_uid: None, instance_uid: None, patient_id: None, modality: None }
    }

    pub fn for_patient(patient_id: impl Into<String>) -> Self {
        Self { study_uid: None, series_uid: None, instance_uid: None, patient_id: Some(patient_id.into()), modality: None }
    }

    /// Build WADO-RS URL for study retrieval
    pub fn build_wado_url(&self, base_url: &str) -> String {
        let mut url = format!("{}/wado/rs", base_url.trim_end_matches('/'));
        if let Some(ref study) = self.study_uid {
            url.push_str(&format!("/studies/{}", study));
        }
        if let Some(ref series) = self.series_uid {
            url.push_str(&format!("/series/{}", series));
        }
        if let Some(ref instance) = self.instance_uid {
            url.push_str(&format!("/instances/{}", instance));
        }
        url
    }
}

/// PACS connection status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacsStatus {
    Connected,
    Disconnected,
    Authenticating,
    Error(String),
}

/// Minimal PACS client abstraction (actual network calls via reqwest in production)
pub struct PacsClient {
    pub config: PacsConfig,
    pub status: PacsStatus,
}

impl PacsClient {
    pub fn new(config: PacsConfig) -> Self {
        Self { config, status: PacsStatus::Disconnected }
    }

    pub fn base_url(&self) -> String {
        let scheme = if self.config.use_tls { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.config.host, self.config.port)
    }

    pub fn wado_url(&self, query: &WadoQuery) -> String {
        query.build_wado_url(&self.base_url())
    }

    /// C-ECHO check (ping)
    pub fn ping_url(&self) -> String {
        format!("{}/wado/rs/studies", self.base_url())
    }
}
