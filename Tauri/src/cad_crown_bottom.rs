// AR-V367 — Crown bottom (Tauri command surface).
//
// One command:
//   * `cad_crown_bottom_generate` — given prep STL + margin polyline + gap params, write the
//     intaglio (inside) shell STL.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_crown::bottom::{
    generate_bottom_offset, polyline_from_array, BottomParams, BottomReport,
};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CrownBottomError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("margin polyline must have at least 3 points")]
    InvalidPolyline,
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, CrownBottomError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| CrownBottomError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| CrownBottomError::Io {
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
    let mut mesh = Mesh::new("prep");
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, CrownBottomError> {
    Err(CrownBottomError::FormatsFeatureMissing)
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), CrownBottomError> {
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
        .map_err(|e| CrownBottomError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| CrownBottomError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), CrownBottomError> {
    Err(CrownBottomError::FormatsFeatureMissing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrownBottomRequest {
    pub prep_stl: PathBuf,
    pub output: PathBuf,
    /// Margin polyline as `[x, y, z]` array. Length ≥ 3.
    pub margin_polyline: Vec<[f64; 3]>,
    #[serde(default = "default_true")]
    pub margin_closed: bool,
    /// Insertion axis (will be normalized). The "inward" direction is `-axis`.
    pub insertion_axis: [f64; 3],
    /// Gap parameters. If omitted, uses material-aware defaults below.
    #[serde(default)]
    pub gap_cement_mm: Option<f64>,
    #[serde(default)]
    pub gap_border_mm: Option<f64>,
    #[serde(default)]
    pub border_width_mm: Option<f64>,
    #[serde(default)]
    pub ramp_mm: Option<f64>,
    #[serde(default)]
    pub max_offset_mm: Option<f64>,
    /// Material name — drives default gap profile if explicit gaps are omitted.
    /// "zirconia" → 50 µm, "emax" → 60 µm, "metal" → 30 µm; default 50 µm.
    #[serde(default)]
    pub material: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrownBottomResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices_offset: usize,
    pub max_displacement_mm: f64,
    pub mean_displacement_mm: f64,
    pub gap_cement_mm: f64,
    pub gap_border_mm: f64,
    pub backend: &'static str,
}

fn material_default_gaps(material: Option<&str>) -> (f64, f64) {
    // (cement, border)
    match material.map(|s| s.to_lowercase()).as_deref() {
        Some("zirconia") => (0.050, 0.010),
        Some("emax") | Some("e.max") | Some("lithium-disilicate") => (0.060, 0.012),
        Some("metal") | Some("cobalt-chrome") | Some("nickel-chrome") => (0.030, 0.008),
        Some("titanium") => (0.040, 0.010),
        _ => (0.050, 0.010),
    }
}

#[tauri::command]
pub fn cad_crown_bottom_generate(
    request: CrownBottomRequest,
) -> Result<CrownBottomResponse, CrownBottomError> {
    if !request.prep_stl.exists() {
        return Err(CrownBottomError::InputNotFound {
            path: request.prep_stl.to_string_lossy().into_owned(),
        });
    }
    if request.margin_polyline.len() < 3 {
        return Err(CrownBottomError::InvalidPolyline);
    }
    let prep = read_stl(&request.prep_stl)?;
    let polyline = polyline_from_array(&request.margin_polyline);

    let (mat_cement, mat_border) = material_default_gaps(request.material.as_deref());
    let params = BottomParams {
        gap_cement_mm: request.gap_cement_mm.unwrap_or(mat_cement),
        gap_border_mm: request.gap_border_mm.unwrap_or(mat_border),
        border_width_mm: request.border_width_mm.unwrap_or(0.6),
        ramp_mm: request.ramp_mm.unwrap_or(0.3),
        max_offset_mm: request.max_offset_mm.unwrap_or(0.150),
    };
    let inward = -Vector3::new(
        request.insertion_axis[0],
        request.insertion_axis[1],
        request.insertion_axis[2],
    );

    let (bottom, report): (Mesh, BottomReport) =
        generate_bottom_offset(&prep, &polyline, request.margin_closed, inward, &params);

    write_stl(&bottom, &request.output)?;

    Ok(CrownBottomResponse {
        output: request.output,
        triangles: bottom.triangle_count(),
        vertices_offset: report.vertices_offset,
        max_displacement_mm: report.max_displacement_mm,
        mean_displacement_mm: report.mean_displacement_mm,
        gap_cement_mm: params.gap_cement_mm,
        gap_border_mm: params.gap_border_mm,
        backend: "tlanticad-crown::bottom",
    })
}
