use tlanticad_core::{Result, TlantiError};
use tlanticad_mesh::Mesh;
use nalgebra::{Point3, Vector3};
use std::path::Path;

/// Import binary STL file
pub async fn import(path: impl AsRef<Path>) -> Result<Mesh> {
    let path = path.as_ref();
    let data = tokio::fs::read(path).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;

    if data.len() < 84 {
        return Err(TlantiError::IoError("STL file too small".into()));
    }

    let num_triangles = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
    let expected = 84 + num_triangles * 50;
    if data.len() < expected {
        return Err(TlantiError::IoError("STL file truncated".into()));
    }

    let mut mesh = Mesh::new(path.file_stem()
        .and_then(|s| s.to_str()).unwrap_or("stl_import"));

    let mut offset = 84;
    for _ in 0..num_triangles {
        let nx = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as f64;
        let ny = f32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]) as f64;
        let nz = f32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]) as f64;
        offset += 12;

        let base = mesh.vertices.len() as u32;
        for _ in 0..3 {
            let x = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as f64;
            let y = f32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]) as f64;
            let z = f32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]) as f64;
            mesh.vertices.push(Point3::new(x, y, z));
            mesh.normals.push(Vector3::new(nx, ny, nz));
            offset += 12;
        }
        offset += 2; // attribute byte count

        mesh.indices.push([base, base + 1, base + 2]);
    }

    Ok(mesh)
}

/// Export mesh as binary STL
pub async fn export(mesh: &Mesh, path: impl AsRef<Path>) -> Result<()> {
    let mut buf = Vec::with_capacity(84 + mesh.indices.len() * 50);

    // 80-byte header
    buf.extend_from_slice(&[0u8; 80]);
    // triangle count
    buf.extend_from_slice(&(mesh.indices.len() as u32).to_le_bytes());

    for tri in &mesh.indices {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        let normal = (v1 - v0).cross(&(v2 - v0)).normalize();

        // Normal
        for c in [normal.x, normal.y, normal.z] {
            buf.extend_from_slice(&(c as f32).to_le_bytes());
        }
        // Vertices
        for v in [v0, v1, v2] {
            for c in [v.x, v.y, v.z] {
                buf.extend_from_slice(&(c as f32).to_le_bytes());
            }
        }
        // Attribute byte count
        buf.extend_from_slice(&0u16.to_le_bytes());
    }

    tokio::fs::write(path, &buf).await
        .map_err(|e| TlantiError::IoError(e.to_string()))?;
    Ok(())
}
