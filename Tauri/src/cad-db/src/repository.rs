use crate::{Database, models::*};
use cad_core::Result;

// ─── Patient repository ───────────────────────────────────────────────────

pub struct PatientRepo<'a>(pub &'a Database);

impl<'a> PatientRepo<'a> {
    pub async fn insert(&self, p: &Patient) -> Result<()> {
        sqlx::query(
            "INSERT INTO patients (id, first_name, last_name, date_of_birth, patient_number, phone, email, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&p.id)
        .bind(&p.first_name)
        .bind(&p.last_name)
        .bind(&p.date_of_birth)
        .bind(&p.patient_number)
        .bind(&p.phone)
        .bind(&p.email)
        .bind(&p.notes)
        .execute(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Patient>> {
        sqlx::query_as::<_, Patient>("SELECT * FROM patients WHERE id = ?")
            .bind(id)
            .fetch_optional(self.0.pool())
            .await
            .map_err(|e| cad_core::CadError::Database(e.to_string()))
    }

    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<Patient>> {
        let pattern = format!("%{query}%");
        sqlx::query_as::<_, Patient>(
            "SELECT * FROM patients
             WHERE first_name LIKE ? OR last_name LIKE ? OR patient_number LIKE ?
             ORDER BY last_name, first_name
             LIMIT ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))
    }

    pub async fn list(&self, offset: i64, limit: i64) -> Result<Vec<Patient>> {
        sqlx::query_as::<_, Patient>(
            "SELECT * FROM patients ORDER BY last_name, first_name LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM patients WHERE id = ?")
            .bind(id)
            .execute(self.0.pool())
            .await
            .map_err(|e| cad_core::CadError::Database(e.to_string()))?;
        Ok(())
    }
}

// ─── Case repository ──────────────────────────────────────────────────────

pub struct CaseRepo<'a>(pub &'a Database);

impl<'a> CaseRepo<'a> {
    pub async fn insert(&self, c: &Case) -> Result<()> {
        sqlx::query(
            "INSERT INTO cases (id, case_number, patient_id, dentist_id, work_type, status, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&c.id)
        .bind(&c.case_number)
        .bind(&c.patient_id)
        .bind(&c.dentist_id)
        .bind(&c.work_type)
        .bind(&c.status)
        .bind(&c.notes)
        .execute(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Case>> {
        sqlx::query_as::<_, Case>("SELECT * FROM cases WHERE id = ?")
            .bind(id)
            .fetch_optional(self.0.pool())
            .await
            .map_err(|e| cad_core::CadError::Database(e.to_string()))
    }

    pub async fn list_for_patient(&self, patient_id: &str) -> Result<Vec<Case>> {
        sqlx::query_as::<_, Case>(
            "SELECT * FROM cases WHERE patient_id = ? ORDER BY modified_at DESC",
        )
        .bind(patient_id)
        .fetch_all(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))
    }

    pub async fn update_status(&self, id: &str, status: &str) -> Result<()> {
        sqlx::query(
            "UPDATE cases SET status = ?, modified_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(status)
        .bind(id)
        .execute(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

        // Log the status change
        sqlx::query(
            "INSERT INTO case_audit_log (case_id, action, details) VALUES (?, 'status_change', ?)",
        )
        .bind(id)
        .bind(format!("{{\"new_status\": \"{status}\"}}"))
        .execute(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<Case>> {
        sqlx::query_as::<_, Case>(
            "SELECT * FROM cases ORDER BY modified_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))
    }
}

// ─── Scan repository ──────────────────────────────────────────────────────

pub struct ScanRepo<'a>(pub &'a Database);

impl<'a> ScanRepo<'a> {
    pub async fn insert(&self, s: &Scan) -> Result<()> {
        sqlx::query(
            "INSERT INTO scans (id, case_id, scan_type, file_path, file_hash, transformation, is_visible)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&s.id)
        .bind(&s.case_id)
        .bind(&s.scan_type)
        .bind(&s.file_path)
        .bind(&s.file_hash)
        .bind(&s.transformation)
        .bind(s.is_visible)
        .execute(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_case(&self, case_id: &str) -> Result<Vec<Scan>> {
        sqlx::query_as::<_, Scan>(
            "SELECT * FROM scans WHERE case_id = ? ORDER BY import_date",
        )
        .bind(case_id)
        .fetch_all(self.0.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_patient_crud() {
        let db = Database::in_memory().await.expect("in-memory DB");
        let repo = PatientRepo(&db);

        let patient = Patient::new("Juan".into(), "García".into());
        repo.insert(&patient).await.expect("insert");

        let found = repo
            .get_by_id(&patient.id)
            .await
            .expect("get")
            .expect("Some");
        assert_eq!(found.first_name, "Juan");
        assert_eq!(found.last_name, "García");
    }

    #[tokio::test]
    async fn test_case_status_transition() {
        let db = Database::in_memory().await.expect("in-memory DB");

        // Insert prerequisite records
        let patient = Patient::new("Ana".into(), "López".into());
        PatientRepo(&db).insert(&patient).await.unwrap();

        let dentist = Dentist::new("Dr. Pérez".into());
        sqlx::query(
            "INSERT INTO dentists (id, name) VALUES (?, ?)",
        )
        .bind(&dentist.id)
        .bind(&dentist.name)
        .execute(db.pool())
        .await
        .unwrap();

        let case = Case::new(patient.id.clone(), dentist.id.clone(), "crown".into());
        let case_repo = CaseRepo(&db);
        case_repo.insert(&case).await.unwrap();

        case_repo
            .update_status(&case.id, "design")
            .await
            .unwrap();

        let updated = case_repo
            .get_by_id(&case.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "design");
    }
}
