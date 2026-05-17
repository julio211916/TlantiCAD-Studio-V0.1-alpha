use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tlanticad_alignment::{
    align_landmarks, align_meshes, apply_transform_to_mesh, AlignmentMode, AlignmentParams,
    AlignmentResult,
};
use tlanticad_mesh::nalgebra::{Point3, Vector3};
use tlanticad_mesh::{load_obj, save_stl, Mesh};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentRegisterLandmarksRequest {
    pub moving_points: Vec<[f64; 3]>,
    pub fixed_points: Vec<[f64; 3]>,
    pub case_folder_path: Option<String>,
    pub output_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentRegisterMeshesRequest {
    pub moving_mesh_path: String,
    pub fixed_mesh_path: String,
    pub case_folder_path: Option<String>,
    pub output_file_name: Option<String>,
    pub write_aligned_mesh: Option<bool>,
    pub mode: Option<AlignmentMode>,
    pub max_iterations: Option<usize>,
    pub tolerance_mm: Option<f64>,
    pub sample_limit: Option<usize>,
    pub max_correspondence_distance_mm: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentRegisterResponse {
    pub result: AlignmentResult,
    pub transform_path: Option<String>,
    pub aligned_mesh_path: Option<String>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn alignment_register_landmarks(
    request: AlignmentRegisterLandmarksRequest,
) -> Result<AlignmentRegisterResponse, String> {
    let result = align_landmarks(&request.moving_points, &request.fixed_points)?;
    let transform_path = write_transform_artifact(
        request.case_folder_path.as_deref(),
        request
            .output_file_name
            .as_deref()
            .unwrap_or("landmark-alignment.json"),
        &result,
    )?;

    Ok(AlignmentRegisterResponse {
        result,
        transform_path,
        aligned_mesh_path: None,
        backend: "rust:tlanticad-alignment",
    })
}

#[tauri::command]
pub fn alignment_register_meshes(
    request: AlignmentRegisterMeshesRequest,
) -> Result<AlignmentRegisterResponse, String> {
    let moving_path = PathBuf::from(&request.moving_mesh_path);
    let fixed_path = PathBuf::from(&request.fixed_mesh_path);
    let moving_mesh = load_mesh(&moving_path)?;
    let fixed_mesh = load_mesh(&fixed_path)?;
    let params = AlignmentParams {
        mode: request.mode.unwrap_or(AlignmentMode::IterativeClosestPoint),
        max_iterations: request.max_iterations.unwrap_or(32),
        tolerance_mm: request.tolerance_mm.unwrap_or(1.0e-4),
        sample_limit: request.sample_limit.unwrap_or(2_000),
        max_correspondence_distance_mm: request.max_correspondence_distance_mm,
    };
    let result = align_meshes(&moving_mesh, &fixed_mesh, &params)?;

    let transform_path = write_transform_artifact(
        request.case_folder_path.as_deref(),
        request
            .output_file_name
            .as_deref()
            .unwrap_or("mesh-alignment.json"),
        &result,
    )?;
    let aligned_mesh_path = if request.write_aligned_mesh.unwrap_or(false) {
        Some(write_aligned_mesh(
            request.case_folder_path.as_deref(),
            &moving_mesh,
            &result,
            request.output_file_name.as_deref(),
        )?)
    } else {
        None
    };

    Ok(AlignmentRegisterResponse {
        result,
        transform_path,
        aligned_mesh_path,
        backend: "rust:tlanticad-alignment",
    })
}

fn load_mesh(path: &Path) -> Result<Mesh, String> {
    if !path.exists() {
        return Err(format!("mesh path does not exist: {}", path.display()));
    }
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "obj" => load_obj(path),
        "stl" => load_stl_indexed(path),
        extension => Err(format!("unsupported alignment mesh extension: {extension}")),
    }
}

#[cfg(feature = "backend-formats")]
fn load_stl_indexed(path: &Path) -> Result<Mesh, String> {
    let file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut reader = BufReader::new(file);
    let indexed = stl_io::read_stl(&mut reader)
        .map_err(|error| format!("parse STL {}: {error}", path.display()))?;
    let mut mesh = Mesh::new(path.file_stem().unwrap_or_default().to_string_lossy());
    mesh.vertices = indexed
        .vertices
        .iter()
        .map(|vertex| Point3::new(vertex[0] as f64, vertex[1] as f64, vertex[2] as f64))
        .collect();
    mesh.indices = indexed
        .faces
        .iter()
        .map(|face| {
            [
                face.vertices[0] as u32,
                face.vertices[1] as u32,
                face.vertices[2] as u32,
            ]
        })
        .collect();
    mesh.normals = indexed
        .faces
        .iter()
        .flat_map(|face| {
            let normal = Vector3::new(
                face.normal[0] as f64,
                face.normal[1] as f64,
                face.normal[2] as f64,
            );
            [normal, normal, normal]
        })
        .take(mesh.vertices.len())
        .collect();
    if mesh.normals.len() != mesh.vertices.len() {
        mesh.calculate_normals();
    }
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn load_stl_indexed(_path: &Path) -> Result<Mesh, String> {
    Err("STL alignment import requires the backend-formats feature".to_string())
}

fn write_transform_artifact(
    case_folder_path: Option<&str>,
    output_file_name: &str,
    result: &AlignmentResult,
) -> Result<Option<String>, String> {
    let Some(case_folder_path) = case_folder_path else {
        return Ok(None);
    };
    let output_dir = PathBuf::from(case_folder_path)
        .join("work")
        .join("alignment");
    std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let output_path = output_dir.join(safe_file_name(output_file_name, "alignment.json"));
    let payload = serde_json::to_vec_pretty(result).map_err(|error| error.to_string())?;
    std::fs::write(&output_path, payload).map_err(|error| error.to_string())?;
    Ok(Some(output_path.to_string_lossy().to_string()))
}

fn write_aligned_mesh(
    case_folder_path: Option<&str>,
    moving_mesh: &Mesh,
    result: &AlignmentResult,
    output_file_name: Option<&str>,
) -> Result<String, String> {
    let case_folder_path = case_folder_path
        .ok_or_else(|| "caseFolderPath is required to write aligned mesh".to_string())?;
    let output_dir = PathBuf::from(case_folder_path)
        .join("work")
        .join("alignment");
    std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let stem = output_file_name.unwrap_or("aligned-moving");
    let output_path = output_dir.join(safe_stl_file_name(stem, "aligned-moving.stl"));
    let aligned = apply_transform_to_mesh(moving_mesh, &result.matrix);
    save_stl(&aligned, &output_path).map_err(|error| error.to_string())?;
    Ok(output_path.to_string_lossy().to_string())
}

fn safe_file_name(input: &str, fallback: &str) -> String {
    let mut safe = input
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '-',
        })
        .collect::<String>();
    if safe.is_empty() || safe == "." || safe == ".." {
        safe = fallback.to_string();
    }
    if !safe.ends_with(".json") && !safe.ends_with(".stl") {
        safe.push_str(".json");
    }
    safe
}

fn safe_stl_file_name(input: &str, fallback: &str) -> String {
    let mut safe = safe_file_name(input, fallback);
    if safe.ends_with(".json") {
        safe.truncate(safe.len() - ".json".len());
        safe.push_str(".stl");
    }
    if !safe.ends_with(".stl") {
        safe.push_str(".stl");
    }
    safe
}
