//! Project repository

use chrono::Utc;
use app_core::types::Project;
use rusqlite::params;
use uuid::Uuid;

use crate::{sqlite::SqliteDb, Result};

/// Repository for project operations
pub struct ProjectRepository {
    db: SqliteDb,
}

impl ProjectRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    /// Create a new project
    pub fn create(&self, project: &Project) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                r#"
                INSERT INTO projects (id, name, description, path, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![
                    project.id.to_string(),
                    project.name,
                    project.description,
                    project.path,
                    project.created_at.to_rfc3339(),
                    project.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    /// Get a project by ID
    pub fn get(&self, id: Uuid) -> Result<Project> {
        self.db.query(|conn| {
            conn.query_row(
                "SELECT id, name, description, path, created_at, updated_at FROM projects WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    let id_str: String = row.get(0)?;
                    let created_at_str: String = row.get(4)?;
                    let updated_at_str: String = row.get(5)?;

                    Ok(Project {
                        id: Uuid::parse_str(&id_str).unwrap_or_default(),
                        name: row.get(1)?,
                        description: row.get(2)?,
                        path: row.get(3)?,
                        created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                            .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                            .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    })
                },
            )
        })
    }

    /// List all projects
    pub fn list(&self) -> Result<Vec<Project>> {
        self.db.query(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, path, created_at, updated_at FROM projects ORDER BY updated_at DESC"
            )?;

            let projects = stmt.query_map([], |row| {
                let id_str: String = row.get(0)?;
                let created_at_str: String = row.get(4)?;
                let updated_at_str: String = row.get(5)?;

                Ok(Project {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    path: row.get(3)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?;

            projects.collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    /// Update a project
    pub fn update(&self, project: &Project) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                r#"
                UPDATE projects SET
                    name = ?2,
                    description = ?3,
                    path = ?4,
                    updated_at = ?5
                WHERE id = ?1
                "#,
                params![
                    project.id.to_string(),
                    project.name,
                    project.description,
                    project.path,
                    Utc::now().to_rfc3339(),
                ],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    /// Delete a project
    pub fn delete(&self, id: Uuid) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute("DELETE FROM projects WHERE id = ?1", params![id.to_string()])?;
            Ok(())
        })?;
        Ok(())
    }
}
