//! TlantiShare — local-first P2P file sharing (V276–V278).
//!
//! Replaces the earlier "dentalshare" cloud concept. Communicates directly
//! with peers on the same LAN, no central server, automatic discovery via
//! mDNS-SD, file transfer over TCP authenticated + encrypted with AES-256-GCM.
//!
//! # Wire protocol — `tlantishare/v1`
//!
//! mDNS service: `_tlantishare._tcp.local.`. TXT records: `alias`, `device_id`,
//! `version`, `platform`, `fingerprint` (sha256 of device_id, used so peers
//! can verify they're talking to the right machine).
//!
//! Per session over TCP:
//! 1. Sender connects, writes 12-byte magic `b"TLANTISHARE\x01"` + 4-byte length
//!    + JSON manifest `{transfer_id, salt_b64, files:[{name,size,sha256}]}`.
//! 2. Receiver emits `tlantishare://incoming-request` to the UI with the
//!    manifest. The UI shows an accept dialog where the user enters the 6-digit
//!    pairing PIN (displayed on the sender side).
//! 3. UI calls `local_share_accept(transfer_id, pin)` → receiver sends
//!    `[0x01]` (accept) or `[0x00]` (reject) byte.
//! 4. Both sides derive AES-256-GCM key via HKDF-SHA256(salt, pin, info=
//!    "tlantishare/v1/key").
//! 5. Sender streams per-file: `[u32 chunk_len][12-byte nonce][ciphertext]`.
//!    Nonce = 4-byte salt || 8-byte counter (big-endian).
//! 6. After last chunk of each file, sender writes a sentinel `[u32 0]`.
//! 7. Receiver writes plaintext to `<staging_dir>/<transfer_id>/<name>`,
//!    verifies SHA256, emits `tlantishare://recv-progress` and finally
//!    `tlantishare://recv-complete`.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use hkdf::Hkdf;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

const SERVICE_TYPE: &str = "_tlantishare._tcp.local.";
const PROTOCOL_MAGIC: &[u8; 12] = b"TLANTISHARE\x01";
const HKDF_INFO: &[u8] = b"tlantishare/v1/key";
const CHUNK_PLAINTEXT_BYTES: usize = 64 * 1024;
const ACCEPT_BYTE: u8 = 0x01;
const REJECT_BYTE: u8 = 0x00;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub transports: Vec<String>,
    pub address: Option<String>,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: u128,
    pub status: String,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdvertiseInfo {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub port: u16,
    pub alias: String,
    pub fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferManifest {
    #[serde(rename = "transferId")]
    pub transfer_id: String,
    #[serde(rename = "saltB64")]
    pub salt_b64: String,
    pub files: Vec<FileEntry>,
    #[serde(rename = "fromAlias")]
    pub from_alias: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendInput {
    #[serde(rename = "peerAddress")]
    pub peer_address: String,
    #[serde(rename = "filePaths")]
    pub file_paths: Vec<String>,
    pub pin: String,
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SendOutput {
    #[serde(rename = "transferId")]
    pub transfer_id: String,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
}

pub struct LocalShareState {
    daemon: Mutex<Option<ServiceDaemon>>,
    advertised_fullname: Mutex<Option<String>>,
    browse_token: Mutex<Option<oneshot::Sender<()>>>,
    listen_token: Mutex<Option<oneshot::Sender<()>>>,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<Option<String>>>>>,
}

impl Default for LocalShareState {
    fn default() -> Self {
        Self {
            daemon: Mutex::new(None),
            advertised_fullname: Mutex::new(None),
            browse_token: Mutex::new(None),
            listen_token: Mutex::new(None),
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn primary_ipv4() -> Result<Ipv4Addr, String> {
    match local_ip_address::local_ip().map_err(|e| e.to_string())? {
        IpAddr::V4(v4) => Ok(v4),
        IpAddr::V6(_) => {
            // Fall back to scanning interfaces for a v4.
            for (_, ip) in local_ip_address::list_afinet_netifas().map_err(|e| e.to_string())? {
                if let IpAddr::V4(v4) = ip {
                    if !v4.is_loopback() {
                        return Ok(v4);
                    }
                }
            }
            Err("no IPv4 LAN interface available".to_string())
        }
    }
}

fn fingerprint_for(device_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    let digest = hasher.finalize();
    B64.encode(&digest[..16])
}

async fn ensure_daemon(state: &LocalShareState) -> Result<ServiceDaemon, String> {
    let mut guard = state.daemon.lock().await;
    if guard.is_none() {
        *guard = Some(ServiceDaemon::new().map_err(|e| e.to_string())?);
    }
    Ok(guard.as_ref().unwrap().clone())
}

#[tauri::command]
pub async fn local_share_advertise(
    state: State<'_, LocalShareState>,
    port: Option<u16>,
    alias: Option<String>,
) -> Result<AdvertiseInfo, String> {
    let port = port.unwrap_or(53318);
    let alias = match alias {
        Some(a) if !a.trim().is_empty() => a,
        _ => hostname::get()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned(),
    };
    let device_id = Uuid::new_v4().to_string();
    let fingerprint = fingerprint_for(&device_id);
    let host_ip = primary_ipv4()?;
    let host_name = format!("{}.local.", alias.replace(' ', "-"));

    let mut props: HashMap<String, String> = HashMap::new();
    props.insert("alias".into(), alias.clone());
    props.insert("device_id".into(), device_id.clone());
    props.insert("version".into(), "1".into());
    props.insert("platform".into(), std::env::consts::OS.into());
    props.insert("fingerprint".into(), fingerprint.clone());

    let host_ip_str = host_ip.to_string();
    let info = ServiceInfo::new(
        SERVICE_TYPE,
        &alias,
        &host_name,
        host_ip_str.as_str(),
        port,
        Some(props),
    )
    .map_err(|e| e.to_string())?;
    let fullname = info.get_fullname().to_string();

    let daemon = ensure_daemon(&state).await?;

    // Re-advertise must replace the previous registration; otherwise the
    // mDNS service leaks a stale entry every time the user toggles
    // sharing. Unregister first, then register the fresh ServiceInfo.
    let mut advertised = state.advertised_fullname.lock().await;
    if let Some(prev) = advertised.take() {
        let _ = daemon.unregister(&prev);
    }
    daemon.register(info).map_err(|e| e.to_string())?;
    *advertised = Some(fullname);

    Ok(AdvertiseInfo {
        device_id,
        port,
        alias,
        fingerprint,
    })
}

#[tauri::command]
pub async fn local_share_stop_advertising(state: State<'_, LocalShareState>) -> Result<(), String> {
    let mut advertised = state.advertised_fullname.lock().await;
    if let Some(fullname) = advertised.take() {
        let daemon = ensure_daemon(&state).await?;
        let _ = daemon.unregister(&fullname);
    }
    Ok(())
}

#[tauri::command]
pub async fn local_share_browse_start(
    app: AppHandle,
    state: State<'_, LocalShareState>,
) -> Result<(), String> {
    let mut token = state.browse_token.lock().await;
    if token.is_some() {
        return Ok(());
    }
    let daemon = ensure_daemon(&state).await?;
    let receiver = daemon.browse(SERVICE_TYPE).map_err(|e| e.to_string())?;
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    *token = Some(cancel_tx);

    let app_handle = app.clone();
    tokio::spawn(async move {
        tokio::pin!(cancel_rx);
        loop {
            tokio::select! {
                _ = &mut cancel_rx => break,
                evt = tokio::task::spawn_blocking({
                    let receiver = receiver.clone();
                    move || receiver.recv()
                }) => {
                    match evt {
                        Ok(Ok(ServiceEvent::ServiceResolved(info))) => {
                            let props = info.get_properties();
                            let alias = props
                                .get_property_val_str("alias")
                                .unwrap_or_else(|| info.get_hostname());
                            let device_id = props
                                .get_property_val_str("device_id")
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| Uuid::new_v4().to_string());
                            let platform_label = props
                                .get_property_val_str("platform")
                                .unwrap_or("desktop");
                            let fingerprint = props
                                .get_property_val_str("fingerprint")
                                .map(|s| s.to_string());
                            let ip: IpAddr = info
                                .get_addresses()
                                .iter()
                                .copied()
                                .find(|a: &IpAddr| !a.is_loopback())
                                .unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
                            let peer = Peer {
                                id: device_id,
                                name: alias.to_string(),
                                platform: platform_label.to_string(),
                                transports: vec!["lan".into()],
                                address: Some(format!("{}:{}", ip, info.get_port())),
                                last_seen_at: now_ms(),
                                status: "idle".into(),
                                fingerprint,
                            };
                            let _ = app_handle.emit("tlantishare://peer-found", &peer);
                        }
                        Ok(Ok(ServiceEvent::ServiceRemoved(_, fullname))) => {
                            let _ = app_handle.emit("tlantishare://peer-removed", &fullname);
                        }
                        Ok(Ok(_)) => {}
                        Ok(Err(_)) => break,
                        Err(_) => break,
                    }
                }
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn local_share_browse_stop(state: State<'_, LocalShareState>) -> Result<(), String> {
    let mut token = state.browse_token.lock().await;
    if let Some(tx) = token.take() {
        let _ = tx.send(());
    }
    Ok(())
}

fn derive_key(pin: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let hk = Hkdf::<Sha256>::new(Some(salt), pin.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(HKDF_INFO, &mut key).map_err(|e| e.to_string())?;
    Ok(key)
}

fn nonce_for(salt4: [u8; 4], counter: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[..4].copy_from_slice(&salt4);
    n[4..].copy_from_slice(&counter.to_be_bytes());
    n
}

async fn read_exact_async<S: AsyncReadExt + Unpin>(
    stream: &mut S,
    buf: &mut [u8],
) -> Result<(), String> {
    stream
        .read_exact(buf)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

async fn write_framed_json<S: AsyncWriteExt + Unpin, T: Serialize>(
    stream: &mut S,
    payload: &T,
) -> Result<(), String> {
    let bytes = serde_json::to_vec(payload).map_err(|e| e.to_string())?;
    let len = (bytes.len() as u32).to_be_bytes();
    stream.write_all(&len).await.map_err(|e| e.to_string())?;
    stream.write_all(&bytes).await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn read_framed_json<S: AsyncReadExt + Unpin, T: for<'de> Deserialize<'de>>(
    stream: &mut S,
) -> Result<T, String> {
    let mut len_buf = [0u8; 4];
    read_exact_async(stream, &mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 1_048_576 {
        return Err(format!("framed json too large: {} bytes", len));
    }
    let mut buf = vec![0u8; len];
    read_exact_async(stream, &mut buf).await?;
    serde_json::from_slice(&buf).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn local_share_send(app: AppHandle, input: SendInput) -> Result<SendOutput, String> {
    if input.pin.is_empty() {
        return Err("pin is required (6-digit pairing code)".into());
    }
    let alias = match input.alias {
        Some(a) if !a.trim().is_empty() => a,
        _ => hostname::get()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned(),
    };

    let mut entries: Vec<(PathBuf, FileEntry)> = Vec::with_capacity(input.file_paths.len());
    let mut total_bytes: u64 = 0;
    for raw in &input.file_paths {
        let p = PathBuf::from(raw);
        if !p.is_file() {
            return Err(format!("file not found: {}", p.display()));
        }
        let meta = std::fs::metadata(&p).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
        hasher.update(&bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        let name = p
            .file_name()
            .ok_or_else(|| format!("invalid filename: {}", p.display()))?
            .to_string_lossy()
            .into_owned();
        entries.push((
            p,
            FileEntry {
                name,
                size: meta.len(),
                sha256,
            },
        ));
        total_bytes += meta.len();
    }

    let transfer_id = Uuid::new_v4().to_string();
    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let mut salt4 = [0u8; 4];
    salt4.copy_from_slice(&salt[..4]);
    let salt_b64 = B64.encode(salt);

    let manifest = TransferManifest {
        transfer_id: transfer_id.clone(),
        salt_b64: salt_b64.clone(),
        files: entries.iter().map(|(_, e)| e.clone()).collect(),
        from_alias: alias,
    };

    let mut stream = TcpStream::connect(&input.peer_address)
        .await
        .map_err(|e| format!("connect {}: {}", input.peer_address, e))?;
    stream
        .write_all(PROTOCOL_MAGIC)
        .await
        .map_err(|e| e.to_string())?;
    write_framed_json(&mut stream, &manifest).await?;

    let mut decision = [0u8; 1];
    read_exact_async(&mut stream, &mut decision).await?;
    if decision[0] != ACCEPT_BYTE {
        return Err("peer rejected the transfer".into());
    }

    let key_bytes = derive_key(&input.pin, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut counter: u64 = 0;
    let mut bytes_sent: u64 = 0;

    for (path, entry) in &entries {
        let bytes = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        for chunk in bytes.chunks(CHUNK_PLAINTEXT_BYTES) {
            let nonce_bytes = nonce_for(salt4, counter);
            let nonce = Nonce::from_slice(&nonce_bytes);
            let ciphertext = cipher
                .encrypt(nonce, chunk)
                .map_err(|e| format!("aes-gcm encrypt: {}", e))?;
            let len = (ciphertext.len() as u32).to_be_bytes();
            stream.write_all(&len).await.map_err(|e| e.to_string())?;
            stream
                .write_all(&nonce_bytes)
                .await
                .map_err(|e| e.to_string())?;
            stream
                .write_all(&ciphertext)
                .await
                .map_err(|e| e.to_string())?;
            counter = counter.wrapping_add(1);
            bytes_sent += chunk.len() as u64;
            let _ = app.emit(
                "tlantishare://send-progress",
                serde_json::json!({
                    "transferId": transfer_id,
                    "bytesSent": bytes_sent,
                    "totalBytes": total_bytes,
                    "currentFile": entry.name,
                }),
            );
        }
        // sentinel between files
        stream
            .write_all(&0u32.to_be_bytes())
            .await
            .map_err(|e| e.to_string())?;
    }
    stream.shutdown().await.ok();

    let _ = app.emit(
        "tlantishare://send-complete",
        serde_json::json!({ "transferId": transfer_id }),
    );

    Ok(SendOutput {
        transfer_id,
        total_bytes,
    })
}

#[tauri::command]
pub async fn local_share_listen(
    app: AppHandle,
    state: State<'_, LocalShareState>,
    port: u16,
    staging_dir: String,
) -> Result<(), String> {
    let mut token_guard = state.listen_token.lock().await;
    if token_guard.is_some() {
        return Ok(());
    }
    let staging = PathBuf::from(staging_dir);
    tokio::fs::create_dir_all(&staging)
        .await
        .map_err(|e| e.to_string())?;

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| format!("bind 0.0.0.0:{} → {}", port, e))?;
    let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
    *token_guard = Some(cancel_tx);

    let pending = state.pending.clone();
    let app_handle = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut cancel_rx => break,
                accept = listener.accept() => {
                    match accept {
                        Ok((stream, peer_addr)) => {
                            let pending = pending.clone();
                            let app_handle = app_handle.clone();
                            let staging = staging.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_incoming(stream, peer_addr.to_string(), staging, pending, app_handle.clone()).await {
                                    let _ = app_handle.emit(
                                        "tlantishare://recv-error",
                                        serde_json::json!({ "error": e }),
                                    );
                                }
                            });
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn local_share_stop_listening(state: State<'_, LocalShareState>) -> Result<(), String> {
    let mut token = state.listen_token.lock().await;
    if let Some(tx) = token.take() {
        let _ = tx.send(());
    }
    Ok(())
}

#[tauri::command]
pub async fn local_share_accept(
    state: State<'_, LocalShareState>,
    transfer_id: String,
    pin: String,
) -> Result<(), String> {
    let mut pending = state.pending.lock().await;
    let tx = pending
        .remove(&transfer_id)
        .ok_or_else(|| format!("no pending transfer: {}", transfer_id))?;
    tx.send(Some(pin))
        .map_err(|_| "receiver task dropped".to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn local_share_reject(
    state: State<'_, LocalShareState>,
    transfer_id: String,
) -> Result<(), String> {
    let mut pending = state.pending.lock().await;
    let tx = pending
        .remove(&transfer_id)
        .ok_or_else(|| format!("no pending transfer: {}", transfer_id))?;
    tx.send(None)
        .map_err(|_| "receiver task dropped".to_string())?;
    Ok(())
}

async fn handle_incoming(
    mut stream: TcpStream,
    peer_addr: String,
    staging: PathBuf,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<Option<String>>>>>,
    app: AppHandle,
) -> Result<(), String> {
    let mut magic = [0u8; 12];
    read_exact_async(&mut stream, &mut magic).await?;
    if &magic != PROTOCOL_MAGIC {
        return Err("bad protocol magic".into());
    }

    let manifest: TransferManifest = read_framed_json(&mut stream).await?;
    let salt_bytes = B64.decode(&manifest.salt_b64).map_err(|e| e.to_string())?;
    if salt_bytes.len() != 16 {
        return Err(format!("bad salt length: {}", salt_bytes.len()));
    }
    let mut salt4 = [0u8; 4];
    salt4.copy_from_slice(&salt_bytes[..4]);

    // Notify the UI so it can show the accept dialog with the manifest details.
    let _ = app.emit(
        "tlantishare://incoming-request",
        serde_json::json!({
            "transferId": manifest.transfer_id,
            "fromAlias": manifest.from_alias,
            "fromAddress": peer_addr,
            "files": manifest.files,
        }),
    );

    let (decision_tx, decision_rx) = oneshot::channel::<Option<String>>();
    {
        let mut p = pending.lock().await;
        p.insert(manifest.transfer_id.clone(), decision_tx);
    }

    let pin = match tokio::time::timeout(std::time::Duration::from_secs(120), decision_rx).await {
        Ok(Ok(Some(pin))) => pin,
        _ => {
            stream
                .write_all(&[REJECT_BYTE])
                .await
                .map_err(|e| e.to_string())?;
            return Err("transfer rejected or timed out".into());
        }
    };

    stream
        .write_all(&[ACCEPT_BYTE])
        .await
        .map_err(|e| e.to_string())?;

    let key_bytes = derive_key(&pin, &salt_bytes)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let target_dir = staging.join(&manifest.transfer_id);
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| e.to_string())?;

    let mut total_received: u64 = 0;
    let total_bytes: u64 = manifest.files.iter().map(|f| f.size).sum();

    for entry in &manifest.files {
        let safe_name = sanitize_filename(&entry.name);
        let dest = target_dir.join(&safe_name);
        let mut file = tokio::fs::File::create(&dest)
            .await
            .map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();

        loop {
            let mut len_buf = [0u8; 4];
            read_exact_async(&mut stream, &mut len_buf).await?;
            let len = u32::from_be_bytes(len_buf) as usize;
            if len == 0 {
                break;
            }
            if len > CHUNK_PLAINTEXT_BYTES + 32 {
                return Err(format!("chunk too large: {} bytes", len));
            }
            let mut nonce_bytes = [0u8; 12];
            read_exact_async(&mut stream, &mut nonce_bytes).await?;
            if nonce_bytes[..4] != salt4 {
                return Err("nonce salt mismatch".into());
            }
            let mut ct = vec![0u8; len];
            read_exact_async(&mut stream, &mut ct).await?;
            let nonce = Nonce::from_slice(&nonce_bytes);
            let plaintext = cipher
                .decrypt(nonce, ct.as_ref())
                .map_err(|e| format!("aes-gcm decrypt: {} (wrong PIN?)", e))?;
            hasher.update(&plaintext);
            file.write_all(&plaintext)
                .await
                .map_err(|e| e.to_string())?;
            total_received += plaintext.len() as u64;
            let _ = app.emit(
                "tlantishare://recv-progress",
                serde_json::json!({
                    "transferId": manifest.transfer_id,
                    "bytesReceived": total_received,
                    "totalBytes": total_bytes,
                    "currentFile": entry.name,
                }),
            );
        }
        file.flush().await.map_err(|e| e.to_string())?;

        let actual_sha = format!("{:x}", hasher.finalize());
        if actual_sha != entry.sha256 {
            return Err(format!(
                "sha256 mismatch for {}: expected {}, got {}",
                entry.name, entry.sha256, actual_sha
            ));
        }
    }

    let _ = app.emit(
        "tlantishare://recv-complete",
        serde_json::json!({
            "transferId": manifest.transfer_id,
            "stagingPath": target_dir.to_string_lossy(),
            "files": manifest.files,
        }),
    );
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "transfer.bin".to_string()
    } else {
        cleaned
    }
}

#[tauri::command]
pub fn local_share_generate_pin() -> String {
    let mut buf = [0u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    let n = u32::from_be_bytes(buf) % 1_000_000;
    format!("{:06}", n)
}

#[tauri::command]
pub fn local_share_local_transports() -> Vec<String> {
    let mut transports = vec!["lan".to_string()];
    if cfg!(target_os = "macos") {
        // AirDrop wraps Apple private APIs — reachable today only via a Swift sidecar.
        // We surface it in the catalog so the UI can render the badge but the send
        // path returns "transport-not-enabled" until V279 sidecar lands.
        transports.push("airdrop".into());
    }
    transports.push("bluetooth".into());
    transports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_derivation_is_deterministic() {
        let salt = [0xAB; 16];
        let a = derive_key("123456", &salt).unwrap();
        let b = derive_key("123456", &salt).unwrap();
        assert_eq!(a, b);
        let c = derive_key("654321", &salt).unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn nonce_carries_salt_prefix() {
        let n = nonce_for([1, 2, 3, 4], 7);
        assert_eq!(&n[..4], &[1, 2, 3, 4]);
        assert_eq!(&n[4..], &7u64.to_be_bytes());
    }

    #[test]
    fn fingerprint_is_stable() {
        let fp = fingerprint_for("device-uuid-123");
        assert_eq!(fp, fingerprint_for("device-uuid-123"));
        assert_ne!(fp, fingerprint_for("other-uuid"));
    }

    #[test]
    fn sanitize_strips_path_traversal() {
        assert_eq!(sanitize_filename("a/b/c.stl"), "a_b_c.stl");
        assert_eq!(sanitize_filename(""), "transfer.bin");
        assert_eq!(sanitize_filename(".."), "transfer.bin");
    }

    #[test]
    fn pin_is_six_digits() {
        for _ in 0..16 {
            let p = local_share_generate_pin();
            assert_eq!(p.len(), 6);
            assert!(p.chars().all(|c| c.is_ascii_digit()));
        }
    }
}

#[allow(dead_code)]
fn _ensure_path_owned(_p: &Path) {}
