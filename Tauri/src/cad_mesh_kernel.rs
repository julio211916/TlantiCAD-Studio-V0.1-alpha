// AR-V363 — Mesh kernel ops (Tauri command surface).
//
// Three commands:
//   * `mesh_kernel_add_remove`  — bulge/remove a region of an STL mesh.
//   * `mesh_kernel_compare`     — compute Hausdorff/RMS between two meshes.
//   * `mesh_kernel_adapt_to_gingiva` — drape a source STL onto a target STL.
//
// All commands operate on paths (consistent with `cad_csg::mesh_op`). Output
// meshes are written as binary STL via the `backend-formats` feature path.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_mesh::{
    adapt::{drape_onto, DrapeOptions, DrapeReport},
    add_remove::{bulge_region, remove_region, AddRemoveReport, BulgeOptions},
    compare::{compare as compare_meshes, CompareReport},
    region::{closest_face, grow_by_radius, FaceRegion},
    Mesh,
};

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MeshKernelError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("kernel error: {message}")]
    Kernel { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
}

// ---------- helpers ----------

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, MeshKernelError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| MeshKernelError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| MeshKernelError::Io {
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
fn read_stl(_path: &PathBuf) -> Result<Mesh, MeshKernelError> {
    Err(MeshKernelError::FormatsFeatureMissing)
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), MeshKernelError> {
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
        .map_err(|e| MeshKernelError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| MeshKernelError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), MeshKernelError> {
    Err(MeshKernelError::FormatsFeatureMissing)
}

fn ensure_exists(path: &PathBuf) -> Result<(), MeshKernelError> {
    if !path.exists() {
        return Err(MeshKernelError::InputNotFound {
            path: path.to_string_lossy().into_owned(),
        });
    }
    Ok(())
}

// ---------- add/remove ----------

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddRemoveOp {
    Add,
    Remove,
    DropFaces,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRemoveRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub op: AddRemoveOp,
    /// Center of the brush in mesh-local coordinates (mm).
    pub center: [f64; 3],
    /// Brush radius in mm.
    pub radius_mm: f64,
    /// For Add/Remove: signed amount in mm. For DropFaces this is ignored.
    #[serde(default)]
    pub amount_mm: f64,
    #[serde(default = "default_falloff")]
    pub falloff: f64,
}

fn default_falloff() -> f64 {
    2.0
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRemoveResponse {
    pub output: PathBuf,
    pub vertices_modified: usize,
    pub faces_removed: usize,
    pub max_displacement_mm: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn mesh_kernel_add_remove(
    request: AddRemoveRequest,
) -> Result<AddRemoveResponse, MeshKernelError> {
    ensure_exists(&request.input)?;
    let mut mesh = read_stl(&request.input)?;
    let seed_point = tlanticad_mesh::nalgebra::Point3::new(
        request.center[0],
        request.center[1],
        request.center[2],
    );
    let seed = closest_face(&mesh, &seed_point).ok_or_else(|| MeshKernelError::Kernel {
        message: "input mesh has no faces".into(),
    })?;
    let region: FaceRegion = grow_by_radius(&mesh, seed, request.radius_mm);

    let report: AddRemoveReport = match request.op {
        AddRemoveOp::Add => {
            let opts = BulgeOptions {
                amount_mm: request.amount_mm.abs(),
                falloff: request.falloff,
                use_falloff: request.falloff > 0.0,
            };
            bulge_region(&mut mesh, &region, &opts)
        }
        AddRemoveOp::Remove => {
            let opts = BulgeOptions {
                amount_mm: -request.amount_mm.abs(),
                falloff: request.falloff,
                use_falloff: request.falloff > 0.0,
            };
            bulge_region(&mut mesh, &region, &opts)
        }
        AddRemoveOp::DropFaces => remove_region(&mut mesh, &region),
    };

    write_stl(&mesh, &request.output)?;
    Ok(AddRemoveResponse {
        output: request.output,
        vertices_modified: report.vertices_modified,
        faces_removed: report.faces_removed,
        max_displacement_mm: report.max_displacement_mm,
        backend: "tlanticad-mesh::add_remove",
    })
}

// ---------- compare ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareRequest {
    pub a: PathBuf,
    pub b: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareResponse {
    #[serde(flatten)]
    pub report: CompareReport,
    pub backend: &'static str,
}

#[tauri::command]
pub fn mesh_kernel_compare(request: CompareRequest) -> Result<CompareResponse, MeshKernelError> {
    ensure_exists(&request.a)?;
    ensure_exists(&request.b)?;
    let a = read_stl(&request.a)?;
    let b = read_stl(&request.b)?;
    let report = compare_meshes(&a, &b);
    Ok(CompareResponse {
        report,
        backend: "tlanticad-mesh::compare",
    })
}

// ---------- adapt to gingiva ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptToGingivaRequest {
    pub source: PathBuf,
    pub target: PathBuf,
    pub output: PathBuf,
    /// Direction "down" toward gingiva (will be normalized).
    pub axis_occlusal: [f64; 3],
    #[serde(default)]
    pub min_distance_mm: f64,
    #[serde(default = "default_true")]
    pub snap_to_gingiva: bool,
    #[serde(default = "default_smoothing_iters")]
    pub even_out_iterations: u32,
    #[serde(default = "default_smoothing_lambda")]
    pub even_out_lambda: f64,
}

fn default_true() -> bool {
    true
}
fn default_smoothing_iters() -> u32 {
    3
}
fn default_smoothing_lambda() -> f64 {
    0.5
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptToGingivaResponse {
    pub output: PathBuf,
    pub vertices_moved: usize,
    pub max_displacement_mm: f64,
    pub mean_displacement_mm: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn mesh_kernel_adapt_to_gingiva(
    request: AdaptToGingivaRequest,
) -> Result<AdaptToGingivaResponse, MeshKernelError> {
    ensure_exists(&request.source)?;
    ensure_exists(&request.target)?;
    let mut source = read_stl(&request.source)?;
    let target = read_stl(&request.target)?;
    let opts = DrapeOptions {
        axis_occlusal: request.axis_occlusal,
        min_distance_mm: request.min_distance_mm,
        snap_to_gingiva: request.snap_to_gingiva,
        even_out_iterations: request.even_out_iterations,
        even_out_lambda: request.even_out_lambda,
    };
    let report: DrapeReport = drape_onto(&mut source, &target, &opts);
    write_stl(&source, &request.output)?;
    Ok(AdaptToGingivaResponse {
        output: request.output,
        vertices_moved: report.vertices_moved,
        max_displacement_mm: report.max_displacement_mm,
        mean_displacement_mm: report.mean_displacement_mm,
        backend: "tlanticad-mesh::adapt",
    })
}
