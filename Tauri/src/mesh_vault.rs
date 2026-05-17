use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::clinical_jobs::{self, ClinicalArtifactRecordRequest, ClinicalJobRecordRequest};
use crate::storage_layout;

const CORE_SCHEMA_SQL: &str = include_str!("../sql/001_core_schema.sql");
const DEFAULT_CHUNK_SIZE_BYTES: usize = 4 * 1024 * 1024;
const MIN_CHUNK_SIZE_BYTES: usize = 256 * 1024;
const MAX_CHUNK_SIZE_BYTES: usize = 16 * 1024 * 1024;
const MAX_IMPORT_SOURCE_BYTES: u64 = 2 * 1024 * 1024 * 1024 * 1024;
const MAX_CALLER_METADATA_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshVaultImportRequest {
    pub workspace: Option<String>,
    #[serde(alias = "case")]
    pub case_id: String,
    pub asset_id: Option<String>,
    #[serde(alias = "source")]
    pub source_path: String,
    pub kind: String,
    #[serde(alias = "moduleHint")]
    pub module_id: Option<String>,
    pub role: Option<String>,
    pub chunk_size_bytes: Option<usize>,
    pub ttl: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshVaultGpuHintsDto {
    pub preferred_upload: String,
    pub index_type: String,
    pub interleaved: bool,
    pub lod_ready: bool,
    pub render_usage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshVaultHandleDto {
    pub mesh_key: String,
    pub case_id: String,
    pub asset_id: String,
    pub kind: String,
    pub format: String,
    pub storage_path: String,
    pub checksum_sha256: String,
    pub bytes: u64,
    pub chunk_size_bytes: u64,
    pub chunk_count: u64,
    pub ttl: String,
    pub gpu_hints: MeshVaultGpuHintsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshVaultImportResponseDto {
    pub asset_id: String,
    pub hash: String,
    pub vault_path: String,
    pub size: u64,
    pub status: String,
}

impl MeshVaultImportResponseDto {
    fn from_handle(handle: &MeshVaultHandleDto, status: &str) -> Self {
        Self {
            asset_id: handle.asset_id.clone(),
            hash: handle.checksum_sha256.clone(),
            vault_path: handle.storage_path.clone(),
            size: handle.bytes,
            status: status.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshVaultJobStatusDto {
    pub job_id: String,
    pub status: String,
    pub progress: f64,
    pub stage: String,
    pub handle: Option<MeshVaultHandleDto>,
    pub response: Option<MeshVaultImportResponseDto>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshVaultImportEventDto {
    pub job_id: String,
    pub case_id: String,
    pub status: String,
    pub progress: f64,
    pub stage: String,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub mesh_key: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChunkRecord {
    index: u64,
    offset_bytes: u64,
    size_bytes: u64,
    checksum_sha256: String,
    storage_path: String,
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    storage_layout::database_path(app)
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
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|value| safe_segment(value, fallback))
        .unwrap_or_else(|| fallback.to_string())
}

fn case_root(app: &AppHandle, case_id: &str) -> Result<PathBuf, String> {
    let conn = open_connection(app)?;
    let stored: Option<String> = conn
        .query_row(
            "SELECT root_path FROM case_folder_manifests WHERE case_id = ?1",
            [case_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not resolve case folder for mesh vault: {error}"))?;

    if let Some(root_path) = stored {
        return Ok(PathBuf::from(root_path));
    }

    storage_layout::fallback_case_root(app, &safe_segment(case_id, "case"))
}

fn mesh_format(source_path: &Path, kind: &str) -> String {
    source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .unwrap_or_else(|| match kind {
            "dicom-series" => "dicom".to_string(),
            "obj-mesh" => "obj".to_string(),
            "ply-mesh" => "ply".to_string(),
            _ => "stl".to_string(),
        })
}

fn asset_type_for_kind(kind: &str, format: &str) -> &'static str {
    match kind {
        "dicom-series" => "dicom",
        "obj-mesh" => "obj",
        "ply-mesh" => "ply",
        "manufacturing-export" if format == "3mf" => "report",
        _ if format == "obj" => "obj",
        _ if format == "ply" => "ply",
        _ if format == "glb" || format == "gltf" => "gltf",
        _ => "stl",
    }
}

fn clamp_chunk_size(value: Option<usize>) -> usize {
    value
        .unwrap_or(DEFAULT_CHUNK_SIZE_BYTES)
        .clamp(MIN_CHUNK_SIZE_BYTES, MAX_CHUNK_SIZE_BYTES)
}

fn gpu_hints_for(format: &str) -> MeshVaultGpuHintsDto {
    MeshVaultGpuHintsDto {
        preferred_upload: "array-buffer-slice".to_string(),
        index_type: if format == "stl" { "uint32" } else { "source" }.to_string(),
        interleaved: false,
        lod_ready: false,
        render_usage: "static-draw-after-worker-parse".to_string(),
    }
}

fn collect_source_files(source_path: &Path) -> Result<Vec<PathBuf>, String> {
    if source_path.is_file() {
        return Ok(vec![source_path.to_path_buf()]);
    }

    if !source_path.is_dir() {
        return Err(format!(
            "Mesh vault import expects a readable file or directory path, got {}",
            source_path.display()
        ));
    }

    let mut pending = vec![source_path.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| {
            format!(
                "Could not read source directory {}: {error}",
                directory.display()
            )
        })? {
            let entry =
                entry.map_err(|error| format!("Could not read source directory entry: {error}"))?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn source_file_bytes(files: &[PathBuf]) -> Result<u64, String> {
    source_file_bytes_with_limit(files, MAX_IMPORT_SOURCE_BYTES)
}

fn source_file_bytes_with_limit(files: &[PathBuf], max_bytes: u64) -> Result<u64, String> {
    files.iter().try_fold(0_u64, |acc, path| {
        let metadata = fs::metadata(path)
            .map_err(|error| format!("Could not inspect source file {}: {error}", path.display()))?;

        if !metadata.is_file() {
            return Err(format!(
                "Mesh vault source entry is not a regular file: {}",
                path.display()
            ));
        }

        let next = acc.checked_add(metadata.len()).ok_or_else(|| {
            format!(
                "Mesh vault source size overflow while inspecting {}",
                path.display()
            )
        })?;

        if next > max_bytes {
            return Err(format!(
                "Mesh vault source is too large: {next} bytes exceeds the {max_bytes} byte guardrail"
            ));
        }

        Ok(next)
    })
}

fn validate_source_path_text(source_path: &str) -> Result<(), String> {
    if source_path.trim().is_empty() {
        return Err("Mesh vault import requires a non-empty source path".to_string());
    }

    if source_path.contains('\0') {
        return Err("Mesh vault source path contains an invalid NUL byte".to_string());
    }

    Ok(())
}

fn validate_caller_metadata(metadata: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(metadata) = metadata else {
        return Ok(());
    };

    if !(metadata.is_object() || metadata.is_null()) {
        return Err("Mesh vault metadata must be a JSON object".to_string());
    }

    let encoded = serde_json::to_vec(metadata)
        .map_err(|error| format!("Could not validate mesh vault metadata: {error}"))?;
    if encoded.len() > MAX_CALLER_METADATA_BYTES {
        return Err(format!(
            "Mesh vault metadata is too large: {} bytes exceeds the {} byte guardrail",
            encoded.len(),
            MAX_CALLER_METADATA_BYTES
        ));
    }

    Ok(())
}

fn validate_import_request(request: &MeshVaultImportRequest) -> Result<(), String> {
    validate_source_path_text(&request.source_path)?;
    validate_caller_metadata(request.metadata.as_ref())
}

fn emit_progress(
    app: &AppHandle,
    job_id: &str,
    case_id: &str,
    status: &str,
    progress: f64,
    stage: &str,
    bytes_written: u64,
    total_bytes: u64,
    mesh_key: Option<String>,
    error: Option<String>,
) {
    let _ = app.emit(
        "mesh-vault://job-progress",
        MeshVaultImportEventDto {
            job_id: job_id.to_string(),
            case_id: case_id.to_string(),
            status: status.to_string(),
            progress,
            stage: stage.to_string(),
            bytes_written,
            total_bytes,
            mesh_key,
            error,
        },
    );
}

fn update_job(
    app: &AppHandle,
    job_id: &str,
    case_id: &str,
    status: &str,
    progress: f64,
    stage: &str,
    result_json: Option<serde_json::Value>,
    error: Option<String>,
) -> Result<(), String> {
    clinical_jobs::clinical_job_record(
        app.clone(),
        ClinicalJobRecordRequest {
            id: Some(job_id.to_string()),
            case_id: Some(case_id.to_string()),
            kind: "mesh-vault-import".to_string(),
            status: Some(status.to_string()),
            progress: Some(progress),
            vendor: Some("tlanticad-rust".to_string()),
            model_id: None,
            checkpoint_sha256: None,
            params_json: Some(json!({ "stage": stage, "runtime": "rust-core" }).to_string()),
            result_json: result_json.map(|value| value.to_string()),
            error,
        },
    )?;
    Ok(())
}

fn job_was_cancelled(app: &AppHandle, job_id: &str) -> Result<bool, String> {
    let conn = open_connection(app)?;
    conn.query_row(
        "SELECT status = 'cancelled' FROM clinical_jobs WHERE id = ?1",
        [job_id],
        |row| row.get::<_, bool>(0),
    )
    .optional()
    .map(|value| value.unwrap_or(false))
    .map_err(|error| format!("Could not check mesh vault cancellation state: {error}"))
}

fn record_case_asset(
    conn: &Connection,
    case_id: &str,
    asset_id: &str,
    kind: &str,
    format: &str,
    source_name: &str,
    storage_path: &Path,
    checksum_sha256: &str,
    bytes: u64,
    role: Option<&str>,
    module_id: Option<&str>,
) -> Result<(), String> {
    let now = now_sql(conn)?;
    let metadata_json = json!({
        "role": role.unwrap_or("mesh-vault"),
        "moduleId": module_id,
        "meshVault": true,
        "format": format,
        "checksumSha256": checksum_sha256,
    })
    .to_string();

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
            asset_id,
            case_id,
            asset_type_for_kind(kind, format),
            storage_path.to_string_lossy().to_string(),
            safe_filename(source_name, "mesh.bin"),
            checksum_sha256,
            bytes as i64,
            metadata_json,
            now,
            now,
        ],
    )
    .map_err(|error| format!("Could not record mesh vault case asset: {error}"))?;

    Ok(())
}

fn record_mesh_handle(
    conn: &Connection,
    handle: &MeshVaultHandleDto,
    metadata: serde_json::Value,
) -> Result<(), String> {
    let now = now_sql(conn)?;
    conn.execute(
        r#"
        INSERT INTO mesh_vault_assets (
            mesh_key, case_id, asset_id, kind, format, storage_path, checksum_sha256,
            file_size_bytes, chunk_size_bytes, chunk_count, ttl, gpu_hints_json,
            metadata_json, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(mesh_key) DO UPDATE SET
            case_id = excluded.case_id,
            asset_id = excluded.asset_id,
            kind = excluded.kind,
            format = excluded.format,
            storage_path = excluded.storage_path,
            checksum_sha256 = excluded.checksum_sha256,
            file_size_bytes = excluded.file_size_bytes,
            chunk_size_bytes = excluded.chunk_size_bytes,
            chunk_count = excluded.chunk_count,
            ttl = excluded.ttl,
            gpu_hints_json = excluded.gpu_hints_json,
            metadata_json = excluded.metadata_json,
            updated_at = excluded.updated_at
        "#,
        params![
            handle.mesh_key,
            handle.case_id,
            handle.asset_id,
            handle.kind,
            handle.format,
            handle.storage_path,
            handle.checksum_sha256,
            handle.bytes as i64,
            handle.chunk_size_bytes as i64,
            handle.chunk_count as i64,
            handle.ttl,
            serde_json::to_string(&handle.gpu_hints)
                .map_err(|error| format!("Could not serialize GPU hints: {error}"))?,
            metadata.to_string(),
            now,
            now,
        ],
    )
    .map_err(|error| format!("Could not record mesh vault handle: {error}"))?;
    Ok(())
}

fn record_chunks(conn: &Connection, mesh_key: &str, chunks: &[ChunkRecord]) -> Result<(), String> {
    let now = now_sql(conn)?;
    for chunk in chunks {
        conn.execute(
            r#"
            INSERT INTO mesh_vault_chunks (
                id, mesh_key, chunk_index, offset_bytes, size_bytes, checksum_sha256,
                storage_path, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(mesh_key, chunk_index) DO UPDATE SET
                offset_bytes = excluded.offset_bytes,
                size_bytes = excluded.size_bytes,
                checksum_sha256 = excluded.checksum_sha256,
                storage_path = excluded.storage_path
            "#,
            params![
                format!("{mesh_key}-chunk-{}", chunk.index),
                mesh_key,
                chunk.index as i64,
                chunk.offset_bytes as i64,
                chunk.size_bytes as i64,
                chunk.checksum_sha256,
                chunk.storage_path,
                now,
            ],
        )
        .map_err(|error| format!("Could not record mesh vault chunk: {error}"))?;
    }
    Ok(())
}

fn handle_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MeshVaultHandleDto> {
    let gpu_hints_json: String = row.get(10)?;
    let format: String = row.get(4)?;
    let gpu_hints = serde_json::from_str::<MeshVaultGpuHintsDto>(&gpu_hints_json)
        .unwrap_or_else(|_| gpu_hints_for(&format));

    Ok(MeshVaultHandleDto {
        mesh_key: row.get(0)?,
        case_id: row.get(1)?,
        asset_id: row.get(2)?,
        kind: row.get(3)?,
        format,
        storage_path: row.get(5)?,
        checksum_sha256: row.get(6)?,
        bytes: row.get::<_, i64>(7)? as u64,
        chunk_size_bytes: row.get::<_, i64>(8)? as u64,
        chunk_count: row.get::<_, i64>(9)? as u64,
        ttl: row.get(11)?,
        gpu_hints,
    })
}

fn load_handle(conn: &Connection, mesh_key: &str) -> Result<Option<MeshVaultHandleDto>, String> {
    conn.query_row(
        r#"
        SELECT mesh_key, case_id, asset_id, kind, format, storage_path, checksum_sha256,
               file_size_bytes, chunk_size_bytes, chunk_count, gpu_hints_json, ttl
        FROM mesh_vault_assets
        WHERE mesh_key = ?1
        "#,
        [mesh_key],
        handle_from_row,
    )
    .optional()
    .map_err(|error| format!("Could not load mesh vault handle: {error}"))
}

fn stable_import_response_for_status(
    status: &str,
    handle: Option<&MeshVaultHandleDto>,
    parsed_response: Option<MeshVaultImportResponseDto>,
) -> Option<MeshVaultImportResponseDto> {
    let response_status = if status == "completed" {
        "completed"
    } else {
        status
    };

    handle
        .map(|handle| MeshVaultImportResponseDto::from_handle(handle, response_status))
        .or_else(|| {
            parsed_response.map(|mut response| {
                if status == "completed" {
                    response.status = "completed".to_string();
                }
                response
            })
        })
}

fn import_file_blocking(
    app: AppHandle,
    job_id: String,
    request: MeshVaultImportRequest,
) -> Result<MeshVaultHandleDto, String> {
    validate_import_request(&request)?;

    let case_id = safe_segment(&request.case_id, "case");
    let asset_id = request
        .asset_id
        .clone()
        .map(|value| safe_segment(&value, "asset"))
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let source_path = PathBuf::from(&request.source_path);
    let import_files = collect_source_files(&source_path)?;
    if import_files.is_empty() {
        return Err(format!(
            "Source path {} does not contain importable files",
            source_path.display()
        ));
    }

    let total_bytes = source_file_bytes(&import_files)?;
    let chunk_size = clamp_chunk_size(request.chunk_size_bytes);
    let format = mesh_format(&source_path, &request.kind);
    let source_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mesh.bin")
        .to_string();

    update_job(
        &app, &job_id, &case_id, "running", 0.02, "prepare", None, None,
    )?;
    emit_progress(
        &app,
        &job_id,
        &case_id,
        "running",
        0.02,
        "prepare",
        0,
        total_bytes,
        None,
        None,
    );

    let target_root = case_root(&app, &case_id)?
        .join("work")
        .join("meshes")
        .join(&asset_id);
    let chunk_root = target_root.join("chunks");
    fs::create_dir_all(&chunk_root)
        .map_err(|error| format!("Could not create mesh chunk directory: {error}"))?;

    let canonical_source = source_path.canonicalize().map_err(|error| {
        format!(
            "Could not resolve source path {}: {error}",
            source_path.display()
        )
    })?;
    let source_is_directory = canonical_source.is_dir();
    let target_path = if source_is_directory {
        target_root.join(safe_filename(&source_name, "dicom-series"))
    } else {
        target_root.join(safe_filename(&source_name, "mesh.bin"))
    };
    if source_is_directory {
        fs::create_dir_all(&target_path).map_err(|error| {
            format!(
                "Could not create DICOM series target {}: {error}",
                target_path.display()
            )
        })?;
    }

    let mut buffer = vec![0_u8; chunk_size];
    let mut file_hasher = Sha256::new();
    let mut chunks = Vec::new();
    let mut bytes_written = 0_u64;
    let mut chunk_index = 0_u64;

    for (file_index, source_file) in import_files.iter().enumerate() {
        let canonical_file = source_file.canonicalize().map_err(|error| {
            format!(
                "Could not resolve source file {}: {error}",
                source_file.display()
            )
        })?;
        let target_file = if source_is_directory {
            target_path.join(format!(
                "{file_index:06}-{}",
                safe_filename(
                    canonical_file
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("dicom.dcm"),
                    "dicom.dcm",
                )
            ))
        } else {
            target_path.clone()
        };

        let mut reader = BufReader::with_capacity(
            chunk_size,
            File::open(&canonical_file).map_err(|error| {
                format!(
                    "Could not open source asset {}: {error}",
                    canonical_file.display()
                )
            })?,
        );
        let mut writer = BufWriter::with_capacity(
            chunk_size,
            File::create(&target_file).map_err(|error| {
                format!(
                    "Could not create asset target {}: {error}",
                    target_file.display()
                )
            })?,
        );
        if source_is_directory {
            let relative_name = canonical_file
                .strip_prefix(&canonical_source)
                .unwrap_or(&canonical_file)
                .to_string_lossy();
            file_hasher.update(relative_name.as_bytes());
        }

        loop {
            if job_was_cancelled(&app, &job_id)? {
                emit_progress(
                    &app,
                    &job_id,
                    &case_id,
                    "cancelled",
                    0.0,
                    "cancelled",
                    bytes_written,
                    total_bytes,
                    None,
                    None,
                );
                return Err("__mesh_vault_cancelled__".to_string());
            }

            let bytes_read = reader
                .read(&mut buffer)
                .map_err(|error| format!("Could not read source asset chunk: {error}"))?;
            if bytes_read == 0 {
                break;
            }

            let chunk_bytes = &buffer[..bytes_read];
            writer
                .write_all(chunk_bytes)
                .map_err(|error| format!("Could not write target asset chunk: {error}"))?;
            file_hasher.update(chunk_bytes);

            let chunk_checksum = {
                let digest = Sha256::digest(chunk_bytes);
                digest
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            };
            let chunk_path = chunk_root.join(format!("chunk-{chunk_index:06}.bin"));
            fs::write(&chunk_path, chunk_bytes).map_err(|error| {
                format!(
                    "Could not persist asset chunk {}: {error}",
                    chunk_path.display()
                )
            })?;

            chunks.push(ChunkRecord {
                index: chunk_index,
                offset_bytes: bytes_written,
                size_bytes: bytes_read as u64,
                checksum_sha256: chunk_checksum,
                storage_path: chunk_path.to_string_lossy().to_string(),
            });
            bytes_written += bytes_read as u64;
            chunk_index += 1;

            let progress = if total_bytes == 0 {
                0.75
            } else {
                0.05 + ((bytes_written as f64 / total_bytes as f64) * 0.75)
            };
            update_job(
                &app,
                &job_id,
                &case_id,
                "running",
                progress,
                "stream-copy",
                None,
                None,
            )?;
            emit_progress(
                &app,
                &job_id,
                &case_id,
                "running",
                progress,
                "stream-copy",
                bytes_written,
                total_bytes,
                None,
                None,
            );
        }

        writer.flush().map_err(|error| {
            format!(
                "Could not flush target asset {}: {error}",
                target_file.display()
            )
        })?;
    }

    let checksum_sha256 = file_hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mesh_key = format!("sha256:{checksum_sha256}");
    let ttl = request.ttl.unwrap_or_else(|| "default".to_string());
    let gpu_hints = gpu_hints_for(&format);

    let handle = MeshVaultHandleDto {
        mesh_key: mesh_key.clone(),
        case_id: case_id.clone(),
        asset_id: asset_id.clone(),
        kind: request.kind.clone(),
        format: format.clone(),
        storage_path: target_path.to_string_lossy().to_string(),
        checksum_sha256: checksum_sha256.clone(),
        bytes: bytes_written,
        chunk_size_bytes: chunk_size as u64,
        chunk_count: chunks.len() as u64,
        ttl,
        gpu_hints,
    };

    update_job(
        &app, &job_id, &case_id, "running", 0.86, "index", None, None,
    )?;
    emit_progress(
        &app,
        &job_id,
        &case_id,
        "running",
        0.86,
        "index",
        bytes_written,
        total_bytes,
        Some(mesh_key.clone()),
        None,
    );

    let conn = open_connection(&app)?;
    record_case_asset(
        &conn,
        &case_id,
        &asset_id,
        &request.kind,
        &format,
        &source_name,
        &target_path,
        &checksum_sha256,
        bytes_written,
        request.role.as_deref(),
        request.module_id.as_deref(),
    )?;
    record_mesh_handle(
        &conn,
        &handle,
        json!({
            "workspace": request.workspace,
            "moduleId": request.module_id,
            "role": request.role.unwrap_or_else(|| "mesh-vault".to_string()),
            "sourcePath": canonical_source.to_string_lossy().to_string(),
            "sourceFileName": source_name,
            "chunks": chunks.len(),
            "callerMetadata": request.metadata.unwrap_or_else(|| json!({})),
        }),
    )?;
    record_chunks(&conn, &mesh_key, &chunks)?;

    clinical_jobs::clinical_artifact_record(
        app.clone(),
        ClinicalArtifactRecordRequest {
            id: Some(format!("{job_id}-mesh-handle")),
            job_id: Some(job_id.clone()),
            case_id: Some(case_id.clone()),
            asset_id: Some(asset_id),
            artifact_type: "mesh-vault-handle".to_string(),
            storage_path: Some(target_path.to_string_lossy().to_string()),
            checksum_sha256: Some(checksum_sha256),
            metadata_json: Some(
                json!({
                    "meshKey": mesh_key,
                    "format": format,
                    "chunkCount": chunks.len(),
                    "chunkSizeBytes": chunk_size,
                    "gpuHints": handle.gpu_hints,
                })
                .to_string(),
            ),
        },
    )?;

    update_job(
        &app,
        &job_id,
        &case_id,
        "completed",
        1.0,
        "completed",
        Some(json!({
            "handle": handle.clone(),
            "response": MeshVaultImportResponseDto::from_handle(&handle, "completed"),
        })),
        None,
    )?;
    emit_progress(
        &app,
        &job_id,
        &case_id,
        "completed",
        1.0,
        "completed",
        bytes_written,
        total_bytes,
        Some(handle.mesh_key.clone()),
        None,
    );

    Ok(handle)
}

#[tauri::command]
pub async fn mesh_vault_import_start(
    app: AppHandle,
    request: MeshVaultImportRequest,
) -> Result<MeshVaultJobStatusDto, String> {
    validate_import_request(&request)?;

    let job_id = Uuid::new_v4().to_string();
    let case_id = safe_segment(&request.case_id, "case");
    let params_json = json!({
        "runtime": "rust-core",
        "workspace": request.workspace.clone(),
        "moduleId": request.module_id.clone(),
        "moduleHint": request.module_id.clone(),
        "sourcePath": request.source_path.clone(),
        "source": request.source_path.clone(),
        "kind": request.kind,
        "chunkSizeBytes": clamp_chunk_size(request.chunk_size_bytes),
    })
    .to_string();

    clinical_jobs::clinical_job_record(
        app.clone(),
        ClinicalJobRecordRequest {
            id: Some(job_id.clone()),
            case_id: Some(case_id.clone()),
            kind: "mesh-vault-import".to_string(),
            status: Some("queued".to_string()),
            progress: Some(0.0),
            vendor: Some("tlanticad-rust".to_string()),
            model_id: None,
            checkpoint_sha256: None,
            params_json: Some(params_json),
            result_json: None,
            error: None,
        },
    )?;

    emit_progress(
        &app, &job_id, &case_id, "queued", 0.0, "queued", 0, 0, None, None,
    );

    let app_for_task = app.clone();
    let job_id_for_task = job_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) =
            import_file_blocking(app_for_task.clone(), job_id_for_task.clone(), request)
        {
            if error == "__mesh_vault_cancelled__" {
                return;
            }
            let _ = update_job(
                &app_for_task,
                &job_id_for_task,
                &case_id,
                "failed",
                0.0,
                "failed",
                None,
                Some(error.clone()),
            );
            emit_progress(
                &app_for_task,
                &job_id_for_task,
                &case_id,
                "failed",
                0.0,
                "failed",
                0,
                0,
                None,
                Some(error),
            );
        }
    });

    Ok(MeshVaultJobStatusDto {
        job_id,
        status: "queued".to_string(),
        progress: 0.0,
        stage: "queued".to_string(),
        handle: None,
        response: None,
        error: None,
    })
}

#[tauri::command]
pub fn mesh_vault_job_status(
    app: AppHandle,
    job_id: String,
) -> Result<MeshVaultJobStatusDto, String> {
    let record = clinical_jobs::clinical_job_get(app, job_id)?;
    let parsed_result = record
        .result_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok());
    let handle = parsed_result
        .as_ref()
        .and_then(|value| value.get("handle"))
        .and_then(|value| serde_json::from_value::<MeshVaultHandleDto>(value.clone()).ok());
    let parsed_response = parsed_result
        .as_ref()
        .and_then(|value| value.get("response"))
        .and_then(|value| serde_json::from_value::<MeshVaultImportResponseDto>(value.clone()).ok());
    let response =
        stable_import_response_for_status(&record.status, handle.as_ref(), parsed_response);
    let stage = serde_json::from_str::<serde_json::Value>(&record.params_json)
        .ok()
        .and_then(|value| {
            value
                .get("stage")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| record.status.clone());

    Ok(MeshVaultJobStatusDto {
        job_id: record.id,
        status: record.status,
        progress: record.progress,
        stage,
        handle,
        response,
        error: record.error,
    })
}

#[tauri::command]
pub fn mesh_vault_cancel(app: AppHandle, job_id: String) -> Result<MeshVaultJobStatusDto, String> {
    clinical_jobs::clinical_job_cancel(app.clone(), job_id.clone())?;
    mesh_vault_job_status(app, job_id)
}

#[tauri::command]
pub fn mesh_vault_find(
    app: AppHandle,
    mesh_key: String,
) -> Result<Option<MeshVaultHandleDto>, String> {
    let conn = open_connection(&app)?;
    load_handle(&conn, &mesh_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_handle() -> MeshVaultHandleDto {
        MeshVaultHandleDto {
            mesh_key: "sha256:abc123".to_string(),
            case_id: "case-123".to_string(),
            asset_id: "asset-456".to_string(),
            kind: "stl-mesh".to_string(),
            format: "stl".to_string(),
            storage_path: "/vault/case-123/work/meshes/asset-456/prep.stl".to_string(),
            checksum_sha256: "abc123".to_string(),
            bytes: 42,
            chunk_size_bytes: DEFAULT_CHUNK_SIZE_BYTES as u64,
            chunk_count: 1,
            ttl: "default".to_string(),
            gpu_hints: gpu_hints_for("stl"),
        }
    }

    #[test]
    fn chunk_size_is_bounded_for_predictable_memory() {
        assert_eq!(clamp_chunk_size(Some(1024)), MIN_CHUNK_SIZE_BYTES);
        assert_eq!(
            clamp_chunk_size(Some(64 * 1024 * 1024)),
            MAX_CHUNK_SIZE_BYTES
        );
        assert_eq!(clamp_chunk_size(None), DEFAULT_CHUNK_SIZE_BYTES);
    }

    #[test]
    fn mesh_format_uses_extension_without_loading_file() {
        assert_eq!(mesh_format(Path::new("/tmp/prep.STL"), "stl-mesh"), "stl");
        assert_eq!(mesh_format(Path::new("/tmp/cbct"), "dicom-series"), "dicom");
    }

    #[test]
    fn gpu_hints_keep_three_upload_off_react_state() {
        let hints = gpu_hints_for("stl");
        assert_eq!(hints.preferred_upload, "array-buffer-slice");
        assert_eq!(hints.render_usage, "static-draw-after-worker-parse");
        assert!(!hints.interleaved);
    }

    #[test]
    fn import_request_accepts_handle_contract_aliases() {
        let request = serde_json::from_value::<MeshVaultImportRequest>(json!({
            "workspace": "cad",
            "case": "case-123",
            "source": "/tmp/prep.stl",
            "kind": "stl-mesh",
            "moduleHint": "crown"
        }))
        .expect("request aliases should deserialize");

        assert_eq!(request.workspace.as_deref(), Some("cad"));
        assert_eq!(request.case_id, "case-123");
        assert_eq!(request.source_path, "/tmp/prep.stl");
        assert_eq!(request.kind, "stl-mesh");
        assert_eq!(request.module_id.as_deref(), Some("crown"));
    }

    #[test]
    fn import_response_exposes_stable_handle_fields() {
        let handle = sample_handle();

        let response = MeshVaultImportResponseDto::from_handle(&handle, "completed");
        let serialized = serde_json::to_value(&response).expect("response should serialize");

        assert_eq!(serialized["assetId"], "asset-456");
        assert_eq!(serialized["hash"], "abc123");
        assert_eq!(
            serialized["vaultPath"],
            "/vault/case-123/work/meshes/asset-456/prep.stl"
        );
        assert_eq!(serialized["size"], 42);
        assert_eq!(serialized["status"], "completed");
    }

    #[test]
    fn completed_status_normalizes_stale_import_response() {
        let stale_response = MeshVaultImportResponseDto {
            asset_id: "asset-456".to_string(),
            hash: "abc123".to_string(),
            vault_path: "/vault/case-123/work/meshes/asset-456/prep.stl".to_string(),
            size: 42,
            status: "running".to_string(),
        };

        let response = stable_import_response_for_status("completed", None, Some(stale_response))
            .expect("completed jobs should keep a stable response");

        assert_eq!(response.status, "completed");
        assert_eq!(response.asset_id, "asset-456");
        assert_eq!(response.hash, "abc123");
    }

    #[test]
    fn completed_status_rebuilds_response_from_handle() {
        let handle = sample_handle();
        let stale_response = MeshVaultImportResponseDto {
            asset_id: "stale-asset".to_string(),
            hash: "stale-hash".to_string(),
            vault_path: "/stale/path.stl".to_string(),
            size: 1,
            status: "running".to_string(),
        };

        let response =
            stable_import_response_for_status("completed", Some(&handle), Some(stale_response))
                .expect("handle should produce response");

        assert_eq!(response.status, "completed");
        assert_eq!(response.asset_id, "asset-456");
        assert_eq!(response.hash, "abc123");
        assert_eq!(response.size, 42);
    }

    #[test]
    fn metadata_validation_rejects_non_object_and_large_payloads() {
        let array_error = validate_caller_metadata(Some(&json!(["bad"])))
            .expect_err("metadata arrays should be rejected");
        assert!(array_error.contains("metadata must be a JSON object"));

        let large_metadata = json!({ "notes": "x".repeat(MAX_CALLER_METADATA_BYTES) });
        let large_error = validate_caller_metadata(Some(&large_metadata))
            .expect_err("oversized metadata should be rejected before DB writes");
        assert!(large_error.contains("metadata is too large"));

        validate_caller_metadata(Some(&json!({ "workflow": "crown-design" })))
            .expect("small metadata objects should pass");
    }

    #[test]
    fn source_path_validation_rejects_empty_and_nul_paths() {
        assert!(validate_source_path_text("   ").is_err());
        assert!(validate_source_path_text("/tmp/prep.stl\0.bad").is_err());
        validate_source_path_text("/tmp/prep.stl").expect("normal paths should pass");
    }

    #[test]
    fn source_size_validation_rejects_over_limit_without_huge_files() {
        let test_root = std::env::temp_dir().join(format!(
            "tlanticad-mesh-vault-source-size-{}",
            Uuid::new_v4()
        ));
        fs::create_dir_all(&test_root).expect("test directory should be created");
        let source_file = test_root.join("small.stl");
        fs::write(&source_file, b"12345").expect("small source file should be written");

        let too_large = source_file_bytes_with_limit(&[source_file.clone()], 4)
            .expect_err("source guardrail should reject files above the configured limit");
        assert!(too_large.contains("source is too large"));

        let total = source_file_bytes_with_limit(&[source_file], 5)
            .expect("source size at the limit should pass");
        assert_eq!(total, 5);

        fs::remove_dir_all(test_root).expect("test directory should be removed");
    }
}
