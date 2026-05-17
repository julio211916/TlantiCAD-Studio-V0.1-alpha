use tlanticad_core::{Result, TlantiError};
use tlanticad_mesh::Mesh;
use nalgebra::Point3;
use std::path::Path;

/// Import ASCII PLY file
pub async fn import(path: impl AsRef<Path>) -> Result<Mesh> {
    let path = path.as_ref();
    let content = tokio::fs::read_to_string(path).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;

    let mut mesh = Mesh::new(path.file_stem()
        .and_then(|s| s.to_str()).unwrap_or("ply_import"));

    let mut lines = content.lines();
    let mut vertex_count = 0usize;
    let mut face_count = 0usize;
    // Parse header
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line == "end_header" {
            break;
        }
        if line.starts_with("element vertex") {
            vertex_count = line.split_whitespace().last()
                .and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if line.starts_with("element face") {
            face_count = line.split_whitespace().last()
                .and_then(|s| s.parse().ok()).unwrap_or(0);
        }
    }

    // Parse vertices
    for _ in 0..vertex_count {
        if let Some(line) = lines.next() {
            let vals: Vec<f64> = line.split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if vals.len() >= 3 {
                mesh.vertices.push(Point3::new(vals[0], vals[1], vals[2]));
            }
        }
    }

    // Parse faces
    for _ in 0..face_count {
        if let Some(line) = lines.next() {
            let vals: Vec<u32> = line.split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if vals.len() >= 4 && vals[0] == 3 {
                mesh.indices.push([vals[1], vals[2], vals[3]]);
            } else if vals.len() >= 5 && vals[0] == 4 {
                mesh.indices.push([vals[1], vals[2], vals[3]]);
                mesh.indices.push([vals[1], vals[3], vals[4]]);
            }
        }
    }

    mesh.calculate_normals();
    Ok(mesh)
}

/// Export mesh as ASCII PLY
pub async fn export(mesh: &Mesh, path: impl AsRef<Path>) -> Result<()> {
    let mut content = String::new();
    content.push_str("ply\n");
    content.push_str("format ascii 1.0\n");
    content.push_str(&format!("element vertex {}\n", mesh.vertices.len()));
    content.push_str("property float x\nproperty float y\nproperty float z\n");
    content.push_str(&format!("element face {}\n", mesh.indices.len()));
    content.push_str("property list uchar uint vertex_indices\n");
    content.push_str("end_header\n");

    for v in &mesh.vertices {
        content.push_str(&format!("{} {} {}\n", v.x, v.y, v.z));
    }
    for tri in &mesh.indices {
        content.push_str(&format!("3 {} {} {}\n", tri[0], tri[1], tri[2]));
    }

    tokio::fs::write(path, content).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;
    Ok(())
}
