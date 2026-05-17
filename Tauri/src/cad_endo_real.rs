// AR-V378 — Endo (Tauri command surface).
//
// Two commands:
//   * `cad_endo_chamber_build` — build a flat-bottom tapered cylinder for endo crown chamber.
//   * `cad_endo_estimate_canal_axis` — PCA-based canal axis estimation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_endo::canal::{estimate_canal_axis, CanalAxis};
use tlanticad_endo::chamber::{build_chamber_mesh, ChamberParams, ChamberReport};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum EndoError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), EndoError> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;
    use stl_io::{Triangle, Vector};
    let triangles: Vec<Triangle> = mesh
        .indices
        .iter()
        .map(|tri| {
            let v0 = mesh.vertices[tri[0] as usize];
            let v1 = mesh.vertices[tri[1] as usize];
            let v2 = mesh.vertices[tri[2] as usize];
            let e1 = v1 - v0;
            let e2 = v2 - v0;
            let n = e1.cross(&e2);
            let len = n.norm().max(f64::EPSILON);
            Triangle {
                normal: Vector::new([(n.x / len) as f32, (n.y / len) as f32, (n.z / len) as f32]),
                vertices: [
                    Vector::new([v0.x as f32, v0.y as f32, v0.z as f32]),
                    Vector::new([v1.x as f32, v1.y as f32, v1.z as f32]),
                    Vector::new([v2.x as f32, v2.y as f32, v2.z as f32]),
                ],
            }
        })
        .collect();
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|e| EndoError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| EndoError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), EndoError> {
    Err(EndoError::FormatsFeatureMissing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndoChamberRequest {
    pub output: PathBuf,
    pub center: [f64; 3],
    pub axis: [f64; 3],
    pub diameter_mm: f64,
    pub depth_mm: f64,
    #[serde(default = "default_taper")]
    pub taper_deg: f64,
    #[serde(default = "default_radial")]
    pub radial_segments: u32,
    #[serde(default = "default_axial")]
    pub axial_segments: u32,
}

fn default_taper() -> f64 {
    3.0
}
fn default_radial() -> u32 {
    32
}
fn default_axial() -> u32 {
    4
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndoChamberResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
    pub watertight: bool,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_endo_chamber_build(
    request: EndoChamberRequest,
) -> Result<EndoChamberResponse, EndoError> {
    let params = ChamberParams {
        center: request.center,
        axis: request.axis,
        diameter_mm: request.diameter_mm,
        depth_mm: request.depth_mm,
        taper_deg: request.taper_deg,
        radial_segments: request.radial_segments,
        axial_segments: request.axial_segments,
    };
    let (mesh, report): (Mesh, ChamberReport) = build_chamber_mesh(&params);
    if mesh.triangle_count() == 0 {
        return Err(EndoError::Invalid {
            message: "chamber parameters produced an empty mesh".into(),
        });
    }
    write_stl(&mesh, &request.output)?;
    Ok(EndoChamberResponse {
        output: request.output,
        triangles: report.triangles,
        vertices: report.vertices,
        volume_mm3: report.volume_mm3,
        watertight: report.watertight,
        backend: "tlanticad-endo::chamber",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndoCanalAxisRequest {
    pub points: Vec<[f64; 3]>,
    /// "Down" direction (axis flips to align with this hemisphere).
    pub occlusal_down: [f64; 3],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndoCanalAxisResponse {
    pub axis: CanalAxis,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_endo_estimate_canal_axis(
    request: EndoCanalAxisRequest,
) -> Result<EndoCanalAxisResponse, EndoError> {
    if request.points.len() < 3 {
        return Err(EndoError::Invalid {
            message: "canal axis estimation needs ≥3 points".into(),
        });
    }
    let pts: Vec<Point3<f64>> = request
        .points
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let down = Vector3::new(
        request.occlusal_down[0],
        request.occlusal_down[1],
        request.occlusal_down[2],
    );
    let axis = estimate_canal_axis(&pts, down).ok_or_else(|| EndoError::Invalid {
        message: "PCA failed (degenerate point cloud)".into(),
    })?;
    Ok(EndoCanalAxisResponse {
        axis,
        backend: "tlanticad-endo::canal",
    })
}
