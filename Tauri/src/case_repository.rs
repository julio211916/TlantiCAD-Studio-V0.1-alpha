use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

const CORE_SCHEMA_SQL: &str = include_str!("../sql/001_core_schema.sql");
const CLINICAL_ORDER_SCHEMA_SQL: &str =
    include_str!("../sql/002_tlantidb_clinical_order_schema.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalAssetDto {
    pub id: String,
    pub name: String,
    pub role: String,
    pub local_path: Option<String>,
    pub tooth_numbers: Option<Vec<i32>>,
    pub module_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalCaseDto {
    pub id: String,
    pub case_number: String,
    pub name: String,
    pub active_module_id: String,
    pub assets: Vec<ClinicalAssetDto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseCreateRequest {
    pub case_number: Option<String>,
    pub name: Option<String>,
    pub active_module_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetWriteRequestDto {
    pub case_id: String,
    pub asset: ClinicalAssetDto,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredAssetRefDto {
    pub asset_id: String,
    pub case_id: String,
    pub local_path: String,
    pub bytes: u64,
    pub checksum_sha256: String,
}

fn app_data_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve app data dir: {error}"))
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_root(app)?.join("tlanticad.db"))
}

fn open_connection(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Could not create app data directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let conn = Connection::open(&path)
        .map_err(|error| format!("Could not open SQLite database {}: {error}", path.display()))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;",
    )
    .map_err(|error| format!("Could not configure SQLite database: {error}"))?;
    conn.execute_batch(CORE_SCHEMA_SQL)
        .map_err(|error| format!("Could not apply core schema: {error}"))?;
    conn.execute_batch(CLINICAL_ORDER_SCHEMA_SQL)
        .map_err(|error| format!("Could not apply clinical order schema: {error}"))?;
    Ok(conn)
}

fn now_sql(conn: &Connection) -> Result<String, String> {
    conn.query_row("SELECT datetime('now')", [], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not read SQLite clock: {error}"))
}

fn safe_segment(value: &str, fallback: &str) -> String {
    let segment: String = value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
        .collect();

    if segment.is_empty() {
        fallback.to_string()
    } else {
        segment
    }
}

fn safe_filename(value: &str, fallback: &str) -> String {
    let file_name = Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(fallback);
    safe_segment(file_name, fallback)
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn infer_asset_type(asset: &ClinicalAssetDto) -> &'static str {
    let lower_name = asset.name.to_lowercase();
    if lower_name.ends_with(".dcm")
        || lower_name.ends_with(".dicom")
        || lower_name.ends_with(".ima")
    {
        "dicom"
    } else if lower_name.ends_with(".stl") {
        "stl"
    } else if lower_name.ends_with(".obj") {
        "obj"
    } else if lower_name.ends_with(".ply") {
        "ply"
    } else if lower_name.ends_with(".gltf") || lower_name.ends_with(".glb") {
        "gltf"
    } else if lower_name.ends_with(".xml") {
        "xml"
    } else if lower_name.ends_with(".pdf") || lower_name.ends_with(".txt") || asset.role == "report"
    {
        "report"
    } else {
        "preview"
    }
}

fn metadata_for_asset(asset: &ClinicalAssetDto, checksum: Option<&str>) -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({
        "role": asset.role,
        "moduleId": asset.module_id,
        "toothNumbers": asset.tooth_numbers,
        "tags": asset.tags,
        "checksumSha256": checksum,
    }))
    .map_err(|error| format!("Could not serialize asset metadata: {error}"))
}

fn upsert_case(conn: &Connection, dental_case: &DentalCaseDto) -> Result<(), String> {
    conn.execute(
        r#"
        INSERT INTO cases (
            id, case_number, patient_alias, title, status, active_jaw, notes, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, 'draft', 'upper', NULL, ?5, ?6)
        ON CONFLICT(id) DO UPDATE SET
            case_number = excluded.case_number,
            patient_alias = excluded.patient_alias,
            title = excluded.title,
            updated_at = excluded.updated_at
        "#,
        params![
            dental_case.id,
            dental_case.case_number,
            dental_case.name,
            dental_case.name,
            dental_case.created_at,
            dental_case.updated_at,
        ],
    )
    .map_err(|error| format!("Could not save case: {error}"))?;

    conn.execute(
        r#"
        INSERT INTO case_modules (id, case_id, module_code, window_label, sync_enabled, state_json, updated_at)
        VALUES (?1, ?2, ?3, NULL, 1, ?4, ?5)
        ON CONFLICT(case_id, module_code) DO UPDATE SET
            state_json = excluded.state_json,
            updated_at = excluded.updated_at
        "#,
        params![
            format!("module-{}-{}", dental_case.id, dental_case.active_module_id),
            dental_case.id,
            dental_case.active_module_id,
            serde_json::json!({ "active": true }).to_string(),
            dental_case.updated_at,
        ],
    )
    .map_err(|error| format!("Could not save active module: {error}"))?;

    for asset in &dental_case.assets {
        upsert_asset(conn, &dental_case.id, asset, None)?;
    }

    Ok(())
}

fn upsert_asset(
    conn: &Connection,
    case_id: &str,
    asset: &ClinicalAssetDto,
    checksum: Option<&str>,
) -> Result<(), String> {
    let updated_at = now_sql(conn)?;
    let local_path = asset.local_path.clone().unwrap_or_default();
    let metadata = metadata_for_asset(asset, checksum)?;
    let file_size = if local_path.is_empty() {
        None
    } else {
        fs::metadata(&local_path)
            .ok()
            .map(|metadata| metadata.len() as i64)
    };

    conn.execute(
        r#"
        INSERT INTO case_assets (
            id, case_id, asset_type, storage_path, filename, mime_type, checksum_sha256,
            file_size_bytes, metadata_json, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(id) DO UPDATE SET
            case_id = excluded.case_id,
            asset_type = excluded.asset_type,
            storage_path = excluded.storage_path,
            filename = excluded.filename,
            checksum_sha256 = excluded.checksum_sha256,
            file_size_bytes = excluded.file_size_bytes,
            metadata_json = excluded.metadata_json,
            updated_at = excluded.updated_at
        "#,
        params![
            asset.id,
            case_id,
            infer_asset_type(asset),
            local_path,
            safe_filename(&asset.name, "asset.bin"),
            checksum,
            file_size,
            metadata,
            asset.created_at,
            updated_at,
        ],
    )
    .map_err(|error| format!("Could not save asset: {error}"))?;

    Ok(())
}

fn load_assets(conn: &Connection, case_id: &str) -> Result<Vec<ClinicalAssetDto>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, filename, storage_path, metadata_json, created_at
            FROM case_assets
            WHERE case_id = ?1
            ORDER BY created_at ASC, filename ASC
            "#,
        )
        .map_err(|error| format!("Could not prepare asset query: {error}"))?;

    let rows = stmt
        .query_map([case_id], |row| {
            let metadata: String = row.get(3)?;
            let parsed: serde_json::Value = serde_json::from_str(&metadata).unwrap_or_default();
            Ok(ClinicalAssetDto {
                id: row.get(0)?,
                name: row.get(1)?,
                role: parsed
                    .get("role")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("other")
                    .to_string(),
                local_path: Some(row.get::<_, String>(2)?).filter(|value| !value.is_empty()),
                tooth_numbers: parsed
                    .get("toothNumbers")
                    .and_then(|value| serde_json::from_value::<Vec<i32>>(value.clone()).ok()),
                module_id: parsed
                    .get("moduleId")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string),
                tags: parsed
                    .get("tags")
                    .and_then(|value| serde_json::from_value::<Vec<String>>(value.clone()).ok()),
                created_at: row.get(4)?,
            })
        })
        .map_err(|error| format!("Could not load assets: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not collect assets: {error}"))
}

fn active_module_for_case(conn: &Connection, case_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT module_code FROM case_modules WHERE case_id = ?1 ORDER BY updated_at DESC LIMIT 1",
        [case_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(|error| format!("Could not query active module: {error}"))
    .map(|value| value.unwrap_or_else(|| "cad".to_string()))
}

fn row_to_case(conn: &Connection, row: &rusqlite::Row<'_>) -> rusqlite::Result<DentalCaseDto> {
    let case_id: String = row.get(0)?;
    let active_module_id =
        active_module_for_case(conn, &case_id).unwrap_or_else(|_| "cad".to_string());
    let assets = load_assets(conn, &case_id).unwrap_or_default();

    Ok(DentalCaseDto {
        id: case_id,
        case_number: row.get(1)?,
        name: row.get(2)?,
        active_module_id,
        assets,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[tauri::command]
pub fn case_create(app: AppHandle, request: CaseCreateRequest) -> Result<DentalCaseDto, String> {
    let conn = open_connection(&app)?;
    let now = now_sql(&conn)?;
    let id = Uuid::new_v4().to_string();
    let case_number = request
        .case_number
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("TC-{}", &id[..8]));
    let name = request
        .name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "New restorative case".to_string());

    let dental_case = DentalCaseDto {
        id,
        case_number,
        name,
        active_module_id: request
            .active_module_id
            .unwrap_or_else(|| "cad".to_string()),
        assets: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    };

    upsert_case(&conn, &dental_case)?;
    Ok(dental_case)
}

#[tauri::command]
pub fn case_save(app: AppHandle, dental_case: DentalCaseDto) -> Result<DentalCaseDto, String> {
    let conn = open_connection(&app)?;
    upsert_case(&conn, &dental_case)?;
    case_get_graph(app, dental_case.id)?
        .ok_or_else(|| "Saved case could not be reloaded".to_string())
}

#[tauri::command]
pub fn case_get_graph(app: AppHandle, case_id: String) -> Result<Option<DentalCaseDto>, String> {
    let conn = open_connection(&app)?;
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, case_number, title, created_at, updated_at
            FROM cases
            WHERE id = ?1
            LIMIT 1
            "#,
        )
        .map_err(|error| format!("Could not prepare case query: {error}"))?;

    stmt.query_row([case_id], |row| row_to_case(&conn, row))
        .optional()
        .map_err(|error| format!("Could not load case graph: {error}"))
}

#[tauri::command]
pub fn case_list(app: AppHandle) -> Result<Vec<DentalCaseDto>, String> {
    let conn = open_connection(&app)?;
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, case_number, title, created_at, updated_at
            FROM cases
            ORDER BY updated_at DESC
            "#,
        )
        .map_err(|error| format!("Could not prepare case list query: {error}"))?;

    let rows = stmt
        .query_map([], |row| row_to_case(&conn, row))
        .map_err(|error| format!("Could not query cases: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not collect case list: {error}"))
}

#[tauri::command]
pub fn case_save_asset(
    app: AppHandle,
    case_id: String,
    asset: ClinicalAssetDto,
) -> Result<ClinicalAssetDto, String> {
    let conn = open_connection(&app)?;
    upsert_asset(&conn, &case_id, &asset, None)?;
    Ok(asset)
}

#[tauri::command]
pub fn asset_write(
    app: AppHandle,
    request: AssetWriteRequestDto,
) -> Result<StoredAssetRefDto, String> {
    let root = app_data_root(&app)?
        .join("cases")
        .join(safe_segment(&request.case_id, "case"))
        .join("assets")
        .join(safe_segment(&request.asset.id, "asset"));
    fs::create_dir_all(&root).map_err(|error| {
        format!(
            "Could not create asset directory {}: {error}",
            root.display()
        )
    })?;

    let filename = safe_filename(&request.asset.name, "asset.bin");
    let target = root.join(filename);
    fs::write(&target, &request.bytes)
        .map_err(|error| format!("Could not write asset {}: {error}", target.display()))?;

    let checksum = hex_digest(&request.bytes);
    let mut stored_asset = request.asset.clone();
    stored_asset.local_path = Some(target.to_string_lossy().to_string());

    let conn = open_connection(&app)?;
    upsert_asset(&conn, &request.case_id, &stored_asset, Some(&checksum))?;

    Ok(StoredAssetRefDto {
        asset_id: stored_asset.id,
        case_id: request.case_id,
        local_path: target.to_string_lossy().to_string(),
        bytes: request.bytes.len() as u64,
        checksum_sha256: checksum,
    })
}

#[tauri::command]
pub fn asset_read(app: AppHandle, local_path: String) -> Result<Vec<u8>, String> {
    let root = app_data_root(&app)?;
    let path = PathBuf::from(local_path);
    if !path.starts_with(&root) {
        return Err("Asset path is outside TlantiCAD app data scope".to_string());
    }

    fs::read(&path).map_err(|error| format!("Could not read asset {}: {error}", path.display()))
}
