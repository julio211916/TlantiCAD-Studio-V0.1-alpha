//! DICOM file viewer/parser
//!
//! Reads DICOM files (X-Ray, CT, MRI, panoramic) and extracts:
//! - Patient metadata (name, ID, study date)
//! - Image pixel data as PNG/base64
//! - Series/study information
//! - Window/level adjustments

use crate::error::ImagingError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// DICOM study metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStudy {
    pub patient_name: Option<String>,
    pub patient_id: Option<String>,
    pub study_date: Option<String>,
    pub study_description: Option<String>,
    pub modality: Option<String>, // CR, CT, MR, OT, DX (dental X-ray)
    pub series_description: Option<String>,
    pub institution_name: Option<String>,
    pub manufacturer: Option<String>,
    pub rows: u32,
    pub columns: u32,
    pub bits_allocated: u16,
    pub pixel_spacing: Option<(f64, f64)>,
    pub window_center: Option<f64>,
    pub window_width: Option<f64>,
    pub file_path: String,
}

/// DICOM image data for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomImage {
    pub study: DicomStudy,
    /// Base64-encoded PNG for frontend rendering
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
}

/// Parse a DICOM file and extract metadata
pub fn parse_dicom_metadata(path: &Path) -> Result<DicomStudy, ImagingError> {
    use dicom::object::open_file;

    let obj = open_file(path).map_err(|e| ImagingError::DicomParse(e.to_string()))?;

    let patient_name = obj
        .element_by_name("PatientName")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let patient_id = obj
        .element_by_name("PatientID")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let study_date = obj
        .element_by_name("StudyDate")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let study_description = obj
        .element_by_name("StudyDescription")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let modality = obj
        .element_by_name("Modality")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let series_description = obj
        .element_by_name("SeriesDescription")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let institution_name = obj
        .element_by_name("InstitutionName")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let manufacturer = obj
        .element_by_name("Manufacturer")
        .ok()
        .and_then(|e| e.to_str().ok().map(|s| s.to_string()));

    let rows = obj
        .element_by_name("Rows")
        .ok()
        .and_then(|e| e.to_int::<u32>().ok())
        .unwrap_or(0);

    let columns = obj
        .element_by_name("Columns")
        .ok()
        .and_then(|e| e.to_int::<u32>().ok())
        .unwrap_or(0);

    let bits_allocated = obj
        .element_by_name("BitsAllocated")
        .ok()
        .and_then(|e| e.to_int::<u16>().ok())
        .unwrap_or(8);

    let window_center = obj
        .element_by_name("WindowCenter")
        .ok()
        .and_then(|e| e.to_str().ok())
        .and_then(|s| s.trim().parse::<f64>().ok());

    let window_width = obj
        .element_by_name("WindowWidth")
        .ok()
        .and_then(|e| e.to_str().ok())
        .and_then(|s| s.trim().parse::<f64>().ok());

    Ok(DicomStudy {
        patient_name,
        patient_id,
        study_date,
        study_description,
        modality,
        series_description,
        institution_name,
        manufacturer,
        rows,
        columns,
        bits_allocated,
        pixel_spacing: None,
        window_center,
        window_width,
        file_path: path.to_string_lossy().to_string(),
    })
}

/// Parse a DICOM file and extract pixel data as base64 PNG
pub fn parse_dicom_image(path: &Path) -> Result<DicomImage, ImagingError> {
    use base64::Engine as _;
    use dicom::object::open_file;
    use dicom_pixeldata::PixelDecoder;
    use image::ImageEncoder;

    let obj = open_file(path).map_err(|e| ImagingError::DicomParse(e.to_string()))?;
    let study = parse_dicom_metadata(path)?;

    // Decode pixel data
    let pixel_data = obj
        .decode_pixel_data()
        .map_err(|e| ImagingError::DicomParse(format!("Pixel decode: {}", e)))?;

    let dynamic_image = pixel_data
        .to_dynamic_image(0)
        .map_err(|e| ImagingError::ImageProcessing(format!("To image: {}", e)))?;

    // Apply window/level if available
    let gray = dynamic_image.to_luma16();
    let (w, h) = gray.dimensions();

    // Convert to PNG buffer
    let mut png_buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_buf);
    encoder
        .write_image(
            dynamic_image.as_bytes(),
            w,
            h,
            dynamic_image.color().into(),
        )
        .map_err(|e| ImagingError::ImageProcessing(e.to_string()))?;

    let image_base64 = base64::engine::general_purpose::STANDARD.encode(&png_buf);

    Ok(DicomImage {
        study,
        image_base64,
        width: w,
        height: h,
    })
}

/// List all DICOM files in a directory
pub fn list_dicom_files(dir: &Path) -> Result<Vec<DicomStudy>, ImagingError> {
    let mut studies = Vec::new();

    if !dir.exists() {
        return Ok(studies);
    }

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext == "dcm" || ext == "dicom" || ext.is_empty() {
                if let Ok(study) = parse_dicom_metadata(path) {
                    studies.push(study);
                }
            }
        }
    }

    Ok(studies)
}
