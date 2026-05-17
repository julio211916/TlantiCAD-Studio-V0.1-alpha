// AR-V369 — Crown feedback (Tauri command surface).
//
// Two commands:
//   * `cad_crown_validate_real`  — full per-tooth feedback (thickness + clearance + warnings).
//   * `cad_crown_constraint_bounds` — fast lookup of material-aware bounds (UI defaults).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_crown::feedback::{
    evaluate_tooth, material_constraint_bounds, ConstraintBounds, ToothFeedbackReport,
};
use tlanticad_mesh::nalgebra::Point3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CrownFeedbackError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, CrownFeedbackError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| CrownFeedbackError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| CrownFeedbackError::Io {
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
    let mut mesh = Mesh::new("crown-feedback");
    mesh.vertices = vertices;
    mesh.indices = indices;
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, CrownFeedbackError> {
    Err(CrownFeedbackError::FormatsFeatureMissing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateCrownRequest {
    pub fdi: u8,
    pub material: String,
    pub crown_outer: PathBuf,
    pub crown_bottom: PathBuf,
    #[serde(default)]
    pub antagonist: Option<PathBuf>,
    #[serde(default)]
    pub overrides: Option<ConstraintBounds>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateCrownResponse {
    pub report: ToothFeedbackReport,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_crown_validate_real(
    request: ValidateCrownRequest,
) -> Result<ValidateCrownResponse, CrownFeedbackError> {
    if !request.crown_outer.exists() {
        return Err(CrownFeedbackError::InputNotFound {
            path: request.crown_outer.to_string_lossy().into_owned(),
        });
    }
    if !request.crown_bottom.exists() {
        return Err(CrownFeedbackError::InputNotFound {
            path: request.crown_bottom.to_string_lossy().into_owned(),
        });
    }
    let outer = read_stl(&request.crown_outer)?;
    let bottom = read_stl(&request.crown_bottom)?;
    let antagonist = match &request.antagonist {
        Some(path) => {
            if !path.exists() {
                return Err(CrownFeedbackError::InputNotFound {
                    path: path.to_string_lossy().into_owned(),
                });
            }
            Some(read_stl(path)?)
        }
        None => None,
    };
    let report = evaluate_tooth(
        request.fdi,
        &request.material,
        &outer,
        &bottom,
        antagonist.as_ref(),
        request.overrides,
    );
    Ok(ValidateCrownResponse {
        report,
        backend: "tlanticad-crown::feedback",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintBoundsRequest {
    pub material: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintBoundsResponse {
    pub bounds: ConstraintBounds,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_crown_constraint_bounds(request: ConstraintBoundsRequest) -> ConstraintBoundsResponse {
    ConstraintBoundsResponse {
        bounds: material_constraint_bounds(&request.material),
        backend: "tlanticad-crown::feedback",
    }
}
