use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClinicalError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<dental_database::DbError> for ClinicalError {
    fn from(err: dental_database::DbError) -> Self {
        match err {
            dental_database::DbError::NotFound(msg) => ClinicalError::NotFound(msg),
            _ => ClinicalError::Database(err.to_string()),
        }
    }
}

pub type ClinicalResult<T> = Result<T, ClinicalError>;
