use crate::Database;
use cad_core::Result;

/// Run all pending migrations in order
pub async fn run(db: &Database) -> Result<()> {
    // Bootstrap the migration tracking table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version  INTEGER PRIMARY KEY,
            name     TEXT NOT NULL,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    let current: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _migrations")
            .fetch_one(db.pool())
            .await
            .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    tracing::debug!("DB schema version: {current}");

    if current < 1 {
        v1_initial_schema(db).await?;
    }
    if current < 2 {
        v2_library_index(db).await?;
    }

    Ok(())
}

// ─── v1 — Initial DentalDB schema ─────────────────────────────────────────

async fn v1_initial_schema(db: &Database) -> Result<()> {
    tracing::info!("Applying migration v1: initial schema");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS patients (
            id             TEXT PRIMARY KEY,
            first_name     TEXT NOT NULL,
            last_name      TEXT NOT NULL,
            date_of_birth  TEXT,
            patient_number TEXT UNIQUE,
            phone          TEXT,
            email          TEXT,
            notes          TEXT,
            created_at     DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_patients_name ON patients(last_name, first_name);
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dentists (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            clinic     TEXT,
            email      TEXT,
            phone      TEXT,
            city       TEXT,
            country    TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_dentists_name ON dentists(name);
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cases (
            id          TEXT PRIMARY KEY,
            case_number TEXT UNIQUE NOT NULL,
            patient_id  TEXT NOT NULL,
            dentist_id  TEXT NOT NULL,
            work_type   TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'new',
            notes       TEXT,
            created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
            modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (patient_id) REFERENCES patients(id) ON DELETE CASCADE,
            FOREIGN KEY (dentist_id) REFERENCES dentists(id)
        );
        CREATE INDEX IF NOT EXISTS idx_cases_patient  ON cases(patient_id);
        CREATE INDEX IF NOT EXISTS idx_cases_status   ON cases(status);
        CREATE INDEX IF NOT EXISTS idx_cases_modified ON cases(modified_at DESC);
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scans (
            id             TEXT PRIMARY KEY,
            case_id        TEXT NOT NULL,
            scan_type      TEXT NOT NULL,
            file_path      TEXT NOT NULL,
            file_hash      TEXT,
            transformation TEXT,
            is_visible     INTEGER NOT NULL DEFAULT 1,
            import_date    DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_scans_case ON scans(case_id);
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS designs (
            id             TEXT PRIMARY KEY,
            case_id        TEXT NOT NULL,
            name           TEXT NOT NULL,
            design_type    TEXT NOT NULL,
            tooth_number   INTEGER,
            mesh_file_path TEXT,
            parameters     TEXT,
            created_at     DATETIME DEFAULT CURRENT_TIMESTAMP,
            modified_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_designs_case ON designs(case_id);
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS materials (
            id               TEXT PRIMARY KEY,
            name             TEXT NOT NULL,
            material_type    TEXT NOT NULL,
            manufacturer     TEXT,
            available_shades TEXT,
            is_active        INTEGER NOT NULL DEFAULT 1
        );
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS case_audit_log (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            case_id    TEXT NOT NULL,
            action     TEXT NOT NULL,
            details    TEXT,
            timestamp  DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_audit_case ON case_audit_log(case_id);
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    // Insert default materials
    sqlx::query(
        "INSERT OR IGNORE INTO materials (id, name, material_type, manufacturer, available_shades)
         VALUES
         ('mat-zr-1',  'Zirconia 3Y-TZP',    'zirconia',     'Generic',   '[\"A1\",\"A2\",\"A3\",\"A3.5\",\"B1\",\"B2\",\"C2\",\"D3\"]'),
         ('mat-pm-1',  'PMMA Temp',           'pmma',         'Generic',   '[\"A1\",\"A2\",\"A3\",\"A3.5\",\"B2\"]'),
         ('mat-ti-1',  'Titanium Grade 5',    'titanium',     'Generic',   NULL),
         ('mat-co-1',  'Cobalt-Chrome',       'cobalt_chrome','Generic',   NULL),
         ('mat-ex-1',  'IPS e.max CAD',       'lithium_disil','Ivoclar',   '[\"A1\",\"A2\",\"A3\",\"A3.5\",\"B1\",\"C1\",\"HT\",\"LT\",\"MO\"]')
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query("INSERT INTO _migrations (version, name) VALUES (1, 'initial_schema')")
        .execute(db.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    tracing::info!("Migration v1 applied successfully");
    Ok(())
}

// ─── v2 — Library index table ─────────────────────────────────────────────

async fn v2_library_index(db: &Database) -> Result<()> {
    tracing::info!("Applying migration v2: library_index");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS library_index (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            manufacturer TEXT,
            lib_type     TEXT NOT NULL,   -- 'implant', 'tooth', 'bar', 'attachment'
            file_path    TEXT NOT NULL,
            is_active    INTEGER NOT NULL DEFAULT 1,
            indexed_at   DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_lib_type ON library_index(lib_type);
        CREATE INDEX IF NOT EXISTS idx_lib_mfr  ON library_index(manufacturer);
        ",
    )
    .execute(db.pool())
    .await
    .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    sqlx::query("INSERT INTO _migrations (version, name) VALUES (2, 'library_index')")
        .execute(db.pool())
        .await
        .map_err(|e| cad_core::CadError::Database(e.to_string()))?;

    tracing::info!("Migration v2 applied successfully");
    Ok(())
}
