//! DICOM study/series/instance management

use serde::{Deserialize, Serialize};
use crate::parser::{DicomMetadata, DicomModality};

/// DICOM Study — groups all series for a patient visit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStudy {
    pub study_instance_uid: String,
    pub patient_id: String,
    pub patient_name: String,
    pub study_date: Option<String>,
    pub study_description: String,
    pub accession_number: String,
    pub series: Vec<DicomSeries>,
}

impl DicomStudy {
    pub fn new(uid: impl Into<String>, patient_id: impl Into<String>) -> Self {
        Self {
            study_instance_uid: uid.into(),
            patient_id: patient_id.into(),
            patient_name: String::new(),
            study_date: None,
            study_description: String::new(),
            accession_number: String::new(),
            series: Vec::new(),
        }
    }

    pub fn cbct_series(&self) -> Vec<&DicomSeries> {
        self.series.iter().filter(|s| s.modality == DicomModality::CT).collect()
    }

    pub fn total_instances(&self) -> usize {
        self.series.iter().map(|s| s.instance_count).sum()
    }
}

/// DICOM Series — collection of images from one acquisition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomSeries {
    pub series_instance_uid: String,
    pub series_number: i32,
    pub modality: DicomModality,
    pub series_description: String,
    pub instance_count: usize,
    pub instances: Vec<DicomMetadata>,
}

impl DicomSeries {
    pub fn new(uid: impl Into<String>, modality: DicomModality) -> Self {
        Self {
            series_instance_uid: uid.into(),
            series_number: 1,
            modality,
            series_description: String::new(),
            instance_count: 0,
            instances: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, meta: DicomMetadata) {
        self.instance_count += 1;
        self.instances.push(meta);
    }

    pub fn is_cbct(&self) -> bool {
        self.modality == DicomModality::CT
    }

    /// Sort instances by instance number
    pub fn sort_instances(&mut self) {
        self.instances.sort_by_key(|m| m.instance_number);
    }
}
