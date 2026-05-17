// AR-V371 — Real abutment loft (Tauri command surface).
//
// Replaces audit no-stubs item #12 (the previous 1-triangle stub in `cad_abutment::generate`).
//
// Two commands:
//   * `cad_abutment_generate_real` — loft from margin polyline + style, write STL.
//   * `cad_abutment_validate_real` — fast UI feedback (limit warnings) without generating.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_abutment::edit::{
    generate_loft, validate, AbutmentEditParams, AbutmentReport, AbutmentStyle, LimitWarning,
};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AbutmentRealError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), AbutmentRealError> {
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
        .map_err(|e| AbutmentRealError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| AbutmentRealError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), AbutmentRealError> {
    Err(AbutmentRealError::FormatsFeatureMissing)
}

fn parse_style(s: &str) -> AbutmentStyle {
    match s.to_lowercase().as_str() {
        "cylindrical" | "cylinder" => AbutmentStyle::Cylindrical,
        "angular" | "angled" => AbutmentStyle::Angular,
        "legacy" => AbutmentStyle::Legacy,
        _ => AbutmentStyle::Standard,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentGenerateRealRequest {
    pub margin_polyline: Vec<[f64; 3]>,
    pub insertion_axis: [f64; 3],
    pub output: PathBuf,
    #[serde(default = "default_style")]
    pub style: String,
    #[serde(default = "default_height")]
    pub height_mm: f64,
    #[serde(default = "default_top_radius")]
    pub top_radius_mm: f64,
    #[serde(default = "default_axial")]
    pub axial_segments: u32,
    #[serde(default = "default_radial")]
    pub radial_segments: u32,
    #[serde(default = "default_screw_diameter")]
    pub screw_channel_diameter_mm: f64,
    #[serde(default)]
    pub screw_channel_angle_deg: f64,
    #[serde(default = "default_bulge")]
    pub anatomic_bulge: f64,
}

fn default_style() -> String {
    "standard".into()
}
fn default_height() -> f64 {
    5.0
}
fn default_top_radius() -> f64 {
    2.0
}
fn default_axial() -> u32 {
    8
}
fn default_radial() -> u32 {
    32
}
fn default_screw_diameter() -> f64 {
    2.3
}
fn default_bulge() -> f64 {
    0.4
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentGenerateRealResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
    pub warnings: Vec<LimitWarning>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_abutment_generate_real(
    request: AbutmentGenerateRealRequest,
) -> Result<AbutmentGenerateRealResponse, AbutmentRealError> {
    if request.margin_polyline.len() < 3 {
        return Err(AbutmentRealError::Invalid {
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
    let params = AbutmentEditParams {
        style: parse_style(&request.style),
        height_mm: request.height_mm,
        top_radius_mm: request.top_radius_mm,
        axial_segments: request.axial_segments,
        radial_segments: request.radial_segments,
        screw_channel_diameter_mm: request.screw_channel_diameter_mm,
        screw_channel_angle_deg: request.screw_channel_angle_deg,
        anatomic_bulge: request.anatomic_bulge,
    };
    let (mesh, report): (Mesh, AbutmentReport) = generate_loft(&polyline, axis, &params);
    write_stl(&mesh, &request.output)?;
    Ok(AbutmentGenerateRealResponse {
        output: request.output,
        triangles: report.triangles,
        vertices: report.vertices,
        volume_mm3: report.volume_mm3,
        warnings: report.warnings,
        backend: "tlanticad-abutment::edit",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentValidateRealRequest {
    pub margin_polyline: Vec<[f64; 3]>,
    #[serde(default = "default_style")]
    pub style: String,
    #[serde(default = "default_height")]
    pub height_mm: f64,
    #[serde(default = "default_top_radius")]
    pub top_radius_mm: f64,
    #[serde(default = "default_axial")]
    pub axial_segments: u32,
    #[serde(default = "default_radial")]
    pub radial_segments: u32,
    #[serde(default = "default_screw_diameter")]
    pub screw_channel_diameter_mm: f64,
    #[serde(default)]
    pub screw_channel_angle_deg: f64,
    #[serde(default = "default_bulge")]
    pub anatomic_bulge: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentValidateRealResponse {
    pub warnings: Vec<LimitWarning>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_abutment_validate_real(
    request: AbutmentValidateRealRequest,
) -> Result<AbutmentValidateRealResponse, AbutmentRealError> {
    let polyline: Vec<Point3<f64>> = request
        .margin_polyline
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let params = AbutmentEditParams {
        style: parse_style(&request.style),
        height_mm: request.height_mm,
        top_radius_mm: request.top_radius_mm,
        axial_segments: request.axial_segments,
        radial_segments: request.radial_segments,
        screw_channel_diameter_mm: request.screw_channel_diameter_mm,
        screw_channel_angle_deg: request.screw_channel_angle_deg,
        anatomic_bulge: request.anatomic_bulge,
    };
    let warnings = validate(&polyline, &params);
    Ok(AbutmentValidateRealResponse {
        warnings,
        backend: "tlanticad-abutment::edit",
    })
}
