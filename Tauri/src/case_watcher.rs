// AR-V382 — Case watcher (Tauri command surface).
//
// Watches the user's case folder via `notify` and emits Tauri events whenever a relevant
// asset (.stl, .dcm, .nii, .nii.gz, .ply, .obj, case manifest) changes. Replaces the
// `AProjectFileWatcher.cs` + `AProjectLockWatcher.cs` from `DentalDB.Import.Common`.
//
// Two commands:
//   * `case_watcher_start`  — begin watching a directory (recursive). Emits
//                             `case-watcher://event` and `case-watcher://error` events.
//   * `case_watcher_stop`   — stop the watcher.
//
// Events are debounced server-side (200 ms) so a single editor save doesn't trigger ten
// notifications.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::event::{EventKind, ModifyKind};
use notify::{recommended_watcher, Event, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

const RELEVANT_EXTENSIONS: &[&str] = &[
    "stl",
    "dcm",
    "nii",
    "ply",
    "obj",
    "json",
    "xml",
    "case",
    "meshvault",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseEventKind {
    Created,
    Modified,
    Removed,
    Renamed,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseEventDto {
    pub kind: CaseEventKind,
    pub paths: Vec<String>,
    pub timestamp_ms: i64,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CaseWatcherError {
    #[error("path not found: {path}")]
    PathNotFound { path: String },
    #[error("path is not a directory: {path}")]
    NotADirectory { path: String },
    #[error("watcher already active for path: {path}")]
    AlreadyWatching { path: String },
    #[error("watcher backend error: {message}")]
    Backend { message: String },
    #[error("watcher state lock poisoned")]
    StateLock,
}

#[derive(Default)]
pub struct CaseWatcherState {
    inner: Mutex<Option<Holder>>,
}

struct Holder {
    /// Type-erased so the state struct can stay simple. We never read it back; we just keep it
    /// alive to keep the watcher running.
    _watcher: Box<dyn Watcher + Send>,
    root: PathBuf,
}

impl std::fmt::Debug for Holder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Holder")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

fn classify(kind: EventKind) -> CaseEventKind {
    match kind {
        EventKind::Create(_) => CaseEventKind::Created,
        EventKind::Modify(ModifyKind::Name(_)) => CaseEventKind::Renamed,
        EventKind::Modify(_) => CaseEventKind::Modified,
        EventKind::Remove(_) => CaseEventKind::Removed,
        _ => CaseEventKind::Other,
    }
}

fn path_is_relevant(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        // No extension: still relevant for case folder manifest changes.
        return path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase().contains("case"))
            .unwrap_or(false);
    };
    let ext_lower = ext.to_lowercase();
    RELEVANT_EXTENSIONS.iter().any(|e| *e == ext_lower)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseWatcherStartRequest {
    pub directory: PathBuf,
    #[serde(default = "default_recursive")]
    pub recursive: bool,
}

fn default_recursive() -> bool {
    true
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseWatcherStartResponse {
    pub directory: PathBuf,
    pub recursive: bool,
    pub backend: &'static str,
}

#[tauri::command]
pub fn case_watcher_start(
    app: AppHandle,
    state: State<'_, CaseWatcherState>,
    request: CaseWatcherStartRequest,
) -> Result<CaseWatcherStartResponse, CaseWatcherError> {
    if !request.directory.exists() {
        return Err(CaseWatcherError::PathNotFound {
            path: request.directory.to_string_lossy().into_owned(),
        });
    }
    if !request.directory.is_dir() {
        return Err(CaseWatcherError::NotADirectory {
            path: request.directory.to_string_lossy().into_owned(),
        });
    }
    let mut guard = state
        .inner
        .lock()
        .map_err(|_| CaseWatcherError::StateLock)?;
    if let Some(holder) = guard.as_ref() {
        return Err(CaseWatcherError::AlreadyWatching {
            path: holder.root.to_string_lossy().into_owned(),
        });
    }

    let root = request.directory.clone();
    let recursive = if request.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };
    let app_handle = app.clone();
    let last_emit = Arc::new(Mutex::new(std::time::Instant::now()));
    let debounce = Duration::from_millis(200);

    let handler = move |result: Result<Event, notify::Error>| match result {
        Ok(event) => {
            let relevant_paths: Vec<String> = event
                .paths
                .iter()
                .filter(|p| path_is_relevant(p))
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            if relevant_paths.is_empty() {
                return;
            }
            if let Ok(mut last) = last_emit.lock() {
                if last.elapsed() < debounce {
                    return;
                }
                *last = std::time::Instant::now();
            }
            let dto = CaseEventDto {
                kind: classify(event.kind),
                paths: relevant_paths,
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
            };
            let _ = app_handle.emit("case-watcher://event", &dto);
        }
        Err(err) => {
            let _ = app_handle.emit(
                "case-watcher://error",
                serde_json::json!({ "message": err.to_string() }),
            );
        }
    };

    let mut watcher = recommended_watcher(handler).map_err(|e| CaseWatcherError::Backend {
        message: format!("init watcher: {e}"),
    })?;
    watcher
        .watch(&root, recursive)
        .map_err(|e| CaseWatcherError::Backend {
            message: format!("watch {}: {e}", root.display()),
        })?;
    *guard = Some(Holder {
        _watcher: Box::new(watcher),
        root: root.clone(),
    });
    Ok(CaseWatcherStartResponse {
        directory: root,
        recursive: request.recursive,
        backend: "notify-rs",
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseWatcherStopResponse {
    pub stopped: bool,
}

#[tauri::command]
pub fn case_watcher_stop(
    state: State<'_, CaseWatcherState>,
) -> Result<CaseWatcherStopResponse, CaseWatcherError> {
    let mut guard = state
        .inner
        .lock()
        .map_err(|_| CaseWatcherError::StateLock)?;
    let stopped = guard.is_some();
    *guard = None;
    Ok(CaseWatcherStopResponse { stopped })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relevant_path_accepts_stl() {
        assert!(path_is_relevant(Path::new("/tmp/foo/bar.stl")));
        assert!(path_is_relevant(Path::new("scan.STL")));
    }

    #[test]
    fn relevant_path_accepts_dicom_and_nifti() {
        assert!(path_is_relevant(Path::new("foo.dcm")));
        assert!(path_is_relevant(Path::new("vol.nii")));
    }

    #[test]
    fn relevant_path_rejects_random() {
        assert!(!path_is_relevant(Path::new("/tmp/foo.png")));
        assert!(!path_is_relevant(Path::new("/tmp/notes.md")));
    }

    #[test]
    fn relevant_path_accepts_case_filename_without_extension() {
        assert!(path_is_relevant(Path::new("/tmp/casefile")));
    }
}
