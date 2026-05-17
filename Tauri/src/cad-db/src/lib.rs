pub mod models;
pub mod repository;
pub mod migrations;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use cad_core::{CadError, Result};

/// Holds the SQLite connection pool
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Open (or create) the SQLite database at `path`
    pub async fn open(path: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(path)
            .map_err(|e| CadError::Database(e.to_string()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|e| CadError::Database(e.to_string()))?;

        let db = Self { pool };
        migrations::run(&db).await?;
        Ok(db)
    }

    /// In-memory database (for tests)
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePool::connect(":memory:")
            .await
            .map_err(|e| CadError::Database(e.to_string()))?;
        let db = Self { pool };
        migrations::run(&db).await?;
        Ok(db)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
