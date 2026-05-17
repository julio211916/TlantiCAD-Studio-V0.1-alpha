//! Odontogram repository

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use dental_core::models::{
    Odontogram,
    OdontogramEntry,
    OdontogramHistory,
    SurfaceCondition,
    UpdateOdontogramEntry,
};
use dental_core::ToothCondition;

use crate::{DbError, DbPool, DbResult};

/// Repository for odontogram entries
pub struct OdontogramRepository {
    pool: DbPool,
}

impl OdontogramRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get full odontogram for a patient
    pub fn get_odontogram(&self, patient_id: Uuid) -> DbResult<Odontogram> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, tooth_number, surface_conditions, primary_condition,
                   treatment_status, is_primary, mobility, notes, updated_at, updated_by
            FROM odontogram_entries
            WHERE patient_id = ?1
            ORDER BY tooth_number
            "#,
        )?;

        let rows = stmt.query_map([patient_id.to_string()], |row| self.map_row(row))?;
        let entries: Vec<OdontogramEntry> = rows.filter_map(|r| r.ok()).collect();

        let last_updated = entries
            .iter()
            .map(|e| e.updated_at)
            .max()
            .unwrap_or_else(Utc::now);

        Ok(Odontogram {
            patient_id,
            entries,
            last_updated,
        })
    }

    /// Get a specific tooth entry
    pub fn get_entry(&self, patient_id: Uuid, tooth_number: i32) -> DbResult<Option<OdontogramEntry>> {
        let conn = self.pool.get()?;

        conn.query_row(
            r#"
            SELECT id, patient_id, tooth_number, surface_conditions, primary_condition,
                   treatment_status, is_primary, mobility, notes, updated_at, updated_by
            FROM odontogram_entries
            WHERE patient_id = ?1 AND tooth_number = ?2
            "#,
            params![patient_id.to_string(), tooth_number],
            |row| self.map_row(row),
        )
        .optional()
        .map_err(|e| DbError::QueryError(e.to_string()))
    }

    /// Upsert an entry and write history if condition changes
    pub fn upsert_entry(&self, mut entry: OdontogramEntry, change_reason: Option<String>) -> DbResult<OdontogramEntry> {
        let conn = self.pool.get()?;
        let now = Utc::now();

        let existing = self.get_entry(entry.patient_id, entry.tooth_number)?;

        if let Some(prev) = existing.as_ref() {
            if prev.primary_condition != entry.primary_condition {
                self.insert_history(
                    entry.patient_id,
                    entry.tooth_number,
                    prev.primary_condition,
                    entry.primary_condition,
                    change_reason.clone(),
                    entry.updated_by,
                    now,
                )?;
            }
        }

        entry.updated_at = now;

        let surface_json = serde_json::to_string(&entry.surface_conditions)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO odontogram_entries (
                id, patient_id, tooth_number, surface_conditions, primary_condition,
                treatment_status, is_primary, mobility, notes, updated_at, updated_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(patient_id, tooth_number) DO UPDATE SET
                surface_conditions = excluded.surface_conditions,
                primary_condition = excluded.primary_condition,
                treatment_status = excluded.treatment_status,
                is_primary = excluded.is_primary,
                mobility = excluded.mobility,
                notes = excluded.notes,
                updated_at = excluded.updated_at,
                updated_by = excluded.updated_by
            "#,
            params![
                entry.id.to_string(),
                entry.patient_id.to_string(),
                entry.tooth_number,
                surface_json,
                entry.primary_condition.to_string(),
                entry.treatment_status,
                if entry.is_primary { 1 } else { 0 },
                entry.mobility,
                entry.notes,
                entry.updated_at.to_rfc3339(),
                entry.updated_by.to_string(),
            ],
        )?;

        Ok(entry)
    }

    /// Update an existing tooth entry with a patch
    pub fn update_tooth(
        &self,
        patient_id: Uuid,
        tooth_number: i32,
        update: UpdateOdontogramEntry,
        updated_by: Uuid,
    ) -> DbResult<OdontogramEntry> {
        let mut entry = match self.get_entry(patient_id, tooth_number)? {
            Some(e) => e,
            None => OdontogramEntry::new(patient_id, tooth_number, updated_by),
        };

        if let Some(condition) = update.primary_condition {
            entry.primary_condition = condition;
        }
        if let Some(surfaces) = update.surface_conditions {
            entry.surface_conditions = surfaces;
        }
        if let Some(notes) = update.notes {
            entry.notes = Some(notes);
        }
        if let Some(mobility) = update.mobility {
            entry.mobility = Some(mobility);
        }
        if let Some(status) = update.treatment_status {
            entry.treatment_status = Some(status);
        }
        if let Some(is_primary) = update.is_primary {
            entry.is_primary = is_primary;
        }

        entry.updated_by = updated_by;

        self.upsert_entry(entry, None)
    }

    /// List odontogram history for a patient
    pub fn list_history(&self, patient_id: Uuid) -> DbResult<Vec<OdontogramHistory>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, tooth_number, previous_condition, new_condition,
                   change_reason, changed_by, changed_at
            FROM odontogram_history
            WHERE patient_id = ?1
            ORDER BY changed_at DESC
            "#,
        )?;

        let rows = stmt.query_map([patient_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let patient_id: String = row.get(1)?;
            let prev: String = row.get(3)?;
            let new: String = row.get(4)?;
            let changed_by: String = row.get(6)?;
            let changed_at_str: String = row.get(7)?;

            let previous_condition: ToothCondition = prev.parse().unwrap_or(ToothCondition::Healthy);
            let new_condition: ToothCondition = new.parse().unwrap_or(ToothCondition::Healthy);

            let changed_at = DateTime::parse_from_rfc3339(&changed_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(OdontogramHistory {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
                tooth_number: row.get(2)?,
                previous_condition,
                new_condition,
                change_reason: row.get(5)?,
                changed_by: Uuid::parse_str(&changed_by).unwrap_or_default(),
                changed_at,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn insert_history(
        &self,
        patient_id: Uuid,
        tooth_number: i32,
        previous_condition: ToothCondition,
        new_condition: ToothCondition,
        change_reason: Option<String>,
        changed_by: Uuid,
        changed_at: DateTime<Utc>,
    ) -> DbResult<()> {
        let conn = self.pool.get()?;
        let id = Uuid::new_v4();

        conn.execute(
            r#"
            INSERT INTO odontogram_history (
                id, patient_id, tooth_number, previous_condition, new_condition,
                change_reason, changed_by, changed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                id.to_string(),
                patient_id.to_string(),
                tooth_number,
                previous_condition.to_string(),
                new_condition.to_string(),
                change_reason,
                changed_by.to_string(),
                changed_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    fn map_row(&self, row: &rusqlite::Row) -> Result<OdontogramEntry, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let surface_json: Option<String> = row.get(3)?;
        let condition_str: String = row.get(4)?;
        let updated_at_str: String = row.get(9)?;
        let updated_by: String = row.get(10)?;

        let primary_condition: ToothCondition = condition_str.parse().unwrap_or(ToothCondition::Healthy);
        let surface_conditions: Vec<SurfaceCondition> = surface_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(OdontogramEntry {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            tooth_number: row.get(2)?,
            surface_conditions,
            primary_condition,
            treatment_status: row.get(5)?,
            is_primary: row.get::<_, i64>(6)? == 1,
            mobility: row.get(7)?,
            notes: row.get(8)?,
            updated_at,
            updated_by: Uuid::parse_str(&updated_by).unwrap_or_default(),
        })
    }
}

/// Periodontogram repository (stores JSON payload)
pub struct PeriodontogramRepository {
    pool: DbPool,
}

impl PeriodontogramRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn get_by_patient(&self, patient_id: Uuid) -> DbResult<Option<serde_json::Value>> {
        let conn = self.pool.get()?;

        let value: Option<String> = conn.query_row(
            "SELECT data FROM periodontograms WHERE patient_id = ?1",
            [patient_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        let json = value
            .and_then(|s| serde_json::from_str(&s).ok());

        Ok(json)
    }

    pub fn save(&self, patient_id: Uuid, data: serde_json::Value, updated_by: Uuid) -> DbResult<serde_json::Value> {
        let conn = self.pool.get()?;
        let now = Utc::now();

        let existing_id: Option<String> = conn.query_row(
            "SELECT id FROM periodontograms WHERE patient_id = ?1",
            [patient_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        let id = existing_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let payload = serde_json::to_string(&data)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO periodontograms (id, patient_id, data, updated_at, updated_by)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(patient_id) DO UPDATE SET
                data = excluded.data,
                updated_at = excluded.updated_at,
                updated_by = excluded.updated_by
            "#,
            params![
                id,
                patient_id.to_string(),
                payload,
                now.to_rfc3339(),
                updated_by.to_string(),
            ],
        )?;

        Ok(data)
    }
}