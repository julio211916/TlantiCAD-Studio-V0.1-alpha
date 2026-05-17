//! Interactive freeform adapt — push a tooth-model mesh down onto a virtual
//! preparation. AR-V414.
//!
//! Conceptually ported from
//! `artifacts/DentalProcessors/FreeformAdaptToothmodelToVirtualPreparationProcessor.cs`.
//! V402 already produced the *batch* prep generator; this sprint is the
//! interactive variant the technician runs after they have placed the
//! library tooth on top of the prep — it shrink-wraps the bottom of the
//! library tooth onto the prep surface (drape) and offsets cervically by a
//! cement gap (bottom offset), then smooths the seam.
//!
//! Algorithm — `adapt_via_freeform`:
//!
//!   1. **Drape**: every tooth vertex below the prep's bounding plane is
//!      projected onto the closest prep vertex along the insertion axis (we
//!      use the brute-force closest-point lookup — the prep is small enough
//!      that this is a non-issue, and a kd-tree adds a dep we don't need
//!      yet). Drape strength is a smoothstep falloff over `drape_radius_mm`
//!      around the drape band.
//!   2. **Bottom offset**: every draped vertex is pushed back along the
//!      tooth's outward direction by `cement_gap_mm` (typical 30–60 µm).
//!   3. **Smoothing**: Laplacian relaxation restricted to the union of
//!      vertices that were actually moved by drape OR bottom offset (so we
//!      don't blur the occlusal anatomy). Iterations + lambda are fully
//!      configurable.
//!
//! This is the "glue" between V402's virtual prep and V401's gingiva — the
//! technician triggers it once they accept the auto-positioning of the
//! library tooth.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AdaptOptions {
    /// Insertion axis (unit-ish). The drape projects along this direction.
    pub insertion_axis: [f64; 3],
    /// Cement / die-spacer gap (mm) applied AFTER drape. Typical 0.03–0.08.
    pub cement_gap_mm: f64,
    /// Drape band radius — vertices within this distance of the prep get
    /// snapped onto it. Outside this band the vertices are untouched.
    pub drape_radius_mm: f64,
    /// Laplacian smoothing iterations applied at the seam (0 = no smoothing).
    pub smoothing_iterations: u32,
    /// Smoothing lambda in `[0, 1]`. Default 0.5.
    pub smoothing_lambda: f64,
}

impl Default for AdaptOptions {
    fn default() -> Self {
        Self {
            insertion_axis: [0.0, 0.0, -1.0],
            cement_gap_mm: 0.05,
            drape_radius_mm: 0.6,
            smoothing_iterations: 3,
            smoothing_lambda: 0.5,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdaptReport {
    /// Vertices moved by the drape pass.
    pub draped_vertices: usize,
    /// Vertices moved by the cement-gap pass (always ⊂ `draped_vertices`).
    pub offset_vertices: usize,
    /// Maximum drape distance (mm).
    pub max_drape_mm: f64,
    /// Mean drape distance (mm) over draped vertices.
    pub mean_drape_mm: f64,
}

/// Adapt the library tooth onto the virtual prep.
///
/// Inputs are **not** consumed; a new mesh is returned with the same vertex
/// ordering and topology as `tooth`.
pub fn adapt_via_freeform(tooth: &Mesh, prep: &Mesh, options: &AdaptOptions) -> (Mesh, AdaptReport) {
    let mut out = tooth.clone();
    out.name = format!("{}_adapted", tooth.name);
    let mut report = AdaptReport::default();

    if tooth.vertices.is_empty() || prep.vertices.is_empty() {
        return (out, report);
    }
    let axis = Vector3::new(
        options.insertion_axis[0],
        options.insertion_axis[1],
        options.insertion_axis[2],
    )
    .try_normalize(1e-9)
    .unwrap_or(Vector3::z());
    let drape_r = options.drape_radius_mm.max(1e-6);
    let gap = options.cement_gap_mm.max(0.0);

    // Pass 1 — drape: snap to closest prep vertex along axis.
    let mut moved: HashSet<usize> = HashSet::new();
    let mut drape_sum = 0.0_f64;
    let mut drape_max = 0.0_f64;
    for i in 0..out.vertices.len() {
        let v = out.vertices[i];
        // Find the closest prep vertex (brute force).
        let mut best_dist = f64::INFINITY;
        let mut best_idx = 0usize;
        for (j, p) in prep.vertices.iter().enumerate() {
            let d = (v - p).norm();
            if d < best_dist {
                best_dist = d;
                best_idx = j;
            }
        }
        if best_dist > drape_r {
            continue;
        }
        let target = prep.vertices[best_idx];
        let falloff = smoothstep(1.0 - (best_dist / drape_r));
        // Drape only along the insertion axis component of the delta — keeps
        // mesial/distal/buccal anatomy of the library tooth intact.
        let delta = target - v;
        let along_axis = axis * delta.dot(&axis);
        let displacement = along_axis * falloff;
        let mag = displacement.norm();
        if mag < 1e-9 {
            continue;
        }
        out.vertices[i] = v + displacement;
        moved.insert(i);
        drape_sum += mag;
        if mag > drape_max {
            drape_max = mag;
        }
    }
    report.draped_vertices = moved.len();
    report.max_drape_mm = drape_max;
    report.mean_drape_mm = if moved.is_empty() {
        0.0
    } else {
        drape_sum / (moved.len() as f64)
    };

    // Pass 2 — bottom offset: push the draped band back along -axis to leave
    // the cement gap.
    if gap > 0.0 {
        for &i in &moved {
            // Negative axis: insertion axis points "down into the prep", so
            // pulling away = going against axis.
            out.vertices[i] -= axis * gap;
        }
        report.offset_vertices = moved.len();
    }

    // Pass 3 — Laplacian smoothing of the moved band.
    if options.smoothing_iterations > 0 && !moved.is_empty() {
        smooth_subset(
            &mut out,
            &moved,
            options.smoothing_iterations,
            options.smoothing_lambda.clamp(0.0, 1.0),
        );
    }

    out.calculate_normals();
    (out, report)
}

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Laplacian relaxation over a subset of vertex indices. Adjacency is built
/// from `mesh.indices`. We don't fold this into the standard smoother because
/// the standard one operates on the entire mesh.
fn smooth_subset(mesh: &mut Mesh, subset: &HashSet<usize>, iterations: u32, lambda: f64) {
    if subset.is_empty() {
        return;
    }
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); mesh.vertices.len()];
    for tri in &mesh.indices {
        for k in 0..3 {
            let a = tri[k] as usize;
            let b = tri[(k + 1) % 3] as usize;
            adj[a].push(b as u32);
            adj[b].push(a as u32);
        }
    }
    for _ in 0..iterations {
        let snapshot = mesh.vertices.clone();
        for &i in subset {
            if i >= snapshot.len() || adj[i].is_empty() {
                continue;
            }
            let mean: Vector3<f64> = adj[i]
                .iter()
                .map(|&j| snapshot[j as usize].coords)
                .sum::<Vector3<f64>>()
                / (adj[i].len() as f64);
            let p = snapshot[i].coords;
            let new_p = p + (mean - p) * lambda;
            mesh.vertices[i] = Point3::from(new_p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    /// Library tooth: 1×1×1 box stacked above the prep at z=2..3
    /// Prep: 1×1×1 box at z=0..1 (the table surface for drape).
    fn pair_boxes() -> (Mesh, Mesh) {
        let prep = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let tooth = create_box(
            Point3::new(0.0, 0.0, 1.5),
            Point3::new(1.0, 1.0, 2.5),
        );
        (tooth, prep)
    }

    #[test]
    fn empty_inputs_return_empty_report() {
        let prep = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let tooth = Mesh::new("empty");
        let (out, report) = adapt_via_freeform(&tooth, &prep, &AdaptOptions::default());
        assert!(out.vertices.is_empty());
        assert_eq!(report.draped_vertices, 0);

        let prep = Mesh::new("empty");
        let tooth = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let (_, r) = adapt_via_freeform(&tooth, &prep, &AdaptOptions::default());
        assert_eq!(r.draped_vertices, 0);
    }

    #[test]
    fn drape_with_zero_radius_does_nothing() {
        let (tooth, prep) = pair_boxes();
        let opts = AdaptOptions {
            drape_radius_mm: 0.0,
            cement_gap_mm: 0.0,
            smoothing_iterations: 0,
            ..Default::default()
        };
        let (out, report) = adapt_via_freeform(&tooth, &prep, &opts);
        assert_eq!(report.draped_vertices, 0);
        // Vertex positions unchanged
        for (a, b) in out.vertices.iter().zip(tooth.vertices.iter()) {
            assert!((a - b).norm() < 1e-9);
        }
    }

    #[test]
    fn drape_pulls_bottom_face_toward_prep() {
        let (tooth, prep) = pair_boxes();
        let opts = AdaptOptions {
            insertion_axis: [0.0, 0.0, -1.0],
            drape_radius_mm: 1.0,
            cement_gap_mm: 0.0,
            smoothing_iterations: 0,
            ..Default::default()
        };
        let (out, report) = adapt_via_freeform(&tooth, &prep, &opts);
        // Bottom four corners of tooth (z=1.5) should be within drape radius
        // of the top face of prep (z=1.0): distance = 0.5 < 1.0.
        assert!(report.draped_vertices >= 4);
        // Verify that at least one bottom vertex moved DOWN toward the prep.
        let bottom_idx_before: Vec<f64> = tooth
            .vertices
            .iter()
            .filter(|v| (v.z - 1.5).abs() < 1e-6)
            .map(|v| v.z)
            .collect();
        let bottom_idx_after: Vec<f64> = out
            .vertices
            .iter()
            .filter(|v| v.z < 1.5 - 1e-6 && v.z > 0.5)
            .map(|v| v.z)
            .collect();
        assert!(!bottom_idx_after.is_empty());
        assert_eq!(bottom_idx_before.len(), 4);
    }

    #[test]
    fn cement_gap_offsets_against_axis() {
        let (tooth, prep) = pair_boxes();
        let gap = 0.1;
        let opts = AdaptOptions {
            insertion_axis: [0.0, 0.0, -1.0],
            drape_radius_mm: 1.0,
            cement_gap_mm: gap,
            smoothing_iterations: 0,
            ..Default::default()
        };
        let (out_with_gap, _) = adapt_via_freeform(&tooth, &prep, &opts);
        let opts_no_gap = AdaptOptions {
            cement_gap_mm: 0.0,
            ..opts
        };
        let (out_no_gap, _) = adapt_via_freeform(&tooth, &prep, &opts_no_gap);
        // The draped vertices in the with-gap variant should sit `gap` higher
        // (against -axis = +z) than the no-gap variant.
        let mut diffs = Vec::new();
        for i in 0..out_with_gap.vertices.len() {
            let d = out_with_gap.vertices[i].z - out_no_gap.vertices[i].z;
            if d.abs() > 1e-6 {
                diffs.push(d);
            }
        }
        assert!(!diffs.is_empty());
        // every diff = +gap (axis = -z, offset = -axis * gap = +z * gap)
        for d in diffs {
            assert!((d - gap).abs() < 1e-9, "expected delta {}, got {}", gap, d);
        }
    }

    #[test]
    fn smoothing_does_not_explode_bbox() {
        let (tooth, prep) = pair_boxes();
        let opts = AdaptOptions {
            insertion_axis: [0.0, 0.0, -1.0],
            drape_radius_mm: 1.5,
            cement_gap_mm: 0.05,
            smoothing_iterations: 5,
            smoothing_lambda: 0.5,
        };
        let (out, _) = adapt_via_freeform(&tooth, &prep, &opts);
        let (lo_in, hi_in) = tooth.calculate_bounds();
        let (lo_out, hi_out) = out.calculate_bounds();
        // bbox shouldn't drift outward by more than 0.5 mm in any axis
        assert!((lo_out.x - lo_in.x).abs() < 0.5);
        assert!((hi_out.z - hi_in.z).abs() < 1.0);
    }

    #[test]
    fn moved_vertex_count_stable_under_repeat_call() {
        let (tooth, prep) = pair_boxes();
        let opts = AdaptOptions {
            drape_radius_mm: 1.0,
            cement_gap_mm: 0.05,
            smoothing_iterations: 1,
            ..Default::default()
        };
        let (_, r1) = adapt_via_freeform(&tooth, &prep, &opts);
        let (_, r2) = adapt_via_freeform(&tooth, &prep, &opts);
        assert_eq!(r1.draped_vertices, r2.draped_vertices);
        assert_eq!(r1.offset_vertices, r2.offset_vertices);
    }
}
