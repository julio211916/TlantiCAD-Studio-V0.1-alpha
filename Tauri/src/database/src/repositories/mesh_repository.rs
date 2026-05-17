//! Mesh repository

use chrono::Utc;
use app_core::types::MeshData;
use rusqlite::params;
use serde_json;
use uuid::Uuid;

use crate::{sqlite::SqliteDb, Result};

/// Repository for mesh operations
pub struct MeshRepository {
    db: SqliteDb,
}

impl MeshRepository {
    pub fn new(db: SqliteDb) -> Self {
        Self { db }
    }

    /// Save mesh metadata to database
    pub fn save(&self, project_id: Uuid, mesh: &MeshData, file_path: Option<&str>) -> Result<()> {
        let metadata = serde_json::to_string(&mesh.transform).ok();

        self.db.execute(|conn| {
            conn.execute(
                r#"
                INSERT OR REPLACE INTO meshes (id, project_id, name, file_path, vertex_count, face_count, metadata, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    mesh.id.to_string(),
                    project_id.to_string(),
                    mesh.name,
                    file_path,
                    mesh.vertex_count() as i64,
                    mesh.face_count() as i64,
                    metadata,
                    mesh.created_at.to_rfc3339(),
                    mesh.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    /// Get mesh by ID
    pub fn get(&self, id: Uuid) -> Result<(MeshData, Option<String>)> {
        self.db.query(|conn| {
            conn.query_row(
                "SELECT id, name, file_path, vertex_count, face_count, metadata, created_at, updated_at FROM meshes WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    let id_str: String = row.get(0)?;
                    let file_path: Option<String> = row.get(2)?;
                    let created_at_str: String = row.get(6)?;
                    let updated_at_str: String = row.get(7)?;

                    let mut mesh = MeshData::new(row.get::<_, String>(1)?);
                    mesh.id = Uuid::parse_str(&id_str).unwrap_or_default();
                    mesh.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());
                    mesh.updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    Ok((mesh, file_path))
                },
            )
        })
    }

    /// List meshes for a project
    pub fn list_by_project(&self, project_id: Uuid) -> Result<Vec<MeshData>> {
        self.db.query(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, vertex_count, face_count, created_at, updated_at FROM meshes WHERE project_id = ?1 ORDER BY name"
            )?;

            let meshes = stmt.query_map(params![project_id.to_string()], |row| {
                let id_str: String = row.get(0)?;
                let created_at_str: String = row.get(4)?;
                let updated_at_str: String = row.get(5)?;

                let mut mesh = MeshData::new(row.get::<_, String>(1)?);
                mesh.id = Uuid::parse_str(&id_str).unwrap_or_default();
                mesh.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                mesh.updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(mesh)
            })?;

            meshes.collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    /// Delete a mesh
    pub fn delete(&self, id: Uuid) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute("DELETE FROM meshes WHERE id = ?1", params![id.to_string()])?;
            Ok(())
        })?;
        Ok(())
    }
}
