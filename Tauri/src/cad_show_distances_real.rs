// AR-V376 — Show distances real (Tauri command surface).
//
// Replaces audit no-stubs item #10 (the previous `cad_show_distances/compute` returned
// hardcoded stats). This command runs a real KD-tree distance scan and returns a full
// histogram + percentile stats + per-vertex distance array (for the shader overlay).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_freeform::distance_shader::{
    compute_distance_field, severity_for, DistanceShaderOptions, DistanceStats, HistogramBucket,
};
use tlanticad_mesh::nalgebra::Point3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ShowDistancesError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, ShowDistancesError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| ShowDistancesError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| ShowDistancesError::Io {
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
    let mut mesh = Mesh::new("show-distances");
    mesh.vertices = vertices;
    mesh.indices = indices;
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, ShowDistancesError> {
    Err(ShowDistancesError::FormatsFeatureMissing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowDistancesRequest {
    pub source: PathBuf,
    pub target: PathBuf,
    /// Number of histogram buckets. Default 16.
    #[serde(default = "default_buckets")]
    pub bucket_count: usize,
    /// If true, also return the full per-vertex distance array (large for big meshes).
    #[serde(default)]
    pub include_per_vertex: bool,
    /// Threshold pair for severity colorization. Defaults to 0.1mm green / 1.0mm red.
    #[serde(default)]
    pub options: Option<ShowDistancesOptionsDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowDistancesOptionsDto {
    pub red_threshold_mm: f64,
    pub green_threshold_mm: f64,
    #[serde(default)]
    pub flag_interpenetration: Option<bool>,
}

fn default_buckets() -> usize {
    16
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowDistancesResponse {
    pub stats: DistanceStats,
    pub histogram: Vec<HistogramBucket>,
    /// Optional per-vertex distance array (only populated if `include_per_vertex` is true).
    pub per_vertex_mm: Option<Vec<f64>>,
    /// Optional per-vertex severity array in [0, 1] (always 0..1; only when per-vertex requested).
    pub per_vertex_severity: Option<Vec<f64>>,
    /// Mirror of the resolved thresholds — useful for the legend in the UI.
    pub red_threshold_mm: f64,
    pub green_threshold_mm: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_show_distances_real(
    request: ShowDistancesRequest,
) -> Result<ShowDistancesResponse, ShowDistancesError> {
    if !request.source.exists() {
        return Err(ShowDistancesError::InputNotFound {
            path: request.source.to_string_lossy().into_owned(),
        });
    }
    if !request.target.exists() {
        return Err(ShowDistancesError::InputNotFound {
            path: request.target.to_string_lossy().into_owned(),
        });
    }
    let source = read_stl(&request.source)?;
    let target = read_stl(&request.target)?;

    let options = match request.options {
        Some(o) => DistanceShaderOptions {
            red_threshold_mm: o.red_threshold_mm,
            green_threshold_mm: o.green_threshold_mm,
            flag_interpenetration: o.flag_interpenetration.unwrap_or(true),
        },
        None => DistanceShaderOptions::default(),
    };

    let (per_vertex, stats, histogram) =
        compute_distance_field(&source, &target, request.bucket_count);

    let (per_vertex_mm, per_vertex_severity) = if request.include_per_vertex {
        let severity: Vec<f64> = per_vertex
            .iter()
            .map(|&d| severity_for(d, &options))
            .collect();
        (Some(per_vertex), Some(severity))
    } else {
        (None, None)
    };

    Ok(ShowDistancesResponse {
        stats,
        histogram,
        per_vertex_mm,
        per_vertex_severity,
        red_threshold_mm: options.red_threshold_mm,
        green_threshold_mm: options.green_threshold_mm,
        backend: "tlanticad-freeform::distance_shader",
    })
}
