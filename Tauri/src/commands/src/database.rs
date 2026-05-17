//! Database-related Tauri commands

use database::SqliteDb;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use crate::{CommandError, CommandResult};

/// Database state
pub struct DatabaseState {
    pub sqlite: Arc<SqliteDb>,
}

/// Execute a raw SQL query (for development/debugging)
#[tauri::command]
pub async fn execute_sql(
    state: State<'_, DatabaseState>,
    query: String,
) -> CommandResult<Vec<Vec<serde_json::Value>>> {
    let conn = state.sqlite.get_connection()?;
    
    let mut stmt = conn.prepare(&query)
        .map_err(|e| CommandError {
            code: "SQL_ERROR".to_string(),
            message: e.to_string(),
        })?;

    let column_count = stmt.column_count();
    let _column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    let rows: Vec<Vec<serde_json::Value>> = stmt
        .query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..column_count {
                let value: rusqlite::types::Value = row.get(i)?;
                let json_value = match value {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(i) => serde_json::json!(i),
                    rusqlite::types::Value::Real(f) => serde_json::json!(f),
                    rusqlite::types::Value::Text(s) => serde_json::json!(s),
                    rusqlite::types::Value::Blob(b) => serde_json::json!(format!("<blob: {} bytes>", b.len())),
                };
                values.push(json_value);
            }
            Ok(values)
        })
        .map_err(|e| CommandError {
            code: "SQL_ERROR".to_string(),
            message: e.to_string(),
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Get database statistics
#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, DatabaseState>,
) -> CommandResult<DatabaseStats> {
    let conn = state.sqlite.get_connection()?;

    // Get table counts
    let project_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
        .unwrap_or(0);

    let mesh_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM meshes", [], |row| row.get(0))
        .unwrap_or(0);

    let model_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ml_models", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(DatabaseStats {
        project_count: project_count as u64,
        mesh_count: mesh_count as u64,
        model_count: model_count as u64,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub project_count: u64,
    pub mesh_count: u64,
    pub model_count: u64,
}
