use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImagingError {
    #[error("DICOM parse error: {0}")]
    DicomParse(String),

    #[error("STL parse error: {0}")]
    StlParse(String),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("DICOM network error: {0}")]
    Network(String),

    #[error("Orthanc PACS error: {0}")]
    OrthancError(String),

    #[error("DICOMweb error: {0}")]
    DicomWebError(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for ImagingError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
