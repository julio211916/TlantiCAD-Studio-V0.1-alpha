//! Mesh file format support

use app_core::types::MeshData;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use tracing::info;

use crate::{MeshError, Result};

/// Supported mesh formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    Obj,
    Stl,
    Ply,
    Gltf,
    Glb,
    Unknown,
}

impl MeshFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "obj" => MeshFormat::Obj,
            "stl" => MeshFormat::Stl,
            "ply" => MeshFormat::Ply,
            "gltf" => MeshFormat::Gltf,
            "glb" => MeshFormat::Glb,
            _ => MeshFormat::Unknown,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(MeshFormat::Unknown)
    }
}

/// Load mesh from file
pub fn load_mesh(path: &Path) -> Result<MeshData> {
    let format = MeshFormat::from_path(path);
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();

    info!("Loading mesh: {:?} (format: {:?})", path, format);

    match format {
        MeshFormat::Obj => load_obj(path, name),
        MeshFormat::Stl => load_stl(path, name),
        MeshFormat::Ply => load_ply(path, name),
        _ => Err(MeshError::InvalidFormat(format!(
            "Unsupported format: {:?}",
            format
        ))),
    }
}

/// Save mesh to file
pub fn save_mesh(mesh: &MeshData, path: &Path) -> Result<()> {
    let format = MeshFormat::from_path(path);

    info!("Saving mesh: {:?} (format: {:?})", path, format);

    match format {
        MeshFormat::Obj => save_obj(mesh, path),
        MeshFormat::Stl => save_stl(mesh, path),
        _ => Err(MeshError::InvalidFormat(format!(
            "Unsupported format for saving: {:?}",
            format
        ))),
    }
}

/// Load OBJ file
fn load_obj(path: &Path, name: String) -> Result<MeshData> {
    use wavefront_obj::obj;

    let file_content =
        std::fs::read_to_string(path).map_err(|e| MeshError::LoadError(e.to_string()))?;

    let obj_set = obj::parse(&file_content).map_err(|e| MeshError::LoadError(e.to_string()))?;

    let mut mesh = MeshData::new(name);

    // Extract first object
    if let Some(object) = obj_set.objects.first() {
        // Vertices
        for v in &object.vertices {
            mesh.vertices.push(v.x as f32);
            mesh.vertices.push(v.y as f32);
            mesh.vertices.push(v.z as f32);
        }

        // Faces/indices
        for geom in &object.geometry {
            for shape in &geom.shapes {
                if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
                    mesh.indices.push(a.0 as u32);
                    mesh.indices.push(b.0 as u32);
                    mesh.indices.push(c.0 as u32);
                }
            }
        }

        // Normals
        for n in &object.normals {
            mesh.normals.push(n.x as f32);
            mesh.normals.push(n.y as f32);
            mesh.normals.push(n.z as f32);
        }

        // UVs
        for t in &object.tex_vertices {
            mesh.uvs.push(t.u as f32);
            mesh.uvs.push(t.v as f32);
        }
    }

    Ok(mesh)
}

/// Load STL file
fn load_stl(path: &Path, name: String) -> Result<MeshData> {
    let file = File::open(path).map_err(|e| MeshError::LoadError(e.to_string()))?;
    let mut reader = BufReader::new(file);

    let stl =
        stl_io::read_stl(&mut reader).map_err(|e| MeshError::LoadError(format!("{:?}", e)))?;

    let mut mesh = MeshData::new(name);

    for (i, triangle) in stl.faces.iter().enumerate() {
        // Add vertices from the mesh's vertex array using the triangle's indices
        for v_idx in &triangle.vertices {
            let vertex = &stl.vertices[*v_idx];
            mesh.vertices.push(vertex[0]);
            mesh.vertices.push(vertex[1]);
            mesh.vertices.push(vertex[2]);
        }

        // Add indices
        let base = (i * 3) as u32;
        mesh.indices.push(base);
        mesh.indices.push(base + 1);
        mesh.indices.push(base + 2);

        // Add normals (same for all 3 vertices of triangle)
        for _ in 0..3 {
            mesh.normals.push(triangle.normal[0]);
            mesh.normals.push(triangle.normal[1]);
            mesh.normals.push(triangle.normal[2]);
        }
    }

    Ok(mesh)
}

/// Load PLY file
fn load_ply(path: &Path, name: String) -> Result<MeshData> {
    use ply_rs::parser::Parser;
    use ply_rs::ply::PropertyAccess;

    let file = File::open(path).map_err(|e| MeshError::LoadError(e.to_string()))?;
    let mut reader = BufReader::new(file);

    let vertex_parser = Parser::<ply_rs::ply::DefaultElement>::new();
    let ply = vertex_parser
        .read_ply(&mut reader)
        .map_err(|e| MeshError::LoadError(format!("{:?}", e)))?;

    let mut mesh = MeshData::new(name);

    // Extract vertices
    if let Some(vertices) = ply.payload.get("vertex") {
        for vertex in vertices {
            let x = vertex.get_float(&"x".to_string()).unwrap_or(0.0);
            let y = vertex.get_float(&"y".to_string()).unwrap_or(0.0);
            let z = vertex.get_float(&"z".to_string()).unwrap_or(0.0);

            mesh.vertices.push(x);
            mesh.vertices.push(y);
            mesh.vertices.push(z);

            // Normals if available
            if let (Some(nx), Some(ny), Some(nz)) = (
                vertex.get_float(&"nx".to_string()),
                vertex.get_float(&"ny".to_string()),
                vertex.get_float(&"nz".to_string()),
            ) {
                mesh.normals.push(nx);
                mesh.normals.push(ny);
                mesh.normals.push(nz);
            }
        }
    }

    // Extract faces
    if let Some(faces) = ply.payload.get("face") {
        for face in faces {
            if let Some(indices) = face.get_list_int(&"vertex_indices".to_string()) {
                if indices.len() >= 3 {
                    mesh.indices.push(indices[0] as u32);
                    mesh.indices.push(indices[1] as u32);
                    mesh.indices.push(indices[2] as u32);
                }
            }
        }
    }

    Ok(mesh)
}

/// Save OBJ file
fn save_obj(mesh: &MeshData, path: &Path) -> Result<()> {
    use std::io::Write;

    let file = File::create(path).map_err(|e| MeshError::SaveError(e.to_string()))?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "# TlantiStudio OBJ Export").map_err(|e| MeshError::SaveError(e.to_string()))?;
    writeln!(writer, "# Vertices: {}", mesh.vertex_count()).map_err(|e| MeshError::SaveError(e.to_string()))?;
    writeln!(writer, "# Faces: {}", mesh.face_count()).map_err(|e| MeshError::SaveError(e.to_string()))?;
    writeln!(writer).map_err(|e| MeshError::SaveError(e.to_string()))?;

    // Write vertices
    for i in 0..mesh.vertex_count() {
        let idx = i * 3;
        writeln!(
            writer,
            "v {} {} {}",
            mesh.vertices[idx],
            mesh.vertices[idx + 1],
            mesh.vertices[idx + 2]
        )
        .map_err(|e| MeshError::SaveError(e.to_string()))?;
    }

    // Write normals
    if !mesh.normals.is_empty() {
        let normal_count = mesh.normals.len() / 3;
        for i in 0..normal_count {
            let idx = i * 3;
            writeln!(
                writer,
                "vn {} {} {}",
                mesh.normals[idx],
                mesh.normals[idx + 1],
                mesh.normals[idx + 2]
            )
            .map_err(|e| MeshError::SaveError(e.to_string()))?;
        }
    }

    // Write faces
    for i in 0..mesh.face_count() {
        let idx = i * 3;
        writeln!(
            writer,
            "f {} {} {}",
            mesh.indices[idx] + 1,
            mesh.indices[idx + 1] + 1,
            mesh.indices[idx + 2] + 1
        )
        .map_err(|e| MeshError::SaveError(e.to_string()))?;
    }

    Ok(())
}

/// Save STL file
fn save_stl(mesh: &MeshData, path: &Path) -> Result<()> {
    let file = File::create(path).map_err(|e| MeshError::SaveError(e.to_string()))?;
    let mut writer = BufWriter::new(file);

    let mut triangles = Vec::new();

    for i in 0..mesh.face_count() {
        let idx = i * 3;
        let i0 = mesh.indices[idx] as usize;
        let i1 = mesh.indices[idx + 1] as usize;
        let i2 = mesh.indices[idx + 2] as usize;

        let v0 = [
            mesh.vertices[i0 * 3],
            mesh.vertices[i0 * 3 + 1],
            mesh.vertices[i0 * 3 + 2],
        ];
        let v1 = [
            mesh.vertices[i1 * 3],
            mesh.vertices[i1 * 3 + 1],
            mesh.vertices[i1 * 3 + 2],
        ];
        let v2 = [
            mesh.vertices[i2 * 3],
            mesh.vertices[i2 * 3 + 1],
            mesh.vertices[i2 * 3 + 2],
        ];

        // Calculate normal
        let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let normal = [
            edge1[1] * edge2[2] - edge1[2] * edge2[1],
            edge1[2] * edge2[0] - edge1[0] * edge2[2],
            edge1[0] * edge2[1] - edge1[1] * edge2[0],
        ];

        triangles.push(stl_io::Triangle {
            normal: stl_io::Normal::new(normal),
            vertices: [
                stl_io::Vertex::new(v0),
                stl_io::Vertex::new(v1),
                stl_io::Vertex::new(v2),
            ],
        });
    }

    stl_io::write_stl(&mut writer, triangles.iter())
        .map_err(|e| MeshError::SaveError(format!("{:?}", e)))?;

    Ok(())
}
