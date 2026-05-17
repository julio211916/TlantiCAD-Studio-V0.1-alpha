use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<dental_database::DbError> for AccountingError {
    fn from(err: dental_database::DbError) -> Self {
        match err {
            dental_database::DbError::NotFound(msg) => AccountingError::NotFound(msg),
            _ => AccountingError::Database(err.to_string()),
        }
    }
}

impl From<r2d2::Error> for AccountingError {
    fn from(err: r2d2::Error) -> Self {
        AccountingError::Database(err.to_string())
    }
}

impl From<dental_database::rusqlite::Error> for AccountingError {
    fn from(err: dental_database::rusqlite::Error) -> Self {
        AccountingError::Database(err.to_string())
    }
}

pub type AccountingResult<T> = Result<T, AccountingError>;
