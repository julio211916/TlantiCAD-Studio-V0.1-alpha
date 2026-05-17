//! SQLite database implementation

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use tracing::info;

use crate::{DbError, Result};

/// SQLite database wrapper
pub struct SqliteDb {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteDb {
    /// Create a new SQLite database connection
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DbError::Sqlite(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(1),
                    Some(e.to_string()),
                ))
            })?;
        }

        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| DbError::Pool(e.to_string()))?;

        let db = Self { pool };
        db.init()?;

        info!("SQLite database initialized at {:?}", path);
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .map_err(|e| DbError::Pool(e.to_string()))?;

        let db = Self { pool };
        db.init()?;

        info!("In-memory SQLite database initialized");
        Ok(db)
    }

    /// Initialize database schema
    fn init(&self) -> Result<()> {
        let conn = self.get_connection()?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Create tables
        conn.execute_batch(
            r#"
            -- Projects table
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Meshes table
            CREATE TABLE IF NOT EXISTS meshes (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                file_path TEXT,
                vertex_count INTEGER NOT NULL DEFAULT 0,
                face_count INTEGER NOT NULL DEFAULT 0,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            -- ML Models table
            CREATE TABLE IF NOT EXISTS ml_models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                model_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                input_shape TEXT,
                output_shape TEXT,
                created_at TEXT NOT NULL
            );

            -- Settings table
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_meshes_project_id ON meshes(project_id);
            CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);
            "#,
        )?;

        Ok(())
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().map_err(|e| DbError::Pool(e.to_string()))
    }

    /// Execute a query with no return value
    pub fn execute<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<()>,
    {
        let conn = self.get_connection()?;
        f(&conn)?;
        Ok(())
    }

    /// Execute a query and return a result
    pub fn query<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<T>,
    {
        let conn = self.get_connection()?;
        let result = f(&conn)?;
        Ok(result)
    }
}

impl Clone for SqliteDb {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_db() {
        let db = SqliteDb::in_memory().unwrap();
        assert!(db.get_connection().is_ok());
    }
}
