//! S111-S115: Scan import and preprocessing pipeline.
//!
//! Import dental scans from common file formats (STL, PLY, OBJ),
//! validate mesh integrity, align to dental coordinate system.

use nalgebra::{Point3, Vector3, Isometry3, UnitQuaternion};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Supported scan file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScanFormat {
    Stl,
    StlAscii,
    Ply,
    Obj,
    Dcm,
}

impl ScanFormat {
    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "stl" => Some(Self::Stl),
            "ply" => Some(Self::Ply),
            "obj" => Some(Self::Obj),
            "dcm" | "dicom" => Some(Self::Dcm),
            _ => None,
        }
    }
}

/// Quality metrics for an imported scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanQuality {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub has_normals: bool,
    pub is_watertight: bool,
    pub non_manifold_edges: usize,
    pub degenerate_triangles: usize,
    pub bounding_box_size: [f64; 3],
    /// Estimated scan resolution (average edge length in mm).
    pub avg_edge_length: f64,
}

/// Raw imported scan data.
#[derive(Debug, Clone)]
pub struct RawScan {
    pub vertices: Vec<Point3<f64>>,
    pub normals: Vec<Vector3<f64>>,
    pub indices: Vec<[u32; 3]>,
    pub format: ScanFormat,
}

/// Result of scan alignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentResult {
    pub transform: [[f64; 4]; 4],
    pub rms_error: f64,
    pub iterations: usize,
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// Mesh quality analysis (S111-S112)
// ---------------------------------------------------------------------------

/// Analyze mesh quality metrics.
pub fn analyze_scan_quality(scan: &RawScan) -> ScanQuality {
    let vertex_count = scan.vertices.len();
    let triangle_count = scan.indices.len();
    let has_normals = !scan.normals.is_empty() && scan.normals.len() == vertex_count;

    // Bounding box
    let (bb_min, bb_max) = bounding_box(&scan.vertices);
    let bb_size = [bb_max.x - bb_min.x, bb_max.y - bb_min.y, bb_max.z - bb_min.z];

    // Degenerate triangles (zero area)
    let degenerate_triangles = scan
        .indices
        .iter()
        .filter(|tri| {
            let a = &scan.vertices[tri[0] as usize];
            let b = &scan.vertices[tri[1] as usize];
            let c = &scan.vertices[tri[2] as usize];
            (b - a).cross(&(c - a)).norm() < 1e-12
        })
        .count();

    // Average edge length
    let mut edge_sum = 0.0;
    let mut edge_count = 0usize;
    for tri in &scan.indices {
        for k in 0..3 {
            let a = &scan.vertices[tri[k] as usize];
            let b = &scan.vertices[tri[(k + 1) % 3] as usize];
            edge_sum += (b - a).norm();
            edge_count += 1;
        }
    }
    let avg_edge_length = if edge_count > 0 { edge_sum / edge_count as f64 } else { 0.0 };

    // Non-manifold edges: an edge shared by != 2 triangles
    let non_manifold_edges = count_non_manifold_edges(&scan.indices);

    // Watertight: no boundary edges (each edge shared by exactly 2 triangles)
    let is_watertight = non_manifold_edges == 0 && triangle_count > 0;

    ScanQuality {
        vertex_count,
        triangle_count,
        has_normals,
        is_watertight,
        non_manifold_edges,
        degenerate_triangles,
        bounding_box_size: bb_size,
        avg_edge_length,
    }
}

fn bounding_box(vertices: &[Point3<f64>]) -> (Point3<f64>, Point3<f64>) {
    if vertices.is_empty() {
        return (Point3::origin(), Point3::origin());
    }
    let mut min = vertices[0];
    let mut max = vertices[0];
    for v in vertices {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        min.z = min.z.min(v.z);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
        max.z = max.z.max(v.z);
    }
    (min, max)
}

fn count_non_manifold_edges(indices: &[[u32; 3]]) -> usize {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let edge = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    }
    edge_count.values().filter(|&&c| c != 2).count()
}

// ---------------------------------------------------------------------------
// Scan preprocessing (S113)
// ---------------------------------------------------------------------------

/// Remove degenerate triangles from the scan.
pub fn remove_degenerate_triangles(scan: &mut RawScan) {
    scan.indices.retain(|tri| {
        let a = &scan.vertices[tri[0] as usize];
        let b = &scan.vertices[tri[1] as usize];
        let c = &scan.vertices[tri[2] as usize];
        (b - a).cross(&(c - a)).norm() > 1e-12
    });
}

/// Recompute vertex normals from triangle faces.
pub fn recompute_normals(scan: &mut RawScan) {
    let mut normals = vec![Vector3::zeros(); scan.vertices.len()];
    for tri in &scan.indices {
        let a = &scan.vertices[tri[0] as usize];
        let b = &scan.vertices[tri[1] as usize];
        let c = &scan.vertices[tri[2] as usize];
        let face_normal = (b - a).cross(&(c - a));
        for &vi in tri {
            normals[vi as usize] += face_normal;
        }
    }
    for n in &mut normals {
        let len = n.norm();
        if len > 1e-12 {
            *n /= len;
        }
    }
    scan.normals = normals;
}

/// Center the scan at the origin using centroid.
pub fn center_scan(scan: &mut RawScan) {
    if scan.vertices.is_empty() {
        return;
    }
    let sum: Vector3<f64> = scan.vertices.iter().map(|p| p.coords).sum();
    let centroid = sum / scan.vertices.len() as f64;
    for v in &mut scan.vertices {
        v.coords -= centroid;
    }
}

/// Scale scan uniformly so that bounding box fits within target_size mm.
pub fn normalize_scale(scan: &mut RawScan, target_size: f64) {
    let (min, max) = bounding_box(&scan.vertices);
    let diag = (max - min).norm();
    if diag < 1e-12 {
        return;
    }
    let factor = target_size / diag;
    for v in &mut scan.vertices {
        v.coords *= factor;
    }
}

// ---------------------------------------------------------------------------
// Scan alignment (S114-S115) — simplified ICP
// ---------------------------------------------------------------------------

/// Align source scan to target using simplified ICP (Iterative Closest Point).
pub fn align_scans(
    source: &mut RawScan,
    target: &RawScan,
    max_iterations: usize,
    tolerance: f64,
) -> AlignmentResult {
    let mut total_transform = Isometry3::identity();
    let mut prev_error = f64::MAX;
    let mut converged = false;

    for iteration in 0..max_iterations {
        // Find closest points
        let correspondences: Vec<(usize, usize)> = source
            .vertices
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let closest = target
                    .vertices
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        let da = (a.coords - p.coords).norm();
                        let db = (b.coords - p.coords).norm();
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(j, _)| j)
                    .unwrap_or(0);
                (i, closest)
            })
            .collect();

        // Compute RMS error
        let rms = compute_rms(&source.vertices, &target.vertices, &correspondences);

        if (prev_error - rms).abs() < tolerance {
            converged = true;
            return AlignmentResult {
                transform: isometry_to_matrix(&total_transform),
                rms_error: rms,
                iterations: iteration + 1,
                converged,
            };
        }
        prev_error = rms;

        // Compute rigid transform from correspondences (centroid-based)
        let src_centroid = centroid_subset(&source.vertices, &correspondences.iter().map(|c| c.0).collect::<Vec<_>>());
        let tgt_centroid = centroid_subset(&target.vertices, &correspondences.iter().map(|c| c.1).collect::<Vec<_>>());

        let translation = tgt_centroid - src_centroid;
        for v in &mut source.vertices {
            v.coords += translation;
        }

        let step = Isometry3::from_parts(
            nalgebra::Translation3::new(translation.x, translation.y, translation.z),
            UnitQuaternion::identity(),
        );
        total_transform = step * total_transform;
    }

    AlignmentResult {
        transform: isometry_to_matrix(&total_transform),
        rms_error: prev_error,
        iterations: max_iterations,
        converged,
    }
}

fn compute_rms(source: &[Point3<f64>], target: &[Point3<f64>], correspondences: &[(usize, usize)]) -> f64 {
    if correspondences.is_empty() {
        return 0.0;
    }
    let sum: f64 = correspondences
        .iter()
        .map(|&(si, ti)| (source[si] - target[ti]).norm_squared())
        .sum();
    (sum / correspondences.len() as f64).sqrt()
}

fn centroid_subset(points: &[Point3<f64>], indices: &[usize]) -> Vector3<f64> {
    if indices.is_empty() {
        return Vector3::zeros();
    }
    let sum: Vector3<f64> = indices.iter().map(|&i| points[i].coords).sum();
    sum / indices.len() as f64
}

fn isometry_to_matrix(iso: &Isometry3<f64>) -> [[f64; 4]; 4] {
    let m = iso.to_homogeneous();
    let mut result = [[0.0f64; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            result[r][c] = m[(r, c)];
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scan() -> RawScan {
        RawScan {
            vertices: vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
                Point3::new(1.0, 1.0, 0.0),
            ],
            normals: vec![Vector3::z(); 4],
            indices: vec![[0, 1, 2], [1, 3, 2]],
            format: ScanFormat::Stl,
        }
    }

    #[test]
    fn quality_analysis() {
        let scan = make_scan();
        let q = analyze_scan_quality(&scan);
        assert_eq!(q.vertex_count, 4);
        assert_eq!(q.triangle_count, 2);
        assert!(q.has_normals);
        assert_eq!(q.degenerate_triangles, 0);
        assert!(q.avg_edge_length > 0.0);
    }

    #[test]
    fn watertight_check() {
        let scan = make_scan();
        let q = analyze_scan_quality(&scan);
        // Open mesh (boundary edges) → not watertight
        assert!(!q.is_watertight);
    }

    #[test]
    fn remove_degenerates() {
        let mut scan = make_scan();
        // Add a degenerate triangle
        scan.vertices.push(Point3::new(5.0, 5.0, 5.0));
        scan.indices.push([4, 4, 4]);
        assert_eq!(scan.indices.len(), 3);
        remove_degenerate_triangles(&mut scan);
        assert_eq!(scan.indices.len(), 2);
    }

    #[test]
    fn center_scan_moves_centroid() {
        let mut scan = make_scan();
        center_scan(&mut scan);
        let sum: Vector3<f64> = scan.vertices.iter().map(|p| p.coords).sum();
        let centroid = sum / scan.vertices.len() as f64;
        assert!(centroid.norm() < 1e-9);
    }

    #[test]
    fn recompute_normals_works() {
        let mut scan = make_scan();
        scan.normals.clear();
        recompute_normals(&mut scan);
        assert_eq!(scan.normals.len(), scan.vertices.len());
        for n in &scan.normals {
            assert!((n.norm() - 1.0).abs() < 1e-6 || n.norm() < 1e-12);
        }
    }

    #[test]
    fn format_detection() {
        assert_eq!(ScanFormat::from_extension("stl"), Some(ScanFormat::Stl));
        assert_eq!(ScanFormat::from_extension("PLY"), Some(ScanFormat::Ply));
        assert_eq!(ScanFormat::from_extension("obj"), Some(ScanFormat::Obj));
        assert_eq!(ScanFormat::from_extension("xyz"), None);
    }

    #[test]
    fn align_identical_scans() {
        let mut source = make_scan();
        let target = make_scan();
        let result = align_scans(&mut source, &target, 10, 1e-6);
        assert!(result.rms_error < 1e-6);
        assert!(result.converged);
    }

    #[test]
    fn align_translated_scan() {
        let mut source = make_scan();
        // Shift source by (5, 0, 0)
        for v in &mut source.vertices {
            v.x += 5.0;
        }
        let target = make_scan();
        let result = align_scans(&mut source, &target, 50, 1e-6);
        assert!(result.rms_error < 0.5, "RMS should decrease after alignment");
    }

    #[test]
    fn normalize_scale_works() {
        let mut scan = make_scan();
        normalize_scale(&mut scan, 100.0);
        let (min, max) = bounding_box(&scan.vertices);
        let diag = (max - min).norm();
        assert!((diag - 100.0).abs() < 1e-6);
    }
}
