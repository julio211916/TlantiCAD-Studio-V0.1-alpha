// AR-V368 — Crown generation 7-step pipeline (Tauri command surface).
//
// One command:
//   * `cad_crown_pipeline_run` — run margin → insertion → bottom → library-fit → feedback →
//                                approximal → connector-weld and write outputs.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tlanticad_crown::pipeline::{
    run_pipeline, CrownPipelineConfig, CrownPipelineReport, PipelineInputs,
};
use tlanticad_mesh::nalgebra::Point3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CrownPipelineError {
    #[error("input path not found: {path}")]
    InputNotFound { path: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("backend-formats feature is required for STL I/O")]
    FormatsFeatureMissing,
}

#[cfg(feature = "backend-formats")]
fn read_stl(path: &PathBuf) -> Result<Mesh, CrownPipelineError> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| CrownPipelineError::Io {
        message: format!("open {}: {}", path.display(), e),
    })?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| CrownPipelineError::Io {
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
    let mut mesh = Mesh::new("pipeline-input");
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    Ok(mesh)
}

#[cfg(not(feature = "backend-formats"))]
fn read_stl(_path: &PathBuf) -> Result<Mesh, CrownPipelineError> {
    Err(CrownPipelineError::FormatsFeatureMissing)
}

#[cfg(feature = "backend-formats")]
fn write_stl(mesh: &Mesh, path: &PathBuf) -> Result<(), CrownPipelineError> {
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
        .map_err(|e| CrownPipelineError::Io {
            message: format!("create {}: {}", path.display(), e),
        })?;
    let mut writer = BufWriter::new(file);
    stl_io::write_stl(&mut writer, triangles.iter()).map_err(|e| CrownPipelineError::Io {
        message: format!("write {}: {}", path.display(), e),
    })?;
    Ok(())
}

#[cfg(not(feature = "backend-formats"))]
fn write_stl(_mesh: &Mesh, _path: &PathBuf) -> Result<(), CrownPipelineError> {
    Err(CrownPipelineError::FormatsFeatureMissing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrownPipelineRequest {
    pub config: CrownPipelineConfig,
    pub prep_stl: PathBuf,
    pub output_outer_stl: PathBuf,
    #[serde(default)]
    pub library_tooth_stl: Option<PathBuf>,
    #[serde(default)]
    pub antagonist_stl: Option<PathBuf>,
    #[serde(default)]
    pub mesial_neighbour_stl: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrownPipelineResponse {
    pub output_outer_stl: PathBuf,
    pub report: CrownPipelineReport,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_crown_pipeline_run(
    request: CrownPipelineRequest,
) -> Result<CrownPipelineResponse, CrownPipelineError> {
    if !request.prep_stl.exists() {
        return Err(CrownPipelineError::InputNotFound {
            path: request.prep_stl.to_string_lossy().into_owned(),
        });
    }
    let prep = read_stl(&request.prep_stl)?;
    let library = match &request.library_tooth_stl {
        Some(p) if p.exists() => Some(read_stl(p)?),
        Some(p) => {
            return Err(CrownPipelineError::InputNotFound {
                path: p.to_string_lossy().into_owned(),
            })
        }
        None => None,
    };
    let antagonist = match &request.antagonist_stl {
        Some(p) if p.exists() => Some(read_stl(p)?),
        Some(p) => {
            return Err(CrownPipelineError::InputNotFound {
                path: p.to_string_lossy().into_owned(),
            })
        }
        None => None,
    };
    let neighbour = match &request.mesial_neighbour_stl {
        Some(p) if p.exists() => Some(read_stl(p)?),
        Some(p) => {
            return Err(CrownPipelineError::InputNotFound {
                path: p.to_string_lossy().into_owned(),
            })
        }
        None => None,
    };

    let inputs = PipelineInputs {
        config: request.config,
        prep_mesh: &prep,
        library_tooth: library.as_ref(),
        antagonist: antagonist.as_ref(),
        mesial_neighbour: neighbour.as_ref(),
    };
    let (report, outer) = run_pipeline(&inputs);
    write_stl(&outer, &request.output_outer_stl)?;

    Ok(CrownPipelineResponse {
        output_outer_stl: request.output_outer_stl,
        report,
        backend: "tlanticad-crown::pipeline",
    })
}
