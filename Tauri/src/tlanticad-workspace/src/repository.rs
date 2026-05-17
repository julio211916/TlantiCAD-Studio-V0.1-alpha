//! Repository pattern for database operations

use crate::Database;
use chrono::{Utc, Datelike};
use sqlx::Row;
use tlanticad_core::{Id, Project, ProjectStatus, Result, TlantiError};
use uuid::Uuid;

/// Repository de proyectos
pub struct ProjectRepository<'a> {
    db: &'a Database,
}

impl<'a> ProjectRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Crear nuevo proyecto
    pub async fn create(&self, patient_name: &str, work_type: &str) -> Result<Project> {
        let id = Uuid::new_v4();
        let case_number = generate_case_number().await?;
        let now = Utc::now();

        // Crear paciente temporal
        let patient_id = Uuid::new_v4();
        let names: Vec<&str> = patient_name.split_whitespace().collect();
        let first_name = names.first().unwrap_or(&"").to_string();
        let last_name = names.get(1..).map(|n| n.join(" ")).unwrap_or_default();

        sqlx::query(
            "INSERT INTO patients (id, first_name, last_name) VALUES (?1, ?2, ?3)"
        )
        .bind(patient_id.to_string())
        .bind(&first_name)
        .bind(&last_name)
        .execute(self.db.pool())
        .await
        .map_err(|e| TlantiError::Database(e.to_string()))?;

        // Crear dentista por defecto
        let dentist_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO dentists (id, name) VALUES (?1, ?2)"
        )
        .bind(dentist_id.to_string())
        .bind("Default Dentist")
        .execute(self.db.pool())
        .await
        .map_err(|e| TlantiError::Database(e.to_string()))?;

        // Crear proyecto
        sqlx::query(
            "INSERT INTO projects (id, case_number, patient_id, dentist_id, work_type, status, created_at, modified_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        )
        .bind(id.to_string())
        .bind(&case_number)
        .bind(patient_id.to_string())
        .bind(dentist_id.to_string())
        .bind(work_type)
        .bind("new")
        .bind(now)
        .bind(now)
        .execute(self.db.pool())
        .await
        .map_err(|e| TlantiError::Database(e.to_string()))?;

        tracing::info!("Created project {} with case number {}", id, case_number);

        Ok(Project {
            id,
            case_number,
            patient_name: patient_name.to_string(),
            dentist: "Default Dentist".to_string(),
            clinic: String::new(),
            created_at: now,
            modified_at: now,
            work_type: parse_work_type(work_type),
            teeth: Vec::new(),
            materials: Vec::new(),
            scans: Vec::new(),
            designs: Vec::new(),
            status: ProjectStatus::New,
            notes: String::new(),
            technician: None,
            is_deleted: false,
            global_shade: None,
            antagonist_scan_mode: None,
            is_imported: false,
        })
    }

    /// Obtener proyecto por ID
    pub async fn get_by_id(&self, id: Id) -> Result<Project> {
        let row = sqlx::query(
            r#"SELECT p.*, pt.first_name, pt.last_name, d.name as dentist_name, d.clinic
               FROM projects p
               JOIN patients pt ON p.patient_id = pt.id
               JOIN dentists d ON p.dentist_id = d.id
               WHERE p.id = ?1"#
        )
        .bind(id.to_string())
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| TlantiError::Database(e.to_string()))?;

        Ok(Project {
            id,
            case_number: row.get("case_number"),
            patient_name: format!("{} {}", 
                row.get::<String, _>("first_name"), 
                row.get::<String, _>("last_name")
            ),
            dentist: row.get("dentist_name"),
            clinic: row.get("clinic"),
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            work_type: parse_work_type(&row.get::<String, _>("work_type")),
            teeth: Vec::new(),
            materials: Vec::new(),
            scans: Vec::new(),
            designs: Vec::new(),
            status: parse_status(&row.get::<String, _>("status")),
            notes: row.get("notes"),
            technician: None,
            is_deleted: false,
            global_shade: None,
            antagonist_scan_mode: None,
            is_imported: false,
        })
    }

    /// Listar todos los proyectos
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Project>> {
        let rows = sqlx::query(
            r#"SELECT p.*, pt.first_name, pt.last_name, d.name as dentist_name, d.clinic
               FROM projects p
               JOIN patients pt ON p.patient_id = pt.id
               JOIN dentists d ON p.dentist_id = d.id
               ORDER BY p.modified_at DESC
               LIMIT ?1 OFFSET ?2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| TlantiError::Database(e.to_string()))?;

        let mut projects = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            projects.push(Project {
                id: Id::parse_str(&id).map_err(|_| TlantiError::Database("Invalid UUID".to_string()))?,
                case_number: row.get("case_number"),
                patient_name: format!("{} {}", 
                    row.get::<String, _>("first_name"), 
                    row.get::<String, _>("last_name")
                ),
                dentist: row.get("dentist_name"),
                clinic: row.get("clinic"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                work_type: parse_work_type(&row.get::<String, _>("work_type")),
                teeth: Vec::new(),
                materials: Vec::new(),
                scans: Vec::new(),
                designs: Vec::new(),
                status: parse_status(&row.get::<String, _>("status")),
                notes: row.get("notes"),
                technician: None,
                is_deleted: false,
                global_shade: None,
                antagonist_scan_mode: None,
                is_imported: false,
            });
        }

        Ok(projects)
    }

    /// Actualizar estado del proyecto
    pub async fn update_status(&self, id: Id, status: ProjectStatus) -> Result<()> {
        sqlx::query(
            "UPDATE projects SET status = ?1, modified_at = ?2 WHERE id = ?3"
        )
        .bind(format!("{:?}", status).to_lowercase())
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(self.db.pool())
        .await
        .map_err(|e| TlantiError::Database(e.to_string()))?;

        Ok(())
    }

    /// Buscar proyectos
    pub async fn search(&self, query: &str) -> Result<Vec<Project>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query(
            r#"SELECT p.*, pt.first_name, pt.last_name, d.name as dentist_name, d.clinic
               FROM projects p
               JOIN patients pt ON p.patient_id = pt.id
               JOIN dentists d ON p.dentist_id = d.id
               WHERE p.case_number LIKE ?1 
                  OR pt.first_name LIKE ?1 
                  OR pt.last_name LIKE ?1
                  OR d.name LIKE ?1
               ORDER BY p.modified_at DESC"#
        )
        .bind(&pattern)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| TlantiError::Database(e.to_string()))?;

        let mut projects = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            projects.push(Project {
                id: Id::parse_str(&id).map_err(|_| TlantiError::Database("Invalid UUID".to_string()))?,
                case_number: row.get("case_number"),
                patient_name: format!("{} {}", 
                    row.get::<String, _>("first_name"), 
                    row.get::<String, _>("last_name")
                ),
                dentist: row.get("dentist_name"),
                clinic: row.get("clinic"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                work_type: parse_work_type(&row.get::<String, _>("work_type")),
                teeth: Vec::new(),
                materials: Vec::new(),
                scans: Vec::new(),
                designs: Vec::new(),
                status: parse_status(&row.get::<String, _>("status")),
                notes: row.get("notes"),
                technician: None,
                is_deleted: false,
                global_shade: None,
                antagonist_scan_mode: None,
                is_imported: false,
            });
        }

        Ok(projects)
    }

    /// Eliminar proyecto
    pub async fn delete(&self, id: Id) -> Result<()> {
        let mut tx = self.db.pool().begin().await.map_err(|e| TlantiError::Database(e.to_string()))?;
        
        sqlx::query("DELETE FROM projects WHERE id = ?1")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await.map_err(|e| TlantiError::Database(e.to_string()))?;
            
        tx.commit().await.map_err(|e| TlantiError::Database(e.to_string()))?;
        Ok(())
    }
}

/// Generar número de caso
async fn generate_case_number() -> Result<String> {
    let year = Utc::now().year();
    let timestamp = Utc::now().timestamp_millis() % 10000;
    Ok(format!("TL-{}-{:04}", year, timestamp))
}

fn parse_work_type(s: &str) -> tlanticad_core::WorkType {
    use tlanticad_core::WorkType::*;
    match s {
        "crown_anatomic" => CrownAnatomic,
        "crown_reduced" => CrownReduced,
        "crown_veneer" => CrownVeneer,
        "crown_inlay" => CrownInlay,
        "crown_onlay" => CrownOnlay,
        "bridge" => Bridge,
        "abutment_custom" => AbutmentCustom,
        "abutment_stock" => AbutmentStock,
        "screw_retained_crown" => ScrewRetainedCrown,
        "bar" => Bar,
        "telescope_primary" => TelescopePrimary,
        "telescope_secondary" => TelescopeSecondary,
        "bite_splint" => BiteSplint,
        "model" => Model,
        "waxup" => WaxUp,
        _ => CrownAnatomic,
    }
}

fn parse_status(s: &str) -> ProjectStatus {
    use ProjectStatus::*;
    match s {
        "new" => New,
        "scan_imported" => ScanImported,
        "margin_defined" => MarginDefined,
        "in_design" => InDesign,
        "design_complete" => DesignComplete,
        "ready_for_manufacturing" => ReadyForManufacturing,
        "manufactured" => Manufactured,
        "delivered" => Delivered,
        _ => New,
    }
}
