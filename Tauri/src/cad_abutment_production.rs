// AR-V372 — Abutment production (Tauri command surface).
//
// Three commands:
//   * `cad_abutment_production_blank` — milling stock cylinder.
//   * `cad_abutment_screw_channel`    — angulated screw channel cylinder (for boolean diff).
//   * `cad_abutment_nesting_puck`     — 98.5 × 16 mm CAM disc + slot positions.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_abutment::production::{
    build_nesting_puck, build_production_blank, build_screw_channel, nesting_slot_positions,
    NestingPuckParams, ProductionBlankParams, ProductionReport, ScrewChannelParams,
};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AbutmentProductionError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), AbutmentProductionError> {
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
        .map_err(|e| AbutmentProductionError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| AbutmentProductionError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), AbutmentProductionError> {
    Err(AbutmentProductionError::FormatsFeatureMissing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionBlankRequest {
    pub output: PathBuf,
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    #[serde(default = "default_blank_diameter")]
    pub diameter_mm: f64,
    #[serde(default = "default_blank_height")]
    pub height_mm: f64,
    #[serde(default = "default_blank_taper")]
    pub taper_deg: f64,
    #[serde(default = "default_blank_radial")]
    pub radial_segments: u32,
    #[serde(default = "default_blank_axial")]
    pub axial_segments: u32,
}

fn default_blank_diameter() -> f64 {
    14.0
}
fn default_blank_height() -> f64 {
    14.5
}
fn default_blank_taper() -> f64 {
    1.5
}
fn default_blank_radial() -> u32 {
    32
}
fn default_blank_axial() -> u32 {
    4
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMeshResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_abutment_production_blank(
    request: ProductionBlankRequest,
) -> Result<ProductionMeshResponse, AbutmentProductionError> {
    let params = ProductionBlankParams {
        diameter_mm: request.diameter_mm,
        height_mm: request.height_mm,
        taper_deg: request.taper_deg,
        radial_segments: request.radial_segments,
        axial_segments: request.axial_segments,
    };
    let origin = Point3::new(request.origin[0], request.origin[1], request.origin[2]);
    let axis = Vector3::new(request.axis[0], request.axis[1], request.axis[2]);
    let (mesh, report): (Mesh, ProductionReport) = build_production_blank(origin, axis, &params);
    write_stl(&mesh, &request.output)?;
    Ok(ProductionMeshResponse {
        output: request.output,
        triangles: report.triangles,
        vertices: report.vertices,
        volume_mm3: report.volume_mm3,
        backend: "tlanticad-abutment::production::blank",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrewChannelRequest {
    pub output: PathBuf,
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    #[serde(default = "default_screw_diameter")]
    pub diameter_mm: f64,
    #[serde(default = "default_screw_length")]
    pub length_mm: f64,
    #[serde(default)]
    pub angle_deg: f64,
    #[serde(default = "default_screw_radial")]
    pub radial_segments: u32,
}

fn default_screw_diameter() -> f64 {
    2.3
}
fn default_screw_length() -> f64 {
    12.0
}
fn default_screw_radial() -> u32 {
    24
}

#[tauri::command]
pub fn cad_abutment_screw_channel(
    request: ScrewChannelRequest,
) -> Result<ProductionMeshResponse, AbutmentProductionError> {
    let params = ScrewChannelParams {
        diameter_mm: request.diameter_mm,
        length_mm: request.length_mm,
        angle_deg: request.angle_deg,
        origin: request.origin,
        axis: request.axis,
        radial_segments: request.radial_segments,
    };
    let (mesh, report) = build_screw_channel(&params);
    write_stl(&mesh, &request.output)?;
    Ok(ProductionMeshResponse {
        output: request.output,
        triangles: report.triangles,
        vertices: report.vertices,
        volume_mm3: report.volume_mm3,
        backend: "tlanticad-abutment::production::screw-channel",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NestingPuckRequest {
    pub output: PathBuf,
    #[serde(default = "default_puck_diameter")]
    pub diameter_mm: f64,
    #[serde(default = "default_puck_thickness")]
    pub thickness_mm: f64,
    #[serde(default = "default_slot_count")]
    pub slot_count: u32,
    #[serde(default = "default_slot_radius")]
    pub slot_radius_mm: f64,
    #[serde(default = "default_slot_diameter")]
    pub slot_diameter_mm: f64,
}

fn default_puck_diameter() -> f64 {
    98.5
}
fn default_puck_thickness() -> f64 {
    16.0
}
fn default_slot_count() -> u32 {
    6
}
fn default_slot_radius() -> f64 {
    35.0
}
fn default_slot_diameter() -> f64 {
    14.0
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NestingPuckResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
    pub slot_positions: Vec<[f64; 3]>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_abutment_nesting_puck(
    request: NestingPuckRequest,
) -> Result<NestingPuckResponse, AbutmentProductionError> {
    let params = NestingPuckParams {
        diameter_mm: request.diameter_mm,
        thickness_mm: request.thickness_mm,
        slot_count: request.slot_count,
        slot_radius_mm: request.slot_radius_mm,
        slot_diameter_mm: request.slot_diameter_mm,
    };
    let (mesh, report) = build_nesting_puck(&params);
    let slots = nesting_slot_positions(&params);
    write_stl(&mesh, &request.output)?;
    Ok(NestingPuckResponse {
        output: request.output,
        triangles: report.triangles,
        vertices: report.vertices,
        volume_mm3: report.volume_mm3,
        slot_positions: slots.into_iter().map(|p| [p.x, p.y, p.z]).collect(),
        backend: "tlanticad-abutment::production::nesting",
    })
}
