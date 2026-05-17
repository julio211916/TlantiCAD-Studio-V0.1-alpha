use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use uuid::Uuid;

use crate::storage_layout;

const CORE_SCHEMA_SQL: &str = include_str!("../sql/001_core_schema.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalJobDto {
    pub id: String,
    pub case_id: Option<String>,
    pub kind: String,
    pub status: String,
    pub progress: f64,
    pub vendor: Option<String>,
    pub model_id: Option<String>,
    pub checkpoint_sha256: Option<String>,
    pub params_json: String,
    pub result_json: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalJobRecordRequest {
    pub id: Option<String>,
    pub case_id: Option<String>,
    pub kind: String,
    pub status: Option<String>,
    pub progress: Option<f64>,
    pub vendor: Option<String>,
    pub model_id: Option<String>,
    pub checkpoint_sha256: Option<String>,
    pub params_json: Option<String>,
    pub result_json: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalArtifactDto {
    pub id: String,
    pub job_id: Option<String>,
    pub case_id: Option<String>,
    pub asset_id: Option<String>,
    pub artifact_type: String,
    pub storage_path: Option<String>,
    pub checksum_sha256: Option<String>,
    pub metadata_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalArtifactRecordRequest {
    pub id: Option<String>,
    pub job_id: Option<String>,
    pub case_id: Option<String>,
    pub asset_id: Option<String>,
    pub artifact_type: String,
    pub storage_path: Option<String>,
    pub checksum_sha256: Option<String>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalCommandEventDto {
    pub id: String,
    pub case_id: Option<String>,
    pub module_id: String,
    pub tool_id: String,
    pub event_type: String,
    pub command_json: String,
    pub inverse_command_json: Option<String>,
    pub target_event_id: Option<String>,
    pub asset_ids_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalCommandRecordRequest {
    pub id: Option<String>,
    pub case_id: Option<String>,
    pub module_id: String,
    pub tool_id: String,
    pub command_json: Option<String>,
    pub inverse_command_json: Option<String>,
    pub target_event_id: Option<String>,
    pub asset_ids_json: Option<String>,
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    storage_layout::database_path(app)
}

fn open_connection(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create app data directory: {error}"))?;
    }
    let conn = Connection::open(&path)
        .map_err(|error| format!("Could not open SQLite database {}: {error}", path.display()))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;",
    )
    .map_err(|error| format!("Could not configure SQLite database: {error}"))?;
    conn.execute_batch(CORE_SCHEMA_SQL)
        .map_err(|error| format!("Could not apply core schema: {error}"))?;
    Ok(conn)
}

fn now_sql(conn: &Connection) -> Result<String, String> {
    conn.query_row("SELECT datetime('now')", [], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not read SQLite clock: {error}"))
}

fn validate_job_status(status: &str) -> Result<(), String> {
    match status {
        "queued" | "running" | "completed" | "failed" | "cancelled" | "manual-review" => Ok(()),
        _ => Err(format!("Unsupported clinical job status: {status}")),
    }
}

fn validate_command_event_type(event_type: &str) -> Result<(), String> {
    match event_type {
        "record" | "undo" | "redo" => Ok(()),
        _ => Err(format!(
            "Unsupported clinical command event type: {event_type}"
        )),
    }
}

fn record_command_event(
    app: AppHandle,
    request: ClinicalCommandRecordRequest,
    event_type: &str,
) -> Result<ClinicalCommandEventDto, String> {
    validate_command_event_type(event_type)?;
    let conn = open_connection(&app)?;
    let now = now_sql(&conn)?;
    let id = request.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let command_json = request.command_json.unwrap_or_else(|| "{}".to_string());
    let asset_ids_json = request.asset_ids_json.unwrap_or_else(|| "[]".to_string());

    conn.execute(
        r#"
        INSERT INTO clinical_command_events (
            id, case_id, module_id, tool_id, event_type, command_json,
            inverse_command_json, target_event_id, asset_ids_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            id,
            request.case_id,
            request.module_id,
            request.tool_id,
            event_type,
            command_json,
            request.inverse_command_json,
            request.target_event_id,
            asset_ids_json,
            now,
        ],
    )
    .map_err(|error| format!("Could not record clinical command event: {error}"))?;

    clinical_command_event_get(app, id)
}

#[tauri::command]
pub fn clinical_command_record(
    app: AppHandle,
    request: ClinicalCommandRecordRequest,
) -> Result<ClinicalCommandEventDto, String> {
    record_command_event(app, request, "record")
}

#[tauri::command]
pub fn clinical_command_undo(
    app: AppHandle,
    request: ClinicalCommandRecordRequest,
) -> Result<ClinicalCommandEventDto, String> {
    record_command_event(app, request, "undo")
}

#[tauri::command]
pub fn clinical_command_redo(
    app: AppHandle,
    request: ClinicalCommandRecordRequest,
) -> Result<ClinicalCommandEventDto, String> {
    record_command_event(app, request, "redo")
}

#[tauri::command]
pub fn clinical_command_event_get(
    app: AppHandle,
    event_id: String,
) -> Result<ClinicalCommandEventDto, String> {
    let conn = open_connection(&app)?;
    conn.query_row(
        r#"
        SELECT id, case_id, module_id, tool_id, event_type, command_json,
               inverse_command_json, target_event_id, asset_ids_json, created_at
        FROM clinical_command_events
        WHERE id = ?1
        "#,
        [event_id],
        |row| {
            Ok(ClinicalCommandEventDto {
                id: row.get(0)?,
                case_id: row.get(1)?,
                module_id: row.get(2)?,
                tool_id: row.get(3)?,
                event_type: row.get(4)?,
                command_json: row.get(5)?,
                inverse_command_json: row.get(6)?,
                target_event_id: row.get(7)?,
                asset_ids_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        },
    )
    .map_err(|error| format!("Could not load clinical command event: {error}"))
}

#[tauri::command]
pub fn clinical_job_record(
    app: AppHandle,
    request: ClinicalJobRecordRequest,
) -> Result<ClinicalJobDto, String> {
    let conn = open_connection(&app)?;
    let now = now_sql(&conn)?;
    let id = request.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let status = request.status.unwrap_or_else(|| "queued".to_string());
    validate_job_status(&status)?;
    let progress = request.progress.unwrap_or(0.0).clamp(0.0, 1.0);
    let params_json = request.params_json.unwrap_or_else(|| "{}".to_string());

    conn.execute(
        r#"
        INSERT INTO clinical_jobs (
            id, case_id, kind, status, progress, vendor, model_id, checkpoint_sha256,
            params_json, result_json, error, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ON CONFLICT(id) DO UPDATE SET
            status = excluded.status,
            progress = excluded.progress,
            vendor = excluded.vendor,
            model_id = excluded.model_id,
            checkpoint_sha256 = excluded.checkpoint_sha256,
            params_json = excluded.params_json,
            result_json = excluded.result_json,
            error = excluded.error,
            updated_at = excluded.updated_at
        "#,
        params![
            id,
            request.case_id,
            request.kind,
            status,
            progress,
            request.vendor,
            request.model_id,
            request.checkpoint_sha256,
            params_json,
            request.result_json,
            request.error,
            now,
            now,
        ],
    )
    .map_err(|error| format!("Could not record clinical job: {error}"))?;

    clinical_job_get(app, id)
}

#[tauri::command]
pub fn clinical_job_get(app: AppHandle, job_id: String) -> Result<ClinicalJobDto, String> {
    let conn = open_connection(&app)?;
    conn.query_row(
        r#"
        SELECT id, case_id, kind, status, progress, vendor, model_id, checkpoint_sha256,
               params_json, result_json, error, created_at, updated_at
        FROM clinical_jobs
        WHERE id = ?1
        "#,
        [job_id],
        |row| {
            Ok(ClinicalJobDto {
                id: row.get(0)?,
                case_id: row.get(1)?,
                kind: row.get(2)?,
                status: row.get(3)?,
                progress: row.get(4)?,
                vendor: row.get(5)?,
                model_id: row.get(6)?,
                checkpoint_sha256: row.get(7)?,
                params_json: row.get(8)?,
                result_json: row.get(9)?,
                error: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        },
    )
    .map_err(|error| format!("Could not load clinical job: {error}"))
}

#[tauri::command]
pub fn clinical_job_list(
    app: AppHandle,
    case_id: Option<String>,
) -> Result<Vec<ClinicalJobDto>, String> {
    let conn = open_connection(&app)?;
    let sql = if case_id.is_some() {
        r#"
        SELECT id, case_id, kind, status, progress, vendor, model_id, checkpoint_sha256,
               params_json, result_json, error, created_at, updated_at
        FROM clinical_jobs
        WHERE case_id = ?1
        ORDER BY created_at DESC
        "#
    } else {
        r#"
        SELECT id, case_id, kind, status, progress, vendor, model_id, checkpoint_sha256,
               params_json, result_json, error, created_at, updated_at
        FROM clinical_jobs
        ORDER BY created_at DESC
        LIMIT 200
        "#
    };
    let mut stmt = conn
        .prepare(sql)
        .map_err(|error| format!("Could not prepare clinical job list: {error}"))?;

    let mapper = |row: &rusqlite::Row<'_>| {
        Ok(ClinicalJobDto {
            id: row.get(0)?,
            case_id: row.get(1)?,
            kind: row.get(2)?,
            status: row.get(3)?,
            progress: row.get(4)?,
            vendor: row.get(5)?,
            model_id: row.get(6)?,
            checkpoint_sha256: row.get(7)?,
            params_json: row.get(8)?,
            result_json: row.get(9)?,
            error: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    };

    let rows = if let Some(case_id) = case_id {
        stmt.query_map([case_id], mapper)
            .map_err(|error| format!("Could not query clinical jobs: {error}"))?
            .collect::<Result<Vec<_>, _>>()
    } else {
        stmt.query_map([], mapper)
            .map_err(|error| format!("Could not query clinical jobs: {error}"))?
            .collect::<Result<Vec<_>, _>>()
    };

    rows.map_err(|error| format!("Could not collect clinical jobs: {error}"))
}

#[tauri::command]
pub fn clinical_job_cancel(app: AppHandle, job_id: String) -> Result<ClinicalJobDto, String> {
    let conn = open_connection(&app)?;
    let now = now_sql(&conn)?;
    conn.execute(
        "UPDATE clinical_jobs SET status = 'cancelled', progress = 0, updated_at = ?1 WHERE id = ?2",
        params![now, job_id],
    )
    .map_err(|error| format!("Could not cancel clinical job: {error}"))?;
    clinical_job_get(app, job_id)
}

#[tauri::command]
pub fn clinical_artifact_record(
    app: AppHandle,
    request: ClinicalArtifactRecordRequest,
) -> Result<ClinicalArtifactDto, String> {
    let conn = open_connection(&app)?;
    let now = now_sql(&conn)?;
    let id = request.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let metadata_json = request.metadata_json.unwrap_or_else(|| "{}".to_string());

    conn.execute(
        r#"
        INSERT INTO clinical_artifacts (
            id, job_id, case_id, asset_id, artifact_type, storage_path, checksum_sha256,
            metadata_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(id) DO UPDATE SET
            job_id = excluded.job_id,
            case_id = excluded.case_id,
            asset_id = excluded.asset_id,
            artifact_type = excluded.artifact_type,
            storage_path = excluded.storage_path,
            checksum_sha256 = excluded.checksum_sha256,
            metadata_json = excluded.metadata_json
        "#,
        params![
            id,
            request.job_id,
            request.case_id,
            request.asset_id,
            request.artifact_type,
            request.storage_path,
            request.checksum_sha256,
            metadata_json,
            now,
        ],
    )
    .map_err(|error| format!("Could not record clinical artifact: {error}"))?;

    Ok(ClinicalArtifactDto {
        id,
        job_id: request.job_id,
        case_id: request.case_id,
        asset_id: request.asset_id,
        artifact_type: request.artifact_type,
        storage_path: request.storage_path,
        checksum_sha256: request.checksum_sha256,
        metadata_json,
        created_at: now,
    })
}
