use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Excel write error: {0}")]
    ExcelWrite(String),

    #[error("Excel read error: {0}")]
    ExcelRead(String),

    #[error("CSV error: {0}")]
    Csv(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for ExportError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
