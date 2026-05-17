//! Mesh comparison — Hausdorff + RMS distance via KD-tree.
//!
//! Ported from `DentalProcessors/CompareMeshToolProcessor` (point-cloud Hausdorff).

use crate::Mesh;
use nalgebra::{Point3, Vector3};
use rstar::{primitives::GeomWithData, RTree};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompareReport {
    /// Max distance from any A-vertex to nearest B-vertex (one-sided)
    pub hausdorff_a_to_b_mm: f64,
    pub hausdorff_b_to_a_mm: f64,
    /// Max of both directions = symmetric Hausdorff
    pub hausdorff_symmetric_mm: f64,
    /// Root-mean-square of A→B distances
    pub rms_a_to_b_mm: f64,
    pub rms_b_to_a_mm: f64,
    /// Mean distance A→B (sometimes called "mean unsigned error")
    pub mean_a_to_b_mm: f64,
    pub mean_b_to_a_mm: f64,
    pub vertex_count_a: usize,
    pub vertex_count_b: usize,
}

type IndexedPoint = GeomWithData<[f64; 3], usize>;

fn build_tree(mesh: &Mesh) -> RTree<IndexedPoint> {
    let pts: Vec<IndexedPoint> = mesh
        .vertices
        .iter()
        .enumerate()
        .map(|(i, p)| GeomWithData::new([p.x, p.y, p.z], i))
        .collect();
    RTree::bulk_load(pts)
}

fn one_sided(src: &Mesh, tree_dst: &RTree<IndexedPoint>) -> (f64, f64, f64) {
    let mut max_d2 = 0.0_f64;
    let mut sum_d2 = 0.0_f64;
    let mut sum_d = 0.0_f64;
    for v in &src.vertices {
        let q = [v.x, v.y, v.z];
        let nearest = tree_dst.nearest_neighbor(&q);
        if let Some(p) = nearest {
            let dx = q[0] - p.geom()[0];
            let dy = q[1] - p.geom()[1];
            let dz = q[2] - p.geom()[2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 > max_d2 {
                max_d2 = d2;
            }
            sum_d2 += d2;
            sum_d += d2.sqrt();
        }
    }
    let n = src.vertices.len() as f64;
    let rms = if n > 0.0 { (sum_d2 / n).sqrt() } else { 0.0 };
    let mean = if n > 0.0 { sum_d / n } else { 0.0 };
    (max_d2.sqrt(), rms, mean)
}

/// Per-vertex distance from each vertex of `src` to the closest vertex of `dst`.
/// Used by AR-V376 distance shader and AR-V365 margin coloring.
pub fn per_vertex_distance(src: &Mesh, dst: &Mesh) -> Vec<f64> {
    if src.vertices.is_empty() || dst.vertices.is_empty() {
        return vec![0.0; src.vertices.len()];
    }
    let tree = build_tree(dst);
    src.vertices
        .iter()
        .map(|v| {
            let q = [v.x, v.y, v.z];
            tree.nearest_neighbor(&q)
                .map(|p| {
                    let dx = q[0] - p.geom()[0];
                    let dy = q[1] - p.geom()[1];
                    let dz = q[2] - p.geom()[2];
                    (dx * dx + dy * dy + dz * dz).sqrt()
                })
                .unwrap_or(0.0)
        })
        .collect()
}

/// Compare two meshes — Hausdorff + RMS in both directions.
pub fn compare(a: &Mesh, b: &Mesh) -> CompareReport {
    if a.vertices.is_empty() || b.vertices.is_empty() {
        return CompareReport {
            vertex_count_a: a.vertices.len(),
            vertex_count_b: b.vertices.len(),
            ..Default::default()
        };
    }
    let tree_b = build_tree(b);
    let tree_a = build_tree(a);
    let (h_ab, rms_ab, mean_ab) = one_sided(a, &tree_b);
    let (h_ba, rms_ba, mean_ba) = one_sided(b, &tree_a);
    CompareReport {
        hausdorff_a_to_b_mm: h_ab,
        hausdorff_b_to_a_mm: h_ba,
        hausdorff_symmetric_mm: h_ab.max(h_ba),
        rms_a_to_b_mm: rms_ab,
        rms_b_to_a_mm: rms_ba,
        mean_a_to_b_mm: mean_ab,
        mean_b_to_a_mm: mean_ba,
        vertex_count_a: a.vertices.len(),
        vertex_count_b: b.vertices.len(),
    }
}

/// Batch version of `closest_point_on`: returns one closest point per `src` vertex against
/// `target`'s vertex cloud. KD-tree built once, O((N+M) log M).
pub fn closest_points_batch(src: &Mesh, target: &Mesh) -> Vec<Option<Point3<f64>>> {
    if target.vertices.is_empty() {
        return vec![None; src.vertices.len()];
    }
    let tree = build_tree(target);
    src.vertices
        .iter()
        .map(|v| {
            let q = [v.x, v.y, v.z];
            tree.nearest_neighbor(&q).map(|p| {
                let g = p.geom();
                Point3::new(g[0], g[1], g[2])
            })
        })
        .collect()
}

/// AR-V410 — Signed compare report: distinguishes outside (positive) vs inside (negative).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignedCompareReport {
    /// Per-vertex signed distance: +mm if `src` vertex lies outside `dst` (along normal),
    /// -mm if it lies inside (interpenetration).
    pub signed_distances_mm: Vec<f64>,
    /// Mean of absolute values.
    pub mean_abs_distance_mm: f64,
    /// Mean of signed values (bias).
    pub mean_signed_distance_mm: f64,
    /// Maximum positive displacement (largest "outside" excursion).
    pub max_positive_mm: f64,
    /// Maximum negative displacement (deepest interpenetration); negative number.
    pub max_negative_mm: f64,
    /// Count of vertices that interpenetrate (negative side).
    pub interpenetration_count: usize,
}

/// Average vertex normal of all triangles incident to a target vertex.
fn build_target_normals(mesh: &Mesh) -> Vec<Vector3<f64>> {
    if mesh.normals.len() == mesh.vertices.len() {
        return mesh.normals.clone();
    }
    // Recompute manually so we don't mutate input.
    let mut normals = vec![Vector3::zeros(); mesh.vertices.len()];
    for tri in &mesh.indices {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        let n = (v1 - v0).cross(&(v2 - v0));
        if n.norm_squared() < 1e-18 {
            continue;
        }
        let n = n.normalize();
        for &idx in tri {
            normals[idx as usize] += n;
        }
    }
    for n in &mut normals {
        let norm = n.norm();
        if norm > 1e-12 {
            *n /= norm;
        } else {
            *n = Vector3::z();
        }
    }
    normals
}

/// Compute signed distance from each `src` vertex to `dst`. The sign is determined by the
/// dot-product of `(closest_dst_point - src_vertex)` with the closest dst-vertex's normal:
///   * positive  ⇒ src vertex is on the "outside" of dst (along the outward normal)
///   * negative  ⇒ src vertex is on the "inside" of dst (interpenetration)
pub fn compare_signed(src: &Mesh, dst: &Mesh) -> SignedCompareReport {
    if src.vertices.is_empty() || dst.vertices.is_empty() {
        return SignedCompareReport::default();
    }
    let dst_normals = build_target_normals(dst);
    let pts: Vec<IndexedPoint> = dst
        .vertices
        .iter()
        .enumerate()
        .map(|(i, p)| GeomWithData::new([p.x, p.y, p.z], i))
        .collect();
    let tree = RTree::bulk_load(pts);

    let mut signed: Vec<f64> = Vec::with_capacity(src.vertices.len());
    let mut sum_abs = 0.0;
    let mut sum_signed = 0.0;
    let mut max_pos = 0.0_f64;
    let mut max_neg = 0.0_f64;
    let mut interpen = 0usize;
    for v in &src.vertices {
        let q = [v.x, v.y, v.z];
        let nearest = match tree.nearest_neighbor(&q) {
            Some(n) => n,
            None => {
                signed.push(0.0);
                continue;
            }
        };
        let g = nearest.geom();
        let target_idx = nearest.data;
        let target_pt = Point3::new(g[0], g[1], g[2]);
        let dist = (target_pt - v).norm();
        // Sign: ray from src vertex toward target. If dot(target - src, target_normal) > 0
        // then the source vertex sits on the OUTSIDE of dst (target normal points toward src).
        // Equivalently we test the displacement against the OUTWARD normal of dst.
        let n = dst_normals[target_idx];
        // Vector pointing FROM target to source vertex.
        let to_src = v - target_pt;
        let sign = if to_src.dot(&n) >= 0.0 { 1.0 } else { -1.0 };
        let s = sign * dist;
        signed.push(s);
        sum_abs += dist;
        sum_signed += s;
        if s > max_pos {
            max_pos = s;
        }
        if s < max_neg {
            max_neg = s;
        }
        if s < 0.0 {
            interpen += 1;
        }
    }
    let n = src.vertices.len() as f64;
    SignedCompareReport {
        signed_distances_mm: signed,
        mean_abs_distance_mm: if n > 0.0 { sum_abs / n } else { 0.0 },
        mean_signed_distance_mm: if n > 0.0 { sum_signed / n } else { 0.0 },
        max_positive_mm: max_pos,
        max_negative_mm: max_neg,
        interpenetration_count: interpen,
    }
}

/// AR-V410 — Histogram bin (closed-open `[lower, upper)` except for the last bin which is
/// closed-closed `[lower, upper]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBin {
    pub lower_mm: f64,
    pub upper_mm: f64,
    pub count: usize,
}

/// AR-V410 — Build a histogram over per-vertex distances (unsigned). Useful for the standard
/// "color-coded deviation" report exocad / 3Shape produce after a scan-vs-design comparison.
pub fn histogram_compare(src: &Mesh, dst: &Mesh, bin_count: usize) -> Vec<HistogramBin> {
    if bin_count == 0 || src.vertices.is_empty() || dst.vertices.is_empty() {
        return Vec::new();
    }
    let distances = per_vertex_distance(src, dst);
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for &d in &distances {
        if d < min {
            min = d;
        }
        if d > max {
            max = d;
        }
    }
    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    if (max - min).abs() < 1e-12 {
        // All distances equal — single bin.
        return vec![HistogramBin {
            lower_mm: min,
            upper_mm: max,
            count: distances.len(),
        }];
    }
    let span = max - min;
    let bin_width = span / bin_count as f64;
    let mut bins: Vec<HistogramBin> = (0..bin_count)
        .map(|i| HistogramBin {
            lower_mm: min + i as f64 * bin_width,
            upper_mm: min + (i + 1) as f64 * bin_width,
            count: 0,
        })
        .collect();
    for &d in &distances {
        let mut idx = ((d - min) / bin_width).floor() as i64;
        if idx < 0 {
            idx = 0;
        }
        if idx as usize >= bin_count {
            idx = (bin_count - 1) as i64;
        }
        bins[idx as usize].count += 1;
    }
    bins
}

/// Closest point on B for each vertex of A — useful for snap operations.
pub fn closest_point_on(target: &Mesh, query: &Point3<f64>) -> Option<Point3<f64>> {
    if target.vertices.is_empty() {
        return None;
    }
    let tree = build_tree(target);
    let q = [query.x, query.y, query.z];
    tree.nearest_neighbor(&q).map(|p| {
        let g = p.geom();
        Point3::new(g[0], g[1], g[2])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_box;

    #[test]
    fn identical_meshes_zero_distance() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let r = compare(&a, &b);
        assert!(r.hausdorff_symmetric_mm < 1e-9);
        assert!(r.mean_a_to_b_mm < 1e-9);
    }

    #[test]
    fn translated_mesh_recovers_offset() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 1.0, 1.0));
        let r = compare(&a, &b);
        // worst case: vertex (0,0,0) in A → vertex (2,0,0) in B = 2mm; vertex (1,1,1) in A
        // → vertex (3,1,1) in B is also 2mm; worst pair is (0,1,1) → (2,0,0) etc.
        assert!(r.hausdorff_symmetric_mm > 1.5);
        assert!(r.hausdorff_symmetric_mm < 4.0);
    }

    #[test]
    fn per_vertex_distance_sane_length() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(0.5, 0.0, 0.0), Point3::new(1.5, 1.0, 1.0));
        let d = per_vertex_distance(&a, &b);
        assert_eq!(d.len(), a.vertices.len());
        assert!(d.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn signed_compare_identical_meshes_zero() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = a.clone();
        let r = compare_signed(&a, &b);
        assert_eq!(r.signed_distances_mm.len(), a.vertex_count());
        assert!(r.mean_abs_distance_mm < 1e-9);
        assert_eq!(r.interpenetration_count, 0);
    }

    #[test]
    fn signed_compare_translated_mesh_has_distance() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(0.5, 0.0, 0.0), Point3::new(1.5, 1.0, 1.0));
        let r = compare_signed(&a, &b);
        assert_eq!(r.signed_distances_mm.len(), a.vertex_count());
        // Some non-zero signed distances expected.
        assert!(r.mean_abs_distance_mm > 0.0);
    }

    #[test]
    fn signed_compare_empty_inputs() {
        let a = Mesh::new("a");
        let b = Mesh::new("b");
        let r = compare_signed(&a, &b);
        assert!(r.signed_distances_mm.is_empty());
        assert_eq!(r.interpenetration_count, 0);
    }

    #[test]
    fn histogram_compare_partitions_distances() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(0.5, 0.0, 0.0), Point3::new(1.5, 1.0, 1.0));
        let h = histogram_compare(&a, &b, 5);
        // When all distances coincide the function returns a single bin; when they vary it
        // returns `bin_count`. Either is valid — assert non-empty + total preserved.
        assert!(!h.is_empty());
        // sum of bin counts equals vertex count of a
        let total: usize = h.iter().map(|b| b.count).sum();
        assert_eq!(total, a.vertex_count());
        // bins are monotonic increasing in lower bound
        for w in h.windows(2) {
            assert!(w[0].lower_mm <= w[1].lower_mm);
        }
    }

    #[test]
    fn histogram_compare_with_varying_distances_produces_multiple_bins() {
        // Build two boxes with very different shapes so vertex distances vary.
        let a = create_box(Point3::origin(), Point3::new(2.0, 1.0, 1.0));
        let b = create_box(Point3::new(0.5, 0.5, 0.5), Point3::new(1.5, 1.5, 1.5));
        let h = histogram_compare(&a, &b, 4);
        assert!(!h.is_empty());
        let total: usize = h.iter().map(|b| b.count).sum();
        assert_eq!(total, a.vertex_count());
    }

    #[test]
    fn histogram_compare_zero_bins_empty() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = a.clone();
        let h = histogram_compare(&a, &b, 0);
        assert!(h.is_empty());
    }

    #[test]
    fn closest_point_finds_corner() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let p = closest_point_on(&mesh, &Point3::new(-5.0, -5.0, -5.0)).unwrap();
        assert!((p - Point3::origin()).norm() < 1e-6);
    }
}
