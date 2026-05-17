//! ComputeRouter — picks the right backend per op + falls back gracefully on errors.
//! MP-110.

use crate::backend::{ComputeBackend, ComputeError, ComputeKind, ComputeOp, ComputeStats};
use crate::cpu::CpuBackend;
use crate::profile::{BenchProfile, EnergyMode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tlanticad_mesh::Mesh;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RouterError {
    #[error("no backend available for op {0:?}")]
    NoBackendForOp(ComputeOp),
    #[error("backend error: {message}")]
    Backend { message: String },
}

impl From<ComputeError> for RouterError {
    fn from(err: ComputeError) -> Self {
        RouterError::Backend {
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterDecision {
    pub op: ComputeOp,
    pub picked: ComputeKind,
    pub fell_back_from: Option<ComputeKind>,
}

/// Owns the backend pool + bench profile. Cheap to clone (Arc inside).
#[derive(Clone)]
pub struct ComputeRouter {
    pool: Arc<Vec<Box<dyn ComputeBackend>>>,
    profile: Arc<BenchProfile>,
    energy_mode: EnergyMode,
}

impl ComputeRouter {
    /// Build the canonical backend pool for this binary. Tauri and other adapters must
    /// call this instead of rebuilding their own CPU/GPU list.
    pub fn available_backend_pool() -> Vec<Box<dyn ComputeBackend>> {
        let mut pool: Vec<Box<dyn ComputeBackend>> = Vec::new();

        #[cfg(feature = "gpu-wgpu")]
        if let Some(backend) = crate::wgpu_backend::WgpuBackend::try_new() {
            pool.push(Box::new(backend));
        }

        pool.push(Box::new(CpuBackend::new()));
        pool
    }

    /// Build a router with a single CPU backend — guaranteed to work in any environment.
    pub fn cpu_only() -> Self {
        let pool: Vec<Box<dyn ComputeBackend>> = vec![Box::new(CpuBackend::new())];
        let profile = BenchProfile::run_cpu_only("default-host");
        Self {
            pool: Arc::new(pool),
            profile: Arc::new(profile),
            energy_mode: EnergyMode::Performance,
        }
    }

    /// Build a router from an explicit pool + pre-baked profile (used after the bench step
    /// at app startup so we don't re-bench every invocation).
    pub fn new(
        pool: Vec<Box<dyn ComputeBackend>>,
        profile: BenchProfile,
        energy_mode: EnergyMode,
    ) -> Self {
        Self {
            pool: Arc::new(pool),
            profile: Arc::new(profile),
            energy_mode,
        }
    }

    /// Build a router from the canonical backend pool and benchmark it for the hot ops.
    pub fn with_available_backends(host_id: impl Into<String>, energy_mode: EnergyMode) -> Self {
        let pool = Self::available_backend_pool();
        let profile = BenchProfile::run_with_backends(
            host_id,
            energy_mode,
            &pool,
            &[ComputeOp::PerVertexDistance, ComputeOp::LaplacianSmooth],
        );
        Self::new(pool, profile, energy_mode)
    }

    pub fn energy_mode(&self) -> EnergyMode {
        self.energy_mode
    }

    /// Backends in the pool that pass the energy-mode filter and `is_runnable`.
    pub fn runnable(&self) -> Vec<&dyn ComputeBackend> {
        self.pool
            .iter()
            .map(|b| b.as_ref())
            .filter(|b| b.is_runnable())
            .filter(|b| {
                matches!(self.energy_mode, EnergyMode::Performance)
                    || b.capabilities().low_power_friendly
            })
            .collect()
    }

    /// Find the backend the profile ranks first for `op`. Falls back to the first runnable
    /// backend if there is no ranking entry.
    fn pick_for(&self, op: ComputeOp) -> Option<&dyn ComputeBackend> {
        let runnable = self.runnable();
        if runnable.is_empty() {
            return None;
        }
        if let Some(best) = self.profile.best_backend_id(op) {
            for b in &runnable {
                if b.capabilities().kind.id() == best {
                    return Some(*b);
                }
            }
        }
        Some(*runnable.first()?)
    }

    /// Public decision surface for adapters that only need to display or persist the pick.
    pub fn picked_kind_for(&self, op: ComputeOp) -> Option<ComputeKind> {
        self.pick_for(op).map(|backend| backend.capabilities().kind)
    }

    /// Dispatch `per_vertex_distance` through the picked backend; on error falls back to CPU
    /// and records the fall-back.
    pub fn per_vertex_distance(
        &self,
        src: &Mesh,
        dst: &Mesh,
    ) -> Result<(Vec<f64>, ComputeStats, RouterDecision), RouterError> {
        let op = ComputeOp::PerVertexDistance;
        let primary = self
            .pick_for(op)
            .ok_or(RouterError::NoBackendForOp(op))?;
        let primary_kind = primary.capabilities().kind;
        match primary.per_vertex_distance(src, dst) {
            Ok((dists, stats)) => Ok((
                dists,
                stats,
                RouterDecision {
                    op,
                    picked: primary_kind,
                    fell_back_from: None,
                },
            )),
            Err(_) if primary_kind != ComputeKind::Cpu => {
                let cpu = CpuBackend::new();
                let (dists, mut stats) = cpu.per_vertex_distance(src, dst)?;
                stats.fell_back_to_cpu = true;
                Ok((
                    dists,
                    stats,
                    RouterDecision {
                        op,
                        picked: ComputeKind::Cpu,
                        fell_back_from: Some(primary_kind),
                    },
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn laplacian_smooth(
        &self,
        mesh: &mut Mesh,
        iterations: u32,
        lambda: f64,
    ) -> Result<(ComputeStats, RouterDecision), RouterError> {
        let op = ComputeOp::LaplacianSmooth;
        let primary = self
            .pick_for(op)
            .ok_or(RouterError::NoBackendForOp(op))?;
        let primary_kind = primary.capabilities().kind;
        match primary.laplacian_smooth(mesh, iterations, lambda) {
            Ok(stats) => Ok((
                stats,
                RouterDecision {
                    op,
                    picked: primary_kind,
                    fell_back_from: None,
                },
            )),
            Err(_) if primary_kind != ComputeKind::Cpu => {
                let cpu = CpuBackend::new();
                let mut stats = cpu.laplacian_smooth(mesh, iterations, lambda)?;
                stats.fell_back_to_cpu = true;
                Ok((
                    stats,
                    RouterDecision {
                        op,
                        picked: ComputeKind::Cpu,
                        fell_back_from: Some(primary_kind),
                    },
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Status-bar display for the currently-picked backend on a given op.
    pub fn status_label(&self, op: ComputeOp) -> String {
        match self.pick_for(op) {
            Some(b) => b.capabilities().kind.display_name().to_string(),
            None => "no backend".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;
    use tlanticad_mesh::create_box;

    #[test]
    fn cpu_only_router_dispatches_distance() {
        let router = ComputeRouter::cpu_only();
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = a.clone();
        let (d, _, decision) = router.per_vertex_distance(&a, &b).unwrap();
        assert_eq!(d.len(), a.vertices.len());
        assert_eq!(decision.picked, ComputeKind::Cpu);
        assert!(decision.fell_back_from.is_none());
    }

    #[test]
    fn cpu_only_router_dispatches_smooth() {
        let router = ComputeRouter::cpu_only();
        let mut mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let (_, decision) = router.laplacian_smooth(&mut mesh, 2, 0.5).unwrap();
        assert_eq!(decision.picked, ComputeKind::Cpu);
    }

    #[test]
    fn status_label_describes_cpu() {
        let router = ComputeRouter::cpu_only();
        let label = router.status_label(ComputeOp::PerVertexDistance);
        assert!(label.contains("CPU"));
    }

    #[test]
    fn canonical_pool_always_contains_cpu() {
        let pool = ComputeRouter::available_backend_pool();
        assert!(pool
            .iter()
            .any(|backend| backend.capabilities().kind == ComputeKind::Cpu));
    }

    #[test]
    fn picked_kind_uses_router_decision() {
        let router = ComputeRouter::cpu_only();
        assert_eq!(
            router.picked_kind_for(ComputeOp::PerVertexDistance),
            Some(ComputeKind::Cpu)
        );
    }
}
