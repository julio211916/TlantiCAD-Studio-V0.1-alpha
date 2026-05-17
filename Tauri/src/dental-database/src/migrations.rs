//! Database migrations

use rusqlite::Connection;
use tracing::{info, warn};

use crate::{schema, DbError, DbResult};

/// Migration version
const CURRENT_VERSION: i32 = 4;

/// Run all pending migrations
pub fn run_all(conn: &Connection) -> DbResult<()> {
    // Create migrations table if not exists
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )
        "#,
        [],
    )?;
    
    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    
    info!("Current database version: {}", current_version);
    
    if current_version >= CURRENT_VERSION {
        info!("Database is up to date");
        return Ok(());
    }
    
    // Run migrations
    for version in (current_version + 1)..=CURRENT_VERSION {
        info!("Running migration version {}", version);
        run_migration(conn, version)?;
        
        // Record migration
        conn.execute(
            "INSERT INTO _migrations (version, name, applied_at) VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![version, migration_name(version)],
        )?;
    }
    
    info!("All migrations completed");
    Ok(())
}

/// Get migration name by version
fn migration_name(version: i32) -> &'static str {
    match version {
        1 => "initial_schema",
        2 => "add_pin_auth",
        3 => "odontogram_tables",
        4 => "support_ticketing",
        _ => "unknown",
    }
}

/// Run a specific migration
fn run_migration(conn: &Connection, version: i32) -> DbResult<()> {
    match version {
        1 => migration_v1_initial_schema(conn),
        2 => migration_v2_add_pin_auth(conn),
        3 => migration_v3_add_odontogram_tables(conn),
        4 => migration_v4_add_support_ticketing(conn),
        _ => Err(DbError::MigrationError(format!(
            "Unknown migration version: {}",
            version
        ))),
    }
}

/// Migration v4: Add support ticketing tables
fn migration_v4_add_support_ticketing(conn: &Connection) -> DbResult<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS support_tickets (
            id TEXT PRIMARY KEY,
            subject TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            priority TEXT NOT NULL DEFAULT 'medium',
            customer_name TEXT NOT NULL,
            customer_email TEXT NOT NULL,
            assigned_to TEXT REFERENCES users(id),
            assigned_to_name TEXT,
            first_response_at TEXT,
            last_response_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS support_ticket_messages (
            id TEXT PRIMARY KEY,
            ticket_id TEXT NOT NULL REFERENCES support_tickets(id) ON DELETE CASCADE,
            body TEXT NOT NULL,
            author_type TEXT NOT NULL,
            author_id TEXT,
            author_name TEXT NOT NULL,
            is_internal INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS support_canned_responses (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_support_tickets_status ON support_tickets(status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_support_tickets_priority ON support_tickets(priority)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_support_tickets_customer_email ON support_tickets(customer_email)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_support_tickets_updated_at ON support_tickets(updated_at)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_support_ticket_messages_ticket_id ON support_ticket_messages(ticket_id)",
        [],
    )?;

    let now = chrono::Utc::now().to_rfc3339();
    let canned = vec![
        (
            "acknowledge",
            "Confirmación de recepción",
            "Gracias por contactarnos. Ya recibimos tu solicitud y un agente la está revisando.",
        ),
        (
            "request_details",
            "Solicitud de más información",
            "Para poder ayudarte mejor, ¿nos puedes compartir más detalles del problema y desde cuándo ocurre?",
        ),
        (
            "resolved",
            "Resolución de caso",
            "El caso quedó resuelto. Si necesitas algo más, responde este ticket y con gusto te ayudamos.",
        ),
    ];

    for (id, title, body) in canned {
        conn.execute(
            "INSERT OR IGNORE INTO support_canned_responses (id, title, body, is_active, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
            rusqlite::params![id, title, body, now],
        )?;
    }

    Ok(())
}

/// Migration v3: Add odontogram history and periodontograms
fn migration_v3_add_odontogram_tables(conn: &Connection) -> DbResult<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS odontogram_history (
            id TEXT PRIMARY KEY,
            patient_id TEXT NOT NULL REFERENCES patients(id),
            tooth_number INTEGER NOT NULL,
            previous_condition TEXT NOT NULL,
            new_condition TEXT NOT NULL,
            change_reason TEXT,
            changed_by TEXT NOT NULL REFERENCES users(id),
            changed_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS periodontograms (
            id TEXT PRIMARY KEY,
            patient_id TEXT NOT NULL UNIQUE REFERENCES patients(id),
            data TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            updated_by TEXT NOT NULL REFERENCES users(id)
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_odontogram_history_patient ON odontogram_history(patient_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_periodontograms_patient ON periodontograms(patient_id)",
        [],
    )?;

    Ok(())
}

/// Migration v2: Add PIN authentication
fn migration_v2_add_pin_auth(conn: &Connection) -> DbResult<()> {
    // Add pin_hash column to users table
    if let Err(e) = conn.execute(
        "ALTER TABLE users ADD COLUMN pin_hash TEXT",
        [],
    ) {
        warn!("Could not add pin_hash column (may already exist): {}", e);
    }
    
    // Make password_hash optional by recreating table with nullable column
    // This is a SQLite limitation - we can't alter column constraints
    // For new installs, the schema already has it nullable
    
    // Remove the default admin user since we'll use onboarding
    conn.execute("DELETE FROM users WHERE username = 'admin'", [])?;
    
    // Remove the default clinic since onboarding will create it
    conn.execute("DELETE FROM clinics WHERE name = 'Clínica Principal'", [])?;
    
    info!("Migration v2 completed: PIN authentication enabled");
    Ok(())
}

/// Migration v1: Initial schema
fn migration_v1_initial_schema(conn: &Connection) -> DbResult<()> {
    // Create all tables
    for sql in schema::CREATE_TABLES {
        conn.execute(sql, []).map_err(|e| {
            DbError::MigrationError(format!("Failed to create table: {}", e))
        })?;
    }
    
    // Create indexes
    for sql in schema::CREATE_INDEXES {
        if let Err(e) = conn.execute(sql, []) {
            warn!("Failed to create index: {} - {}", sql, e);
        }
    }
    
    // Insert default data
    insert_default_data(conn)?;
    
    Ok(())
}

/// Insert default/seed data
fn insert_default_data(conn: &Connection) -> DbResult<()> {
    use chrono::Utc;
    use uuid::Uuid;
    
    let now = Utc::now().to_rfc3339();
    
    // Default admin user (password: admin123)
    let admin_id = Uuid::new_v4().to_string();
    conn.execute(
        r#"
        INSERT OR IGNORE INTO users (id, username, email, password_hash, first_name, last_name, role, active, created_at, updated_at)
        VALUES (?1, 'admin', 'admin@tlantistudio.com', '$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0$hash', 'Admin', 'System', 'admin', 1, ?2, ?2)
        "#,
        rusqlite::params![admin_id, now],
    )?;
    
    // Default clinic
    let clinic_id = Uuid::new_v4().to_string();
    conn.execute(
        r#"
        INSERT OR IGNORE INTO clinics (id, name, address, city, phone, is_main, active, created_at, updated_at)
        VALUES (?1, 'Clínica Principal', 'Dirección', 'Ciudad', '555-0000', 1, 1, ?2, ?2)
        "#,
        rusqlite::params![clinic_id, now],
    )?;
    
    // Insert common dental procedures
    insert_procedures(conn, &now)?;
    
    // Insert default settings
    let settings = vec![
        ("app.name", "TlantiStudio Dental", "Application name"),
        ("app.version", "1.0.0", "Application version"),
        ("invoice.prefix", "INV", "Invoice number prefix"),
        ("invoice.counter", "1", "Invoice number counter"),
        ("patient.prefix", "PAC", "Patient number prefix"),
        ("patient.counter", "1", "Patient number counter"),
        ("appointment.slot_duration", "30", "Default appointment slot in minutes"),
        ("appointment.reminder_hours", "24,2", "Hours before appointment to send reminders"),
        ("tax.default_rate", "16", "Default tax rate percentage"),
        ("currency.code", "MXN", "Currency code"),
        ("currency.symbol", "$", "Currency symbol"),
    ];
    
    for (key, value, description) in settings {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value, description, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![key, value, description, now],
        )?;
    }
    
    Ok(())
}

/// Insert common dental procedures
fn insert_procedures(conn: &Connection, now: &str) -> DbResult<()> {
    use uuid::Uuid;
    
    let procedures = vec![
        // Diagnostic
        ("D0120", "Evaluación Oral Periódica", "diagnostic", "300", 15),
        ("D0140", "Evaluación Oral Limitada", "diagnostic", "250", 15),
        ("D0150", "Evaluación Oral Completa", "diagnostic", "500", 30),
        ("D0210", "Radiografías (Serie Completa)", "diagnostic", "800", 20),
        ("D0220", "Radiografía Periapical", "diagnostic", "100", 5),
        ("D0274", "Radiografías Bite-Wing (4)", "diagnostic", "350", 15),
        ("D0330", "Radiografía Panorámica", "diagnostic", "450", 15),
        
        // Preventive
        ("D1110", "Limpieza Dental (Adulto)", "preventive", "600", 45),
        ("D1120", "Limpieza Dental (Niño)", "preventive", "400", 30),
        ("D1206", "Aplicación de Flúor", "preventive", "200", 10),
        ("D1351", "Sellador (Por Diente)", "preventive", "250", 15),
        
        // Restorative
        ("D2140", "Amalgama - 1 Superficie", "restorative", "450", 30),
        ("D2150", "Amalgama - 2 Superficies", "restorative", "600", 40),
        ("D2330", "Resina - 1 Superficie Anterior", "restorative", "550", 30),
        ("D2331", "Resina - 2 Superficies Anterior", "restorative", "700", 40),
        ("D2391", "Resina - 1 Superficie Posterior", "restorative", "600", 35),
        ("D2392", "Resina - 2 Superficies Posterior", "restorative", "800", 45),
        ("D2393", "Resina - 3 Superficies Posterior", "restorative", "1000", 55),
        ("D2740", "Corona de Porcelana", "prosthodontic", "6000", 90),
        ("D2750", "Corona Metal-Porcelana", "prosthodontic", "5000", 90),
        
        // Endodontic
        ("D3310", "Endodoncia Anterior", "endodontic", "3500", 60),
        ("D3320", "Endodoncia Premolar", "endodontic", "4500", 75),
        ("D3330", "Endodoncia Molar", "endodontic", "5500", 90),
        
        // Periodontic
        ("D4341", "Raspado y Alisado (Por Cuadrante)", "periodontic", "1200", 45),
        ("D4910", "Mantenimiento Periodontal", "periodontic", "800", 45),
        
        // Oral Surgery
        ("D7140", "Extracción Simple", "oral_surgery", "800", 20),
        ("D7210", "Extracción Quirúrgica", "oral_surgery", "1500", 30),
        ("D7220", "Tercero Molar (Tejido Blando)", "oral_surgery", "2500", 45),
        ("D7230", "Tercero Molar (Parcialmente Impactado)", "oral_surgery", "3500", 60),
        ("D7240", "Tercero Molar (Totalmente Impactado)", "oral_surgery", "4500", 75),
        
        // Prosthodontic
        ("D5110", "Dentadura Completa Superior", "prosthodontic", "8000", 120),
        ("D5120", "Dentadura Completa Inferior", "prosthodontic", "8000", 120),
        ("D5211", "Dentadura Parcial Superior", "prosthodontic", "6000", 90),
        ("D5212", "Dentadura Parcial Inferior", "prosthodontic", "6000", 90),
        
        // Cosmetic
        ("COSM01", "Blanqueamiento Dental", "cosmetic", "3500", 60),
        ("COSM02", "Carilla de Porcelana", "cosmetic", "7000", 90),
        ("COSM03", "Carilla de Resina", "cosmetic", "2500", 60),
        
        // Implant
        ("D6010", "Implante Dental", "implant", "18000", 120),
        ("D6058", "Corona sobre Implante", "implant", "8000", 90),
    ];
    
    for (code, name, category, price, duration) in procedures {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            r#"
            INSERT OR IGNORE INTO procedures (id, code, name, category, default_price, duration_minutes, active, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)
            "#,
            rusqlite::params![id, code, name, category, price, duration, now],
        )?;
    }
    
    Ok(())
}

/// Check if a migration has been applied
pub fn is_migration_applied(conn: &Connection, version: i32) -> DbResult<bool> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM _migrations WHERE version = ?1",
        [version],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Get list of applied migrations
pub fn get_applied_migrations(conn: &Connection) -> DbResult<Vec<(i32, String, String)>> {
    let mut stmt = conn.prepare("SELECT version, name, applied_at FROM _migrations ORDER BY version")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;
    
    let mut migrations = Vec::new();
    for row in rows {
        migrations.push(row?);
    }
    Ok(migrations)
}

/// Rollback last migration (for development)
#[cfg(debug_assertions)]
pub fn rollback_last(conn: &Connection) -> DbResult<()> {
    let version: Option<i32> = conn.query_row(
        "SELECT MAX(version) FROM _migrations",
        [],
        |row| row.get(0),
    ).ok();
    
    if let Some(v) = version {
        warn!("Rolling back migration version {}", v);
        conn.execute("DELETE FROM _migrations WHERE version = ?1", [v])?;
    }
    
    Ok(())
}
