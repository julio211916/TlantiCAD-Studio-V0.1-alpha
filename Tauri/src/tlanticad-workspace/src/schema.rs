//! Database schema - Replica la estructura de DentalDB

/// Tabla de proyectos/casos (Jobs)
pub const CREATE_PROJECTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    case_number TEXT UNIQUE NOT NULL,
    patient_id TEXT NOT NULL,
    dentist_id TEXT NOT NULL,
    technician_id TEXT,
    work_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'new',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,
    is_deleted BOOLEAN DEFAULT 0,
    is_imported BOOLEAN DEFAULT 0,
    import_source TEXT, -- ej. 'exocad', 'intraoral', 'local'
    global_shade TEXT,
    antagonist_scan_mode TEXT,
    notes TEXT,
    multidie_mode BOOLEAN DEFAULT 0,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (dentist_id) REFERENCES dentists(id),
    FOREIGN KEY (technician_id) REFERENCES technicians(id)
);

CREATE INDEX IF NOT EXISTS idx_projects_case_number ON projects(case_number);
CREATE INDEX IF NOT EXISTS idx_projects_patient ON projects(patient_id);
CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);
CREATE INDEX IF NOT EXISTS idx_projects_deleted ON projects(is_deleted);
CREATE INDEX IF NOT EXISTS idx_projects_dates ON projects(created_at);
"#;

/// Tabla de pacientes
pub const CREATE_PATIENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS patients (
    id TEXT PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    date_of_birth DATE,
    patient_id TEXT UNIQUE,
    phone TEXT,
    email TEXT,
    address TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_patients_name ON patients(last_name, first_name);
"#;

/// Tabla de dentistas/clínicas (Clientes)
pub const CREATE_DENTISTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS dentists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    clinic TEXT,
    email TEXT,
    phone TEXT,
    address TEXT,
    city TEXT,
    country TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_dentists_name ON dentists(name);
"#;

/// Tabla de Técnicos de Laboratorio
pub const CREATE_TECHNICIANS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS technicians (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_technicians_name ON technicians(name);
"#;

/// Tabla de dientes por proyecto
pub const CREATE_TEETH_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS teeth (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    tooth_number INTEGER NOT NULL,
    restoration_type TEXT, -- E.g. 'Anatomic crown', 'Coping', 'Pontic', 'Waxup'
    is_present BOOLEAN DEFAULT 1,
    is_prepared BOOLEAN DEFAULT 0,
    antagonist INTEGER,
    margin_line TEXT, -- JSON array of points
    design_id TEXT,
    material_id TEXT,
    shade TEXT,
    multidie_name TEXT, -- Ej. para el "die position to be used" en Multidie mode
    implant_type TEXT, -- E.g. 'Custom Abutment', 'Screw Retained', 'Stock'
    notes TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (material_id) REFERENCES materials(id),
    UNIQUE(project_id, tooth_number)
);

CREATE INDEX IF NOT EXISTS idx_teeth_project ON teeth(project_id);
"#;

/// Tabla de conectores entre dientes
pub const CREATE_CONNECTORS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS connectors (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    from_tooth INTEGER NOT NULL,
    to_tooth INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'green', -- 'green'(crear), 'grey'(no crear), 'red'(bloqueado)
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(project_id, from_tooth, to_tooth)
);

CREATE INDEX IF NOT EXISTS idx_connectors_project ON connectors(project_id);
"#;

/// Tabla de scans
pub const CREATE_SCANS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS scans (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    scan_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_hash TEXT,
    transformation TEXT, -- JSON matrix4
    is_visible BOOLEAN DEFAULT 1,
    opacity REAL DEFAULT 1.0,
    import_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_scans_project ON scans(project_id);
"#;

/// Tabla de diseños
pub const CREATE_DESIGNS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS designs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    design_type TEXT NOT NULL,
    tooth_number INTEGER,
    mesh_file_path TEXT,
    parameters TEXT, -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_designs_project ON designs(project_id);
"#;

/// Tabla de materiales
pub const CREATE_MATERIALS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS materials (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    material_type TEXT NOT NULL,
    manufacturer TEXT,
    available_shades TEXT, -- JSON array
    milling_params TEXT, -- JSON
    is_active BOOLEAN DEFAULT 1
);
"#;

/// Tabla de tipos de trabajo
pub const CREATE_WORK_TYPES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS work_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    available_processors TEXT, -- JSON array
    default_parameters TEXT -- JSON
);
"#;

/// Tabla de actividad/historial
pub const CREATE_ACTIVITY_LOG_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_activity_project ON activity_log(project_id);
"#;

/// Tabla de configuración del usuario
pub const CREATE_USER_SETTINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS user_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
"#;

/// Tabla de librerías de implantes
pub const CREATE_IMPLANT_LIBRARIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS implant_libraries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    manufacturer TEXT NOT NULL,
    connection_type TEXT,
    file_path TEXT NOT NULL,
    is_active BOOLEAN DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_implant_manufacturer ON implant_libraries(manufacturer);
"#;

/// Tabla de implantes
pub const CREATE_IMPLANTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS implants (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL,
    name TEXT NOT NULL,
    diameter REAL NOT NULL,
    length REAL NOT NULL,
    platform TEXT,
    restorative_components TEXT, -- JSON
    FOREIGN KEY (library_id) REFERENCES implant_libraries(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_implants_library ON implants(library_id);
"#;
