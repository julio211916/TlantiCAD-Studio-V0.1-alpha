use tlanticad_core::{Result, TlantiError};
use tlanticad_mesh::Mesh;
use nalgebra::Point3;
use std::path::Path;

/// Import OBJ file
pub async fn import(path: impl AsRef<Path>) -> Result<Mesh> {
    let path = path.as_ref();
    let content = tokio::fs::read_to_string(path).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;

    let mut mesh = Mesh::new(path.file_stem()
        .and_then(|s| s.to_str()).unwrap_or("obj_import"));

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("v ") {
            let parts: Vec<f64> = line[2..].split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() >= 3 {
                mesh.vertices.push(Point3::new(parts[0], parts[1], parts[2]));
            }
        } else if line.starts_with("f ") {
            let indices: Vec<u32> = line[2..].split_whitespace()
                .filter_map(|s| {
                    // Handle "v", "v/vt", "v/vt/vn" formats
                    s.split('/').next()?.parse::<u32>().ok().map(|i| i - 1)
                })
                .collect();
            // Triangulate faces with more than 3 vertices
            if indices.len() >= 3 {
                for i in 1..indices.len() - 1 {
                    mesh.indices.push([indices[0], indices[i] as u32, indices[i + 1] as u32]);
                }
            }
        }
    }

    mesh.calculate_normals();
    Ok(mesh)
}

/// Export mesh as OBJ
pub async fn export(mesh: &Mesh, path: impl AsRef<Path>) -> Result<()> {
    let mut content = String::with_capacity(mesh.vertices.len() * 40 + mesh.indices.len() * 20);

    content.push_str("# TlantiCAD OBJ export\n");
    for v in &mesh.vertices {
        content.push_str(&format!("v {} {} {}\n", v.x, v.y, v.z));
    }
    for n in &mesh.normals {
        content.push_str(&format!("vn {} {} {}\n", n.x, n.y, n.z));
    }
    for tri in &mesh.indices {
        content.push_str(&format!("f {}//{} {}//{} {}//{}\n",
            tri[0] + 1, tri[0] + 1,
            tri[1] + 1, tri[1] + 1,
            tri[2] + 1, tri[2] + 1));
    }

    tokio::fs::write(path, content).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;
    Ok(())
}
