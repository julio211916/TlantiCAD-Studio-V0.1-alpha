//! PostgreSQL database implementation for cloud sync

use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

use crate::{DbError, Result};

/// PostgreSQL database wrapper
pub struct PostgresDb {
    pool: PgPool,
}

impl PostgresDb {
    /// Create a new PostgreSQL connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| DbError::Postgres(e.to_string()))?;

        let db = Self { pool };
        db.init().await?;

        info!("PostgreSQL database connected");
        Ok(db)
    }

    /// Initialize database schema
    async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            -- Projects table
            CREATE TABLE IF NOT EXISTS projects (
                id UUID PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                path VARCHAR(1024) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL
            );

            -- Create index
            CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Postgres(e.to_string()))?;

        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl Clone for PostgresDb {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
