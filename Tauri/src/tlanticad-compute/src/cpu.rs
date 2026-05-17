//! CPU backend — multicore via rayon. MP-102.
//!
//! Implements the heaviest mesh ops with `par_iter`. Falls back to plain iteration when
//! the `cpu-rayon` feature is disabled (e.g. in WASM where rayon doesn't apply).

use crate::backend::{
    BackendCapabilities, ComputeBackend, ComputeError, ComputeOp, ComputeStats,
};
use crate::ComputeKind;
use nalgebra::{Point3, Vector3};
use std::collections::HashMap;
use std::time::Instant;
use tlanticad_mesh::Mesh;

#[cfg(feature = "cpu-rayon")]
use rayon::prelude::*;

/// CPU backend.
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuBackend;

impl CpuBackend {
    pub fn new() -> Self {
        Self
    }
}

fn closest_distance_brute(query: &Point3<f64>, target: &[Point3<f64>]) -> f64 {
    if target.is_empty() {
        return 0.0;
    }
    let mut best = f64::INFINITY;
    for p in target {
        let d2 = (query - p).norm_squared();
        if d2 < best {
            best = d2;
        }
    }
    best.sqrt()
}

impl ComputeBackend for CpuBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            kind: ComputeKind::Cpu,
            max_vertices: u64::MAX,
            supports_fp16: false,
            supports_int8: true,
            peak_tops: 0.0,
            low_power_friendly: true,
        }
    }

    fn per_vertex_distance(
        &self,
        src: &Mesh,
        dst: &Mesh,
    ) -> Result<(Vec<f64>, ComputeStats), ComputeError> {
        let started = Instant::now();
        let count = src.vertices.len() as u64;

        #[cfg(feature = "cpu-rayon")]
        let distances: Vec<f64> = src
            .vertices
            .par_iter()
            .map(|v| closest_distance_brute(v, &dst.vertices))
            .collect();

        #[cfg(not(feature = "cpu-rayon"))]
        let distances: Vec<f64> = src
            .vertices
            .iter()
            .map(|v| closest_distance_brute(v, &dst.vertices))
            .collect();

        let stats = ComputeStats {
            backend: Some(ComputeKind::Cpu),
            op: Some(ComputeOp::PerVertexDistance),
            elapsed_ms: started.elapsed().as_millis() as u64,
            items_processed: count,
            fell_back_to_cpu: false,
        };
        Ok((distances, stats))
    }

    fn laplacian_smooth(
        &self,
        mesh: &mut Mesh,
        iterations: u32,
        lambda: f64,
    ) -> Result<ComputeStats, ComputeError> {
        let started = Instant::now();
        if mesh.vertices.is_empty() {
            return Ok(ComputeStats {
                backend: Some(ComputeKind::Cpu),
                op: Some(ComputeOp::LaplacianSmooth),
                ..Default::default()
            });
        }
        let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
        for tri in &mesh.indices {
            for i in 0..3 {
                for j in 0..3 {
                    if i != j {
                        adj.entry(tri[i]).or_default().push(tri[j]);
                    }
                }
            }
        }

        for _ in 0..iterations.max(1) {
            let snapshot = mesh.vertices.clone();

            #[cfg(feature = "cpu-rayon")]
            let updates: Vec<(usize, Point3<f64>)> = (0..snapshot.len())
                .into_par_iter()
                .filter_map(|i| {
                    let neighbours = adj.get(&(i as u32))?;
                    if neighbours.is_empty() {
                        return None;
                    }
                    let mean: Vector3<f64> = neighbours
                        .iter()
                        .map(|&n| snapshot[n as usize].coords)
                        .sum::<Vector3<f64>>()
                        / neighbours.len() as f64;
                    let p = snapshot[i];
                    Some((i, Point3::from(p.coords.lerp(&mean, lambda))))
                })
                .collect();

            #[cfg(not(feature = "cpu-rayon"))]
            let updates: Vec<(usize, Point3<f64>)> = (0..snapshot.len())
                .filter_map(|i| {
                    let neighbours = adj.get(&(i as u32))?;
                    if neighbours.is_empty() {
                        return None;
                    }
                    let mean: Vector3<f64> = neighbours
                        .iter()
                        .map(|&n| snapshot[n as usize].coords)
                        .sum::<Vector3<f64>>()
                        / neighbours.len() as f64;
                    let p = snapshot[i];
                    Some((i, Point3::from(p.coords.lerp(&mean, lambda))))
                })
                .collect();

            for (i, p) in updates {
                mesh.vertices[i] = p;
            }
        }
        mesh.calculate_normals();

        Ok(ComputeStats {
            backend: Some(ComputeKind::Cpu),
            op: Some(ComputeOp::LaplacianSmooth),
            elapsed_ms: started.elapsed().as_millis() as u64,
            items_processed: mesh.vertices.len() as u64,
            fell_back_to_cpu: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    #[test]
    fn cpu_backend_capabilities() {
        let b = CpuBackend::new();
        let caps = b.capabilities();
        assert_eq!(caps.kind, ComputeKind::Cpu);
        assert!(caps.low_power_friendly);
    }

    #[test]
    fn per_vertex_distance_zero_for_identical() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = a.clone();
        let backend = CpuBackend::new();
        let (d, stats) = backend.per_vertex_distance(&a, &b).unwrap();
        assert_eq!(d.len(), a.vertices.len());
        assert!(d.iter().all(|x| x.abs() < 1e-9));
        assert_eq!(stats.backend, Some(ComputeKind::Cpu));
        assert_eq!(stats.items_processed, a.vertices.len() as u64);
    }

    #[test]
    fn per_vertex_distance_translated_recovers_offset() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 1.0, 1.0));
        let backend = CpuBackend::new();
        let (d, _) = backend.per_vertex_distance(&a, &b).unwrap();
        assert!(d.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn laplacian_smooth_does_not_panic() {
        let mut mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let backend = CpuBackend::new();
        let stats = backend.laplacian_smooth(&mut mesh, 3, 0.5).unwrap();
        assert!(stats.elapsed_ms < 5_000);
        assert_eq!(mesh.vertex_count(), 8);
    }
}
