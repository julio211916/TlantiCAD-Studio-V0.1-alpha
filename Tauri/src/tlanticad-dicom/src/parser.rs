//! DICOM tag parsing and metadata extraction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DICOM tag (group, element) pair
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DicomTag(pub u16, pub u16);

impl DicomTag {
    // Patient tags
    pub const PATIENT_NAME:         DicomTag = DicomTag(0x0010, 0x0010);
    pub const PATIENT_ID:           DicomTag = DicomTag(0x0010, 0x0020);
    pub const PATIENT_BIRTHDATE:    DicomTag = DicomTag(0x0010, 0x0030);
    pub const PATIENT_SEX:          DicomTag = DicomTag(0x0010, 0x0040);

    // Study tags
    pub const STUDY_INSTANCE_UID:   DicomTag = DicomTag(0x0020, 0x000D);
    pub const STUDY_DATE:           DicomTag = DicomTag(0x0008, 0x0020);
    pub const STUDY_DESCRIPTION:    DicomTag = DicomTag(0x0008, 0x1030);
    pub const ACCESSION_NUMBER:     DicomTag = DicomTag(0x0008, 0x0050);

    // Series tags
    pub const SERIES_INSTANCE_UID:  DicomTag = DicomTag(0x0020, 0x000E);
    pub const SERIES_NUMBER:        DicomTag = DicomTag(0x0020, 0x0011);
    pub const MODALITY:             DicomTag = DicomTag(0x0008, 0x0060);
    pub const SERIES_DESCRIPTION:   DicomTag = DicomTag(0x0008, 0x103E);

    // Image tags
    pub const SOP_INSTANCE_UID:     DicomTag = DicomTag(0x0008, 0x0018);
    pub const ROWS:                 DicomTag = DicomTag(0x0028, 0x0010);
    pub const COLUMNS:              DicomTag = DicomTag(0x0028, 0x0011);
    pub const PIXEL_SPACING:        DicomTag = DicomTag(0x0028, 0x0030);
    pub const SLICE_THICKNESS:      DicomTag = DicomTag(0x0050, 0x0050);
    pub const BITS_ALLOCATED:       DicomTag = DicomTag(0x0028, 0x0100);
    pub const BITS_STORED:          DicomTag = DicomTag(0x0028, 0x0101);
    pub const PIXEL_REPRESENTATION: DicomTag = DicomTag(0x0028, 0x0103);
    pub const RESCALE_INTERCEPT:    DicomTag = DicomTag(0x0028, 0x1052);
    pub const RESCALE_SLOPE:        DicomTag = DicomTag(0x0028, 0x1053);
    pub const IMAGE_POSITION:       DicomTag = DicomTag(0x0020, 0x0032);
    pub const IMAGE_ORIENTATION:    DicomTag = DicomTag(0x0020, 0x0037);
    pub const INSTANCE_NUMBER:      DicomTag = DicomTag(0x0020, 0x0013);
    pub const WINDOW_CENTER:        DicomTag = DicomTag(0x0028, 0x1050);
    pub const WINDOW_WIDTH:         DicomTag = DicomTag(0x0028, 0x1051);
}

impl std::fmt::Display for DicomTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:04X},{:04X})", self.0, self.1)
    }
}

/// DICOM VR (Value Representation)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vr {
    CS, LO, SH, PN, DA, TM, DT, DS, IS, US, SS, UL, SL, FL, FD, UI, AT, SQ, OB, OW, UN,
}

/// DICOM data element value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DicomValue {
    Str(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Floats(Vec<f64>),
    Bytes(Vec<u8>),
    Sequence(Vec<DicomDataset>),
}

pub type DicomDataset = HashMap<String, DicomValue>;

/// Parsed DICOM file metadata (no pixel data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomMetadata {
    pub sop_instance_uid: String,
    pub study_instance_uid: String,
    pub series_instance_uid: String,
    pub patient_name: String,
    pub patient_id: String,
    pub modality: DicomModality,
    pub rows: u32,
    pub columns: u32,
    pub instance_number: i32,
    pub slice_position: f64,
    pub pixel_spacing: [f64; 2],
    pub slice_thickness: f64,
    pub rescale_intercept: f64,
    pub rescale_slope: f64,
    pub window_center: f64,
    pub window_width: f64,
    pub bits_allocated: u16,
}

impl Default for DicomMetadata {
    fn default() -> Self {
        Self {
            sop_instance_uid: String::new(),
            study_instance_uid: String::new(),
            series_instance_uid: String::new(),
            patient_name: String::new(),
            patient_id: String::new(),
            modality: DicomModality::CT,
            rows: 512,
            columns: 512,
            instance_number: 0,
            slice_position: 0.0,
            pixel_spacing: [0.3, 0.3],
            slice_thickness: 0.3,
            rescale_intercept: -1024.0,
            rescale_slope: 1.0,
            window_center: 400.0,
            window_width: 2000.0,
            bits_allocated: 16,
        }
    }
}

/// DICOM modality codes relevant for dental
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DicomModality {
    /// Computed Tomography (CBCT)
    CT,
    /// Digital X-ray
    CR,
    /// Panoramic X-ray
    DX,
    /// Intraoral X-ray
    IO,
    /// Optical Surface Scan
    OT,
    /// Secondary Capture
    SC,
    Unknown(String),
}

impl DicomModality {
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "CT" => DicomModality::CT,
            "CR" => DicomModality::CR,
            "DX" => DicomModality::DX,
            "IO" => DicomModality::IO,
            "OT" => DicomModality::OT,
            "SC" => DicomModality::SC,
            other => DicomModality::Unknown(other.to_string()),
        }
    }
}

/// Apply rescale slope/intercept to get Hounsfield Units
pub fn to_hounsfield(raw_value: i16, slope: f64, intercept: f64) -> f64 {
    raw_value as f64 * slope + intercept
}

/// Parse pixel spacing string "0.3\\0.3" → [0.3, 0.3]
pub fn parse_pixel_spacing(s: &str) -> [f64; 2] {
    let parts: Vec<f64> = s.split('\\').filter_map(|v| v.parse().ok()).collect();
    if parts.len() >= 2 { [parts[0], parts[1]] } else { [1.0, 1.0] }
}

/// Parse DICOM date string YYYYMMDD → formatted
pub fn parse_dicom_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s.trim(), "%Y%m%d").ok()
}
