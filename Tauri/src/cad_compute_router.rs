// MP-101 / MP-102 / MP-110 — Compute router (Tauri command surface).
//
// Three commands:
//   * `cad_compute_run_bench` — runs the synthetic bench across all available backends and
//     persists the resulting `BenchProfile` to <app_data>/compute-profile.json.
//   * `cad_compute_status` — returns the current backend ranking + status label per op.
//   * `cad_compute_set_energy_mode` — toggle between Performance and LowPower routing.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tlanticad_compute::{
    BenchProfile, BoundaryBenchProfile, BoundaryBenchRequest, ComputeKind, ComputeOp,
    ComputeRouter, EnergyMode,
};

#[derive(Default)]
pub struct ComputeRouterState {
    inner: Mutex<Option<ComputeRouter>>,
    profile: Mutex<Option<BenchProfile>>,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ComputeCmdError {
    #[error("filesystem: {message}")]
    Fs { message: String },
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

fn profile_path(app: &AppHandle) -> Result<PathBuf, ComputeCmdError> {
    let root = app.path().app_data_dir().map_err(|e| ComputeCmdError::Fs {
        message: format!("app_data_dir: {e}"),
    })?;
    Ok(root.join("compute-profile.json"))
}

fn boundary_profile_path(app: &AppHandle) -> Result<PathBuf, ComputeCmdError> {
    let root = app.path().app_data_dir().map_err(|e| ComputeCmdError::Fs {
        message: format!("app_data_dir: {e}"),
    })?;
    Ok(root.join("compute-boundary-profile.json"))
}

fn host_id() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "unknown-host".to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBenchRequest {
    #[serde(default)]
    pub energy_mode: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBenchResponse {
    pub profile: BenchProfile,
    pub picked_for_distance: ComputeKind,
    pub picked_for_smooth: ComputeKind,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_compute_run_bench(
    app: AppHandle,
    state: State<'_, ComputeRouterState>,
    request: RunBenchRequest,
) -> Result<RunBenchResponse, ComputeCmdError> {
    let energy_mode = match request.energy_mode.as_deref() {
        Some("low-power") | Some("lowPower") | Some("battery") => EnergyMode::LowPower,
        _ => EnergyMode::Performance,
    };
    let pool = ComputeRouter::available_backend_pool();
    let profile = BenchProfile::run_with_backends(
        host_id(),
        energy_mode,
        &pool,
        &[ComputeOp::PerVertexDistance, ComputeOp::LaplacianSmooth],
    );

    let path = profile_path(&app)?;
    profile.save_to(&path).map_err(|e| ComputeCmdError::Fs {
        message: format!("write {}: {}", path.display(), e),
    })?;

    let router = ComputeRouter::new(pool, profile.clone(), energy_mode);
    let picked_distance = router
        .picked_kind_for(ComputeOp::PerVertexDistance)
        .unwrap_or(ComputeKind::Cpu);
    let picked_smooth = router
        .picked_kind_for(ComputeOp::LaplacianSmooth)
        .unwrap_or(ComputeKind::Cpu);

    if let Ok(mut guard) = state.inner.lock() {
        *guard = Some(router);
    }
    if let Ok(mut guard) = state.profile.lock() {
        *guard = Some(profile.clone());
    }

    Ok(RunBenchResponse {
        profile,
        picked_for_distance: picked_distance,
        picked_for_smooth: picked_smooth,
        backend: "tlanticad-compute::router",
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeStatusResponse {
    pub host_id: String,
    pub energy_mode: EnergyMode,
    pub status_per_vertex_distance: String,
    pub status_laplacian_smooth: String,
    pub backends_available: Vec<&'static str>,
    pub profile_persisted: bool,
}

#[tauri::command]
pub fn cad_compute_status(
    app: AppHandle,
    state: State<'_, ComputeRouterState>,
) -> Result<ComputeStatusResponse, ComputeCmdError> {
    let path = profile_path(&app).ok();
    let persisted = path.as_ref().map(|p| p.exists()).unwrap_or(false);

    let router = state
        .inner
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(ComputeRouter::cpu_only);

    let backends_available: Vec<&'static str> = router
        .runnable()
        .iter()
        .map(|b| b.capabilities().kind.id())
        .collect();

    Ok(ComputeStatusResponse {
        host_id: host_id(),
        energy_mode: router.energy_mode(),
        status_per_vertex_distance: router.status_label(ComputeOp::PerVertexDistance),
        status_laplacian_smooth: router.status_label(ComputeOp::LaplacianSmooth),
        backends_available,
        profile_persisted: persisted,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetEnergyModeRequest {
    pub mode: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetEnergyModeResponse {
    pub mode: EnergyMode,
}

#[tauri::command]
pub fn cad_compute_set_energy_mode(
    state: State<'_, ComputeRouterState>,
    request: SetEnergyModeRequest,
) -> Result<SetEnergyModeResponse, ComputeCmdError> {
    let mode = match request.mode.as_str() {
        "low-power" | "lowPower" | "battery" => EnergyMode::LowPower,
        "performance" | "perf" => EnergyMode::Performance,
        other => {
            return Err(ComputeCmdError::Invalid {
                message: format!("unknown energy mode: {other}"),
            })
        }
    };
    let profile = state
        .profile
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| BenchProfile::run_cpu_only(host_id()));
    let pool = ComputeRouter::available_backend_pool();
    let router = ComputeRouter::new(pool, profile.clone(), mode);
    if let Ok(mut guard) = state.inner.lock() {
        *guard = Some(router);
    }
    if let Ok(mut guard) = state.profile.lock() {
        *guard = Some(profile);
    }
    Ok(SetEnergyModeResponse { mode })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBoundaryBenchRequest {
    #[serde(default)]
    pub sample_vertices: Option<usize>,
    #[serde(default)]
    pub energy_mode: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBoundaryBenchResponse {
    pub profile: BoundaryBenchProfile,
    pub profile_path: String,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_compute_run_boundary_bench(
    app: AppHandle,
    request: RunBoundaryBenchRequest,
) -> Result<RunBoundaryBenchResponse, ComputeCmdError> {
    let energy_mode = match request.energy_mode.as_deref() {
        Some("low-power") | Some("lowPower") | Some("battery") => Some(EnergyMode::LowPower),
        Some("performance") | Some("perf") => Some(EnergyMode::Performance),
        Some(other) => {
            return Err(ComputeCmdError::Invalid {
                message: format!("unknown energy mode: {other}"),
            })
        }
        None => None,
    };
    let profile = BoundaryBenchProfile::run(
        host_id(),
        BoundaryBenchRequest {
            sample_vertices: request.sample_vertices,
            energy_mode,
        },
    );
    let path = boundary_profile_path(&app)?;
    profile.save_to(&path).map_err(|e| ComputeCmdError::Fs {
        message: format!("write {}: {}", path.display(), e),
    })?;

    Ok(RunBoundaryBenchResponse {
        profile,
        profile_path: path.to_string_lossy().into_owned(),
        backend: "tlanticad-compute::boundary",
    })
}
