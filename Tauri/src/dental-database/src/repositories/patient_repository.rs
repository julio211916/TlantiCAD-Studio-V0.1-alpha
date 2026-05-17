//! Patient repository

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use dental_core::models::{
    CreatePatient, Patient, PatientAddress, PatientFilters, PatientListItem, UpdatePatient,
};
use dental_core::{Gender, IdDocumentType};

use crate::{DbError, DbPool, DbResult};
use super::{Pagination, PaginatedResult};

/// Patient repository
pub struct PatientRepository {
    pool: DbPool,
}

impl PatientRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new patient
    pub fn create(&self, data: CreatePatient) -> DbResult<Patient> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        // Generate patient number
        let patient_number = self.generate_patient_number(&conn)?;
        
        let patient = Patient::new(
            data.first_name,
            data.last_name,
            data.birth_date,
            data.gender,
            data.phone,
        );
        
        let address_json = data.address.map(|a| serde_json::to_string(&a).ok()).flatten();
        let emergency_json = data.emergency_contact.map(|e| serde_json::to_string(&e).ok()).flatten();
        let medical_json = data.medical_history.map(|m| serde_json::to_string(&m).ok()).flatten();
        
        conn.execute(
            r#"
            INSERT INTO patients (
                id, patient_number, first_name, last_name, middle_name,
                birth_date, gender, phone, phone_secondary, email,
                address, id_document, id_document_type, occupation,
                emergency_contact, allergies, medical_history, notes,
                active, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21
            )
            "#,
            params![
                id.to_string(),
                patient_number,
                patient.first_name,
                patient.last_name,
                data.middle_name,
                patient.birth_date.to_string(),
                patient.gender.to_string(),
                patient.phone,
                data.phone_secondary,
                data.email,
                address_json,
                data.id_document,
                data.id_document_type.map(|t| t.to_string()),
                data.occupation,
                emergency_json,
                data.allergies,
                medical_json,
                data.notes,
                true,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;
        
        self.find_by_id(id)
    }
    
    /// Find patient by ID
    pub fn find_by_id(&self, id: Uuid) -> DbResult<Patient> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            r#"
            SELECT id, patient_number, first_name, last_name, middle_name,
                   birth_date, gender, phone, phone_secondary, email,
                   address, id_document, id_document_type, occupation, workplace,
                   emergency_contact, allergies, medical_history, referral_source,
                   notes, photo_url, insurance, preferred_reminder,
                   active, created_at, updated_at
            FROM patients
            WHERE id = ?1
            "#,
            [id.to_string()],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Patient {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Find patient by patient number
    pub fn find_by_patient_number(&self, patient_number: &str) -> DbResult<Patient> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            r#"
            SELECT id, patient_number, first_name, last_name, middle_name,
                   birth_date, gender, phone, phone_secondary, email,
                   address, id_document, id_document_type, occupation, workplace,
                   emergency_contact, allergies, medical_history, referral_source,
                   notes, photo_url, insurance, preferred_reminder,
                   active, created_at, updated_at
            FROM patients
            WHERE patient_number = ?1
            "#,
            [patient_number],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Patient {}", patient_number)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Update patient
    pub fn update(&self, id: Uuid, data: UpdatePatient) -> DbResult<Patient> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        
        // Get current patient
        let current = self.find_by_id(id)?;
        
        let first_name = data.first_name.unwrap_or(current.first_name);
        let last_name = data.last_name.unwrap_or(current.last_name);
        let phone = data.phone.unwrap_or(current.phone);
        let gender = data.gender.unwrap_or(current.gender);
        let birth_date = data.birth_date.unwrap_or(current.birth_date);
        let active = data.active.unwrap_or(current.active);
        
        let address_json = data.address.map(|a| serde_json::to_string(&a).ok()).flatten()
            .or(current.address.map(|a| serde_json::to_string(&a).ok()).flatten());
        
        conn.execute(
            r#"
            UPDATE patients SET
                first_name = ?2, last_name = ?3, middle_name = ?4,
                birth_date = ?5, gender = ?6, phone = ?7, phone_secondary = ?8,
                email = ?9, address = ?10, id_document = ?11, id_document_type = ?12,
                occupation = ?13, workplace = ?14, notes = ?15, photo_url = ?16,
                preferred_reminder = ?17, active = ?18, updated_at = ?19
            WHERE id = ?1
            "#,
            params![
                id.to_string(),
                first_name,
                last_name,
                data.middle_name.or(current.middle_name),
                birth_date.to_string(),
                gender.to_string(),
                phone,
                data.phone_secondary.or(current.phone_secondary),
                data.email.or(current.email),
                address_json,
                data.id_document.or(current.id_document),
                data.id_document_type.map(|t| t.to_string()).or(current.id_document_type.map(|t| t.to_string())),
                data.occupation.or(current.occupation),
                data.workplace.or(current.workplace),
                data.notes.or(current.notes),
                data.photo_url.or(current.photo_url),
                data.preferred_reminder.or(current.preferred_reminder),
                active,
                now.to_rfc3339(),
            ],
        )?;
        
        self.find_by_id(id)
    }
    
    /// Delete patient (soft delete)
    pub fn delete(&self, id: Uuid) -> DbResult<()> {
        let conn = self.pool.get()?;
        
        conn.execute(
            "UPDATE patients SET active = 0, updated_at = ?2 WHERE id = ?1",
            params![id.to_string(), Utc::now().to_rfc3339()],
        )?;
        
        Ok(())
    }
    
    /// List patients with pagination and filters
    pub fn list(&self, filters: PatientFilters, pagination: Pagination) -> DbResult<PaginatedResult<PatientListItem>> {
        let conn = self.pool.get()?;
        
        let mut where_clauses = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(ref query) = filters.query {
            where_clauses.push("(first_name LIKE ?1 OR last_name LIKE ?1 OR phone LIKE ?1 OR email LIKE ?1 OR patient_number LIKE ?1)");
            params.push(Box::new(format!("%{}%", query)));
        }
        
        if let Some(active_only) = filters.active_only {
            if active_only {
                where_clauses.push("active = 1");
            }
        }
        
        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };
        
        // Get total count
        let count_sql = format!("SELECT COUNT(*) FROM patients {}", where_sql);
        let total: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;
        
        // Get items
        let sql = format!(
            r#"
            SELECT id, patient_number, first_name || ' ' || last_name as full_name,
                   phone, email, birth_date, active
            FROM patients
            {}
            ORDER BY last_name, first_name
            LIMIT {} OFFSET {}
            "#,
            where_sql,
            pagination.limit(),
            pagination.offset()
        );
        
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let birth_date_str: String = row.get(5)?;
            let birth_date = chrono::NaiveDate::parse_from_str(&birth_date_str, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive());
            let age = (chrono::Utc::now().date_naive() - birth_date).num_days() / 365;
            
            Ok(PatientListItem {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                patient_number: row.get(1)?,
                full_name: row.get(2)?,
                phone: row.get(3)?,
                email: row.get(4)?,
                age: age as i32,
                last_visit: None,
                next_appointment: None,
                balance: rust_decimal::Decimal::ZERO,
                active: row.get(6)?,
            })
        })?;
        
        let items: Vec<PatientListItem> = rows.filter_map(|r| r.ok()).collect();
        
        Ok(PaginatedResult::new(items, total as u64, &pagination))
    }
    
    /// Search patients
    pub fn search(&self, query: &str, limit: usize) -> DbResult<Vec<PatientListItem>> {
        let conn = self.pool.get()?;
        let search_term = format!("%{}%", query);
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_number, first_name || ' ' || last_name as full_name,
                   phone, email, birth_date, active
            FROM patients
            WHERE active = 1 AND (
                first_name LIKE ?1 OR last_name LIKE ?1 OR 
                phone LIKE ?1 OR email LIKE ?1 OR patient_number LIKE ?1
            )
            ORDER BY last_name, first_name
            LIMIT ?2
            "#
        )?;
        
        let rows = stmt.query_map(params![search_term, limit as i32], |row| {
            let birth_date_str: String = row.get(5)?;
            let birth_date = chrono::NaiveDate::parse_from_str(&birth_date_str, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive());
            let age = (chrono::Utc::now().date_naive() - birth_date).num_days() / 365;
            
            Ok(PatientListItem {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                patient_number: row.get(1)?,
                full_name: row.get(2)?,
                phone: row.get(3)?,
                email: row.get(4)?,
                age: age as i32,
                last_visit: None,
                next_appointment: None,
                balance: rust_decimal::Decimal::ZERO,
                active: row.get(6)?,
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    /// Count patients
    pub fn count(&self, active_only: bool) -> DbResult<i64> {
        let conn = self.pool.get()?;
        let sql = if active_only {
            "SELECT COUNT(*) FROM patients WHERE active = 1"
        } else {
            "SELECT COUNT(*) FROM patients"
        };
        
        conn.query_row(sql, [], |row| row.get(0))
            .map_err(|e| DbError::QueryError(e.to_string()))
    }
    
    /// Generate next patient number
    fn generate_patient_number(&self, conn: &rusqlite::Connection) -> DbResult<String> {
        // Get prefix and counter from settings
        let prefix: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'patient.prefix'",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "PAC".to_string());
        
        let counter: i64 = conn.query_row(
            "SELECT value FROM settings WHERE key = 'patient.counter'",
            [],
            |row| {
                let val: String = row.get(0)?;
                Ok(val.parse::<i64>().unwrap_or(1))
            },
        ).unwrap_or(1);
        
        let year = chrono::Utc::now().format("%Y");
        let patient_number = format!("{}-{}-{:05}", prefix, year, counter);
        
        // Increment counter
        conn.execute(
            "UPDATE settings SET value = ?1, updated_at = ?2 WHERE key = 'patient.counter'",
            params![(counter + 1).to_string(), Utc::now().to_rfc3339()],
        )?;
        
        Ok(patient_number)
    }
    
    /// Map database row to Patient
    fn map_row(&self, row: &rusqlite::Row) -> Result<Patient, rusqlite::Error> {
        let id: String = row.get(0)?;
        let birth_date_str: String = row.get(5)?;
        let gender_str: String = row.get(6)?;
        let created_at_str: String = row.get(24)?;
        let updated_at_str: String = row.get(25)?;
        
        let birth_date = chrono::NaiveDate::parse_from_str(&birth_date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive());
        
        let gender: Gender = gender_str.parse().unwrap_or(Gender::Other);
        
        let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let address_json: Option<String> = row.get(10)?;
        let address: Option<PatientAddress> = address_json
            .and_then(|json| serde_json::from_str(&json).ok());
        
        let id_doc_type_str: Option<String> = row.get(12)?;
        let id_document_type: Option<IdDocumentType> = id_doc_type_str
            .and_then(|s| s.parse().ok());
        
        Ok(Patient {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_number: row.get(1)?,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
            middle_name: row.get(4)?,
            birth_date,
            gender,
            phone: row.get(7)?,
            phone_secondary: row.get(8)?,
            email: row.get(9)?,
            address,
            id_document: row.get(11)?,
            id_document_type,
            occupation: row.get(13)?,
            workplace: row.get(14)?,
            emergency_contact: None, // Parse from JSON if needed
            allergies: row.get(16)?,
            medical_history: None, // Parse from JSON if needed
            referral_source: row.get(18)?,
            notes: row.get(19)?,
            photo_url: row.get(20)?,
            insurance: None, // Parse from JSON if needed
            preferred_reminder: row.get(22)?,
            active: row.get(23)?,
            created_at,
            updated_at,
        })
    }
}
