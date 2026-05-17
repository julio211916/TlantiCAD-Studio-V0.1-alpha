//! Database migrations

use crate::Database;
use tlanticad_core::Result;

/// Run all migrations
pub async fn migrate(db: &Database) -> Result<()> {
    // Migration tracking
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS __migrations (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(db.pool())
    .await
    .map_err(|e| tlanticad_core::TlantiError::Database(e.to_string()))?;

    let current_version: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM __migrations")
        .fetch_one(db.pool())
        .await
        .map_err(|e| tlanticad_core::TlantiError::Database(e.to_string()))?;

    tracing::info!("Current database version: {}", current_version);

    // Apply pending migrations
    if current_version < 1 {
        migrate_v1(db).await?;
    }

    Ok(())
}

async fn migrate_v1(db: &Database) -> Result<()> {
    tracing::info!("Applying migration v1...");
    
    // Add activity log table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS activity_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id TEXT NOT NULL,
            action TEXT NOT NULL,
            details TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )"
    )
    .execute(db.pool())
    .await
    .map_err(|e| tlanticad_core::TlantiError::Database(e.to_string()))?;

    // Record migration
    sqlx::query("INSERT INTO __migrations (version) VALUES (1)")
        .execute(db.pool())
        .await
        .map_err(|e| tlanticad_core::TlantiError::Database(e.to_string()))?;

    tracing::info!("Migration v1 applied successfully");
    Ok(())
}
