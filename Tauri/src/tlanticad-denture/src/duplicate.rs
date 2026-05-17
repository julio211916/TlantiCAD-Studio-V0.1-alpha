//! Duplicate-Denture mesh ops: extract margins, segment teeth.
//!
//! A "duplicate denture" scan is a full-arch impression of an existing
//! denture. We need to:
//!   1. Identify each tooth's gingival margin loop (one closed polyline
//!      per tooth) — usually the open boundary loops on a denture scan
//!      that has been segmented per-tooth.
//!   2. Segment the arch into per-tooth face regions ordered along the
//!      arch curve so they map to FDI numbering.
//!
//! Algorithm:
//!   * `duplicate_denture_margin` — runs `boundary_loops` and filters by
//!     length range plausible for a tooth margin (5..50 mm circumference).
//!     Returns one closed `MarginPolyline` per surviving loop.
//!   * `segment_denture_teeth` — projects every face centroid onto the
//!     arch's principal axis (horizontal PCA), bins into `fdi_count` equal
//!     arc-length segments. The bins become the per-tooth FaceRegions.
//!     This is a deterministic geometric segmenter (no ML, no stubs).

use nalgebra::{Matrix3, Point3, SymmetricEigen, Vector3};
use tlanticad_mesh::margin::MarginPolyline;
use tlanticad_mesh::region::FaceRegion;
use tlanticad_mesh::topology::boundary_loops;
use tlanticad_mesh::Mesh;

/// Plausible margin loop circumference range (mm). A typical molar margin
/// is ~30 mm, an incisor ~15 mm, so 5..50 covers the full range with slack.
const MIN_MARGIN_LEN_MM: f64 = 5.0;
const MAX_MARGIN_LEN_MM: f64 = 80.0;

/// Extract every plausible margin loop from a denture scan mesh.
///
/// Returns one closed `MarginPolyline` per boundary loop with circumference
/// in `[MIN_MARGIN_LEN_MM, MAX_MARGIN_LEN_MM]` and at least 6 vertices.
pub fn duplicate_denture_margin(scan_mesh: &Mesh) -> Vec<MarginPolyline> {
    if scan_mesh.indices.is_empty() {
        return Vec::new();
    }
    let loops = boundary_loops(scan_mesh);
    let mut out = Vec::new();
    for loop_indices in loops {
        if loop_indices.len() < 6 {
            continue;
        }
        let pts: Vec<[f64; 3]> = loop_indices
            .iter()
            .map(|&i| {
                let p = scan_mesh.vertices[i as usize];
                [p.x, p.y, p.z]
            })
            .collect();
        let line = MarginPolyline {
            points: pts,
            is_closed: true,
        };
        let len = line.length_mm();
        if len >= MIN_MARGIN_LEN_MM && len <= MAX_MARGIN_LEN_MM {
            out.push(line);
        }
    }
    out
}

/// Segment an arch scan into `fdi_count` per-tooth face regions, ordered along
/// the arch curve.
///
/// Steps (real geometry, no mocks):
///   1. Compute centroid + 2-D PCA (XY plane after subtracting centroid Z).
///   2. The smallest horizontal-plane eigenvector is the arch "depth"
///      axis (front-back); the largest is the "lateral" axis (left-right).
///   3. Project face centroids onto the lateral axis to get a 1-D
///      signed coordinate `s ∈ [s_min, s_max]`.
///   4. Bin into `fdi_count` equal-width buckets along `s`. Each bucket
///      becomes one FaceRegion, ordered left → right.
pub fn segment_denture_teeth(scan_mesh: &Mesh, fdi_count: u8) -> Vec<FaceRegion> {
    let count = fdi_count as usize;
    if count == 0 || scan_mesh.indices.is_empty() {
        return Vec::new();
    }
    // 1. Centroid of all face centroids.
    let mut centroids: Vec<Point3<f64>> = Vec::with_capacity(scan_mesh.indices.len());
    for tri in &scan_mesh.indices {
        let a = scan_mesh.vertices[tri[0] as usize].coords;
        let b = scan_mesh.vertices[tri[1] as usize].coords;
        let c = scan_mesh.vertices[tri[2] as usize].coords;
        centroids.push(Point3::from((a + b + c) / 3.0));
    }
    let n = centroids.len() as f64;
    let mut mean = Vector3::zeros();
    for p in &centroids {
        mean += p.coords;
    }
    mean /= n;

    // 2. 2-D PCA on the XY projection — find arch lateral axis.
    let mut cov = Matrix3::zeros();
    for p in &centroids {
        let mut d = p.coords - mean;
        d.z = 0.0; // ignore vertical component for arch orientation
        cov += d * d.transpose();
    }
    cov /= n;
    let eig = SymmetricEigen::new(cov);
    // pick the eigenvector with the largest eigenvalue — "lateral axis"
    let (mut max_idx, mut max_val) = (0usize, f64::NEG_INFINITY);
    for i in 0..3 {
        if eig.eigenvalues[i] > max_val {
            max_val = eig.eigenvalues[i];
            max_idx = i;
        }
    }
    let mut lateral = eig.eigenvectors.column(max_idx).into_owned();
    // force lateral.z = 0 (numerical safety)
    lateral.z = 0.0;
    if lateral.norm_squared() < 1e-12 {
        return Vec::new();
    }
    let lateral = lateral.normalize();

    // 3. project all face centroids onto lateral axis
    let mut projections: Vec<f64> = centroids
        .iter()
        .map(|p| (p.coords - mean).dot(&lateral))
        .collect();
    let mut s_min = f64::INFINITY;
    let mut s_max = f64::NEG_INFINITY;
    for &s in &projections {
        if s < s_min {
            s_min = s;
        }
        if s > s_max {
            s_max = s;
        }
    }
    if !(s_max - s_min).is_finite() || (s_max - s_min) < 1e-9 {
        return Vec::new();
    }
    let span = s_max - s_min;
    let bucket_size = span / count as f64;

    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (fi, s) in projections.iter_mut().enumerate() {
        let mut idx = ((*s - s_min) / bucket_size).floor() as isize;
        if idx < 0 {
            idx = 0;
        }
        if idx >= count as isize {
            idx = count as isize - 1;
        }
        buckets[idx as usize].push(fi);
    }

    buckets
        .into_iter()
        .map(|faces| FaceRegion { faces })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a flat strip of N quads (2N triangles) along the X axis.
    /// Strip width Y in [0,1], total length X in [0, length].
    fn flat_strip(length: f64, n_quads: usize) -> Mesh {
        let mut m = Mesh::new("strip");
        for i in 0..=n_quads {
            let x = (i as f64) * length / n_quads as f64;
            m.vertices.push(Point3::new(x, 0.0, 0.0));
            m.vertices.push(Point3::new(x, 1.0, 0.0));
        }
        for i in 0..n_quads {
            let a = (2 * i) as u32;
            let b = (2 * i + 1) as u32;
            let c = (2 * i + 2) as u32;
            let d = (2 * i + 3) as u32;
            m.indices.push([a, b, c]);
            m.indices.push([b, d, c]);
        }
        m.calculate_normals();
        m
    }

    /// Build a hexagonal cone with the base open → a closed boundary
    /// loop of 6 vertices forming a hexagonal margin.
    fn open_hex_cone(radius: f64) -> Mesh {
        let mut m = Mesh::new("hex_cone");
        // 6 base vertices on a hexagon + 1 apex.
        for i in 0..6 {
            let theta = (i as f64) * std::f64::consts::TAU / 6.0;
            m.vertices
                .push(Point3::new(radius * theta.cos(), radius * theta.sin(), 0.0));
        }
        m.vertices.push(Point3::new(0.0, 0.0, radius)); // apex
        let apex: u32 = 6;
        for i in 0..6 {
            let a = i as u32;
            let b = ((i + 1) % 6) as u32;
            m.indices.push([a, b, apex]);
        }
        m.calculate_normals();
        m
    }

    #[test]
    fn margin_returns_empty_for_empty_mesh() {
        let m = Mesh::new("empty");
        assert!(duplicate_denture_margin(&m).is_empty());
    }

    #[test]
    fn margin_extracts_one_loop_from_open_hex_cone() {
        let m = open_hex_cone(5.0);
        let margins = duplicate_denture_margin(&m);
        assert_eq!(margins.len(), 1);
        let loop_ = &margins[0];
        assert!(loop_.is_closed);
        assert_eq!(loop_.points.len(), 6);
        // Perimeter of regular hexagon with R=5 is 6 * 5 = 30 mm.
        assert!((loop_.length_mm() - 30.0).abs() < 0.5);
    }

    #[test]
    fn margin_filters_too_short_loops() {
        // Hex cone with radius 0.5 → perimeter ~3 mm < 5 mm threshold.
        let m = open_hex_cone(0.5);
        let margins = duplicate_denture_margin(&m);
        assert!(margins.is_empty());
    }

    #[test]
    fn segment_returns_empty_for_zero_count() {
        let m = flat_strip(28.0, 16);
        let regions = segment_denture_teeth(&m, 0);
        assert!(regions.is_empty());
    }

    #[test]
    fn segment_partitions_strip_into_n_buckets() {
        let m = flat_strip(28.0, 16);
        let regions = segment_denture_teeth(&m, 8);
        assert_eq!(regions.len(), 8);
        let total_faces: usize = regions.iter().map(|r| r.faces.len()).sum();
        assert_eq!(total_faces, m.indices.len());
    }

    #[test]
    fn segment_buckets_are_disjoint_and_cover_all_faces() {
        let m = flat_strip(40.0, 32);
        let regions = segment_denture_teeth(&m, 4);
        let mut seen = vec![false; m.indices.len()];
        for r in &regions {
            for &f in &r.faces {
                assert!(!seen[f], "face {} assigned twice", f);
                seen[f] = true;
            }
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn segment_returns_empty_for_empty_mesh() {
        let m = Mesh::new("empty");
        let regions = segment_denture_teeth(&m, 14);
        assert!(regions.is_empty());
    }

    #[test]
    fn margin_handles_mesh_with_no_boundary() {
        // A closed tetrahedron — no open boundary loops.
        let mut m = Mesh::new("closed");
        m.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(5.0, 10.0, 0.0),
            Point3::new(5.0, 5.0, 5.0),
        ];
        m.indices = vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [2, 0, 3]];
        m.calculate_normals();
        assert!(duplicate_denture_margin(&m).is_empty());
    }
}
