// AR-V364 — Insertion direction (Tauri command surface).
//
// Two commands:
//   * `cad_insertion_compute` — compute insertion axis for a single mesh + per-vertex severity.
//   * `cad_insertion_unify_bridge` — unify multiple per-tooth axes into a bridge axis.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_geometry::insertion::{
    classify_undercut, detect_insertion_axis, secondary_axis, severity_counts, unify_bridge_axes,
    InsertionAxis, SeverityCounts, UndercutThresholds,
};
use tlanticad_mesh::nalgebra::Vector3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum InsertionError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("mesh has no vertex normals")]
    NoNormals,
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, InsertionError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| InsertionError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| InsertionError::Io {
        message: format!("parse {}: {}", path.display(), e),
    })?;
    let vertices: Vec<tlanticad_mesh::nalgebra::Point3<f64>> = stl
        .vertices
        .iter()
        .map(|v| tlanticad_mesh::nalgebra::Point3::new(v[0] as f64, v[1] as f64, v[2] as f64))
        .collect();
    let indices: Vec<[u32; 3]> = stl
        .faces
        .iter()
        .map(|f| {
            [
                f.vertices[0] as u32,
                f.vertices[1] as u32,
                f.vertices[2] as u32,
            ]
        })
        .collect();
    let mut mesh = Mesh::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "mesh".into()),
    );
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, InsertionError> {
    Err(InsertionError::FormatsFeatureMissing)
}

// ---------- compute axis for a single tooth ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertionComputeRequest {
    pub input: PathBuf,
    /// "Up" hint — typically world +Z. Will be normalized.
    pub occlusal_hint: [f64; 3],
    /// If set, also compute the secondary axis given this mesial-distal direction.
    #[serde(default)]
    pub mesial_distal: Option<[f64; 3]>,
    /// Threshold overrides (optional). Defaults from exocad: warn at 95°, error at 105°.
    #[serde(default)]
    pub thresholds: Option<UndercutThresholdsDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndercutThresholdsDto {
    pub warn_dot: f64,
    pub error_dot: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertionComputeResponse {
    pub axis: InsertionAxis,
    pub secondary: Option<[f64; 3]>,
    pub severity_counts: SeverityCounts,
    pub vertex_count: usize,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_insertion_compute(
    request: InsertionComputeRequest,
) -> Result<InsertionComputeResponse, InsertionError> {
    if !request.input.exists() {
        return Err(InsertionError::InputNotFound {
            path: request.input.to_string_lossy().into_owned(),
        });
    }
    let mesh = read_stl(&request.input)?;
    if mesh.normals.is_empty() {
        return Err(InsertionError::NoNormals);
    }
    let hint = Vector3::new(
        request.occlusal_hint[0],
        request.occlusal_hint[1],
        request.occlusal_hint[2],
    );
    let axis = detect_insertion_axis(&mesh.normals, hint).ok_or(InsertionError::NoNormals)?;
    let axis_vec = Vector3::new(axis.axis[0], axis.axis[1], axis.axis[2]);

    let secondary = request.mesial_distal.map(|md| {
        let s = secondary_axis(axis_vec, Vector3::new(md[0], md[1], md[2]));
        [s.x, s.y, s.z]
    });

    let thresholds = request
        .thresholds
        .map(|t| UndercutThresholds {
            warn_dot: t.warn_dot,
            error_dot: t.error_dot,
        })
        .unwrap_or_default();
    let severities = classify_undercut(&mesh.normals, axis_vec, &thresholds);
    let counts = severity_counts(&severities);

    Ok(InsertionComputeResponse {
        axis,
        secondary,
        severity_counts: counts,
        vertex_count: mesh.normals.len(),
        backend: "tlanticad-geometry::insertion",
    })
}

// ---------- unify bridge axes ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifyBridgeRequest {
    /// Per-tooth axes (each as `[x, y, z]`).
    pub axes: Vec<[f64; 3]>,
    /// Optional per-tooth weights (default 1.0 each).
    #[serde(default)]
    pub weights: Option<Vec<f64>>,
    /// Maximum allowed deviation per tooth from the unified axis (degrees).
    #[serde(default = "default_max_deviation")]
    pub max_deviation_deg: f64,
}

fn default_max_deviation() -> f64 {
    8.0
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifyBridgeResponse {
    pub unified: [f64; 3],
    pub max_deviation_deg: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_insertion_unify_bridge(
    request: UnifyBridgeRequest,
) -> Result<UnifyBridgeResponse, InsertionError> {
    if request.axes.is_empty() {
        return Err(InsertionError::Invalid {
            message: "axes must not be empty".into(),
        });
    }
    let axes: Vec<Vector3<f64>> = request
        .axes
        .iter()
        .map(|a| Vector3::new(a[0], a[1], a[2]))
        .collect();
    let weights = request.weights.as_deref();
    let (unified, dev) =
        unify_bridge_axes(&axes, weights, request.max_deviation_deg).ok_or_else(|| {
            InsertionError::Invalid {
                message: "unification failed".into(),
            }
        })?;
    Ok(UnifyBridgeResponse {
        unified: [unified.x, unified.y, unified.z],
        max_deviation_deg: dev,
        backend: "tlanticad-geometry::insertion",
    })
}
