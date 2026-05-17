// AR-V374 — Freeform brush (Tauri command surface).
//
// Three commands:
//   * `cad_freeform_paint_pull`     — push/pull along normal.
//   * `cad_freeform_paint_smooth`   — local Laplacian smoothing.
//   * `cad_freeform_paint_drape`    — drag along arbitrary direction.
// Plus `cad_freeform_emergence_profile` — generate emergence profile shell.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_freeform::paint_pull::{
    build_emergence_profile, paint_drape, paint_pull, paint_smooth, BrushParams, BrushReport,
};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FreeformError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, FreeformError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| FreeformError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| FreeformError::Io {
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
    let mut mesh = Mesh::new("freeform-input");
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, FreeformError> {
    Err(FreeformError::FormatsFeatureMissing)
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), FreeformError> {
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
        .map_err(|e| FreeformError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| FreeformError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), FreeformError> {
    Err(FreeformError::FormatsFeatureMissing)
}

fn brush_from_request(
    center: [f64; 3],
    radius_mm: f64,
    strength: f64,
    falloff: f64,
) -> BrushParams {
    BrushParams {
        center,
        radius_mm,
        strength,
        falloff,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaintPullRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub center: [f64; 3],
    pub radius_mm: f64,
    pub amount_mm: f64,
    #[serde(default = "default_strength")]
    pub strength: f64,
    #[serde(default = "default_falloff")]
    pub falloff: f64,
}

fn default_strength() -> f64 {
    1.0
}
fn default_falloff() -> f64 {
    2.0
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrushOpResponse {
    pub output: PathBuf,
    pub vertices_affected: usize,
    pub max_displacement_mm: f64,
    pub mean_displacement_mm: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_freeform_paint_pull(
    request: PaintPullRequest,
) -> Result<BrushOpResponse, FreeformError> {
    if !request.input.exists() {
        return Err(FreeformError::InputNotFound {
            path: request.input.to_string_lossy().into_owned(),
        });
    }
    let mut mesh = read_stl(&request.input)?;
    let params = brush_from_request(
        request.center,
        request.radius_mm,
        request.strength,
        request.falloff,
    );
    let report: BrushReport = paint_pull(&mut mesh, &params, request.amount_mm);
    write_stl(&mesh, &request.output)?;
    Ok(BrushOpResponse {
        output: request.output,
        vertices_affected: report.vertices_affected,
        max_displacement_mm: report.max_displacement_mm,
        mean_displacement_mm: report.mean_displacement_mm,
        backend: "tlanticad-freeform::paint_pull",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaintSmoothRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub center: [f64; 3],
    pub radius_mm: f64,
    #[serde(default = "default_smooth_iters")]
    pub iterations: u32,
    #[serde(default = "default_strength")]
    pub strength: f64,
    #[serde(default = "default_falloff")]
    pub falloff: f64,
}

fn default_smooth_iters() -> u32 {
    3
}

#[tauri::command]
pub fn cad_freeform_paint_smooth(
    request: PaintSmoothRequest,
) -> Result<BrushOpResponse, FreeformError> {
    if !request.input.exists() {
        return Err(FreeformError::InputNotFound {
            path: request.input.to_string_lossy().into_owned(),
        });
    }
    let mut mesh = read_stl(&request.input)?;
    let params = brush_from_request(
        request.center,
        request.radius_mm,
        request.strength,
        request.falloff,
    );
    let report = paint_smooth(&mut mesh, &params, request.iterations);
    write_stl(&mesh, &request.output)?;
    Ok(BrushOpResponse {
        output: request.output,
        vertices_affected: report.vertices_affected,
        max_displacement_mm: report.max_displacement_mm,
        mean_displacement_mm: report.mean_displacement_mm,
        backend: "tlanticad-freeform::paint_smooth",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaintDrapeRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub center: [f64; 3],
    pub radius_mm: f64,
    pub direction_mm: [f64; 3],
    #[serde(default = "default_strength")]
    pub strength: f64,
    #[serde(default = "default_falloff")]
    pub falloff: f64,
}

#[tauri::command]
pub fn cad_freeform_paint_drape(
    request: PaintDrapeRequest,
) -> Result<BrushOpResponse, FreeformError> {
    if !request.input.exists() {
        return Err(FreeformError::InputNotFound {
            path: request.input.to_string_lossy().into_owned(),
        });
    }
    let mut mesh = read_stl(&request.input)?;
    let params = brush_from_request(
        request.center,
        request.radius_mm,
        request.strength,
        request.falloff,
    );
    let report = paint_drape(&mut mesh, &params, request.direction_mm);
    write_stl(&mesh, &request.output)?;
    Ok(BrushOpResponse {
        output: request.output,
        vertices_affected: report.vertices_affected,
        max_displacement_mm: report.max_displacement_mm,
        mean_displacement_mm: report.mean_displacement_mm,
        backend: "tlanticad-freeform::paint_drape",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergenceProfileRequest {
    pub margin_polyline: Vec<[f64; 3]>,
    pub insertion_axis: [f64; 3],
    pub output: PathBuf,
    pub height_mm: f64,
    pub top_radius_mm: f64,
    #[serde(default = "default_axial")]
    pub axial_segments: u32,
}

fn default_axial() -> u32 {
    8
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergenceProfileResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_freeform_emergence_profile(
    request: EmergenceProfileRequest,
) -> Result<EmergenceProfileResponse, FreeformError> {
    if request.margin_polyline.len() < 3 {
        return Err(FreeformError::Invalid {
            message: "margin polyline must have at least 3 points".into(),
        });
    }
    let polyline: Vec<Point3<f64>> = request
        .margin_polyline
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let axis = Vector3::new(
        request.insertion_axis[0],
        request.insertion_axis[1],
        request.insertion_axis[2],
    );
    let mesh = build_emergence_profile(
        &polyline,
        axis,
        request.height_mm,
        request.top_radius_mm,
        request.axial_segments,
    );
    write_stl(&mesh, &request.output)?;
    Ok(EmergenceProfileResponse {
        output: request.output,
        triangles: mesh.triangle_count(),
        vertices: mesh.vertex_count(),
        backend: "tlanticad-freeform::paint_pull::emergence",
    })
}
