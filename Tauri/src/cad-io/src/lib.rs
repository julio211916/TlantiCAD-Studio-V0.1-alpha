pub mod stl;
pub mod obj;
pub mod ply;

pub use cad_core::{CadError, Result, HeMesh};

/// Detect format and load a mesh from any supported file
pub fn load_mesh(path: &std::path::Path) -> Result<HeMesh> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "stl" => stl::read(path),
        "obj" => obj::read(path),
        "ply" => ply::read(path),
        other => Err(CadError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("Unsupported file extension: .{other}"),
        ))),
    }
}

/// Save a mesh to disk; format selected by extension
pub fn save_mesh(mesh: &HeMesh, path: &std::path::Path) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "stl" => stl::write_binary(mesh, path),
        "obj" => obj::write(mesh, path),
        other => Err(CadError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("Cannot write .{other} format"),
        ))),
    }
}
