// AR-V375 — Freeform specialty (Tauri command surface).
//
// Three commands:
//   * `cad_freeform_bar_create`         — multi-anchor bar (Round / Oval / DolderEgg / Hader).
//   * `cad_freeform_telescope_create`   — primary + secondary telescope pair.
//   * `cad_freeform_post_and_core_create` — post (canal fit) + core (above entrance).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_freeform::specialty::{
    build_multi_anchor_bar, build_post_and_core, build_telescope_pair, BarParams, BarProfile,
    PostAndCoreParams, SpecialtyReport, TelescopeParams,
};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SpecialtyError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), SpecialtyError> {
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
        .map_err(|e| SpecialtyError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| SpecialtyError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), SpecialtyError> {
    Err(SpecialtyError::FormatsFeatureMissing)
}

fn parse_bar_profile(s: &str) -> BarProfile {
    match s.to_lowercase().as_str() {
        "oval" => BarProfile::Oval,
        "dolder-egg" | "dolder" | "dolderegg" => BarProfile::DolderEgg,
        "hader" => BarProfile::Hader,
        _ => BarProfile::Round,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarRequest {
    pub output: PathBuf,
    pub anchors: Vec<[f64; 3]>,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_width")]
    pub width_mm: f64,
    #[serde(default = "default_height")]
    pub height_mm: f64,
    #[serde(default = "default_up")]
    pub occlusal_up: [f64; 3],
    #[serde(default = "default_radial")]
    pub radial_segments: u32,
}

fn default_profile() -> String {
    "round".into()
}
fn default_width() -> f64 {
    2.5
}
fn default_height() -> f64 {
    2.5
}
fn default_up() -> [f64; 3] {
    [0.0, 0.0, 1.0]
}
fn default_radial() -> u32 {
    16
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialtyMeshResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub volume_mm3: f64,
    pub watertight_hint: bool,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_freeform_bar_create(
    request: BarRequest,
) -> Result<SpecialtyMeshResponse, SpecialtyError> {
    if request.anchors.len() < 2 {
        return Err(SpecialtyError::Invalid {
            message: "bar requires at least 2 anchors".into(),
        });
    }
    let params = BarParams {
        anchors: request.anchors,
        profile: parse_bar_profile(&request.profile),
        width_mm: request.width_mm,
        height_mm: request.height_mm,
        occlusal_up: request.occlusal_up,
        radial_segments: request.radial_segments,
    };
    let (mesh, report): (Mesh, SpecialtyReport) = build_multi_anchor_bar(&params);
    write_stl(&mesh, &request.output)?;
    Ok(SpecialtyMeshResponse {
        output: request.output,
        triangles: report.triangles,
        vertices: report.vertices,
        volume_mm3: report.volume_mm3,
        watertight_hint: report.watertight_hint,
        backend: "tlanticad-freeform::specialty::bar",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelescopeRequest {
    pub output_primary: PathBuf,
    pub output_secondary: PathBuf,
    pub base: [f64; 3],
    pub occlusal_axis: [f64; 3],
    #[serde(default = "default_height_5_5")]
    pub primary_height_mm: f64,
    #[serde(default = "default_radius_3")]
    pub primary_radius_mm: f64,
    #[serde(default = "default_taper_4")]
    pub primary_taper_deg: f64,
    #[serde(default = "default_gap_25")]
    pub gap_mm: f64,
    #[serde(default = "default_secondary_05")]
    pub secondary_thickness_mm: f64,
    #[serde(default = "default_radial_32")]
    pub radial_segments: u32,
}

fn default_height_5_5() -> f64 {
    5.5
}
fn default_radius_3() -> f64 {
    3.0
}
fn default_taper_4() -> f64 {
    4.0
}
fn default_gap_25() -> f64 {
    0.025
}
fn default_secondary_05() -> f64 {
    0.5
}
fn default_radial_32() -> u32 {
    32
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelescopeResponse {
    pub primary: SpecialtyMeshResponse,
    pub secondary: SpecialtyMeshResponse,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_freeform_telescope_create(
    request: TelescopeRequest,
) -> Result<TelescopeResponse, SpecialtyError> {
    let base = Point3::new(request.base[0], request.base[1], request.base[2]);
    let axis = Vector3::new(
        request.occlusal_axis[0],
        request.occlusal_axis[1],
        request.occlusal_axis[2],
    );
    let params = TelescopeParams {
        primary_height_mm: request.primary_height_mm,
        primary_radius_mm: request.primary_radius_mm,
        primary_taper_deg: request.primary_taper_deg,
        gap_mm: request.gap_mm,
        secondary_thickness_mm: request.secondary_thickness_mm,
        radial_segments: request.radial_segments,
    };
    let (primary_mesh, secondary_mesh, p_report, s_report) =
        build_telescope_pair(base, axis, &params);
    write_stl(&primary_mesh, &request.output_primary)?;
    write_stl(&secondary_mesh, &request.output_secondary)?;
    Ok(TelescopeResponse {
        primary: SpecialtyMeshResponse {
            output: request.output_primary,
            triangles: p_report.triangles,
            vertices: p_report.vertices,
            volume_mm3: p_report.volume_mm3,
            watertight_hint: p_report.watertight_hint,
            backend: "tlanticad-freeform::specialty::telescope-primary",
        },
        secondary: SpecialtyMeshResponse {
            output: request.output_secondary,
            triangles: s_report.triangles,
            vertices: s_report.vertices,
            volume_mm3: s_report.volume_mm3,
            watertight_hint: s_report.watertight_hint,
            backend: "tlanticad-freeform::specialty::telescope-secondary",
        },
        backend: "tlanticad-freeform::specialty",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAndCoreRequest {
    pub output: PathBuf,
    pub canal_entrance: [f64; 3],
    pub canal_axis: [f64; 3],
    #[serde(default = "default_post_length")]
    pub post_length_mm: f64,
    #[serde(default = "default_core_height")]
    pub core_height_mm: f64,
    #[serde(default = "default_post_diameter")]
    pub post_diameter_mm: f64,
    #[serde(default = "default_core_diameter")]
    pub core_diameter_mm: f64,
    #[serde(default = "default_post_taper")]
    pub post_taper_deg: f64,
    #[serde(default = "default_core_taper")]
    pub core_taper_deg: f64,
    #[serde(default = "default_pc_radial")]
    pub radial_segments: u32,
}

fn default_post_length() -> f64 {
    8.0
}
fn default_core_height() -> f64 {
    4.0
}
fn default_post_diameter() -> f64 {
    1.4
}
fn default_core_diameter() -> f64 {
    4.0
}
fn default_post_taper() -> f64 {
    2.0
}
fn default_core_taper() -> f64 {
    6.0
}
fn default_pc_radial() -> u32 {
    24
}

#[tauri::command]
pub fn cad_freeform_post_and_core_create(
    request: PostAndCoreRequest,
) -> Result<SpecialtyMeshResponse, SpecialtyError> {
    let entrance = Point3::new(
        request.canal_entrance[0],
        request.canal_entrance[1],
        request.canal_entrance[2],
    );
    let axis = Vector3::new(
        request.canal_axis[0],
        request.canal_axis[1],
        request.canal_axis[2],
    );
    let params = PostAndCoreParams {
        post_length_mm: request.post_length_mm,
        core_height_mm: request.core_height_mm,
        post_diameter_mm: request.post_diameter_mm,
        core_diameter_mm: request.core_diameter_mm,
        post_taper_deg: request.post_taper_deg,
        core_taper_deg: request.core_taper_deg,
        radial_segments: request.radial_segments,
    };
    let (mesh, report) = build_post_and_core(entrance, axis, &params);
    write_stl(&mesh, &request.output)?;
    Ok(SpecialtyMeshResponse {
        output: request.output,
        triangles: report.triangles,
        vertices: report.vertices,
        volume_mm3: report.volume_mm3,
        watertight_hint: report.watertight_hint,
        backend: "tlanticad-freeform::specialty::post-and-core",
    })
}
