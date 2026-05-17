// AR-V379 + AR-V380 — DICOM segmentation toolbox + fixation guide (Tauri command surface).
//
// Five commands:
//   * `cad_dicom_threshold_voxels`   — apply HU threshold to a flat volume buffer.
//   * `cad_dicom_region_grow_3d`     — 6-connected BFS region grow.
//   * `cad_dicom_marching_cubes`     — extract iso-surface mesh from a binary mask.
//   * `cad_guide_extract_gingiva`    — extract gingiva-contact face subset.
//   * `cad_guide_build_sleeve`       — build a single guide sleeve (cylindrical).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_dicom::segmentation::{
    marching_cubes_lite, region_grow_3d, threshold_voxels, RegionGrowReport, ThresholdParams,
    VolumeShape,
};
use tlanticad_implant::fixation_guide::{
    build_guide_sleeve, extract_gingiva_contact, GingivaContactParams, GingivaContactReport,
    GuideSleeveParams,
};
use tlanticad_mesh::nalgebra::Point3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DicomSegError {
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
fn read_stl(path: &PathBuf) -> Result<Mesh, DicomSegError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| DicomSegError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| DicomSegError::Io {
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
    let mut mesh = Mesh::new("dicom-seg-input");
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, DicomSegError> {
    Err(DicomSegError::FormatsFeatureMissing)
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), DicomSegError> {
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
        .map_err(|e| DicomSegError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| DicomSegError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), DicomSegError> {
    Err(DicomSegError::FormatsFeatureMissing)
}

// ---------- threshold ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdRequest {
    pub volume: Vec<i16>,
    pub low: i16,
    pub high: i16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdResponse {
    pub mask: Vec<u8>,
    pub voxel_count: usize,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_dicom_threshold_voxels(request: ThresholdRequest) -> ThresholdResponse {
    let params = ThresholdParams {
        low: request.low,
        high: request.high,
    };
    let mask = threshold_voxels(&request.volume, &params);
    let count = mask.iter().filter(|&&v| v == 1).count();
    ThresholdResponse {
        mask,
        voxel_count: count,
        backend: "tlanticad-dicom::segmentation::threshold",
    }
}

// ---------- region grow ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionGrowRequest {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    pub mask: Vec<u8>,
    pub seed: [u32; 3],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionGrowResponse {
    pub visited: Vec<u8>,
    pub report: RegionGrowReport,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_dicom_region_grow_3d(
    request: RegionGrowRequest,
) -> Result<RegionGrowResponse, DicomSegError> {
    let shape = VolumeShape {
        size_x: request.size_x,
        size_y: request.size_y,
        size_z: request.size_z,
    };
    if request.mask.len() != shape.voxel_count() {
        return Err(DicomSegError::Invalid {
            message: format!(
                "mask length {} != expected {} (size_x*size_y*size_z)",
                request.mask.len(),
                shape.voxel_count()
            ),
        });
    }
    let (visited, report) = region_grow_3d(
        &shape,
        &request.mask,
        (request.seed[0], request.seed[1], request.seed[2]),
    );
    Ok(RegionGrowResponse {
        visited,
        report,
        backend: "tlanticad-dicom::segmentation::region_grow",
    })
}

// ---------- marching cubes ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarchingCubesRequest {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    pub mask: Vec<u8>,
    pub voxel_size_mm: f64,
    pub output: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarchingCubesResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_dicom_marching_cubes(
    request: MarchingCubesRequest,
) -> Result<MarchingCubesResponse, DicomSegError> {
    let shape = VolumeShape {
        size_x: request.size_x,
        size_y: request.size_y,
        size_z: request.size_z,
    };
    if request.mask.len() != shape.voxel_count() {
        return Err(DicomSegError::Invalid {
            message: "mask length mismatch".into(),
        });
    }
    let (verts, indices) = marching_cubes_lite(&shape, &request.mask, request.voxel_size_mm);
    if verts.is_empty() {
        return Err(DicomSegError::Invalid {
            message: "marching cubes produced empty mesh (mask is all zero)".into(),
        });
    }
    let mut mesh = Mesh::new("dicom-iso-surface");
    mesh.vertices = verts
        .into_iter()
        .map(|v| Point3::new(v[0], v[1], v[2]))
        .collect();
    mesh.indices = indices;
    mesh.calculate_normals();
    let triangles = mesh.triangle_count();
    let vert_count = mesh.vertex_count();
    write_stl(&mesh, &request.output)?;
    Ok(MarchingCubesResponse {
        output: request.output,
        triangles,
        vertices: vert_count,
        backend: "tlanticad-dicom::segmentation::marching_cubes",
    })
}

// ---------- guide: extract gingiva contact ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideExtractRequest {
    pub base_stl: PathBuf,
    pub output: PathBuf,
    pub into_tissue_axis: [f64; 3],
    #[serde(default = "default_dot")]
    pub normal_dot_threshold: f64,
    #[serde(default = "default_min_component")]
    pub min_component_faces: usize,
}

fn default_dot() -> f64 {
    0.6
}
fn default_min_component() -> usize {
    50
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideExtractResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub report: GingivaContactReport,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_guide_extract_gingiva(
    request: GuideExtractRequest,
) -> Result<GuideExtractResponse, DicomSegError> {
    if !request.base_stl.exists() {
        return Err(DicomSegError::InputNotFound {
            path: request.base_stl.to_string_lossy().into_owned(),
        });
    }
    let base = read_stl(&request.base_stl)?;
    let params = GingivaContactParams {
        into_tissue_axis: request.into_tissue_axis,
        normal_dot_threshold: request.normal_dot_threshold,
        min_component_faces: request.min_component_faces,
    };
    let (contact, report) = extract_gingiva_contact(&base, &params);
    let triangles = contact.triangle_count();
    let vertices = contact.vertex_count();
    write_stl(&contact, &request.output)?;
    Ok(GuideExtractResponse {
        output: request.output,
        triangles,
        vertices,
        report,
        backend: "tlanticad-implant::fixation_guide",
    })
}

// ---------- guide: build sleeve ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideSleeveRequest {
    pub output: PathBuf,
    pub center: [f64; 3],
    pub axis: [f64; 3],
    pub diameter_mm: f64,
    pub length_mm: f64,
    #[serde(default = "default_radial_24")]
    pub radial_segments: u32,
}

fn default_radial_24() -> u32 {
    24
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideSleeveResponse {
    pub output: PathBuf,
    pub triangles: usize,
    pub vertices: usize,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_guide_build_sleeve(
    request: GuideSleeveRequest,
) -> Result<GuideSleeveResponse, DicomSegError> {
    let params = GuideSleeveParams {
        center: request.center,
        axis: request.axis,
        diameter_mm: request.diameter_mm,
        length_mm: request.length_mm,
        radial_segments: request.radial_segments,
    };
    let mesh = build_guide_sleeve(&params);
    let triangles = mesh.triangle_count();
    let vertices = mesh.vertex_count();
    write_stl(&mesh, &request.output)?;
    Ok(GuideSleeveResponse {
        output: request.output,
        triangles,
        vertices,
        backend: "tlanticad-implant::fixation_guide::sleeve",
    })
}
