// V118 + V200 — CAD CSG Bridge (Tauri command surface).
//
// Exposes a single `cad_csg::mesh_op` command. The frontend submits a JSON
// description of a boolean op and a list of input mesh paths; the backend
// reads them, runs the operation through `manifold-csg`, writes the output
// STL, and returns metadata (triangle count, watertight flag, volume, genus).
//
// Until the optional features `backend-manifold-csg` / `backend-formats` are
// enabled the command returns a structured `CsgUnsupported` error so the
// frontend can fall back to the Python `cad_merge` mock backend without
// crashing.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CsgOp {
    Union,
    Subtract,
    Intersect,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // fields are consumed only when the `backend-manifold-csg` feature is enabled
pub struct MeshOpRequest {
    pub op: CsgOp,
    /// Input STL paths. The first path is the base; remaining paths are the
    /// operands (subtract / union / intersect with the base, applied left-to-right).
    pub inputs: Vec<PathBuf>,
    /// Where to write the output STL.
    pub output: PathBuf,
    /// Apply manifold post-process repair (closes small gaps under 0.05 mm).
    #[serde(default)]
    pub repair: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshOpResponse {
    pub output: PathBuf,
    pub triangles: u64,
    pub watertight: bool,
    pub volume_mm3: f64,
    pub genus: i32,
    pub backend: &'static str,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
#[allow(dead_code)]
pub enum MeshOpError {
    #[error("CSG kernel not available — enable feature `backend-manifold-csg`")]
    CsgUnsupported,
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("invalid input vector — at least 2 inputs required for {op:?}")]
    InvalidInputs { op: CsgOp },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("kernel error: {message}")]
    Kernel { message: String },
}

#[tauri::command]
pub fn mesh_op(request: MeshOpRequest) -> Result<MeshOpResponse, MeshOpError> {
    if request.inputs.len() < 2 {
        return Err(MeshOpError::InvalidInputs { op: request.op });
    }
    for path in &request.inputs {
        if !path.exists() {
            return Err(MeshOpError::InputNotFound {
                path: path.to_string_lossy().into_owned(),
            });
        }
    }

    #[cfg(all(feature = "backend-manifold-csg", feature = "backend-formats"))]
    {
        run_with_manifold(&request)
    }

    #[cfg(not(all(feature = "backend-manifold-csg", feature = "backend-formats")))]
    {
        let _ = request;
        Err(MeshOpError::CsgUnsupported)
    }
}

#[cfg(all(feature = "backend-manifold-csg", feature = "backend-formats"))]
fn run_with_manifold(request: &MeshOpRequest) -> Result<MeshOpResponse, MeshOpError> {
    use std::fs::{File, OpenOptions};
    use std::io::{BufReader, BufWriter};

    use manifold_csg::Manifold;
    use stl_io::{IndexedMesh, Triangle, Vector};

    fn load_stl(path: &PathBuf) -> Result<IndexedMesh, MeshOpError> {
        let file = File::open(path).map_err(|e| MeshOpError::Io {
            message: format!("open {}: {}", path.display(), e),
        })?;
        let mut reader = BufReader::new(file);
        stl_io::read_stl(&mut reader).map_err(|e| MeshOpError::Io {
            message: format!("parse {}: {}", path.display(), e),
        })
    }

    fn indexed_to_manifold(mesh: &IndexedMesh) -> Result<Manifold, MeshOpError> {
        let mut vert_props: Vec<f64> = Vec::with_capacity(mesh.vertices.len() * 3);
        for v in &mesh.vertices {
            vert_props.push(v[0] as f64);
            vert_props.push(v[1] as f64);
            vert_props.push(v[2] as f64);
        }
        let mut tri_indices: Vec<u64> = Vec::with_capacity(mesh.faces.len() * 3);
        for face in &mesh.faces {
            tri_indices.push(face.vertices[0] as u64);
            tri_indices.push(face.vertices[1] as u64);
            tri_indices.push(face.vertices[2] as u64);
        }
        Manifold::from_mesh_f64(&vert_props, 3, &tri_indices).map_err(|e| MeshOpError::Kernel {
            message: format!("manifold construction: {:?}", e),
        })
    }

    fn manifold_to_stl(manifold: &Manifold, out_path: &PathBuf) -> Result<u64, MeshOpError> {
        let (verts, n_props, indices) = manifold.to_mesh_f64();
        if n_props < 3 {
            return Err(MeshOpError::Kernel {
                message: format!("expected ≥3 vertex props, got {}", n_props),
            });
        }
        let triangles: Vec<Triangle> = indices
            .chunks_exact(3)
            .map(|tri| {
                let v0_off = tri[0] as usize * n_props;
                let v1_off = tri[1] as usize * n_props;
                let v2_off = tri[2] as usize * n_props;
                let v0 = Vector::new([
                    verts[v0_off] as f32,
                    verts[v0_off + 1] as f32,
                    verts[v0_off + 2] as f32,
                ]);
                let v1 = Vector::new([
                    verts[v1_off] as f32,
                    verts[v1_off + 1] as f32,
                    verts[v1_off + 2] as f32,
                ]);
                let v2 = Vector::new([
                    verts[v2_off] as f32,
                    verts[v2_off + 1] as f32,
                    verts[v2_off + 2] as f32,
                ]);
                // Recompute normal from CCW vertex order.
                let ax = v1[0] - v0[0];
                let ay = v1[1] - v0[1];
                let az = v1[2] - v0[2];
                let bx = v2[0] - v0[0];
                let by = v2[1] - v0[1];
                let bz = v2[2] - v0[2];
                let nx = ay * bz - az * by;
                let ny = az * bx - ax * bz;
                let nz = ax * by - ay * bx;
                let len = (nx * nx + ny * ny + nz * nz).sqrt().max(f32::EPSILON);
                Triangle {
                    normal: Vector::new([nx / len, ny / len, nz / len]),
                    vertices: [v0, v1, v2],
                }
            })
            .collect();

        let count = triangles.len() as u64;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(out_path)
            .map_err(|e| MeshOpError::Io {
                message: format!("create {}: {}", out_path.display(), e),
            })?;
        let mut writer = BufWriter::new(file);
        stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| MeshOpError::Io {
            message: format!("write {}: {}", out_path.display(), e),
        })?;
        Ok(count)
    }

    // Load and convert all inputs.
    let mut iter = request.inputs.iter();
    let first_path = iter.next().expect("invariant: ≥2 inputs validated above");
    let mut accumulator = indexed_to_manifold(&load_stl(first_path)?)?;
    for path in iter {
        let next = indexed_to_manifold(&load_stl(path)?)?;
        accumulator = match request.op {
            CsgOp::Union => accumulator.union(&next),
            CsgOp::Subtract => accumulator.difference(&next),
            CsgOp::Intersect => accumulator.intersection(&next),
        };
    }

    let triangles = manifold_to_stl(&accumulator, &request.output)?;
    Ok(MeshOpResponse {
        output: request.output.clone(),
        triangles,
        watertight: !accumulator.is_empty(),
        volume_mm3: accumulator.volume(),
        genus: accumulator.genus(),
        backend: "manifold-csg",
    })
}
