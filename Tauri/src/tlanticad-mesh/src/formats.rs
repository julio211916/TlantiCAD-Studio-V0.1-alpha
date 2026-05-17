//! Mesh format I/O: STL, OBJ, PLY

use nalgebra::{Point3, Vector3};
use crate::Mesh;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::path::Path;

/// Load a mesh from STL (ASCII)
pub fn load_stl(path: &Path) -> Result<Mesh, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut mesh = Mesh::new(path.file_stem().unwrap_or_default().to_string_lossy());
    let mut current_normal = Vector3::zeros();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let trimmed = line.trim();
        if trimmed.starts_with("facet normal") {
            let parts: Vec<f64> = trimmed.split_whitespace().skip(2)
                .filter_map(|s| s.parse().ok()).collect();
            if parts.len() == 3 {
                current_normal = Vector3::new(parts[0], parts[1], parts[2]);
            }
        } else if trimmed.starts_with("vertex") {
            let parts: Vec<f64> = trimmed.split_whitespace().skip(1)
                .filter_map(|s| s.parse().ok()).collect();
            if parts.len() == 3 {
                mesh.vertices.push(Point3::new(parts[0], parts[1], parts[2]));
                mesh.normals.push(current_normal);
            }
        } else if trimmed == "endfacet" {
            let n = mesh.vertices.len() as u32;
            if n >= 3 {
                mesh.indices.push([n - 3, n - 2, n - 1]);
            }
        }
    }
    Ok(mesh)
}

/// Save a mesh to STL (ASCII)
pub fn save_stl(mesh: &Mesh, path: &Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut w = BufWriter::new(file);
    writeln!(w, "solid {}", mesh.name).map_err(|e| e.to_string())?;
    for tri in &mesh.indices {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        let normal = (v1 - v0).cross(&(v2 - v0)).normalize();
        writeln!(w, "  facet normal {} {} {}", normal.x, normal.y, normal.z).map_err(|e| e.to_string())?;
        writeln!(w, "    outer loop").map_err(|e| e.to_string())?;
        for v in [v0, v1, v2] {
            writeln!(w, "      vertex {} {} {}", v.x, v.y, v.z).map_err(|e| e.to_string())?;
        }
        writeln!(w, "    endloop").map_err(|e| e.to_string())?;
        writeln!(w, "  endfacet").map_err(|e| e.to_string())?;
    }
    writeln!(w, "endsolid {}", mesh.name).map_err(|e| e.to_string())?;
    Ok(())
}

/// Load a mesh from OBJ
pub fn load_obj(path: &Path) -> Result<Mesh, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut mesh = Mesh::new(path.file_stem().unwrap_or_default().to_string_lossy());

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let trimmed = line.trim();
        if trimmed.starts_with("v ") {
            let parts: Vec<f64> = trimmed.split_whitespace().skip(1)
                .filter_map(|s| s.parse().ok()).collect();
            if parts.len() >= 3 {
                mesh.vertices.push(Point3::new(parts[0], parts[1], parts[2]));
            }
        } else if trimmed.starts_with("f ") {
            let indices: Vec<u32> = trimmed.split_whitespace().skip(1)
                .filter_map(|s| s.split('/').next()?.parse::<u32>().ok().map(|i| i - 1))
                .collect();
            // Triangulate face
            for i in 1..indices.len().saturating_sub(1) {
                mesh.indices.push([indices[0], indices[i], indices[i + 1]]);
            }
        }
    }
    mesh.calculate_normals();
    Ok(mesh)
}

/// Save a mesh to OBJ
pub fn save_obj(mesh: &Mesh, path: &Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut w = BufWriter::new(file);
    writeln!(w, "# TlantiCAD OBJ export - {}", mesh.name).map_err(|e| e.to_string())?;
    for v in &mesh.vertices {
        writeln!(w, "v {} {} {}", v.x, v.y, v.z).map_err(|e| e.to_string())?;
    }
    for n in &mesh.normals {
        writeln!(w, "vn {} {} {}", n.x, n.y, n.z).map_err(|e| e.to_string())?;
    }
    for tri in &mesh.indices {
        writeln!(w, "f {}//{} {}//{} {}//{}", 
            tri[0]+1, tri[0]+1, tri[1]+1, tri[1]+1, tri[2]+1, tri[2]+1)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
