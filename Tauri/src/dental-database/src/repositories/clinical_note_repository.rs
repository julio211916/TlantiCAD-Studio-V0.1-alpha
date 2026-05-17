//! Clinical note repository

use chrono::{DateTime, Utc};
use rusqlite::{params, params_from_iter, types::Value};
use uuid::Uuid;

use dental_core::models::{ClinicalNote, ClinicalNoteFilters, CreateClinicalNote, UpdateClinicalNote};
use dental_core::ClinicalNoteType;

use crate::{DbError, DbPool, DbResult};

/// Clinical note repository
pub struct ClinicalNoteRepository {
    pool: DbPool,
}

impl ClinicalNoteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Create a new clinical note
    pub fn create(&self, data: CreateClinicalNote) -> DbResult<ClinicalNote> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();

        let attachments_json = data
            .attachments
            .unwrap_or_default();
        let attachments = serde_json::to_string(&attachments_json)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO clinical_notes (
                id, patient_id, appointment_id, user_id,
                note_type, content, attachments, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                id.to_string(),
                data.patient_id.to_string(),
                data.appointment_id.map(|a| a.to_string()),
                data.user_id.to_string(),
                data.note_type.to_string(),
                data.content,
                attachments,
                now.to_rfc3339(),
            ],
        )?;

        self.find_by_id(id)
    }

    /// Find clinical note by ID
    pub fn find_by_id(&self, id: Uuid) -> DbResult<ClinicalNote> {
        let conn = self.pool.get()?;

        conn.query_row(
            r#"
            SELECT id, patient_id, appointment_id, user_id, note_type,
                   content, attachments, created_at
            FROM clinical_notes
            WHERE id = ?1
            "#,
            [id.to_string()],
            |row| self.map_row(row),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("ClinicalNote {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }

    /// Update clinical note
    pub fn update(&self, id: Uuid, data: UpdateClinicalNote) -> DbResult<ClinicalNote> {
        let conn = self.pool.get()?;
        let current = self.find_by_id(id)?;

        let note_type = data.note_type.unwrap_or(current.note_type);
        let content = data.content.unwrap_or(current.content);
        let attachments = data.attachments.unwrap_or(current.attachments);
        let attachments_json = serde_json::to_string(&attachments)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        conn.execute(
            r#"
            UPDATE clinical_notes SET
                note_type = ?2,
                content = ?3,
                attachments = ?4
            WHERE id = ?1
            "#,
            params![
                id.to_string(),
                note_type.to_string(),
                content,
                attachments_json,
            ],
        )?;

        self.find_by_id(id)
    }

    /// Delete clinical note
    pub fn delete(&self, id: Uuid) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM clinical_notes WHERE id = ?1", [id.to_string()])?;
        Ok(())
    }

    /// List notes with filters
    pub fn list(&self, filters: ClinicalNoteFilters) -> DbResult<Vec<ClinicalNote>> {
        let conn = self.pool.get()?;

        let mut conditions: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        if let Some(patient_id) = filters.patient_id {
            conditions.push("patient_id = ?".to_string());
            values.push(Value::from(patient_id.to_string()));
        }

        if let Some(appointment_id) = filters.appointment_id {
            conditions.push("appointment_id = ?".to_string());
            values.push(Value::from(appointment_id.to_string()));
        }

        if let Some(user_id) = filters.user_id {
            conditions.push("user_id = ?".to_string());
            values.push(Value::from(user_id.to_string()));
        }

        if let Some(note_type) = filters.note_type {
            conditions.push("note_type = ?".to_string());
            values.push(Value::from(note_type.to_string()));
        }

        if let Some(date_from) = filters.date_from {
            conditions.push("created_at >= ?".to_string());
            values.push(Value::from(date_from.to_rfc3339()));
        }

        if let Some(date_to) = filters.date_to {
            conditions.push("created_at <= ?".to_string());
            values.push(Value::from(date_to.to_rfc3339()));
        }

        let where_sql = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT id, patient_id, appointment_id, user_id, note_type,
                   content, attachments, created_at
            FROM clinical_notes
            {}
            ORDER BY created_at DESC
            "#,
            where_sql
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(values), |row| self.map_row(row))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// List notes for a patient
    pub fn list_by_patient(&self, patient_id: Uuid) -> DbResult<Vec<ClinicalNote>> {
        self.list(ClinicalNoteFilters {
            patient_id: Some(patient_id),
            ..ClinicalNoteFilters::default()
        })
    }

    /// List notes for an appointment
    pub fn list_by_appointment(&self, appointment_id: Uuid) -> DbResult<Vec<ClinicalNote>> {
        self.list(ClinicalNoteFilters {
            appointment_id: Some(appointment_id),
            ..ClinicalNoteFilters::default()
        })
    }

    fn map_row(&self, row: &rusqlite::Row) -> Result<ClinicalNote, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let appointment_id: Option<String> = row.get(2)?;
        let user_id: String = row.get(3)?;
        let note_type_str: String = row.get(4)?;
        let attachments_json: String = row.get(6)?;
        let created_at_str: String = row.get(7)?;

        let note_type = note_type_str
            .parse::<ClinicalNoteType>()
            .unwrap_or(ClinicalNoteType::Examination);

        let attachments: Vec<String> = serde_json::from_str(&attachments_json).unwrap_or_default();

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(ClinicalNote {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            appointment_id: appointment_id.and_then(|s| Uuid::parse_str(&s).ok()),
            user_id: Uuid::parse_str(&user_id).unwrap_or_default(),
            note_type,
            content: row.get(5)?,
            attachments,
            created_at,
        })
    }
}
