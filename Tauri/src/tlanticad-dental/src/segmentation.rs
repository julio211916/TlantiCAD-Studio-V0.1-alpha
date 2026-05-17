//! S3+S121-S125: Dental scan segmentation
//!
//! Separate gingiva, teeth, and preparation areas via multi-algorithm approach.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Segmentation label for each triangle/vertex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentLabel {
    Gingiva,
    Tooth(u8),       // FDI tooth number
    Preparation(u8), // Prepared tooth
    Implant(u8),
    Unknown,
}

/// Algorithm selection for segmentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentationAlgorithm {
    /// Curvature threshold (original).
    Curvature,
    /// K-means clustering in normal-space.
    KMeans,
    /// Watershed on curvature field.
    Watershed,
    /// Multi-algo consensus (best quality).
    Combined,
}

/// Segmentation result.
#[derive(Debug, Clone)]
pub struct SegmentationResult {
    /// Label per triangle.
    pub triangle_labels: Vec<SegmentLabel>,
    /// Label per vertex.
    pub vertex_labels: Vec<SegmentLabel>,
    /// Number of distinct segments found.
    pub segment_count: usize,
}

/// A warning about potential segmentation issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationWarning {
    pub kind: WarningKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningKind {
    MissingTeeth,
    OverlappingSegments,
    TinySegment,
    OpenBoundary,
}

/// Interproximal region between two adjacent teeth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterproximalRegion {
    pub tooth_a: u8,
    pub tooth_b: u8,
    /// Estimated contact area (mm²).
    pub contact_area: f64,
    /// Minimum gap distance (mm).
    pub gap_distance: f64,
}

// ---------------------------------------------------------------------------
// Main API
// ---------------------------------------------------------------------------

/// Segment with the chosen algorithm.
pub fn segment(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    curvature_threshold: f64,
    algorithm: SegmentationAlgorithm,
) -> SegmentationResult {
    match algorithm {
        SegmentationAlgorithm::Curvature => {
            segment_by_curvature(vertices, normals, indices, curvature_threshold)
        }
        SegmentationAlgorithm::KMeans => {
            segment_by_kmeans(vertices, normals, indices, curvature_threshold)
        }
        SegmentationAlgorithm::Watershed => {
            segment_by_watershed(vertices, indices, curvature_threshold)
        }
        SegmentationAlgorithm::Combined => {
            segment_combined(vertices, normals, indices, curvature_threshold)
        }
    }
}

/// Multi-algorithm consensus segmentation.
pub fn segment_combined(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    curvature_threshold: f64,
) -> SegmentationResult {
    let r1 = segment_by_curvature(vertices, normals, indices, curvature_threshold);
    let r2 = segment_by_kmeans(vertices, normals, indices, curvature_threshold);
    let r3 = segment_by_watershed(vertices, indices, curvature_threshold);

    // Consensus: majority vote per vertex
    let n = vertices.len();
    let mut vertex_labels = vec![SegmentLabel::Unknown; n];
    for i in 0..n {
        let a = r1.vertex_labels[i];
        let b = r2.vertex_labels[i];
        let c = r3.vertex_labels[i];
        vertex_labels[i] = if a == b || a == c {
            a
        } else if b == c {
            b
        } else {
            a // fallback to curvature
        };
    }

    let triangle_labels = majority_vote_triangles(&vertex_labels, indices);
    let segment_count = count_unique_segments(&vertex_labels);

    SegmentationResult { triangle_labels, vertex_labels, segment_count }
}

// ---------------------------------------------------------------------------
// Curvature-based segmentation (original)
// ---------------------------------------------------------------------------

/// Segment based on curvature thresholds (concavity at gingival sulcus).
pub fn segment_by_curvature(
    vertices: &[Point3<f64>],
    _normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    curvature_threshold: f64,
) -> SegmentationResult {
    let curvatures = compute_vertex_curvatures(vertices, indices);
    let n_verts = vertices.len();
    let mut vertex_labels = vec![SegmentLabel::Unknown; n_verts];

    // Mark high-curvature vertices as boundaries
    let mut is_boundary = vec![false; n_verts];
    for (i, &k) in curvatures.iter().enumerate() {
        if k.abs() > curvature_threshold {
            is_boundary[i] = true;
        }
    }

    // Flood-fill connected components
    let adjacency = build_adjacency(vertices.len(), indices);
    let mut visited = vec![false; n_verts];
    let mut components: Vec<Vec<usize>> = Vec::new();

    for start in 0..n_verts {
        if visited[start] || is_boundary[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start];
        while let Some(vi) = stack.pop() {
            if visited[vi] || is_boundary[vi] {
                continue;
            }
            visited[vi] = true;
            component.push(vi);
            for &ni in &adjacency[vi] {
                if !visited[ni] && !is_boundary[ni] {
                    stack.push(ni);
                }
            }
        }
        if !component.is_empty() {
            components.push(component);
        }
    }

    // Largest component = gingiva, others = teeth
    if let Some((gi, _)) = components.iter().enumerate().max_by_key(|(_, c)| c.len()) {
        for &vi in &components[gi] {
            vertex_labels[vi] = SegmentLabel::Gingiva;
        }
    }

    let mut tooth_num = 11u8;
    for component in &components {
        if vertex_labels.get(component.first().copied().unwrap_or(0))
            == Some(&SegmentLabel::Gingiva)
        {
            continue;
        }
        for &vi in component {
            vertex_labels[vi] = SegmentLabel::Tooth(tooth_num);
        }
        tooth_num = next_fdi(tooth_num);
    }

    // Boundaries → gingiva
    for (i, b) in is_boundary.iter().enumerate() {
        if *b {
            vertex_labels[i] = SegmentLabel::Gingiva;
        }
    }

    let triangle_labels = majority_vote_triangles(&vertex_labels, indices);

    SegmentationResult {
        segment_count: components.len(),
        triangle_labels,
        vertex_labels,
    }
}

// ---------------------------------------------------------------------------
// K-means normal clustering
// ---------------------------------------------------------------------------

/// Segment via K-means in normal-space (groups regions with similar orientation).
pub fn segment_by_kmeans(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    _curvature_threshold: f64,
) -> SegmentationResult {
    let n = vertices.len();
    if n == 0 {
        return SegmentationResult {
            triangle_labels: Vec::new(),
            vertex_labels: Vec::new(),
            segment_count: 0,
        };
    }

    let k = 20.min(n); // max clusters
    let max_iter = 30;

    // Initialize centroids by sampling evenly
    let step = n / k.max(1);
    let mut centroids: Vec<Vector3<f64>> = (0..k)
        .map(|i| normals[i * step].normalize())
        .collect();

    let mut labels = vec![0usize; n];

    for _ in 0..max_iter {
        // Assign each vertex to nearest centroid
        let mut changed = false;
        for (vi, normal) in normals.iter().enumerate() {
            let best = centroids
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let da = (normal - *a).norm_squared();
                    let db = (normal - *b).norm_squared();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
            if labels[vi] != best {
                labels[vi] = best;
                changed = true;
            }
        }
        if !changed {
            break;
        }
        // Update centroids
        let mut sums = vec![Vector3::zeros(); k];
        let mut counts = vec![0usize; k];
        for (vi, &l) in labels.iter().enumerate() {
            sums[l] += normals[vi];
            counts[l] += 1;
        }
        for (ci, sum) in sums.iter().enumerate() {
            if counts[ci] > 0 {
                centroids[ci] = (sum / counts[ci] as f64).normalize();
            }
        }
    }

    // Convert cluster labels to SegmentLabels (largest = gingiva)
    let mut sizes = vec![0usize; k];
    for &l in &labels {
        sizes[l] += 1;
    }
    let gingiva_cluster = sizes
        .iter()
        .enumerate()
        .max_by_key(|(_, &s)| s)
        .map(|(i, _)| i)
        .unwrap_or(0);

    let mut vertex_labels = vec![SegmentLabel::Unknown; n];
    let mut tooth_num = 11u8;
    for cluster in 0..k {
        if sizes[cluster] == 0 {
            continue;
        }
        if cluster == gingiva_cluster {
            for (vi, &l) in labels.iter().enumerate() {
                if l == cluster {
                    vertex_labels[vi] = SegmentLabel::Gingiva;
                }
            }
        } else {
            for (vi, &l) in labels.iter().enumerate() {
                if l == cluster {
                    vertex_labels[vi] = SegmentLabel::Tooth(tooth_num);
                }
            }
            tooth_num = next_fdi(tooth_num);
        }
    }

    let triangle_labels = majority_vote_triangles(&vertex_labels, indices);
    let segment_count = count_unique_segments(&vertex_labels);

    SegmentationResult { triangle_labels, vertex_labels, segment_count }
}

// ---------------------------------------------------------------------------
// Watershed segmentation
// ---------------------------------------------------------------------------

/// Watershed segmentation on curvature field.
pub fn segment_by_watershed(
    vertices: &[Point3<f64>],
    indices: &[[u32; 3]],
    _curvature_threshold: f64,
) -> SegmentationResult {
    let n = vertices.len();
    if n == 0 {
        return SegmentationResult {
            triangle_labels: Vec::new(),
            vertex_labels: Vec::new(),
            segment_count: 0,
        };
    }

    let curvatures = compute_vertex_curvatures(vertices, indices);
    let adjacency = build_adjacency(n, indices);

    // Sort vertices by curvature ascending (local minima first = seed points)
    let mut sorted_indices: Vec<usize> = (0..n).collect();
    sorted_indices.sort_by(|&a, &b| {
        curvatures[a]
            .partial_cmp(&curvatures[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut basin = vec![usize::MAX; n]; // basin assignment
    let mut next_basin = 0usize;

    for &vi in &sorted_indices {
        if basin[vi] != usize::MAX {
            continue;
        }
        // Check if any neighbor already has a basin
        let neighbor_basins: Vec<usize> = adjacency[vi]
            .iter()
            .filter_map(|&ni| {
                if basin[ni] != usize::MAX {
                    Some(basin[ni])
                } else {
                    None
                }
            })
            .collect();

        if neighbor_basins.is_empty() {
            // New basin
            basin[vi] = next_basin;
            next_basin += 1;
        } else {
            // Assign to most common neighbor basin
            basin[vi] = *neighbor_basins.first().unwrap_or(&0);
        }
    }

    // Convert basins to SegmentLabels
    let mut sizes = vec![0usize; next_basin];
    for &b in &basin {
        if b < next_basin {
            sizes[b] += 1;
        }
    }
    let gingiva_basin = sizes
        .iter()
        .enumerate()
        .max_by_key(|(_, &s)| s)
        .map(|(i, _)| i)
        .unwrap_or(0);

    let mut vertex_labels = vec![SegmentLabel::Unknown; n];
    let mut tooth_num = 11u8;
    for b in 0..next_basin {
        if sizes[b] == 0 {
            continue;
        }
        if b == gingiva_basin {
            for (vi, &bi) in basin.iter().enumerate() {
                if bi == b {
                    vertex_labels[vi] = SegmentLabel::Gingiva;
                }
            }
        } else {
            for (vi, &bi) in basin.iter().enumerate() {
                if bi == b {
                    vertex_labels[vi] = SegmentLabel::Tooth(tooth_num);
                }
            }
            tooth_num = next_fdi(tooth_num);
        }
    }

    let triangle_labels = majority_vote_triangles(&vertex_labels, indices);
    let segment_count = count_unique_segments(&vertex_labels);

    SegmentationResult { triangle_labels, vertex_labels, segment_count }
}

// ---------------------------------------------------------------------------
// Gingiva detection (S122)
// ---------------------------------------------------------------------------

/// Separate gingiva from teeth based on curvature.
/// Returns (gingiva_face_indices, tooth_face_indices).
pub fn detect_gingiva(
    vertices: &[Point3<f64>],
    indices: &[[u32; 3]],
    curvature_threshold: f64,
) -> (Vec<usize>, Vec<usize>) {
    let curvatures = compute_vertex_curvatures(vertices, indices);

    let mut gingiva_faces = Vec::new();
    let mut tooth_faces = Vec::new();

    for (fi, tri) in indices.iter().enumerate() {
        let avg_k = (curvatures[tri[0] as usize]
            + curvatures[tri[1] as usize]
            + curvatures[tri[2] as usize])
            / 3.0;
        if avg_k < curvature_threshold {
            gingiva_faces.push(fi);
        } else {
            tooth_faces.push(fi);
        }
    }

    (gingiva_faces, tooth_faces)
}

// ---------------------------------------------------------------------------
// Interproximal regions (S123)
// ---------------------------------------------------------------------------

/// Detect interproximal regions between adjacent segmented teeth.
pub fn detect_interproximal(
    vertices: &[Point3<f64>],
    indices: &[[u32; 3]],
    result: &SegmentationResult,
) -> Vec<InterproximalRegion> {
    let mut regions = Vec::new();
    let adjacency = build_adjacency(vertices.len(), indices);

    // Find pairs of different Tooth labels that share boundary edges
    let mut seen_pairs = Vec::new();

    for (vi, label) in result.vertex_labels.iter().enumerate() {
        if let SegmentLabel::Tooth(a) = label {
            for &ni in &adjacency[vi] {
                if let SegmentLabel::Tooth(b) = result.vertex_labels[ni] {
                    if a != &b {
                        let pair = if *a < b { (*a, b) } else { (b, *a) };
                        if !seen_pairs.contains(&pair) {
                            seen_pairs.push(pair);
                            let gap = (vertices[vi] - vertices[ni]).norm();
                            regions.push(InterproximalRegion {
                                tooth_a: pair.0,
                                tooth_b: pair.1,
                                contact_area: 0.5, // approximation
                                gap_distance: gap,
                            });
                        }
                    }
                }
            }
        }
    }

    regions
}

// ---------------------------------------------------------------------------
// Manual segmentation correction (S124)
// ---------------------------------------------------------------------------

/// Merge two segments into one in the result.
pub fn merge_segments(result: &mut SegmentationResult, keep_label: SegmentLabel, remove_label: SegmentLabel) {
    for vl in result.vertex_labels.iter_mut() {
        if *vl == remove_label {
            *vl = keep_label;
        }
    }
    for tl in result.triangle_labels.iter_mut() {
        if *tl == remove_label {
            *tl = keep_label;
        }
    }
    result.segment_count = count_unique_segments(&result.vertex_labels);
}

/// Reassign specific vertices to a new label.
pub fn reassign_vertices(result: &mut SegmentationResult, vertex_ids: &[usize], new_label: SegmentLabel) {
    for &vi in vertex_ids {
        if vi < result.vertex_labels.len() {
            result.vertex_labels[vi] = new_label;
        }
    }
    result.segment_count = count_unique_segments(&result.vertex_labels);
}

// ---------------------------------------------------------------------------
// Validation (S125)
// ---------------------------------------------------------------------------

/// Validate segmentation quality.
pub fn validate_segmentation(result: &SegmentationResult) -> Vec<SegmentationWarning> {
    let mut warnings = Vec::new();

    // Count teeth found
    let mut tooth_numbers = Vec::new();
    for label in &result.vertex_labels {
        if let SegmentLabel::Tooth(n) = label {
            if !tooth_numbers.contains(n) {
                tooth_numbers.push(*n);
            }
        }
    }

    if tooth_numbers.is_empty() {
        warnings.push(SegmentationWarning {
            kind: WarningKind::MissingTeeth,
            message: "No teeth detected in segmentation".into(),
        });
    }

    // Check for tiny segments (< 10 vertices)
    let mut seg_sizes: Vec<(SegmentLabel, usize)> = Vec::new();
    for label in &result.vertex_labels {
        if let Some(entry) = seg_sizes.iter_mut().find(|(l, _)| l == label) {
            entry.1 += 1;
        } else {
            seg_sizes.push((*label, 1));
        }
    }
    for (label, size) in &seg_sizes {
        if *size < 10 && *label != SegmentLabel::Unknown {
            warnings.push(SegmentationWarning {
                kind: WarningKind::TinySegment,
                message: format!("Segment {:?} has only {} vertices", label, size),
            });
        }
    }

    // Check gingiva exists
    let has_gingiva = result
        .vertex_labels
        .iter()
        .any(|l| *l == SegmentLabel::Gingiva);
    if !has_gingiva {
        warnings.push(SegmentationWarning {
            kind: WarningKind::MissingTeeth,
            message: "No gingiva region detected".into(),
        });
    }

    warnings
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_adjacency(n_verts: usize, indices: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n_verts];
    for tri in indices {
        for k in 0..3 {
            let a = tri[k] as usize;
            let b = tri[(k + 1) % 3] as usize;
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }
    adj
}

fn compute_vertex_curvatures(vertices: &[Point3<f64>], indices: &[[u32; 3]]) -> Vec<f64> {
    let adjacency = build_adjacency(vertices.len(), indices);
    vertices
        .iter()
        .enumerate()
        .map(|(vi, p)| {
            let neighbors = &adjacency[vi];
            if neighbors.len() < 3 {
                return 0.0;
            }
            let mut laplacian = Vector3::zeros();
            for &ni in neighbors {
                laplacian += vertices[ni] - p;
            }
            laplacian /= neighbors.len() as f64;
            laplacian.norm()
        })
        .collect()
}

fn majority_vote_triangles(vertex_labels: &[SegmentLabel], indices: &[[u32; 3]]) -> Vec<SegmentLabel> {
    indices
        .iter()
        .map(|tri| {
            let a = vertex_labels[tri[0] as usize];
            let b = vertex_labels[tri[1] as usize];
            let c = vertex_labels[tri[2] as usize];
            if a == b || a == c {
                a
            } else if b == c {
                b
            } else {
                a
            }
        })
        .collect()
}

fn count_unique_segments(labels: &[SegmentLabel]) -> usize {
    let mut unique = Vec::new();
    for l in labels {
        if !unique.contains(l) {
            unique.push(*l);
        }
    }
    unique.len()
}

fn next_fdi(current: u8) -> u8 {
    match current {
        18 => 21,
        28 => 31,
        38 => 41,
        48 => 11,
        _ => current + 1,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_mesh() -> (Vec<Point3<f64>>, Vec<Vector3<f64>>, Vec<[u32; 3]>) {
        // Simple quad mesh (4 triangles)
        let verts = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.5, 0.5, 1.0), // elevated center
        ];
        let normals = vec![
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        let indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
        (verts, normals, indices)
    }

    #[test]
    fn curvature_produces_labels() {
        let (v, n, i) = make_test_mesh();
        let result = segment_by_curvature(&v, &n, &i, 0.01);
        assert_eq!(result.vertex_labels.len(), v.len());
        assert_eq!(result.triangle_labels.len(), i.len());
    }

    #[test]
    fn kmeans_produces_labels() {
        let (v, n, i) = make_test_mesh();
        let result = segment_by_kmeans(&v, &n, &i, 0.01);
        assert_eq!(result.vertex_labels.len(), v.len());
    }

    #[test]
    fn watershed_produces_labels() {
        let (v, _n, i) = make_test_mesh();
        let result = segment_by_watershed(&v, &i, 0.01);
        assert_eq!(result.vertex_labels.len(), v.len());
    }

    #[test]
    fn combined_produces_labels() {
        let (v, n, i) = make_test_mesh();
        let result = segment_combined(&v, &n, &i, 0.01);
        assert_eq!(result.vertex_labels.len(), v.len());
    }

    #[test]
    fn gingiva_detection() {
        let (v, _n, i) = make_test_mesh();
        let (gingiva, tooth) = detect_gingiva(&v, &i, 0.01);
        assert_eq!(gingiva.len() + tooth.len(), i.len());
    }

    #[test]
    fn validation_empty() {
        let result = SegmentationResult {
            vertex_labels: vec![SegmentLabel::Unknown; 5],
            triangle_labels: vec![SegmentLabel::Unknown; 4],
            segment_count: 0,
        };
        let warnings = validate_segmentation(&result);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn merge_segments_works() {
        let mut result = SegmentationResult {
            vertex_labels: vec![
                SegmentLabel::Tooth(11),
                SegmentLabel::Tooth(12),
                SegmentLabel::Tooth(11),
            ],
            triangle_labels: vec![SegmentLabel::Tooth(11)],
            segment_count: 2,
        };
        merge_segments(&mut result, SegmentLabel::Tooth(11), SegmentLabel::Tooth(12));
        assert!(result.vertex_labels.iter().all(|l| *l == SegmentLabel::Tooth(11)));
    }

    #[test]
    fn next_fdi_wraps() {
        assert_eq!(next_fdi(18), 21);
        assert_eq!(next_fdi(28), 31);
        assert_eq!(next_fdi(38), 41);
        assert_eq!(next_fdi(48), 11);
        assert_eq!(next_fdi(11), 12);
    }
}

// ---------------------------------------------------------------------------
// S102: Notation-aware helpers
// ---------------------------------------------------------------------------

use crate::notation::{ToothId, NotationSystem};

/// Convert tooth segments to notation-aware labels.
/// Returns a map from segment index → ToothId for all Tooth() labels.
pub fn segment_to_tooth_ids(result: &SegmentationResult) -> hashbrown::HashMap<usize, ToothId> {
    let mut map = hashbrown::HashMap::new();
    for (i, label) in result.vertex_labels.iter().enumerate() {
        if let SegmentLabel::Tooth(fdi) = label {
            if let Some(tid) = ToothId::from_fdi(*fdi) {
                map.insert(i, tid);
            }
        }
    }
    map
}

/// Generate a per-vertex Universal number label from FDI-based segmentation.
pub fn segment_labels_to_universal(result: &SegmentationResult) -> Vec<Option<u8>> {
    result
        .vertex_labels
        .iter()
        .map(|label| match label {
            SegmentLabel::Tooth(fdi) => crate::notation::fdi_to_universal(*fdi),
            _ => None,
        })
        .collect()
}

/// Relabel a segmentation from one notation system to another (FDI↔Universal).
pub fn relabel_segments(
    result: &mut SegmentationResult,
    target: NotationSystem,
) {
    match target {
        NotationSystem::Universal => {
            for label in result.vertex_labels.iter_mut().chain(result.triangle_labels.iter_mut()) {
                if let SegmentLabel::Tooth(fdi) = *label {
                    if let Some(uni) = crate::notation::fdi_to_universal(fdi) {
                        *label = SegmentLabel::Tooth(uni);
                    }
                }
            }
        }
        NotationSystem::Fdi => {
            for label in result.vertex_labels.iter_mut().chain(result.triangle_labels.iter_mut()) {
                if let SegmentLabel::Tooth(uni) = *label {
                    if let Some(fdi) = crate::notation::universal_to_fdi(uni) {
                        *label = SegmentLabel::Tooth(fdi);
                    }
                }
            }
        }
        NotationSystem::Palmer => {} // Palmer requires quadrant context; skip for now
    }
}

#[cfg(test)]
mod notation_tests {
    use super::*;

    #[test]
    fn segment_to_tooth_ids_maps() {
        let result = SegmentationResult {
            vertex_labels: vec![SegmentLabel::Tooth(11), SegmentLabel::Gingiva, SegmentLabel::Tooth(21)],
            triangle_labels: vec![],
            segment_count: 2,
        };
        let map = segment_to_tooth_ids(&result);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&0));
        assert!(map.contains_key(&2));
    }

    #[test]
    fn universal_labels_conversion() {
        let result = SegmentationResult {
            vertex_labels: vec![SegmentLabel::Tooth(11), SegmentLabel::Gingiva],
            triangle_labels: vec![],
            segment_count: 1,
        };
        let uni = segment_labels_to_universal(&result);
        assert_eq!(uni.len(), 2);
        assert!(uni[0].is_some()); // FDI 11 → Universal 8
        assert!(uni[1].is_none());
    }

    #[test]
    fn relabel_fdi_to_universal() {
        let mut result = SegmentationResult {
            vertex_labels: vec![SegmentLabel::Tooth(11)],
            triangle_labels: vec![SegmentLabel::Tooth(11)],
            segment_count: 1,
        };
        relabel_segments(&mut result, NotationSystem::Universal);
        // FDI 11 → Universal 8
        assert_eq!(result.vertex_labels[0], SegmentLabel::Tooth(8));
        assert_eq!(result.triangle_labels[0], SegmentLabel::Tooth(8));
    }
}
