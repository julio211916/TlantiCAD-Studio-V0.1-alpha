//! Appointment repository

use chrono::{DateTime, Utc, NaiveDate};
use rusqlite::params;
use uuid::Uuid;

use dental_core::models::{
    Appointment, AppointmentListItem, CreateAppointment, UpdateAppointment,
};
use dental_core::AppointmentStatus;

use crate::{DbError, DbPool, DbResult};

/// Appointment repository
pub struct AppointmentRepository {
    pool: DbPool,
}

impl AppointmentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new appointment
    pub fn create(&self, data: CreateAppointment, created_by: Uuid) -> DbResult<Appointment> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        let procedures_json = data.procedures.map(|p| serde_json::to_string(&p).ok()).flatten();
        
        conn.execute(
            r#"
            INSERT INTO appointments (
                id, patient_id, doctor_id, datetime, duration_minutes,
                chair_number, clinic_id, status, reason, procedures,
                notes, color, created_at, updated_at, created_by
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
            )
            "#,
            params![
                id.to_string(),
                data.patient_id.to_string(),
                data.doctor_id.to_string(),
                data.datetime.to_rfc3339(),
                data.duration_minutes,
                data.chair_number,
                data.clinic_id.map(|c| c.to_string()),
                "scheduled",
                data.reason,
                procedures_json,
                data.notes,
                data.color,
                now.to_rfc3339(),
                now.to_rfc3339(),
                created_by.to_string(),
            ],
        )?;
        
        self.find_by_id(id)
    }
    
    /// Find appointment by ID
    pub fn find_by_id(&self, id: Uuid) -> DbResult<Appointment> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            r#"
            SELECT id, patient_id, doctor_id, datetime, duration_minutes,
                   chair_number, clinic_id, status, reason, procedures,
                   notes, internal_notes, confirmation_sent, reminder_sent,
                   checked_in_at, started_at, completed_at, cancel_reason,
                   recurring_group_id, color, created_at, updated_at, created_by
            FROM appointments
            WHERE id = ?1
            "#,
            [id.to_string()],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Appointment {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Update appointment
    pub fn update(&self, id: Uuid, data: UpdateAppointment) -> DbResult<Appointment> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let current = self.find_by_id(id)?;
        
        let doctor_id = data.doctor_id.unwrap_or(current.doctor_id);
        let datetime = data.datetime.unwrap_or(current.datetime);
        let duration = data.duration_minutes.unwrap_or(current.duration_minutes);
        let status = data.status.unwrap_or(current.status);
        
        conn.execute(
            r#"
            UPDATE appointments SET
                doctor_id = ?2, datetime = ?3, duration_minutes = ?4,
                chair_number = ?5, status = ?6, reason = ?7, notes = ?8,
                internal_notes = ?9, cancel_reason = ?10, color = ?11,
                updated_at = ?12
            WHERE id = ?1
            "#,
            params![
                id.to_string(),
                doctor_id.to_string(),
                datetime.to_rfc3339(),
                duration,
                data.chair_number.or(current.chair_number),
                status.to_string(),
                data.reason.or(current.reason),
                data.notes.or(current.notes),
                data.internal_notes.or(current.internal_notes),
                data.cancel_reason.or(current.cancel_reason),
                data.color.or(current.color),
                now.to_rfc3339(),
            ],
        )?;
        
        self.find_by_id(id)
    }
    
    /// Update appointment status
    pub fn update_status(&self, id: Uuid, status: AppointmentStatus) -> DbResult<()> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        
        let extra_field = match status {
            AppointmentStatus::CheckedIn => Some(("checked_in_at", now.to_rfc3339())),
            AppointmentStatus::InProgress => Some(("started_at", now.to_rfc3339())),
            AppointmentStatus::Completed => Some(("completed_at", now.to_rfc3339())),
            _ => None,
        };
        
        if let Some((field, value)) = extra_field {
            conn.execute(
                &format!("UPDATE appointments SET status = ?2, {} = ?3, updated_at = ?4 WHERE id = ?1", field),
                params![id.to_string(), status.to_string(), value, now.to_rfc3339()],
            )?;
        } else {
            conn.execute(
                "UPDATE appointments SET status = ?2, updated_at = ?3 WHERE id = ?1",
                params![id.to_string(), status.to_string(), now.to_rfc3339()],
            )?;
        }
        
        Ok(())
    }
    
    /// Delete appointment
    pub fn delete(&self, id: Uuid) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM appointments WHERE id = ?1", [id.to_string()])?;
        Ok(())
    }
    
    /// List appointments by date range
    pub fn list_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        doctor_id: Option<Uuid>,
    ) -> DbResult<Vec<AppointmentListItem>> {
        let conn = self.pool.get()?;
        
        let sql = if doctor_id.is_some() {
            r#"
            SELECT a.id, a.patient_id, p.first_name || ' ' || p.last_name,
                   a.doctor_id, u.first_name || ' ' || u.last_name,
                   a.datetime, a.duration_minutes, a.chair_number, a.status,
                   a.reason, a.color
            FROM appointments a
            JOIN patients p ON p.id = a.patient_id
            JOIN users u ON u.id = a.doctor_id
            WHERE a.datetime >= ?1 AND a.datetime <= ?2 AND a.doctor_id = ?3
            ORDER BY a.datetime
            "#
        } else {
            r#"
            SELECT a.id, a.patient_id, p.first_name || ' ' || p.last_name,
                   a.doctor_id, u.first_name || ' ' || u.last_name,
                   a.datetime, a.duration_minutes, a.chair_number, a.status,
                   a.reason, a.color
            FROM appointments a
            JOIN patients p ON p.id = a.patient_id
            JOIN users u ON u.id = a.doctor_id
            WHERE a.datetime >= ?1 AND a.datetime <= ?2
            ORDER BY a.datetime
            "#
        };
        
        let mut stmt = conn.prepare(sql)?;
        
        let items: Vec<AppointmentListItem> = if let Some(doc_id) = doctor_id {
            let rows = stmt.query_map(params![start.to_rfc3339(), end.to_rfc3339(), doc_id.to_string()], |row| self.map_list_item(row))?;
            rows.filter_map(|r| r.ok()).collect()
        } else {
            let rows = stmt.query_map(params![start.to_rfc3339(), end.to_rfc3339()], |row| self.map_list_item(row))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        
        Ok(items)
    }
    
    /// Get appointments for a patient
    pub fn list_by_patient(&self, patient_id: Uuid) -> DbResult<Vec<AppointmentListItem>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT a.id, a.patient_id, p.first_name || ' ' || p.last_name,
                   a.doctor_id, u.first_name || ' ' || u.last_name,
                   a.datetime, a.duration_minutes, a.chair_number, a.status,
                   a.reason, a.color
            FROM appointments a
            JOIN patients p ON p.id = a.patient_id
            JOIN users u ON u.id = a.doctor_id
            WHERE a.patient_id = ?1
            ORDER BY a.datetime DESC
            "#
        )?;
        
        let rows = stmt.query_map([patient_id.to_string()], |row| self.map_list_item(row))?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    /// Get today's appointments
    pub fn get_today(&self, doctor_id: Option<Uuid>) -> DbResult<Vec<AppointmentListItem>> {
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_end = Utc::now().date_naive().and_hms_opt(23, 59, 59).unwrap();
        
        let start = DateTime::<Utc>::from_naive_utc_and_offset(today_start, Utc);
        let end = DateTime::<Utc>::from_naive_utc_and_offset(today_end, Utc);
        
        self.list_by_date_range(start, end, doctor_id)
    }
    
    /// Count appointments by status for a date
    pub fn count_by_date(&self, date: NaiveDate) -> DbResult<std::collections::HashMap<AppointmentStatus, i32>> {
        let conn = self.pool.get()?;
        let date_str = date.to_string();
        
        let mut stmt = conn.prepare(
            "SELECT status, COUNT(*) FROM appointments WHERE date(datetime) = ?1 GROUP BY status"
        )?;
        
        let rows = stmt.query_map([date_str], |row| {
            let status_str: String = row.get(0)?;
            let count: i32 = row.get(1)?;
            Ok((status_str, count))
        })?;
        
        let mut result = std::collections::HashMap::new();
        for row in rows {
            if let Ok((status_str, count)) = row {
                if let Ok(status) = status_str.parse::<AppointmentStatus>() {
                    result.insert(status, count);
                }
            }
        }
        
        Ok(result)
    }
    
    fn map_row(&self, row: &rusqlite::Row) -> Result<Appointment, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let doctor_id: String = row.get(2)?;
        let datetime_str: String = row.get(3)?;
        let status_str: String = row.get(7)?;
        let created_at_str: String = row.get(20)?;
        let updated_at_str: String = row.get(21)?;
        let created_by: String = row.get(22)?;
        
        let datetime = DateTime::parse_from_rfc3339(&datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let status: AppointmentStatus = status_str.parse().unwrap_or(AppointmentStatus::Scheduled);
        
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        Ok(Appointment {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            doctor_id: Uuid::parse_str(&doctor_id).unwrap_or_default(),
            datetime,
            duration_minutes: row.get(4)?,
            chair_number: row.get(5)?,
            clinic_id: row.get::<_, Option<String>>(6)?.and_then(|s| Uuid::parse_str(&s).ok()),
            status,
            reason: row.get(8)?,
            procedures: row.get(9)?,
            notes: row.get(10)?,
            internal_notes: row.get(11)?,
            confirmation_sent: row.get(12)?,
            reminder_sent: row.get(13)?,
            checked_in_at: None,
            started_at: None,
            completed_at: None,
            cancel_reason: row.get(17)?,
            recurring_group_id: None,
            color: row.get(19)?,
            created_at,
            updated_at,
            created_by: Uuid::parse_str(&created_by).unwrap_or_default(),
        })
    }
    
    fn map_list_item(&self, row: &rusqlite::Row) -> Result<AppointmentListItem, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let doctor_id: String = row.get(3)?;
        let datetime_str: String = row.get(5)?;
        let status_str: String = row.get(8)?;
        
        let datetime = DateTime::parse_from_rfc3339(&datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let status: AppointmentStatus = status_str.parse().unwrap_or(AppointmentStatus::Scheduled);
        
        Ok(AppointmentListItem {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            patient_name: row.get(2)?,
            doctor_id: Uuid::parse_str(&doctor_id).unwrap_or_default(),
            doctor_name: row.get(4)?,
            datetime,
            duration_minutes: row.get(6)?,
            chair_number: row.get(7)?,
            status,
            reason: row.get(9)?,
            color: row.get(10)?,
        })
    }
}
