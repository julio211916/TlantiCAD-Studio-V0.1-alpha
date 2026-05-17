//! TlantiCAD Database Module
//! 
//! Replica la funcionalidad de DentalDB de Exocad usando SQLite
//! Gestiona casos, pacientes, doctores y trabajos

pub mod schema;
pub mod repository;
pub mod migrations;
pub mod queries;

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;
use tlanticad_core::{Result, TlantiError};

pub use repository::*;
pub use schema::*;

/// Conexión a la base de datos
#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Crear nueva conexión a la base de datos
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref();
        
        // Crear directorio si no existe
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| TlantiError::Database(e.to_string()))?;
        }

        if !db_path.exists() {
            std::fs::File::create(&db_path).map_err(|e| TlantiError::Database(e.to_string()))?;
        }

        let conn_str = format!("sqlite://{}", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&conn_str)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        let db = Self { pool };
        db.init().await?;
        
        Ok(db)
    }

    /// Inicializar schema
    async fn init(&self) -> Result<()> {
        sqlx::query(&schema::CREATE_PROJECTS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_PATIENTS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_DENTISTS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_TECHNICIANS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_TEETH_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_CONNECTORS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_SCANS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_DESIGNS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_MATERIALS_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        sqlx::query(&schema::CREATE_WORK_TYPES_TABLE)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;

        // Insertar work types por defecto
        self.seed_work_types().await?;
        self.seed_materials().await?;

        tracing::info!("Database initialized successfully");
        Ok(())
    }

    /// Insertar work types por defecto
    async fn seed_work_types(&self) -> Result<()> {
        

        let work_types = vec![
            ("crown_anatomic", "Anatomic Crown", "CRW-A"),
            ("crown_reduced", "Reduced Crown", "CRW-R"),
            ("crown_veneer", "Veneer", "VNR"),
            ("crown_inlay", "Inlay", "INL"),
            ("crown_onlay", "Onlay", "ONL"),
            ("bridge", "Bridge", "BRG"),
            ("abutment_custom", "Custom Abutment", "ABT-C"),
            ("abutment_stock", "Stock Abutment", "ABT-S"),
            ("screw_retained_crown", "Screw-Retained Crown", "SRC"),
            ("bar", "Bar", "BAR"),
            ("telescope_primary", "Telescope Primary", "TEL-P"),
            ("telescope_secondary", "Telescope Secondary", "TEL-S"),
            ("bite_splint", "Bite Splint", "SPL"),
            ("model", "Model", "MOD"),
            ("waxup", "WaxUp", "WAX"),
        ];

        for (id, name, code) in work_types {
            sqlx::query(
                "INSERT OR IGNORE INTO work_types (id, name, code, available_processors) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(id)
            .bind(name)
            .bind(code)
            .bind("[]")
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Insertar materiales por defecto
    async fn seed_materials(&self) -> Result<()> {
        let materials = vec![
            ("Zirconia", "zirconia", "Zirconia", "[\"A1\",\"A2\",\"A3\",\"B1\",\"B2\"]"),
            ("Lithium Disilicate", "lithium_disilicate", "E.max", "[\"A1\",\"A2\",\"A3\",\"B1\"]"),
            ("PMMA", "pmma", "PMMA", "[\"A1\",\"A2\",\"A3\"]"),
            ("PEEK", "peek", "PEEK", "[\"pink\",\"white\"]"),
            ("Titanium", "titanium", "Ti", "[\"Ti\"]"),
            ("Cobalt Chrome", "cobalt_chrome", "CoCr", "[\"metal\"]"),
            ("Composite", "composite", "Composite", "[\"A2\",\"A3\"]"),
            ("Wax", "wax", "Wax", "[\"wax\"]"),
        ];

        for (name, id, manufacturer, shades) in materials {
            sqlx::query(
                "INSERT OR IGNORE INTO materials (id, name, manufacturer, available_shades) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(id)
            .bind(name)
            .bind(manufacturer)
            .bind(shades)
            .execute(&self.pool)
            .await
            .map_err(|e| TlantiError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Obtener pool de conexiones
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Cerrar conexión
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new(":memory:").await.unwrap();
        assert!(!db.pool.is_closed());
    }
}
