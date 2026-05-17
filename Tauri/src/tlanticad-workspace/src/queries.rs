//! Predefined SQL queries for common operations

/// Get project count by status
pub const PROJECT_COUNT_BY_STATUS: &str = r#"
SELECT status, COUNT(*) as count 
FROM projects 
GROUP BY status
"#;

/// Get recent projects
pub const RECENT_PROJECTS: &str = r#"
SELECT p.*, pt.first_name, pt.last_name, d.name as dentist_name
FROM projects p
JOIN patients pt ON p.patient_id = pt.id
JOIN dentists d ON p.dentist_id = d.id
ORDER BY p.modified_at DESC
LIMIT ?1
"#;

/// Get project with all related data
pub const PROJECT_FULL: &str = r#"
SELECT 
    p.*,
    pt.first_name as patient_first,
    pt.last_name as patient_last,
    pt.date_of_birth,
    d.name as dentist_name,
    d.clinic,
    d.email as dentist_email
FROM projects p
JOIN patients pt ON p.patient_id = pt.id
JOIN dentists d ON p.dentist_id = d.id
WHERE p.id = ?1
"#;

/// Get teeth for project
pub const PROJECT_TEETH: &str = r#"
SELECT * FROM teeth WHERE project_id = ?1 ORDER BY tooth_number
"#;

/// Get scans for project
pub const PROJECT_SCANS: &str = r#"
SELECT * FROM scans WHERE project_id = ?1 ORDER BY import_date
"#;

/// Get designs for project
pub const PROJECT_DESIGNS: &str = r#"
SELECT * FROM designs WHERE project_id = ?1 ORDER BY created_at
"#;

/// Search patients
pub const SEARCH_PATIENTS: &str = r#"
SELECT * FROM patients 
WHERE first_name LIKE ?1 OR last_name LIKE ?1 OR patient_id LIKE ?1
ORDER BY last_name, first_name
LIMIT ?2
"#;

/// Get activity log for project
pub const PROJECT_ACTIVITY: &str = r#"
SELECT * FROM activity_log 
WHERE project_id = ?1 
ORDER BY timestamp DESC
LIMIT ?2
"#;

/// Update project modified timestamp
pub const TOUCH_PROJECT: &str = r#"
UPDATE projects SET modified_at = CURRENT_TIMESTAMP WHERE id = ?1
"#;

/// Get available materials for work type
pub const MATERIALS_FOR_WORK_TYPE: &str = r#"
SELECT m.* FROM materials m
JOIN work_types w ON w.id = ?1
WHERE m.is_active = 1
ORDER BY m.name
"#;
