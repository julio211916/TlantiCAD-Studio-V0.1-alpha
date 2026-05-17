//! Treatment repository

use chrono::{DateTime, Utc};
use rusqlite::params;
use rust_decimal::Decimal;
use uuid::Uuid;

use dental_core::models::{Treatment, CreateTreatment, TreatmentWithDetails};
use dental_core::TreatmentStatus;
use rust_decimal::prelude::FromPrimitive;

use crate::{DbError, DbPool, DbResult};

/// Treatment repository
pub struct TreatmentRepository {
    pool: DbPool,
}

#[derive(Debug, Clone)]
pub struct TreatmentAggregate {
    pub procedure_name: String,
    pub count: i64,
    pub revenue: Decimal,
}

impl TreatmentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new treatment
    pub fn create(&self, data: CreateTreatment) -> DbResult<Treatment> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        let discount = data.discount.unwrap_or(Decimal::ZERO);
        let final_price = data.price - discount;
        let surfaces_json = data.surfaces.map(|s| serde_json::to_string(&s).ok()).flatten();
        
        conn.execute(
            r#"
            INSERT INTO treatments (
                id, patient_id, appointment_id, treatment_plan_id, procedure_id,
                doctor_id, tooth_number, surfaces, quadrant, status,
                price, discount, final_price, notes, planned_date,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17
            )
            "#,
            params![
                id.to_string(),
                data.patient_id.to_string(),
                data.appointment_id.map(|a| a.to_string()),
                data.treatment_plan_id.map(|t| t.to_string()),
                data.procedure_id.to_string(),
                data.doctor_id.to_string(),
                data.tooth_number,
                surfaces_json,
                data.quadrant,
                "planned",
                data.price.to_string(),
                discount.to_string(),
                final_price.to_string(),
                data.notes,
                data.planned_date.map(|d| d.to_rfc3339()),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;
        
        self.find_by_id(id)
    }
    
    /// Find treatment by ID
    pub fn find_by_id(&self, id: Uuid) -> DbResult<Treatment> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            r#"
            SELECT id, patient_id, appointment_id, treatment_plan_id, procedure_id,
                   doctor_id, tooth_number, surfaces, quadrant, status,
                   price, discount, final_price, notes, planned_date,
                   completed_at, warranty_until, created_at, updated_at
            FROM treatments
            WHERE id = ?1
            "#,
            [id.to_string()],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Treatment {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Update treatment status
    pub fn update_status(&self, id: Uuid, status: TreatmentStatus) -> DbResult<()> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        
        let completed_at = if status == TreatmentStatus::Completed {
            Some(now.to_rfc3339())
        } else {
            None
        };
        
        conn.execute(
            "UPDATE treatments SET status = ?2, completed_at = ?3, updated_at = ?4 WHERE id = ?1",
            params![id.to_string(), status.to_string(), completed_at, now.to_rfc3339()],
        )?;
        
        Ok(())
    }
    
    /// List treatments by patient
    pub fn list_by_patient(&self, patient_id: Uuid) -> DbResult<Vec<TreatmentWithDetails>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT t.id, t.patient_id, t.appointment_id, t.treatment_plan_id, t.procedure_id,
                   t.doctor_id, t.tooth_number, t.surfaces, t.quadrant, t.status,
                   t.price, t.discount, t.final_price, t.notes, t.planned_date,
                   t.completed_at, t.warranty_until, t.created_at, t.updated_at,
                   pr.name, pr.code,
                   p.first_name || ' ' || p.last_name,
                   u.first_name || ' ' || u.last_name
            FROM treatments t
            JOIN procedures pr ON pr.id = t.procedure_id
            JOIN patients p ON p.id = t.patient_id
            JOIN users u ON u.id = t.doctor_id
            WHERE t.patient_id = ?1
            ORDER BY t.created_at DESC
            "#
        )?;
        
        let rows = stmt.query_map([patient_id.to_string()], |row| {
            let treatment = self.map_row(row)?;
            Ok(TreatmentWithDetails {
                treatment,
                procedure_name: row.get(19)?,
                procedure_code: row.get(20)?,
                patient_name: row.get(21)?,
                doctor_name: row.get(22)?,
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    /// List treatments by appointment
    pub fn list_by_appointment(&self, appointment_id: Uuid) -> DbResult<Vec<Treatment>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, appointment_id, treatment_plan_id, procedure_id,
                   doctor_id, tooth_number, surfaces, quadrant, status,
                   price, discount, final_price, notes, planned_date,
                   completed_at, warranty_until, created_at, updated_at
            FROM treatments
            WHERE appointment_id = ?1
            ORDER BY created_at
            "#
        )?;
        
        let rows = stmt.query_map([appointment_id.to_string()], |row| self.map_row(row))?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get top treatments by revenue
    pub fn get_top_treatments(&self, limit: i64) -> DbResult<Vec<TreatmentAggregate>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT pr.name,
                   COUNT(t.id) as total_count,
                   COALESCE(SUM(CAST(t.final_price AS REAL)), 0) as total_revenue
            FROM treatments t
            JOIN procedures pr ON pr.id = t.procedure_id
            GROUP BY pr.name
            ORDER BY total_revenue DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map([limit], |row| {
            let revenue: f64 = row.get(2)?;
            Ok(TreatmentAggregate {
                procedure_name: row.get(0)?,
                count: row.get(1)?,
                revenue: Decimal::from_f64(revenue).unwrap_or(Decimal::ZERO),
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    fn map_row(&self, row: &rusqlite::Row) -> Result<Treatment, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let procedure_id: String = row.get(4)?;
        let doctor_id: String = row.get(5)?;
        let status_str: String = row.get(9)?;
        let price_str: String = row.get(10)?;
        let discount_str: String = row.get(11)?;
        let final_price_str: String = row.get(12)?;
        let created_at_str: String = row.get(17)?;
        let updated_at_str: String = row.get(18)?;
        
        let status: TreatmentStatus = status_str.parse().unwrap_or(TreatmentStatus::Planned);
        let price: Decimal = price_str.parse().unwrap_or(Decimal::ZERO);
        let discount: Decimal = discount_str.parse().unwrap_or(Decimal::ZERO);
        let final_price: Decimal = final_price_str.parse().unwrap_or(Decimal::ZERO);
        
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        Ok(Treatment {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            appointment_id: row.get::<_, Option<String>>(2)?.and_then(|s| Uuid::parse_str(&s).ok()),
            treatment_plan_id: row.get::<_, Option<String>>(3)?.and_then(|s| Uuid::parse_str(&s).ok()),
            procedure_id: Uuid::parse_str(&procedure_id).unwrap_or_default(),
            doctor_id: Uuid::parse_str(&doctor_id).unwrap_or_default(),
            tooth_number: row.get(6)?,
            surfaces: None, // Parse from JSON if needed
            quadrant: row.get(8)?,
            status,
            price,
            discount,
            final_price,
            notes: row.get(13)?,
            planned_date: None,
            completed_at: None,
            warranty_until: None,
            created_at,
            updated_at,
        })
    }
}
