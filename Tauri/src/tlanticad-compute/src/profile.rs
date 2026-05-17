//! BenchProfile — micro-benchmark each available backend at startup and persist the ranking.
//! Used by `ComputeRouter` to pick the fastest accelerator for each op.

use crate::backend::{ComputeBackend, ComputeKind, ComputeOp};
use crate::cpu::CpuBackend;
use chrono::Utc;
use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tlanticad_mesh::create_box;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EnergyMode {
    /// Default — best-of-class accelerator regardless of power.
    Performance,
    /// Battery / quiet — prefer iGPU + CPU, avoid dGPU + dedicated NPUs.
    LowPower,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    pub backend: ComputeKind,
    pub op: ComputeOp,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

/// Persistent profile of every benchmark run on this machine.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchProfile {
    pub generated_at: String,
    pub host_id: String,
    pub energy_mode: Option<EnergyMode>,
    /// Map `op id` → ordered list of backend ids best-first.
    pub ranking: HashMap<String, Vec<String>>,
    pub results: Vec<BenchResult>,
}

impl BenchProfile {
    pub fn empty(host_id: impl Into<String>) -> Self {
        Self {
            generated_at: Utc::now().to_rfc3339(),
            host_id: host_id.into(),
            energy_mode: Some(EnergyMode::Performance),
            ranking: HashMap::new(),
            results: Vec::new(),
        }
    }

    /// Run a synthetic mesh through every backend in `backends` for the supplied op set.
    /// Returns the populated profile.
    pub fn run_with_backends(
        host_id: impl Into<String>,
        energy_mode: EnergyMode,
        backends: &[Box<dyn ComputeBackend>],
        ops: &[ComputeOp],
    ) -> Self {
        let mut profile = Self::empty(host_id);
        profile.energy_mode = Some(energy_mode);
        let synthetic = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));

        for op in ops {
            let mut entries: Vec<(ComputeKind, u64)> = Vec::new();
            for backend in backends {
                if !backend.is_runnable() {
                    continue;
                }
                let kind = backend.capabilities().kind;
                if matches!(energy_mode, EnergyMode::LowPower) && !backend.capabilities().low_power_friendly {
                    continue;
                }
                let outcome = match op {
                    ComputeOp::PerVertexDistance => backend
                        .per_vertex_distance(&synthetic, &synthetic)
                        .map(|(_, stats)| stats),
                    ComputeOp::LaplacianSmooth => {
                        let mut clone = synthetic.clone();
                        backend.laplacian_smooth(&mut clone, 1, 0.5)
                    }
                    other => {
                        profile.results.push(BenchResult {
                            backend: kind,
                            op: *other,
                            elapsed_ms: 0,
                            error: Some("op not benched in default suite".into()),
                        });
                        continue;
                    }
                };
                match outcome {
                    Ok(stats) => {
                        profile.results.push(BenchResult {
                            backend: kind,
                            op: *op,
                            elapsed_ms: stats.elapsed_ms,
                            error: None,
                        });
                        entries.push((kind, stats.elapsed_ms));
                    }
                    Err(err) => {
                        profile.results.push(BenchResult {
                            backend: kind,
                            op: *op,
                            elapsed_ms: 0,
                            error: Some(format!("{err}")),
                        });
                    }
                }
            }
            entries.sort_by_key(|(_, t)| *t);
            let ids: Vec<String> = entries.into_iter().map(|(k, _)| k.id().to_string()).collect();
            profile.ranking.insert(op_id(*op).to_string(), ids);
        }
        profile.generated_at = Utc::now().to_rfc3339();
        profile
    }

    /// Convenience: bench using only the always-available CPU backend.
    pub fn run_cpu_only(host_id: impl Into<String>) -> Self {
        let backends: Vec<Box<dyn ComputeBackend>> = vec![Box::new(CpuBackend::new())];
        Self::run_with_backends(
            host_id,
            EnergyMode::Performance,
            &backends,
            &[ComputeOp::PerVertexDistance, ComputeOp::LaplacianSmooth],
        )
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

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        serde_json::from_str(&raw).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("parse: {e}"))
        })
    }

    /// Best backend id for `op`, if a ranking exists.
    pub fn best_backend_id(&self, op: ComputeOp) -> Option<&str> {
        self.ranking
            .get(op_id(op))
            .and_then(|v| v.first().map(|s| s.as_str()))
    }
}

pub fn op_id(op: ComputeOp) -> &'static str {
    match op {
        ComputeOp::PerVertexDistance => "per-vertex-distance",
        ComputeOp::RegionGrow => "region-grow",
        ComputeOp::LaplacianSmooth => "laplacian-smooth",
        ComputeOp::MarchingCubes => "marching-cubes",
        ComputeOp::VoxelRegionGrow => "voxel-region-grow",
        ComputeOp::MeshDecimate => "mesh-decimate",
        ComputeOp::OnnxInference => "onnx-inference",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_cpu_only_produces_ranking() {
        let profile = BenchProfile::run_cpu_only("test-host");
        assert!(!profile.ranking.is_empty());
        let best = profile.best_backend_id(ComputeOp::PerVertexDistance).unwrap();
        assert_eq!(best, ComputeKind::Cpu.id());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("tlanticad-compute-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("compute-profile.json");
        let profile = BenchProfile::run_cpu_only("ci-host");
        profile.save_to(&path).unwrap();
        let loaded = BenchProfile::load(&path).unwrap();
        assert_eq!(loaded.host_id, "ci-host");
        assert!(!loaded.results.is_empty());
    }

    #[test]
    fn low_power_mode_keeps_cpu() {
        let backends: Vec<Box<dyn ComputeBackend>> = vec![Box::new(CpuBackend::new())];
        let profile = BenchProfile::run_with_backends(
            "lpm-host",
            EnergyMode::LowPower,
            &backends,
            &[ComputeOp::PerVertexDistance],
        );
        let best = profile.best_backend_id(ComputeOp::PerVertexDistance).unwrap();
        assert_eq!(best, ComputeKind::Cpu.id());
    }
}
