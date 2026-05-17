// V238 + V239 + V240 + V241 — Cross-platform case storage stack.
//
//   - V238: resolve_paths     — surface OS-aware folders to the frontend
//   - V239: validate_case     — verify a .tlanticase folder against the schema
//   - V240: case_blob_*       — AES-GCM blob encryption (uses aes-gcm crate)
//   - V241: audit_append      — append-only HMAC-SHA256 chained log
//
// On the front-end the wrappers live in `lib/case-storage-bridge.ts`; both
// browser preview and Tauri runtime can call them, with Tauri returning real
// data and the browser falling back to deterministic stubs.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::storage_layout;

// ─── V238 — paths ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseStoragePaths {
    pub app_data: PathBuf,
    pub tlanticad_data_root: PathBuf,
    pub database: PathBuf,
    pub cases: PathBuf,
    pub documents: PathBuf,
    pub default_library: PathBuf,
    pub default_cases_root: PathBuf,
    pub patients_index: PathBuf,
    pub models: PathBuf,
    pub exports: PathBuf,
    pub cache: PathBuf,
    pub temp: PathBuf,
    pub logs: PathBuf,
    pub backups: PathBuf,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CaseStorageError {
    #[error("could not resolve app data directory")]
    AppDataUnavailable,
    #[error("could not resolve documents directory")]
    DocumentsUnavailable,
    #[error("io error: {message}")]
    Io { message: String },
    #[error("validation error: {message}")]
    Validation { message: String },
    #[error("crypto error: {message}")]
    Crypto { message: String },
    #[error("audit error: {message}")]
    Audit { message: String },
}

#[tauri::command]
pub fn resolve_paths(app: AppHandle) -> Result<CaseStoragePaths, CaseStorageError> {
    let path_resolver = app.path();
    let raw_app_data = path_resolver
        .app_data_dir()
        .map_err(|_| CaseStorageError::AppDataUnavailable)?;
    let documents = path_resolver
        .document_dir()
        .map_err(|_| CaseStorageError::DocumentsUnavailable)?;
    let layout = storage_layout::ensure_data_layout(&app)
        .map_err(|message| CaseStorageError::Io { message })?;

    Ok(CaseStoragePaths {
        app_data: raw_app_data,
        tlanticad_data_root: layout.root,
        database: layout.database_path,
        cases: layout.cases_dir.clone(),
        documents,
        default_library: layout.libraries_dir,
        default_cases_root: layout.cases_dir,
        patients_index: layout.patients_index_dir,
        models: layout.models_dir,
        exports: layout.exports_dir,
        cache: layout.cache_dir,
        temp: layout.temp_dir,
        logs: layout.logs_dir,
        backups: layout.backups_dir,
    })
}

#[tauri::command]
pub fn tlanticad_data_root_get(app: AppHandle) -> Result<CaseStoragePaths, CaseStorageError> {
    resolve_paths(app)
}

// ─── V239 — manifest validator ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // created_at / updated_at consumed by future audit-trail integration
pub struct CaseManifest {
    pub case_schema_version: String,
    pub case_id: String,
    pub case_number: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseValidationReport {
    pub ok: bool,
    pub case_id: Option<String>,
    pub case_number: Option<String>,
    pub schema_version: Option<String>,
    pub assets_dir_present: bool,
    pub interop_dir_present: bool,
    pub audit_log_present: bool,
    pub messages: Vec<String>,
}

#[tauri::command]
pub fn validate_case(folder: PathBuf) -> Result<CaseValidationReport, CaseStorageError> {
    let mut messages: Vec<String> = Vec::new();
    if !folder.exists() {
        return Err(CaseStorageError::Validation {
            message: format!("folder does not exist: {}", folder.display()),
        });
    }

    let manifest_path = folder.join("manifest.json");
    if !manifest_path.exists() {
        messages.push("manifest.json missing".to_string());
        return Ok(CaseValidationReport {
            ok: false,
            case_id: None,
            case_number: None,
            schema_version: None,
            assets_dir_present: folder.join("assets").exists(),
            interop_dir_present: folder.join("interop").exists(),
            audit_log_present: folder.join("audit.log").exists()
                || folder.join("logs").join("audit.log").exists(),
            messages,
        });
    }

    let raw = fs::read_to_string(&manifest_path).map_err(|e| CaseStorageError::Io {
        message: format!("read manifest: {e}"),
    })?;
    let manifest: CaseManifest =
        serde_json::from_str(&raw).map_err(|e| CaseStorageError::Validation {
            message: format!("manifest parse: {e}"),
        })?;

    if !manifest.case_schema_version.starts_with("tlanticase/") {
        messages.push(format!(
            "unexpected schema version: {} (expected tlanticase/*)",
            manifest.case_schema_version
        ));
    }
    if manifest.case_id.is_empty() {
        messages.push("caseId is empty".to_string());
    }
    if manifest.case_number.is_empty() {
        messages.push("caseNumber is empty".to_string());
    }

    let assets_dir_present = folder.join("assets").exists();
    let interop_dir_present = folder.join("interop").exists();
    let audit_log_present =
        folder.join("audit.log").exists() || folder.join("logs").join("audit.log").exists();

    if !assets_dir_present {
        messages.push("assets/ folder missing".to_string());
    }

    Ok(CaseValidationReport {
        ok: messages.is_empty(),
        case_id: Some(manifest.case_id),
        case_number: Some(manifest.case_number),
        schema_version: Some(manifest.case_schema_version),
        assets_dir_present,
        interop_dir_present,
        audit_log_present,
        messages,
    })
}

// ─── V240 — AES-GCM blob encryption ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptRequest {
    /// Plaintext to encrypt (UTF-8). For binary use `case_blob_encrypt_bytes`
    /// in a future iteration.
    pub plaintext: String,
    /// 32-byte key — base64 encoded.
    pub key_b64: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedBlob {
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptRequest {
    pub key_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

#[tauri::command]
pub fn case_blob_encrypt(request: EncryptRequest) -> Result<EncryptedBlob, CaseStorageError> {
    use aes_gcm::aead::{Aead, KeyInit, OsRng, Payload};
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine as _;
    use rand::RngCore;

    let key_bytes = B64
        .decode(&request.key_b64)
        .map_err(|e| CaseStorageError::Crypto {
            message: format!("invalid key_b64: {e}"),
        })?;
    if key_bytes.len() != 32 {
        return Err(CaseStorageError::Crypto {
            message: format!("key must be 32 bytes, got {}", key_bytes.len()),
        });
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| CaseStorageError::Crypto {
        message: format!("cipher init: {e}"),
    })?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: request.plaintext.as_bytes(),
                aad: b"tlanticase",
            },
        )
        .map_err(|e| CaseStorageError::Crypto {
            message: format!("encrypt: {e}"),
        })?;

    Ok(EncryptedBlob {
        nonce_b64: B64.encode(nonce_bytes),
        ciphertext_b64: B64.encode(ciphertext),
    })
}

#[tauri::command]
pub fn case_blob_decrypt(request: DecryptRequest) -> Result<String, CaseStorageError> {
    use aes_gcm::aead::{Aead, KeyInit, Payload};
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine as _;

    let key_bytes = B64
        .decode(&request.key_b64)
        .map_err(|e| CaseStorageError::Crypto {
            message: format!("invalid key_b64: {e}"),
        })?;
    let nonce_bytes = B64
        .decode(&request.nonce_b64)
        .map_err(|e| CaseStorageError::Crypto {
            message: format!("invalid nonce_b64: {e}"),
        })?;
    let ciphertext = B64
        .decode(&request.ciphertext_b64)
        .map_err(|e| CaseStorageError::Crypto {
            message: format!("invalid ciphertext_b64: {e}"),
        })?;

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| CaseStorageError::Crypto {
        message: format!("cipher init: {e}"),
    })?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &ciphertext,
                aad: b"tlanticase",
            },
        )
        .map_err(|e| CaseStorageError::Crypto {
            message: format!("decrypt: {e}"),
        })?;
    String::from_utf8(plaintext).map_err(|e| CaseStorageError::Crypto {
        message: format!("utf8: {e}"),
    })
}

// ─── V241 — audit log ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditAppendRequest {
    pub case_folder: PathBuf,
    pub event: serde_json::Value,
    /// 32-byte HMAC key, base64 encoded.
    pub hmac_key_b64: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub seq: u64,
    pub timestamp: String,
    pub prev_hmac: String,
    pub event: serde_json::Value,
    pub hmac: String,
}

#[tauri::command]
pub fn audit_append(request: AuditAppendRequest) -> Result<AuditEntry, CaseStorageError> {
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine as _;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use std::io::Write as _;

    let hmac_key = B64
        .decode(&request.hmac_key_b64)
        .map_err(|e| CaseStorageError::Audit {
            message: format!("invalid hmac_key_b64: {e}"),
        })?;

    let log_path = request.case_folder.join("audit.log");
    fs::create_dir_all(&request.case_folder).map_err(|e| CaseStorageError::Io {
        message: format!("create folder: {e}"),
    })?;

    // Read previous tail to extract `seq` + `hmac` of last line.
    let (seq, prev_hmac) = if log_path.exists() {
        let raw = fs::read_to_string(&log_path).map_err(|e| CaseStorageError::Io {
            message: format!("read log: {e}"),
        })?;
        let last = raw.lines().rev().find(|l| !l.is_empty());
        match last {
            Some(line) => match serde_json::from_str::<AuditEntry>(line) {
                Ok(prev) => (prev.seq + 1, prev.hmac),
                Err(_) => (1, "genesis".to_string()),
            },
            None => (1, "genesis".to_string()),
        }
    } else {
        (1, "genesis".to_string())
    };

    let timestamp = chrono::Utc::now().to_rfc3339();
    // HMAC payload: seq | timestamp | prev_hmac | event_json
    let event_json =
        serde_json::to_string(&request.event).map_err(|e| CaseStorageError::Audit {
            message: format!("serialise event: {e}"),
        })?;
    let payload = format!("{seq}|{timestamp}|{prev_hmac}|{event_json}");

    let mut mac =
        Hmac::<Sha256>::new_from_slice(&hmac_key).map_err(|e| CaseStorageError::Audit {
            message: format!("hmac init: {e}"),
        })?;
    mac.update(payload.as_bytes());
    let mac_bytes = mac.finalize().into_bytes();
    let hmac_b64 = B64.encode(mac_bytes);

    let entry = AuditEntry {
        seq,
        timestamp,
        prev_hmac,
        event: request.event,
        hmac: hmac_b64,
    };
    let line = serde_json::to_string(&entry).map_err(|e| CaseStorageError::Audit {
        message: format!("serialise entry: {e}"),
    })? + "\n";

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| CaseStorageError::Io {
            message: format!("open log: {e}"),
        })?;
    file.write_all(line.as_bytes())
        .map_err(|e| CaseStorageError::Io {
            message: format!("append log: {e}"),
        })?;

    Ok(entry)
}
