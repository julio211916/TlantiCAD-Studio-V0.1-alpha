//! TlantiCAD DICOM Module
//! Full DICOM parsing, CBCT volume reconstruction, PACS connectivity

pub mod parser;
pub mod cbct;
pub mod pacs;
pub mod series;

// AR-V379 — DICOM segmentation toolbox (sinus, CT, surface mesh edit)
pub mod segmentation;

// AR-V413 — full maxillary sinus segmentation pipeline
pub mod sinus_seg;

pub use parser::*;
pub use cbct::*;
pub use series::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_dicom_tag_constants() {
        assert_eq!(DicomTag::PATIENT_NAME, DicomTag(0x0010, 0x0010));
        assert_eq!(DicomTag::MODALITY, DicomTag(0x0008, 0x0060));
    }

    #[test]
    fn test_dicom_modality_from_str() {
        assert_eq!(DicomModality::from_str("CT"), DicomModality::CT);
        assert_eq!(DicomModality::from_str("DX"), DicomModality::DX);
        assert!(matches!(DicomModality::from_str("FOOBAR"), DicomModality::Unknown(_)));
    }

    #[test]
    fn test_dicom_metadata_default() {
        let m = DicomMetadata::default();
        assert_eq!(m.rows, 512);
        assert_eq!(m.columns, 512);
        assert_eq!(m.modality, DicomModality::CT);
        assert_eq!(m.bits_allocated, 16);
    }

    #[test]
    fn test_to_hounsfield() {
        let hu = to_hounsfield(1000, 1.0, -1024.0);
        assert!((hu - (-24.0)).abs() < 0.01);
    }

    #[test]
    fn test_to_hounsfield_with_slope() {
        let hu = to_hounsfield(500, 2.0, -100.0);
        assert!((hu - 900.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_pixel_spacing() {
        let ps = parse_pixel_spacing("0.3\\0.3");
        assert!((ps[0] - 0.3).abs() < 0.001);
        assert!((ps[1] - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_parse_pixel_spacing_invalid() {
        let ps = parse_pixel_spacing("invalid");
        assert_eq!(ps, [1.0, 1.0]);
    }

    #[test]
    fn test_parse_dicom_date_valid() {
        let d = parse_dicom_date("20240115");
        assert!(d.is_some());
        let d = d.unwrap();
        assert_eq!(d.year(), 2024);
        assert_eq!(d.month(), 1);
        assert_eq!(d.day(), 15);
    }

    #[test]
    fn test_parse_dicom_date_invalid() {
        let d = parse_dicom_date("not-a-date");
        assert!(d.is_none());
    }

    #[test]
    fn test_dicom_metadata_serialize_roundtrip() {
        let m = DicomMetadata::default();
        let json = serde_json::to_string(&m).unwrap();
        let m2: DicomMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m2.rows, 512);
    }
}
