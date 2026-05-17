//! TlantiCAD Import/Export Module

pub mod stl;
pub mod obj;
pub mod ply;
pub mod dicom;

use tlanticad_core::Result;
use tlanticad_mesh::Mesh;
use std::path::Path;

pub async fn import_mesh(path: impl AsRef<Path>) -> Result<Mesh> {
    let path = path.as_ref();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match ext.as_str() {
        "stl" => stl::import(path).await,
        "obj" => obj::import(path).await,
        "ply" => ply::import(path).await,
        _ => Err(tlanticad_core::TlantiError::IoError(format!(
            "Unsupported file format: {}", ext
        ))),
    }
}

pub async fn export_mesh(mesh: &Mesh, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match ext.as_str() {
        "stl" => stl::export(mesh, path).await,
        "obj" => obj::export(mesh, path).await,
        "ply" => ply::export(mesh, path).await,
        _ => Err(tlanticad_core::TlantiError::IoError(format!(
            "Unsupported file format: {}", ext
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::Mesh;

    #[tokio::test]
    async fn test_import_unsupported_format() {
        let result = import_mesh("/tmp/fake.xyz").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_export_unsupported_format() {
        let mesh = Mesh::new("test");
        let result = export_mesh(&mesh, "/tmp/fake.xyz").await;
        assert!(result.is_err());
    }

    fn make_triangle() -> Mesh {
        use nalgebra::{Point3, Vector3};
        let mut mesh = Mesh::new("tri");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        mesh.normals = vec![Vector3::new(0.0, 0.0, 1.0); 3];
        mesh.indices = vec![[0, 1, 2]];
        mesh
    }

    #[tokio::test]
    async fn test_stl_roundtrip() {
        let mesh = make_triangle();
        let path = "/tmp/tlanticad_test_roundtrip.stl";
        export_mesh(&mesh, path).await.unwrap();
        let loaded = import_mesh(path).await.unwrap();
        assert!(!loaded.vertices.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_obj_roundtrip() {
        let mesh = make_triangle();
        let path = "/tmp/tlanticad_test_roundtrip.obj";
        export_mesh(&mesh, path).await.unwrap();
        let loaded = import_mesh(path).await.unwrap();
        assert!(!loaded.vertices.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_ply_roundtrip() {
        let mesh = make_triangle();
        let path = "/tmp/tlanticad_test_roundtrip.ply";
        export_mesh(&mesh, path).await.unwrap();
        let loaded = import_mesh(path).await.unwrap();
        assert!(!loaded.vertices.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_import_nonexistent_stl() {
        let result = import_mesh("/tmp/nonexistent_file_12345.stl").await;
        assert!(result.is_err());
    }
}
