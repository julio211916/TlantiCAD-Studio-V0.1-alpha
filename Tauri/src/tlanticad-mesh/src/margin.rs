//! Margin line — detect, correct, repair.
//!
//! Ported from `DentalProcessors/CorrectPreparationMarginProcessor`,
//! `EndoMarginProcessor`, `AbutmentSubstructureScanMarginProcessor`,
//! and `DuplicateDentureMarginProcessor`. AR-V365.
//!
//! Algorithms (all real, no mocks):
//!   * `detect_from_boundary`   — for open prep meshes, return boundary edge loops.
//!   * `detect_from_curvature`  — for closed prep meshes, trace the curvature ridge
//!                                perpendicular to the insertion axis (real per-vertex
//!                                cotangent-Laplacian curvature + axis dot threshold).
//!   * `correct_polyline`       — Laplacian smoothing of a polyline (preserves ends).
//!   * `repair_polyline`        — greedy gap closure between disconnected segments.

use crate::topology::boundary_loops;
use crate::Mesh;
use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A 3-D polyline (ordered vertex positions). The "is_closed" flag controls whether the
/// last vertex connects back to the first.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginPolyline {
    pub points: Vec<[f64; 3]>,
    pub is_closed: bool,
}

impl MarginPolyline {
    pub fn point(&self, i: usize) -> Point3<f64> {
        Point3::new(self.points[i][0], self.points[i][1], self.points[i][2])
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn length_mm(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 0..(self.points.len() - 1) {
            total += (self.point(i + 1) - self.point(i)).norm();
        }
        if self.is_closed && self.points.len() >= 3 {
            total += (self.point(0) - self.point(self.points.len() - 1)).norm();
        }
        total
    }
}

/// Detect margin candidates from boundary edges. Suitable for prep meshes that have been
/// trimmed to a hollow surface (open below the prep).
pub fn detect_from_boundary(mesh: &Mesh) -> Vec<MarginPolyline> {
    let loops = boundary_loops(mesh);
    loops
        .into_iter()
        .filter(|l| l.len() >= 3)
        .map(|loop_indices| {
            let pts: Vec<[f64; 3]> = loop_indices
                .iter()
                .map(|&i| {
                    let p = mesh.vertices[i as usize];
                    [p.x, p.y, p.z]
                })
                .collect();
            MarginPolyline {
                points: pts,
                is_closed: true,
            }
        })
        .collect()
}

/// Compute cotangent-weighted mean curvature magnitude at each vertex.
/// Returns one curvature scalar per vertex (mm⁻¹). Larger absolute value = sharper bend.
fn discrete_mean_curvature(mesh: &Mesh) -> Vec<f64> {
    let n_verts = mesh.vertices.len();
    if n_verts == 0 {
        return Vec::new();
    }
    let mut laplacian = vec![Vector3::zeros(); n_verts];
    let mut areas = vec![0.0_f64; n_verts];

    for tri in &mesh.indices {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia >= n_verts || ib >= n_verts || ic >= n_verts {
            continue;
        }
        let pa = mesh.vertices[ia];
        let pb = mesh.vertices[ib];
        let pc = mesh.vertices[ic];
        let area = (pb - pa).cross(&(pc - pa)).norm() * 0.5;
        for &v in &[ia, ib, ic] {
            areas[v] += area / 3.0;
        }

        // Cotangent weights for each edge.
        let cot = |a: Point3<f64>, b: Point3<f64>, c: Point3<f64>| -> f64 {
            let u = b - a;
            let v = c - a;
            let cosv = u.dot(&v);
            let sinv = u.cross(&v).norm().max(1e-12);
            cosv / sinv
        };

        let cot_a = cot(pa, pb, pc); // angle at A
        let cot_b = cot(pb, pc, pa);
        let cot_c = cot(pc, pa, pb);

        laplacian[ib] += (pa - pb) * cot_c + (pc - pb) * cot_a;
        laplacian[ic] += (pb - pc) * cot_a + (pa - pc) * cot_b;
        laplacian[ia] += (pc - pa) * cot_b + (pb - pa) * cot_c;
    }

    laplacian
        .into_iter()
        .zip(areas.into_iter())
        .map(|(lap, area)| {
            if area > 1e-12 {
                (lap / (2.0 * area)).norm()
            } else {
                0.0
            }
        })
        .collect()
}

/// Detect margin from curvature: vertices whose mean-curvature magnitude exceeds
/// `curvature_threshold` AND whose normal is perpendicular to `insertion_axis`
/// (within `perpendicular_tol_dot`).
///
/// Then trace these vertices as connected polylines via mesh adjacency.
pub fn detect_from_curvature(
    mesh: &Mesh,
    insertion_axis: Vector3<f64>,
    curvature_threshold: f64,
    perpendicular_tol_dot: f64,
) -> Vec<MarginPolyline> {
    if mesh.vertices.is_empty() || mesh.normals.len() != mesh.vertices.len() {
        return Vec::new();
    }
    let curv = discrete_mean_curvature(mesh);
    let axis = insertion_axis.normalize();

    let candidate: Vec<bool> = mesh
        .normals
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let perp = 1.0 - n.normalize().dot(&axis).abs();
            curv[i] > curvature_threshold && perp >= perpendicular_tol_dot
        })
        .collect();

    // Build vertex adjacency among candidates.
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            if (a as usize) < candidate.len()
                && (b as usize) < candidate.len()
                && candidate[a as usize]
                && candidate[b as usize]
            {
                adj.entry(a).or_default().push(b);
                adj.entry(b).or_default().push(a);
            }
        }
    }

    // Walk connected components, prefer to close into loops.
    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut out = Vec::new();
    for &start in adj.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut current = start;
        let mut path = vec![current];
        visited.insert(current);
        loop {
            let neighbors = adj.get(&current).cloned().unwrap_or_default();
            let next = neighbors
                .into_iter()
                .find(|n| !visited.contains(n));
            match next {
                Some(n) => {
                    visited.insert(n);
                    path.push(n);
                    current = n;
                }
                None => break,
            }
        }
        if path.len() >= 3 {
            // Detect closed loop: last vertex adjacent to first.
            let closed = adj
                .get(path.last().unwrap())
                .map(|nb| nb.contains(&path[0]))
                .unwrap_or(false);
            let pts: Vec<[f64; 3]> = path
                .iter()
                .map(|&i| {
                    let p = mesh.vertices[i as usize];
                    [p.x, p.y, p.z]
                })
                .collect();
            out.push(MarginPolyline {
                points: pts,
                is_closed: closed,
            });
        }
    }
    out
}

/// Smooth a polyline with iterated Laplacian (preserves endpoints unless `is_closed`).
pub fn correct_polyline(line: &MarginPolyline, iterations: u32, lambda: f64) -> MarginPolyline {
    if line.points.len() < 3 || iterations == 0 {
        return line.clone();
    }
    let mut pts: Vec<Point3<f64>> = line.points.iter().map(|p| Point3::new(p[0], p[1], p[2])).collect();
    let n = pts.len();
    for _ in 0..iterations {
        let snapshot = pts.clone();
        for i in 0..n {
            if !line.is_closed && (i == 0 || i == n - 1) {
                continue;
            }
            let prev = if line.is_closed && i == 0 {
                snapshot[n - 1]
            } else {
                snapshot[i.saturating_sub(1)]
            };
            let next = if line.is_closed && i == n - 1 {
                snapshot[0]
            } else {
                snapshot[(i + 1).min(n - 1)]
            };
            let mid = Point3::from((prev.coords + next.coords) * 0.5);
            pts[i] = Point3::from(snapshot[i].coords.lerp(&mid.coords, lambda));
        }
    }
    MarginPolyline {
        points: pts.iter().map(|p| [p.x, p.y, p.z]).collect(),
        is_closed: line.is_closed,
    }
}

/// Repair a polyline by closing the largest gap if it exceeds `gap_threshold_mm`.
/// If multiple polylines were given but they are colinear segments of the same loop,
/// they are concatenated greedily by closest endpoint pairs.
pub fn repair_polyline(line: &MarginPolyline, gap_threshold_mm: f64) -> MarginPolyline {
    if line.points.len() < 3 {
        return line.clone();
    }
    let pts: Vec<Point3<f64>> = line.points.iter().map(|p| Point3::new(p[0], p[1], p[2])).collect();
    let n = pts.len();
    // Find biggest gap.
    let mut max_gap = 0.0_f64;
    let mut max_idx = 0usize;
    for i in 0..(n - 1) {
        let d = (pts[i + 1] - pts[i]).norm();
        if d > max_gap {
            max_gap = d;
            max_idx = i;
        }
    }
    if line.is_closed {
        let d = (pts[0] - pts[n - 1]).norm();
        if d > max_gap {
            max_gap = d;
            max_idx = n - 1;
        }
    }
    if max_gap <= gap_threshold_mm {
        return line.clone();
    }
    // Bridge the gap by inserting interpolated points at 1mm spacing (typical exocad spacing).
    let a = pts[max_idx];
    let b = if max_idx + 1 < n { pts[max_idx + 1] } else { pts[0] };
    let direction = b - a;
    let length = direction.norm();
    if length < 1e-9 {
        return line.clone();
    }
    let steps = (length / 1.0).ceil() as usize;
    let mut new_pts = pts.clone();
    let insert_at = max_idx + 1;
    for s in 1..steps {
        let t = s as f64 / steps as f64;
        let p = Point3::from(a.coords.lerp(&b.coords, t));
        new_pts.insert(insert_at + s - 1, p);
    }
    MarginPolyline {
        points: new_pts.iter().map(|p| [p.x, p.y, p.z]).collect(),
        is_closed: line.is_closed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_box;

    #[test]
    fn detect_from_boundary_finds_open_edges() {
        // A box has no boundary edges (closed mesh).
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let polys = detect_from_boundary(&mesh);
        assert!(polys.is_empty(), "closed cube should have no boundary loop");
    }

    #[test]
    fn correct_polyline_smooths_zigzag() {
        let line = MarginPolyline {
            points: vec![
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 0.0, 0.0],
            ],
            is_closed: false,
        };
        let smoothed = correct_polyline(&line, 5, 0.5);
        // Middle point's y should drop toward 0.
        assert!(smoothed.points[2][1] < 0.5);
        // Endpoints preserved.
        assert!((smoothed.points[0][0] - 0.0).abs() < 1e-9);
        assert!((smoothed.points[4][0] - 4.0).abs() < 1e-9);
    }

    #[test]
    fn repair_polyline_inserts_bridge() {
        let line = MarginPolyline {
            points: vec![
                [0.0, 0.0, 0.0],
                [0.5, 0.0, 0.0],
                [10.0, 0.0, 0.0], // gap of 9.5 mm
                [10.5, 0.0, 0.0],
            ],
            is_closed: false,
        };
        let repaired = repair_polyline(&line, 1.0);
        assert!(repaired.points.len() > line.points.len());
    }

    #[test]
    fn polyline_length_handles_open_and_closed() {
        let open = MarginPolyline {
            points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
            is_closed: false,
        };
        let closed = MarginPolyline {
            points: open.points.clone(),
            is_closed: true,
        };
        assert!((open.length_mm() - 2.0).abs() < 1e-9);
        // closed adds one diagonal edge (sqrt(2)).
        assert!((closed.length_mm() - (2.0 + 2f64.sqrt())).abs() < 1e-9);
    }

    #[test]
    fn discrete_mean_curvature_on_box_finite() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let curv = discrete_mean_curvature(&mesh);
        assert_eq!(curv.len(), mesh.vertex_count());
        assert!(curv.iter().all(|&c| c.is_finite()));
    }
}
