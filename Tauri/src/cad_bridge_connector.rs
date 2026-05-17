// AR-V366 — Bridge connector (Tauri command surface).
//
// One command:
//   * `cad_bridge_connector_create` — given two STL crowns + parameters, find the closest
//     anchor pair, generate an elliptical loft, and write the connector STL.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_bridge::connector::{
    closest_pair, generate_connector_mesh, ConnectorAnchors, ConnectorParams, ConnectorType,
    LoftOptions,
};
use tlanticad_mesh::nalgebra::Point3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum BridgeConnectorError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
    #[error("could not find anchor pair: meshes are empty or disjoint at infinity")]
    NoAnchors,
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, BridgeConnectorError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| BridgeConnectorError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| BridgeConnectorError::Io {
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
    let mut mesh = Mesh::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "mesh".into()),
    );
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, BridgeConnectorError> {
    Err(BridgeConnectorError::FormatsFeatureMissing)
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), BridgeConnectorError> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;
    use stl_io::{Triangle, Vector};
    let triangles: Vec<Triangle> = mesh
        .indices
        .iter()
        .map(|tri| {
            let v0 = mesh.vertices[tri[0] as usize];
            let v1 = mesh.vertices[tri[1] as usize];
            let v2 = mesh.vertices[tri[2] as usize];
            let e1 = v1 - v0;
            let e2 = v2 - v0;
            let n = e1.cross(&e2);
            let len = n.norm().max(f64::EPSILON);
            Triangle {
                normal: Vector::new([(n.x / len) as f32, (n.y / len) as f32, (n.z / len) as f32]),
                vertices: [
                    Vector::new([v0.x as f32, v0.y as f32, v0.z as f32]),
                    Vector::new([v1.x as f32, v1.y as f32, v1.z as f32]),
                    Vector::new([v2.x as f32, v2.y as f32, v2.z as f32]),
                ],
            }
        })
        .collect();
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|e| BridgeConnectorError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| BridgeConnectorError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), BridgeConnectorError> {
    Err(BridgeConnectorError::FormatsFeatureMissing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeConnectorRequest {
    pub crown_a: PathBuf,
    pub crown_b: PathBuf,
    pub output: PathBuf,
    /// Width (mesiodistal, mm). Default 3.0.
    #[serde(default = "default_width")]
    pub width_mm: f64,
    /// Height (occlusogingival, mm). Default 3.0.
    #[serde(default = "default_height")]
    pub height_mm: f64,
    /// Anatomic up vector (occlusal direction). Used to orient the ellipse. Default +Z.
    #[serde(default = "default_up")]
    pub occlusal_up: [f64; 3],
    /// Connector type ("rigid", "semi-precision", "precision"). Default rigid.
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default = "default_axial")]
    pub axial_segments: u32,
    #[serde(default = "default_radial")]
    pub radial_segments: u32,
}

fn default_width() -> f64 {
    3.0
}
fn default_height() -> f64 {
    3.0
}
fn default_up() -> [f64; 3] {
    [0.0, 0.0, 1.0]
}
fn default_kind() -> String {
    "rigid".into()
}
fn default_axial() -> u32 {
    8
}
fn default_radial() -> u32 {
    16
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeConnectorResponse {
    pub output: PathBuf,
    pub anchor_a: [f64; 3],
    pub anchor_b: [f64; 3],
    pub triangles: usize,
    pub cross_section_mm2: f64,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_bridge_connector_create(
    request: BridgeConnectorRequest,
) -> Result<BridgeConnectorResponse, BridgeConnectorError> {
    if !request.crown_a.exists() {
        return Err(BridgeConnectorError::InputNotFound {
            path: request.crown_a.to_string_lossy().into_owned(),
        });
    }
    if !request.crown_b.exists() {
        return Err(BridgeConnectorError::InputNotFound {
            path: request.crown_b.to_string_lossy().into_owned(),
        });
    }
    let a = read_stl(&request.crown_a)?;
    let b = read_stl(&request.crown_b)?;
    let (pa, pb) = closest_pair(&a, &b).ok_or(BridgeConnectorError::NoAnchors)?;

    let kind = match request.kind.to_lowercase().as_str() {
        "semi-precision" | "semi_precision" | "semiprecision" => ConnectorType::SemiPrecision,
        "precision" => ConnectorType::Precision,
        _ => ConnectorType::Rigid,
    };
    // Cross-section area for an ellipse = π × (w/2) × (h/2).
    let area = std::f64::consts::PI * (request.width_mm / 2.0) * (request.height_mm / 2.0);
    let params = ConnectorParams {
        connector_type: kind,
        height: request.height_mm,
        width: request.width_mm,
        cross_section_area: area,
    };
    let opts = LoftOptions {
        axial_segments: request.axial_segments.max(2),
        radial_segments: request.radial_segments.max(6),
    };
    let anchors = ConnectorAnchors {
        a: [pa.x, pa.y, pa.z],
        b: [pb.x, pb.y, pb.z],
        occlusal_up: request.occlusal_up,
    };
    let mesh = generate_connector_mesh(&anchors, &params, &opts);
    write_stl(&mesh, &request.output)?;

    Ok(BridgeConnectorResponse {
        output: request.output,
        anchor_a: anchors.a,
        anchor_b: anchors.b,
        triangles: mesh.triangle_count(),
        cross_section_mm2: area,
        backend: "tlanticad-bridge::connector",
    })
}
