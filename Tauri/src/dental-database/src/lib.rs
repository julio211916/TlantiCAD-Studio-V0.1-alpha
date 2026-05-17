//! TlantiStudio Dental Database Layer
//!
//! SQLite database implementation with connection pooling and repositories.

pub mod schema;
pub mod migrations;
pub mod repositories;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use thiserror::Error;
use tracing::info;

/// Database error types
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),
    
    #[error("Migration error: {0}")]
    MigrationError(String),
    
    #[error("Pool error: {0}")]
    PoolError(String),
    
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    
    #[error("R2D2 error: {0}")]
    R2D2Error(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<r2d2::Error> for DbError {
    fn from(e: r2d2::Error) -> Self {
        DbError::R2D2Error(e.to_string())
    }
}

/// Result type alias for database operations
pub type DbResult<T> = Result<T, DbError>;

/// Database connection pool type
pub type DbPool = Pool<SqliteConnectionManager>;

/// Database connection type
pub type DbConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub pool_size: u32,
    pub create_if_missing: bool,
    pub run_migrations: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "tlanti_dental.db".to_string(),
            pool_size: 10,
            create_if_missing: true,
            run_migrations: true,
        }
    }
}

/// Main database struct
pub struct Database {
    pool: DbPool,
    config: DatabaseConfig,
}

impl Database {
    /// Create a new database instance
    pub fn new(config: DatabaseConfig) -> DbResult<Self> {
        info!("Initializing database at: {}", config.path);
        
        // Create SQLite connection manager
        let manager = if config.create_if_missing {
            SqliteConnectionManager::file(&config.path)
        } else {
            if !Path::new(&config.path).exists() {
                return Err(DbError::ConnectionError(format!(
                    "Database file not found: {}",
                    config.path
                )));
            }
            SqliteConnectionManager::file(&config.path)
        };
        
        // Create connection pool
        let pool = Pool::builder()
            .max_size(config.pool_size)
            .build(manager)?;
        
        let db = Self { pool, config };
        
        // Run migrations if enabled
        if db.config.run_migrations {
            db.run_migrations()?;
        }
        
        info!("Database initialized successfully");
        Ok(db)
    }
    
    /// Create an in-memory database (for testing)
    pub fn in_memory() -> DbResult<Self> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)?;
        
        let db = Self {
            pool,
            config: DatabaseConfig {
                path: ":memory:".to_string(),
                pool_size: 1,
                create_if_missing: true,
                run_migrations: true,
            },
        };
        
        db.run_migrations()?;
        Ok(db)
    }
    
    /// Get a connection from the pool
    pub fn get_connection(&self) -> DbResult<DbConnection> {
        self.pool.get().map_err(|e| DbError::PoolError(e.to_string()))
    }
    
    /// Run all pending migrations
    pub fn run_migrations(&self) -> DbResult<()> {
        let conn = self.get_connection()?;
        migrations::run_all(&conn)?;
        Ok(())
    }
    
    /// Get the connection pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
    
    /// Check database health
    pub fn health_check(&self) -> DbResult<bool> {
        let conn = self.get_connection()?;
        let _: i32 = conn.query_row("SELECT 1", [], |row| row.get(0))?;
        Ok(true)
    }
    
    /// Get database file size in bytes
    pub fn file_size(&self) -> DbResult<u64> {
        if self.config.path == ":memory:" {
            return Ok(0);
        }
        
        std::fs::metadata(&self.config.path)
            .map(|m| m.len())
            .map_err(|e| DbError::ConnectionError(e.to_string()))
    }
    
    /// Vacuum the database to reclaim space
    pub fn vacuum(&self) -> DbResult<()> {
        let conn = self.get_connection()?;
        conn.execute("VACUUM", [])?;
        Ok(())
    }
    
    /// Begin a transaction
    pub fn transaction<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&Connection) -> DbResult<T>,
    {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}

// Re-export repositories
pub use repositories::*;

// Re-export rusqlite for use by dental-commands
pub use rusqlite;
