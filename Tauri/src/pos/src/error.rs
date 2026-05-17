use thiserror::Error;

#[derive(Error, Debug)]
pub enum PosError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Stock error: {0}")]
    Stock(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<dental_database::DbError> for PosError {
    fn from(err: dental_database::DbError) -> Self {
        match err {
            dental_database::DbError::NotFound(msg) => PosError::NotFound(msg),
            dental_database::DbError::QueryError(msg) if msg.contains("Insufficient stock") => {
                PosError::Stock(msg)
            }
            _ => PosError::Database(err.to_string()),
        }
    }
}

pub type PosResult<T> = Result<T, PosError>;
