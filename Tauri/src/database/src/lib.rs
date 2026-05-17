//! Database layer for TlantiStudio
//!
//! Provides SQLite for local storage and PostgreSQL for cloud sync.

pub mod migrations;
pub mod postgres;
pub mod repositories;
pub mod sqlite;

pub use repositories::*;
pub use sqlite::SqliteDb;

use app_core::error::CoreError;
use thiserror::Error;

/// Database-specific errors
#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("PostgreSQL error: {0}")]
    Postgres(String),

    #[error("Connection pool error: {0}")]
    Pool(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<DbError> for CoreError {
    fn from(err: DbError) -> Self {
        CoreError::Database(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DbError>;
