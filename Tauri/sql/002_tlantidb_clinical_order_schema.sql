-- TlantiDB clinical-order schema.
--
-- Inspired by DentalDB-style case organization:
-- laboratory -> practice/client -> patient/technician -> case/treatment -> tooth work.
-- This is intentionally not a clone of exocad DentalDB table names or numeric
-- composite IDs. TlantiCAD keeps UUID/text IDs, JSON metadata where it helps
-- offline sync, and explicit module/workload fields for the workstation shell.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS practices (
  id TEXT PRIMARY KEY,
  laboratory_id TEXT REFERENCES laboratories(id) ON DELETE SET NULL,
  external_ref TEXT,
  name TEXT NOT NULL,
  language TEXT NOT NULL DEFAULT 'es-MX',
  country_code TEXT,
  city TEXT,
  postal_code TEXT,
  street TEXT,
  email TEXT,
  flags INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(laboratory_id, external_ref)
);

CREATE TABLE IF NOT EXISTS patients (
  id TEXT PRIMARY KEY,
  practice_id TEXT REFERENCES practices(id) ON DELETE SET NULL,
  external_ref TEXT,
  first_name TEXT,
  last_name TEXT,
  display_name TEXT NOT NULL,
  date_of_birth TEXT,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(practice_id, external_ref)
);

CREATE TABLE IF NOT EXISTS technicians (
  id TEXT PRIMARY KEY,
  laboratory_id TEXT REFERENCES laboratories(id) ON DELETE SET NULL,
  external_ref TEXT,
  display_name TEXT NOT NULL,
  email TEXT,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(laboratory_id, external_ref)
);

CREATE TABLE IF NOT EXISTS work_type_catalog (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  category TEXT NOT NULL,
  legacy_restoration_type TEXT,
  default_minutes INTEGER NOT NULL DEFAULT 0,
  color_hex TEXT,
  is_active INTEGER NOT NULL DEFAULT 1,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS material_catalog (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  material_type TEXT NOT NULL,
  manufacturer TEXT,
  available_shades_json TEXT,
  is_active INTEGER NOT NULL DEFAULT 1,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS case_order_context (
  case_id TEXT PRIMARY KEY REFERENCES cases(id) ON DELETE CASCADE,
  practice_id TEXT REFERENCES practices(id) ON DELETE SET NULL,
  patient_id TEXT REFERENCES patients(id) ON DELETE SET NULL,
  technician_id TEXT REFERENCES technicians(id) ON DELETE SET NULL,
  order_number TEXT,
  due_at TEXT,
  locked_by TEXT,
  imported_from_path TEXT,
  imported_order_id TEXT,
  workload_id TEXT,
  workload_label TEXT,
  module_target TEXT,
  work_parameters_hash TEXT,
  work_parameters_signature TEXT,
  status_detail TEXT,
  created_by_hash TEXT,
  last_saved_by_hash TEXT,
  metadata_json TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS case_tooth_work (
  id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
  tooth_code TEXT NOT NULL,
  jaw TEXT CHECK(jaw IN ('upper', 'lower')),
  work_type_id TEXT REFERENCES work_type_catalog(id) ON DELETE SET NULL,
  material_id TEXT REFERENCES material_catalog(id) ON DELETE SET NULL,
  restoration_type TEXT,
  material_type TEXT,
  shade TEXT,
  production_method TEXT,
  implant_mode TEXT,
  is_antagonist INTEGER NOT NULL DEFAULT 0,
  is_selected INTEGER NOT NULL DEFAULT 1,
  flags INTEGER NOT NULL DEFAULT 0,
  notes TEXT,
  work_time_minutes INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(case_id, tooth_code)
);

CREATE TABLE IF NOT EXISTS tooth_work_parameters (
  id TEXT PRIMARY KEY,
  tooth_work_id TEXT NOT NULL REFERENCES case_tooth_work(id) ON DELETE CASCADE,
  parameter_key TEXT NOT NULL,
  value_kind TEXT NOT NULL CHECK(value_kind IN ('text', 'number', 'boolean', 'json')),
  text_value TEXT,
  number_value REAL,
  boolean_value INTEGER,
  json_value TEXT,
  source TEXT NOT NULL DEFAULT 'tlanticad',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(tooth_work_id, parameter_key)
);

CREATE TABLE IF NOT EXISTS tooth_work_parameter_dependencies (
  id TEXT PRIMARY KEY,
  parent_parameter_id TEXT NOT NULL REFERENCES tooth_work_parameters(id) ON DELETE CASCADE,
  dependent_key TEXT NOT NULL,
  dependent_value TEXT,
  dependent_kind TEXT NOT NULL DEFAULT 'visibility',
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS work_parameter_blobs (
  hash TEXT PRIMARY KEY,
  content BLOB NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'application/json',
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS work_parameter_versions (
  id TEXT PRIMARY KEY,
  case_id TEXT REFERENCES cases(id) ON DELETE CASCADE,
  blob_hash TEXT NOT NULL REFERENCES work_parameter_blobs(hash) ON DELETE RESTRICT,
  source TEXT NOT NULL CHECK(source IN ('local', 'import', 'share', 'generated')),
  source_label TEXT,
  last_modified_at TEXT NOT NULL,
  metadata_json TEXT
);

CREATE TABLE IF NOT EXISTS treatment_custom_fields (
  id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
  field_key TEXT NOT NULL,
  field_value TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(case_id, field_key)
);

CREATE INDEX IF NOT EXISTS idx_practices_laboratory ON practices(laboratory_id);
CREATE INDEX IF NOT EXISTS idx_patients_practice ON patients(practice_id);
CREATE INDEX IF NOT EXISTS idx_patients_name ON patients(last_name, first_name, display_name);
CREATE INDEX IF NOT EXISTS idx_technicians_laboratory ON technicians(laboratory_id);
CREATE INDEX IF NOT EXISTS idx_case_order_practice ON case_order_context(practice_id);
CREATE INDEX IF NOT EXISTS idx_case_order_patient ON case_order_context(patient_id);
CREATE INDEX IF NOT EXISTS idx_case_tooth_work_case ON case_tooth_work(case_id);
CREATE INDEX IF NOT EXISTS idx_case_tooth_work_type ON case_tooth_work(work_type_id);
CREATE INDEX IF NOT EXISTS idx_tooth_work_parameters_work ON tooth_work_parameters(tooth_work_id);
CREATE INDEX IF NOT EXISTS idx_work_parameter_versions_case ON work_parameter_versions(case_id);

INSERT OR IGNORE INTO practices (
  id, laboratory_id, external_ref, name, language, country_code, city, postal_code, street, email, flags, created_at, updated_at
) VALUES (
  'practice-default', NULL, 'default', 'Default practice', 'es-MX', 'MX', '', '', '', NULL, 0, datetime('now'), datetime('now')
);

INSERT OR IGNORE INTO work_type_catalog (
  id, label, category, legacy_restoration_type, default_minutes, color_hex, is_active, metadata_json, created_at, updated_at
) VALUES
  ('anatomic-crown', 'Anatomic crown', 'crowns-copings', 'anatomic-crown', 35, '#7F4BD8', 1, NULL, datetime('now'), datetime('now')),
  ('coping', 'Coping', 'crowns-copings', 'anatomic-coping', 20, '#2BB6A4', 1, NULL, datetime('now'), datetime('now')),
  ('anatomic-pontic', 'Anatomic pontic', 'pontics-mockup', 'pontic', 40, '#A03B4A', 1, NULL, datetime('now'), datetime('now')),
  ('inlay-onlay', 'Inlay / Onlay', 'inlays-onlays-veneers', 'inlay-onlay', 25, '#1E7F5F', 1, NULL, datetime('now'), datetime('now')),
  ('veneer', 'Veneer', 'inlays-onlays-veneers', 'veneer', 30, '#3CA8D4', 1, NULL, datetime('now'), datetime('now')),
  ('bite-splint', 'Bite splint', 'removables-appliances', 'bite-splint', 55, '#B7791F', 1, NULL, datetime('now'), datetime('now')),
  ('implant-crown', 'Implant crown', 'crowns-copings', 'implant-crown', 55, '#F59E0B', 1, NULL, datetime('now'), datetime('now'));

INSERT OR IGNORE INTO material_catalog (
  id, label, material_type, manufacturer, available_shades_json, is_active, metadata_json, created_at, updated_at
) VALUES
  ('zirconia', 'Zirconia', 'zirconia', 'Generic', '["A1","A2","A3","A3.5","B1","B2","C1","C2","D3"]', 1, NULL, datetime('now'), datetime('now')),
  ('pmma', 'PMMA Temp', 'pmma', 'Generic', '["A1","A2","A3","B1","B2"]', 1, NULL, datetime('now'), datetime('now')),
  ('lithium-disilicate', 'Lithium disilicate', 'lithium-disilicate', 'Generic', '["A1","A2","A3","B1","C1","HT","LT"]', 1, NULL, datetime('now'), datetime('now')),
  ('titanium', 'Titanium Grade 5', 'titanium', 'Generic', NULL, 1, NULL, datetime('now'), datetime('now')),
  ('cobalt-chrome', 'Cobalt-Chrome', 'cobalt-chrome', 'Generic', NULL, 1, NULL, datetime('now'), datetime('now'));
