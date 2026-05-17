// AR-V381 — Parameters store (Tauri command surface).
//
// Persistent SQLite-backed parameter store. Reuses the same `tlanticad.db` file as
// `case_repository` (one DB per app data dir).
//
// Schema (created on first use):
//   CREATE TABLE tlanticad_parameters (
//       scope         TEXT PRIMARY KEY,   -- 'global', 'case:<uuid>', 'tooth:<fdi>'
//       payload_json  TEXT NOT NULL,
//       version       INTEGER NOT NULL DEFAULT 1,
//       updated_at    TEXT NOT NULL
//   );
//
// Five commands:
//   * `parameters_load`        — get JSON payload for a scope (returns defaults if absent).
//   * `parameters_save`        — upsert JSON payload.
//   * `parameters_reset`       — delete row → load returns defaults next time.
//   * `parameters_list_scopes` — list every scope persisted (UI scope selector).
//   * `parameters_export_json` — dump every row to a single JSON file.

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const SCHEMA_SQL: &str = "
    CREATE TABLE IF NOT EXISTS tlanticad_parameters (
        scope        TEXT PRIMARY KEY,
        payload_json TEXT NOT NULL,
        version      INTEGER NOT NULL DEFAULT 1,
        updated_at   TEXT NOT NULL
    );
";

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ParametersError {
    #[error("filesystem: {message}")]
    Fs { message: String },
    #[error("sqlite: {message}")]
    Sqlite { message: String },
    #[error("invalid json: {message}")]
    Json { message: String },
    #[error("scope must not be empty")]
    EmptyScope,
}

fn database_path(app: &AppHandle) -> Result<PathBuf, ParametersError> {
    let root = app.path().app_data_dir().map_err(|e| ParametersError::Fs {
        message: format!("app_data_dir: {e}"),
    })?;
    fs::create_dir_all(&root).map_err(|e| ParametersError::Fs {
        message: format!("create {}: {}", root.display(), e),
    })?;
    Ok(root.join("tlanticad.db"))
}

fn open(app: &AppHandle) -> Result<Connection, ParametersError> {
    let path = database_path(app)?;
    let conn = Connection::open(&path).map_err(|e| ParametersError::Sqlite {
        message: format!("open {}: {}", path.display(), e),
    })?;
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| ParametersError::Sqlite {
            message: format!("schema init: {e}"),
        })?;
    Ok(conn)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterRecord {
    pub scope: String,
    pub payload_json: String,
    pub version: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadRequest {
    pub scope: String,
    /// Default JSON payload returned when the scope row does not exist. If omitted, returns
    /// `{}` so the frontend can layer its own defaults.
    #[serde(default)]
    pub default_json: Option<String>,
}

#[tauri::command]
pub fn parameters_load(
    app: AppHandle,
    request: LoadRequest,
) -> Result<ParameterRecord, ParametersError> {
    if request.scope.trim().is_empty() {
        return Err(ParametersError::EmptyScope);
    }
    let conn = open(&app)?;
    let row = conn
        .query_row(
            "SELECT scope, payload_json, version, updated_at FROM tlanticad_parameters WHERE scope = ?1",
            params![request.scope],
            |row| {
                Ok(ParameterRecord {
                    scope: row.get(0)?,
                    payload_json: row.get(1)?,
                    version: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| ParametersError::Sqlite {
            message: format!("select: {e}"),
        })?;
    Ok(row.unwrap_or_else(|| ParameterRecord {
        scope: request.scope.clone(),
        payload_json: request.default_json.unwrap_or_else(|| "{}".to_string()),
        version: 0,
        updated_at: Utc::now().to_rfc3339(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRequest {
    pub scope: String,
    pub payload_json: String,
}

#[tauri::command]
pub fn parameters_save(
    app: AppHandle,
    request: SaveRequest,
) -> Result<ParameterRecord, ParametersError> {
    if request.scope.trim().is_empty() {
        return Err(ParametersError::EmptyScope);
    }
    // Validate JSON to fail loudly instead of corrupting the row.
    serde_json::from_str::<serde_json::Value>(&request.payload_json).map_err(|e| {
        ParametersError::Json {
            message: format!("payload not valid JSON: {e}"),
        }
    })?;
    let conn = open(&app)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO tlanticad_parameters (scope, payload_json, version, updated_at)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(scope) DO UPDATE SET
             payload_json = excluded.payload_json,
             version = tlanticad_parameters.version + 1,
             updated_at = excluded.updated_at",
        params![request.scope, request.payload_json, now],
    )
    .map_err(|e| ParametersError::Sqlite {
        message: format!("upsert: {e}"),
    })?;
    let record = conn
        .query_row(
            "SELECT scope, payload_json, version, updated_at FROM tlanticad_parameters WHERE scope = ?1",
            params![request.scope],
            |row| {
                Ok(ParameterRecord {
                    scope: row.get(0)?,
                    payload_json: row.get(1)?,
                    version: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .map_err(|e| ParametersError::Sqlite {
            message: format!("read-back: {e}"),
        })?;
    Ok(record)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetRequest {
    pub scope: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetResponse {
    pub scope: String,
    pub deleted: bool,
}

#[tauri::command]
pub fn parameters_reset(
    app: AppHandle,
    request: ResetRequest,
) -> Result<ResetResponse, ParametersError> {
    if request.scope.trim().is_empty() {
        return Err(ParametersError::EmptyScope);
    }
    let conn = open(&app)?;
    let rows = conn
        .execute(
            "DELETE FROM tlanticad_parameters WHERE scope = ?1",
            params![request.scope],
        )
        .map_err(|e| ParametersError::Sqlite {
            message: format!("delete: {e}"),
        })?;
    Ok(ResetResponse {
        scope: request.scope,
        deleted: rows > 0,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeSummary {
    pub scope: String,
    pub version: i64,
    pub updated_at: String,
}

#[tauri::command]
pub fn parameters_list_scopes(app: AppHandle) -> Result<Vec<ScopeSummary>, ParametersError> {
    let conn = open(&app)?;
    let mut stmt = conn
        .prepare("SELECT scope, version, updated_at FROM tlanticad_parameters ORDER BY scope")
        .map_err(|e| ParametersError::Sqlite {
            message: format!("prepare: {e}"),
        })?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ScopeSummary {
                scope: row.get(0)?,
                version: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| ParametersError::Sqlite {
            message: format!("query: {e}"),
        })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| ParametersError::Sqlite {
            message: format!("row: {e}"),
        })?);
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub output: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResponse {
    pub output: PathBuf,
    pub records: usize,
}

#[tauri::command]
pub fn parameters_export_json(
    app: AppHandle,
    request: ExportRequest,
) -> Result<ExportResponse, ParametersError> {
    let conn = open(&app)?;
    let mut stmt = conn
        .prepare("SELECT scope, payload_json, version, updated_at FROM tlanticad_parameters")
        .map_err(|e| ParametersError::Sqlite {
            message: format!("prepare: {e}"),
        })?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ParameterRecord {
                scope: row.get(0)?,
                payload_json: row.get(1)?,
                version: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| ParametersError::Sqlite {
            message: format!("query: {e}"),
        })?;
    let records: Vec<ParameterRecord> = rows.filter_map(|r| r.ok()).collect();
    let json = serde_json::to_string_pretty(&records).map_err(|e| ParametersError::Json {
        message: format!("serialise: {e}"),
    })?;
    if let Some(parent) = request.output.parent() {
        fs::create_dir_all(parent).map_err(|e| ParametersError::Fs {
            message: format!("create {}: {}", parent.display(), e),
        })?;
    }
    fs::write(&request.output, json).map_err(|e| ParametersError::Fs {
        message: format!("write {}: {}", request.output.display(), e),
    })?;
    let count = records.len();
    Ok(ExportResponse {
        output: request.output,
        records: count,
    })
}
