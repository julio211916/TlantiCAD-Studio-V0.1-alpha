//! Renderer/compute boundary spike.
//!
//! This module measures the cost of keeping mesh-heavy work behind Rust instead of
//! serialising full mesh buffers through Tauri IPC into React/Three.

use crate::backend::{ComputeBackend, ComputeOp};
use crate::cpu::CpuBackend;
use crate::profile::EnergyMode;
use chrono::Utc;
use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;
use tlanticad_mesh::Mesh;

const DEFAULT_SAMPLE_VERTICES: usize = 4_096;
const MAX_SAMPLE_VERTICES: usize = 25_000;
const JSON_IPC_WARNING_BYTES: u64 = 8 * 1024 * 1024;
const FRAME_BUDGET_MS: f64 = 16.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryBenchRequest {
    pub sample_vertices: Option<usize>,
    pub energy_mode: Option<EnergyMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryBenchProfile {
    pub generated_at: String,
    pub host_id: String,
    pub energy_mode: EnergyMode,
    pub sample_vertices: usize,
    pub sample_triangles: usize,
    pub mesh_buffer_bytes: u64,
    pub json_ipc_bytes: u64,
    pub transform_ipc_bytes: u64,
    pub json_serialise_ms: f64,
    pub cpu_smooth_ms: f64,
    pub cpu_items_per_ms: f64,
    pub ipc_payload_ratio: f64,
    pub frame_budget_ms: f64,
    pub recommended_boundary: BoundaryRecommendation,
    pub results: Vec<BoundaryBenchResult>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundaryRecommendation {
    KeepThreeRenderOnly,
    RustComputeThreeRender,
    RustWgpuCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryBenchResult {
    pub op: String,
    pub elapsed_ms: f64,
    pub items_processed: u64,
    pub payload_bytes: u64,
    pub owner: BoundaryOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundaryOwner {
    ReactUi,
    ThreeRenderer,
    TauriCommand,
    RustCompute,
    RustWgpuCandidate,
}

impl BoundaryBenchProfile {
    pub fn run(host_id: impl Into<String>, request: BoundaryBenchRequest) -> Self {
        let host_id = host_id.into();
        let requested_vertices = request.sample_vertices.unwrap_or(DEFAULT_SAMPLE_VERTICES);
        let sample_vertices = requested_vertices.clamp(64, MAX_SAMPLE_VERTICES);
        let energy_mode = request.energy_mode.unwrap_or(EnergyMode::Performance);
        let mut mesh = synthetic_grid(sample_vertices);
        let sample_vertices = mesh.vertex_count();
        let sample_triangles = mesh.triangle_count();
        let mesh_buffer_bytes = estimate_gpu_buffer_bytes(&mesh);
        let transform_ipc_bytes = 16 * std::mem::size_of::<f32>() as u64;

        let serialise_started = Instant::now();
        let json_ipc_bytes = serde_json::to_vec(&mesh).map(|raw| raw.len() as u64).unwrap_or(0);
        let json_serialise_ms = elapsed_ms(serialise_started);

        let backend = CpuBackend::new();
        let smooth_started = Instant::now();
        let smooth_outcome = backend.laplacian_smooth(&mut mesh, 2, 0.35);
        let cpu_smooth_ms = elapsed_ms(smooth_started);
        let items_processed = smooth_outcome
            .ok()
            .map(|stats| stats.items_processed)
            .unwrap_or(sample_vertices as u64);
        let cpu_items_per_ms = if cpu_smooth_ms <= f64::EPSILON {
            items_processed as f64
        } else {
            items_processed as f64 / cpu_smooth_ms
        };

        let ipc_payload_ratio = if transform_ipc_bytes == 0 {
            0.0
        } else {
            json_ipc_bytes as f64 / transform_ipc_bytes as f64
        };

        let recommended_boundary = recommend_boundary(json_ipc_bytes, cpu_smooth_ms, sample_vertices);

        let mut notes = vec![
            "React should send ids, transforms and tool params only; full mesh buffers stay in Rust/filesystem or GPU buffers.".to_string(),
            "Three.js keeps camera, materials, selection overlays and final drawing until a real native wgpu viewport exists.".to_string(),
            "Rust owns mesh cleanup, smoothing, distance fields, decimation, CSG, validation and export jobs.".to_string(),
        ];

        if json_ipc_bytes >= JSON_IPC_WARNING_BYTES {
            notes.push("JSON IPC crossed the 8 MB warning threshold; avoid returning vertex arrays through Tauri.".to_string());
        }
        if cpu_smooth_ms > FRAME_BUDGET_MS {
            notes.push("CPU smoothing exceeded a 16 ms frame budget; keep it out of the UI/render loop and run as a job.".to_string());
        }

        Self {
            generated_at: Utc::now().to_rfc3339(),
            host_id,
            energy_mode,
            sample_vertices,
            sample_triangles,
            mesh_buffer_bytes,
            json_ipc_bytes,
            transform_ipc_bytes,
            json_serialise_ms,
            cpu_smooth_ms,
            cpu_items_per_ms,
            ipc_payload_ratio,
            frame_budget_ms: FRAME_BUDGET_MS,
            recommended_boundary,
            results: vec![
                BoundaryBenchResult {
                    op: "json-ipc-serialise-mesh".to_string(),
                    elapsed_ms: json_serialise_ms,
                    items_processed: sample_vertices as u64,
                    payload_bytes: json_ipc_bytes,
                    owner: BoundaryOwner::TauriCommand,
                },
                BoundaryBenchResult {
                    op: "laplacian-smooth-cpu".to_string(),
                    elapsed_ms: cpu_smooth_ms,
                    items_processed,
                    payload_bytes: mesh_buffer_bytes,
                    owner: BoundaryOwner::RustCompute,
                },
                BoundaryBenchResult {
                    op: format!("{:?}", ComputeOp::LaplacianSmooth),
                    elapsed_ms: cpu_smooth_ms,
                    items_processed,
                    payload_bytes: mesh_buffer_bytes,
                    owner: if cpu_smooth_ms > FRAME_BUDGET_MS {
                        BoundaryOwner::RustWgpuCandidate
                    } else {
                        BoundaryOwner::RustCompute
                    },
                },
            ],
            notes,
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("serialise: {e}"))
        })?;
        std::fs::write(path, json)
    }
}

fn recommend_boundary(
    json_ipc_bytes: u64,
    cpu_smooth_ms: f64,
    sample_vertices: usize,
) -> BoundaryRecommendation {
    if cpu_smooth_ms > FRAME_BUDGET_MS * 2.0 || sample_vertices > 20_000 {
        BoundaryRecommendation::RustWgpuCandidate
    } else if json_ipc_bytes >= JSON_IPC_WARNING_BYTES || cpu_smooth_ms > FRAME_BUDGET_MS {
        BoundaryRecommendation::RustComputeThreeRender
    } else {
        BoundaryRecommendation::KeepThreeRenderOnly
    }
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1_000.0
}

fn estimate_gpu_buffer_bytes(mesh: &Mesh) -> u64 {
    let positions = mesh.vertices.len() as u64 * 3 * std::mem::size_of::<f32>() as u64;
    let normals = mesh.vertices.len() as u64 * 3 * std::mem::size_of::<f32>() as u64;
    let indices = mesh.indices.len() as u64 * 3 * std::mem::size_of::<u32>() as u64;
    positions + normals + indices
}

fn synthetic_grid(target_vertices: usize) -> Mesh {
    let side = (target_vertices as f64).sqrt().ceil().max(8.0) as usize;
    let mut mesh = Mesh::new("boundary-grid");
    mesh.vertices.reserve(side * side);
    mesh.indices.reserve((side - 1) * (side - 1) * 2);

    for y in 0..side {
        for x in 0..side {
            let z = ((x as f64) * 0.13).sin() * ((y as f64) * 0.11).cos() * 0.08;
            mesh.vertices.push(Point3::new(x as f64, y as f64, z));
        }
    }

    for y in 0..(side - 1) {
        for x in 0..(side - 1) {
            let a = (y * side + x) as u32;
            let b = a + 1;
            let c = ((y + 1) * side + x) as u32;
            let d = c + 1;
            mesh.indices.push([a, c, b]);
            mesh.indices.push([b, c, d]);
        }
    }

    mesh.calculate_normals();
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_bench_caps_requested_vertices() {
        let profile = BoundaryBenchProfile::run(
            "ci-host",
            BoundaryBenchRequest {
                sample_vertices: Some(MAX_SAMPLE_VERTICES * 2),
                energy_mode: Some(EnergyMode::LowPower),
            },
        );
        assert!(profile.sample_vertices <= MAX_SAMPLE_VERTICES + 400);
        assert_eq!(profile.energy_mode, EnergyMode::LowPower);
        assert!(profile.mesh_buffer_bytes > 0);
        assert!(profile.json_ipc_bytes > profile.transform_ipc_bytes);
    }

    #[test]
    fn boundary_bench_reports_owners_and_notes() {
        let profile = BoundaryBenchProfile::run(
            "ci-host",
            BoundaryBenchRequest {
                sample_vertices: Some(256),
                energy_mode: None,
            },
        );
        assert!(profile.results.iter().any(|result| matches!(result.owner, BoundaryOwner::RustCompute)));
        assert!(profile.notes.iter().any(|note| note.contains("React should send ids")));
    }
}
