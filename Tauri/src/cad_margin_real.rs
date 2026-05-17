// AR-V365 — Margin processors (Tauri command surface).
//
// Three commands:
//   * `cad_margin_detect_real`  — detect margin from boundary or curvature ridge.
//   * `cad_margin_correct_real` — Laplacian-smooth a polyline.
//   * `cad_margin_repair_real`  — close gaps in a polyline.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_mesh::margin::{
    correct_polyline, detect_from_boundary, detect_from_curvature, repair_polyline, MarginPolyline,
};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MarginError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("polyline is empty or too short")]
    EmptyPolyline,
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, MarginError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| MarginError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| MarginError::Io {
        message: format!("parse {}: {}", path.display(), e),
    })?;
    let vertices: Vec<Point3<f64>> = stl
        .vertices
        .iter()
        .map(|v| Point3::new(v[0] as f64, v[1] as f64, v[2] as f64))
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
    let mut mesh = Mesh::new("margin-input");
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, MarginError> {
    Err(MarginError::FormatsFeatureMissing)
}

// ---------- detect ----------

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DetectMode {
    /// Returns boundary edge loops (open prep meshes).
    Boundary,
    /// Curvature-ridge tracing (closed prep meshes).
    Curvature,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginDetectRequest {
    pub input: PathBuf,
    pub mode: DetectMode,
    /// Required for `Curvature` mode: prep insertion direction (will be normalized).
    #[serde(default)]
    pub insertion_axis: Option<[f64; 3]>,
    /// Curvature threshold (mm⁻¹) — only used in Curvature mode. Typical 0.5..1.5.
    #[serde(default = "default_curv_thresh")]
    pub curvature_threshold: f64,
    /// Perpendicular tolerance — 1.0 = strictly perpendicular, 0.7 = within ~45°.
    #[serde(default = "default_perp_tol")]
    pub perpendicular_tol_dot: f64,
}

fn default_curv_thresh() -> f64 {
    0.5
}
fn default_perp_tol() -> f64 {
    0.7
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginDetectResponse {
    pub polylines: Vec<MarginPolyline>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_margin_detect_real(
    request: MarginDetectRequest,
) -> Result<MarginDetectResponse, MarginError> {
    if !request.input.exists() {
        return Err(MarginError::InputNotFound {
            path: request.input.to_string_lossy().into_owned(),
        });
    }
    let mesh = read_stl(&request.input)?;
    let polylines = match request.mode {
        DetectMode::Boundary => detect_from_boundary(&mesh),
        DetectMode::Curvature => {
            let axis = request.insertion_axis.unwrap_or([0.0, 0.0, 1.0]);
            let v = Vector3::new(axis[0], axis[1], axis[2]);
            detect_from_curvature(
                &mesh,
                v,
                request.curvature_threshold,
                request.perpendicular_tol_dot,
            )
        }
    };
    Ok(MarginDetectResponse {
        polylines,
        backend: match request.mode {
            DetectMode::Boundary => "tlanticad-mesh::margin::boundary",
            DetectMode::Curvature => "tlanticad-mesh::margin::curvature",
        },
    })
}

// ---------- correct ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginCorrectRequest {
    pub polyline: MarginPolyline,
    #[serde(default = "default_correct_iters")]
    pub iterations: u32,
    #[serde(default = "default_correct_lambda")]
    pub lambda: f64,
}

fn default_correct_iters() -> u32 {
    5
}
fn default_correct_lambda() -> f64 {
    0.5
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginCorrectResponse {
    pub polyline: MarginPolyline,
    pub original_length_mm: f64,
    pub corrected_length_mm: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_margin_correct_real(
    request: MarginCorrectRequest,
) -> Result<MarginCorrectResponse, MarginError> {
    if request.polyline.is_empty() {
        return Err(MarginError::EmptyPolyline);
    }
    let original_length = request.polyline.length_mm();
    let corrected = correct_polyline(&request.polyline, request.iterations, request.lambda);
    let new_length = corrected.length_mm();
    Ok(MarginCorrectResponse {
        polyline: corrected,
        original_length_mm: original_length,
        corrected_length_mm: new_length,
        backend: "tlanticad-mesh::margin::correct",
    })
}

// ---------- repair ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginRepairRequest {
    pub polyline: MarginPolyline,
    #[serde(default = "default_gap_threshold")]
    pub gap_threshold_mm: f64,
}

fn default_gap_threshold() -> f64 {
    1.5
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginRepairResponse {
    pub polyline: MarginPolyline,
    pub points_inserted: usize,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_margin_repair_real(
    request: MarginRepairRequest,
) -> Result<MarginRepairResponse, MarginError> {
    if request.polyline.is_empty() {
        return Err(MarginError::EmptyPolyline);
    }
    let before = request.polyline.len();
    let repaired = repair_polyline(&request.polyline, request.gap_threshold_mm);
    let after = repaired.len();
    Ok(MarginRepairResponse {
        polyline: repaired,
        points_inserted: after.saturating_sub(before),
        backend: "tlanticad-mesh::margin::repair",
    })
}
