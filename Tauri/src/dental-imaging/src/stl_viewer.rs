//! STL 3D Mesh Viewer
//!
//! Parses STL files (dental molds, crowns, aligners) and returns
//! mesh data for WebGPU/Three.js rendering in the frontend.

use crate::error::ImagingError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A 3D vertex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A triangle face
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triangle {
    pub normal: Vertex,
    pub vertices: [Vertex; 3],
}

/// STL mesh data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StlMesh {
    pub name: String,
    pub triangle_count: usize,
    /// Flat vertex buffer [x,y,z, x,y,z, ...] for WebGPU
    pub vertices: Vec<f32>,
    /// Flat normal buffer [nx,ny,nz, ...] per vertex
    pub normals: Vec<f32>,
    /// Bounding box min/max
    pub bbox_min: [f32; 3],
    pub bbox_max: [f32; 3],
    pub file_path: String,
    pub file_size_bytes: u64,
}

/// STL file metadata (lightweight, no mesh data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StlInfo {
    pub name: String,
    pub triangle_count: usize,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub bbox_min: [f32; 3],
    pub bbox_max: [f32; 3],
}

/// Parse an STL file and return full mesh data for rendering
pub fn parse_stl(path: &Path) -> Result<StlMesh, ImagingError> {
    let file = std::fs::File::open(path).map_err(|e| {
        ImagingError::FileNotFound(format!("{}: {}", path.display(), e))
    })?;

    let mut reader = std::io::BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader)
        .map_err(|e| ImagingError::StlParse(e.to_string()))?;

    let triangle_count = stl.faces.len();
    let mut vertices = Vec::with_capacity(triangle_count * 9);
    let mut normals = Vec::with_capacity(triangle_count * 9);
    let mut bbox_min = [f32::MAX; 3];
    let mut bbox_max = [f32::MIN; 3];

    for face in &stl.faces {
        let n = &face.normal;
        for v_idx in &face.vertices {
            let v = &stl.vertices[*v_idx];
            vertices.push(v[0]);
            vertices.push(v[1]);
            vertices.push(v[2]);
            normals.push(n[0]);
            normals.push(n[1]);
            normals.push(n[2]);

            for i in 0..3 {
                bbox_min[i] = bbox_min[i].min(v[i]);
                bbox_max[i] = bbox_max[i].max(v[i]);
            }
        }
    }

    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(StlMesh {
        name,
        triangle_count,
        vertices,
        normals,
        bbox_min,
        bbox_max,
        file_path: path.to_string_lossy().to_string(),
        file_size_bytes: file_size,
    })
}

/// Parse STL metadata only (fast, no vertex data)
pub fn parse_stl_info(path: &Path) -> Result<StlInfo, ImagingError> {
    let mesh = parse_stl(path)?;
    Ok(StlInfo {
        name: mesh.name,
        triangle_count: mesh.triangle_count,
        file_path: mesh.file_path,
        file_size_bytes: mesh.file_size_bytes,
        bbox_min: mesh.bbox_min,
        bbox_max: mesh.bbox_max,
    })
}

/// List all STL files in a directory
pub fn list_stl_files(dir: &Path) -> Result<Vec<StlInfo>, ImagingError> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("stl") {
                    if let Ok(info) = parse_stl_info(path) {
                        files.push(info);
                    }
                }
            }
        }
    }

    Ok(files)
}
