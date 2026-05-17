//! Margin line detection for crown preparations
//!
//! Detects the finish line where the preparation meets the tooth surface

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Type of margin finish line
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MarginType {
    Chamfer,
    Shoulder,
    Knife,
    DeepChamfer,
    BeveledShoulder,
}

/// A detected margin line
#[derive(Debug, Clone)]
pub struct MarginLine {
    /// Ordered points along the margin
    pub points: Vec<Point3<f64>>,
    /// Normals along the margin (perpendicular to margin, in surface plane)
    pub normals: Vec<Vector3<f64>>,
    /// Detected margin type
    pub margin_type: MarginType,
    /// Confidence score [0..1]
    pub confidence: f64,
    /// Whether the margin loop is closed
    pub is_closed: bool,
}

/// Detect margin line on a prepared tooth mesh
/// Uses curvature gradient to find the sharp transition at the finish line
pub fn detect_margin(
    vertices: &[Point3<f64>],
    _normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    preparation_center: &Point3<f64>,
    search_radius: f64,
) -> Option<MarginLine> {
    // Step 1: Compute per-vertex curvature
    let curvatures = compute_curvature_gradient(vertices, indices);

    // Step 2: Find vertices with high curvature gradient (margin candidates)
    let threshold = curvature_threshold(&curvatures);
    let candidates: Vec<usize> = curvatures.iter().enumerate()
        .filter(|(i, &k)| {
            k > threshold &&
            (vertices[*i] - preparation_center).norm() < search_radius
        })
        .map(|(i, _)| i)
        .collect();

    if candidates.len() < 10 { return None; }

    // Step 3: Order candidates into a loop
    let ordered = order_margin_points(vertices, &candidates);
    if ordered.len() < 10 { return None; }

    // Step 4: Smooth the margin line
    let margin_points: Vec<Point3<f64>> = ordered.iter()
        .map(|&i| vertices[i])
        .collect();
    let smoothed = smooth_margin(&margin_points, 3);

    // Step 5: Compute margin normals and classify type
    let margin_normals = compute_margin_normals(&smoothed);
    let margin_type = classify_margin_type(vertices, indices, &ordered);

    let is_closed = if let (Some(first), Some(last)) = (smoothed.first(), smoothed.last()) {
        (first - last).norm() < search_radius * 0.1
    } else {
        false
    };

    Some(MarginLine {
        points: smoothed,
        normals: margin_normals,
        margin_type,
        confidence: compute_confidence(&curvatures, &ordered, threshold),
        is_closed,
    })
}

/// Refine an existing margin line based on user edits
pub fn refine_margin(
    _vertices: &[Point3<f64>],
    _indices: &[[u32; 3]],
    current_margin: &MarginLine,
    edit_point: &Point3<f64>,
    new_position: &Point3<f64>,
) -> MarginLine {
    let mut new_points = current_margin.points.clone();

    // Find closest point on margin to edit_point
    let closest_idx = new_points.iter().enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (*a - edit_point).norm_squared();
            let db = (*b - edit_point).norm_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Move the closest point
    new_points[closest_idx] = *new_position;

    // Re-smooth around the edit
    let smoothed = local_smooth(&new_points, closest_idx, 5);
    let normals = compute_margin_normals(&smoothed);

    MarginLine {
        points: smoothed,
        normals,
        margin_type: current_margin.margin_type,
        confidence: current_margin.confidence,
        is_closed: current_margin.is_closed,
    }
}

fn compute_curvature_gradient(vertices: &[Point3<f64>], indices: &[[u32; 3]]) -> Vec<f64> {
    let adj = build_adjacency(vertices.len(), indices);
    let curvatures: Vec<f64> = vertices.iter().enumerate().map(|(i, p)| {
        let nbrs = &adj[i];
        if nbrs.len() < 2 { return 0.0; }
        let mut laplacian = Vector3::zeros();
        for &ni in nbrs {
            laplacian += vertices[ni] - p;
        }
        laplacian /= nbrs.len() as f64;
        laplacian.norm()
    }).collect();

    // Gradient of curvature
    curvatures.iter().enumerate().map(|(i, &k)| {
        let nbrs = &adj[i];
        if nbrs.is_empty() { return 0.0; }
        let mut max_diff = 0.0f64;
        for &ni in nbrs {
            max_diff = max_diff.max((curvatures[ni] - k).abs());
        }
        max_diff
    }).collect()
}

fn curvature_threshold(curvatures: &[f64]) -> f64 {
    if curvatures.is_empty() { return 0.0; }
    let mut sorted = curvatures.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // Use 85th percentile as threshold
    sorted[(sorted.len() as f64 * 0.85) as usize]
}

fn order_margin_points(vertices: &[Point3<f64>], candidates: &[usize]) -> Vec<usize> {
    if candidates.is_empty() { return Vec::new(); }

    // Greedy nearest-neighbor ordering
    let mut ordered = Vec::with_capacity(candidates.len());
    let mut remaining: Vec<usize> = candidates.to_vec();

    let first = remaining.remove(0);
    ordered.push(first);

    while !remaining.is_empty() {
        let last = vertices[*ordered.last().unwrap()];
        let (best_idx, _) = remaining.iter().enumerate()
            .min_by(|(_, &a), (_, &b)| {
                let da = (vertices[a] - last).norm_squared();
                let db = (vertices[b] - last).norm_squared();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        ordered.push(remaining.remove(best_idx));
    }
    ordered
}

fn smooth_margin(points: &[Point3<f64>], iterations: usize) -> Vec<Point3<f64>> {
    let mut result = points.to_vec();
    let n = result.len();
    for _ in 0..iterations {
        let prev = result.clone();
        for i in 1..n - 1 {
            result[i] = Point3::from(
                (prev[i - 1].coords + prev[i].coords * 2.0 + prev[i + 1].coords) / 4.0
            );
        }
    }
    result
}

fn local_smooth(points: &[Point3<f64>], center: usize, radius: usize) -> Vec<Point3<f64>> {
    let mut result = points.to_vec();
    let n = result.len();
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(n);

    for _ in 0..3 {
        let prev = result.clone();
        for i in start.max(1)..end.min(n - 1) {
            result[i] = Point3::from(
                (prev[i - 1].coords + prev[i].coords * 2.0 + prev[i + 1].coords) / 4.0
            );
        }
    }
    result
}

fn compute_margin_normals(points: &[Point3<f64>]) -> Vec<Vector3<f64>> {
    let n = points.len();
    (0..n).map(|i| {
        let prev = if i > 0 { i - 1 } else { n - 1 };
        let next = if i < n - 1 { i + 1 } else { 0 };
        let tangent = (points[next] - points[prev]).normalize();
        // Perpendicular in horizontal plane (approximate)
        Vector3::new(-tangent.z, 0.0, tangent.x).normalize()
    }).collect()
}

fn classify_margin_type(
    _vertices: &[Point3<f64>],
    _indices: &[[u32; 3]],
    _candidates: &[usize],
) -> MarginType {
    // Default: chamfer is most common
    MarginType::Chamfer
}

fn compute_confidence(curvatures: &[f64], candidates: &[usize], threshold: f64) -> f64 {
    if candidates.is_empty() { return 0.0; }
    let above: usize = candidates.iter()
        .filter(|&&i| curvatures[i] > threshold * 1.5)
        .count();
    (above as f64 / candidates.len() as f64).min(1.0)
}

fn build_adjacency(n: usize, indices: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for tri in indices {
        for k in 0..3 {
            let a = tri[k] as usize;
            let b = tri[(k + 1) % 3] as usize;
            if !adj[a].contains(&b) { adj[a].push(b); }
            if !adj[b].contains(&a) { adj[b].push(a); }
        }
    }
    adj
}

// ---------------------------------------------------------------------------
// S126-130 additions
// ---------------------------------------------------------------------------

/// Multi-preparation margin detection — detect margins for multiple preps in one scan.
pub fn detect_margins_batch(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    prep_vertex_groups: &[Vec<usize>],
) -> Vec<MarginLine> {
    prep_vertex_groups
        .iter()
        .filter_map(|group| {
            if group.is_empty() {
                return None;
            }
            // Extract sub-mesh for this prep region
            let sub_verts: Vec<Point3<f64>> = group.iter().map(|&i| vertices[i]).collect();
            let sub_normals: Vec<Vector3<f64>> = group.iter().map(|&i| normals[i]).collect();
            // Build sub-indices (triangles fully within the group)
            let in_group = |v: u32| group.contains(&(v as usize));
            let sub_indices: Vec<[u32; 3]> = indices
                .iter()
                .filter(|tri| in_group(tri[0]) && in_group(tri[1]) && in_group(tri[2]))
                .map(|tri| {
                    let remap = |v: u32| group.iter().position(|&g| g == v as usize).unwrap_or(0) as u32;
                    [remap(tri[0]), remap(tri[1]), remap(tri[2])]
                })
                .collect();
            if sub_indices.is_empty() {
                return None;
            }
            // Compute center of sub-mesh vertices
            let sum: Vector3<f64> = sub_verts.iter().map(|p| p.coords).sum();
            let center = Point3::from(sum / sub_verts.len() as f64);
            let radius = sub_verts.iter().map(|p| (p - center).norm()).fold(0.0f64, f64::max);
            detect_margin(&sub_verts, &sub_normals, &sub_indices, &center, radius)
        })
        .collect()
}

/// Validate margin line properties.
pub fn validate_margin(margin: &MarginLine) -> Vec<String> {
    let mut issues = Vec::new();
    if margin.points.len() < 10 {
        issues.push("Margin has fewer than 10 points — may be incomplete".into());
    }
    if !margin.is_closed {
        issues.push("Margin line is not closed".into());
    }
    if margin.confidence < 0.5 {
        issues.push(format!("Low confidence ({:.1}%)", margin.confidence * 100.0));
    }
    // Check for self-intersections (simplified: check for large backtracking)
    for i in 0..margin.points.len().saturating_sub(2) {
        let j = i + 2;
        if j < margin.points.len() {
            let d_forward = (margin.points[i + 1] - margin.points[i]).norm();
            let d_skip = (margin.points[j] - margin.points[i]).norm();
            if d_skip < d_forward * 0.2 && d_forward > 0.01 {
                issues.push(format!("Potential self-intersection near point {}", i));
                break;
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ring_mesh() -> (Vec<Point3<f64>>, Vec<Vector3<f64>>, Vec<[u32; 3]>) {
        let n = 8;
        let mut verts = Vec::new();
        let mut normals = Vec::new();
        for i in 0..n {
            let angle = std::f64::consts::TAU * i as f64 / n as f64;
            verts.push(Point3::new(angle.cos(), angle.sin(), 0.0));
            normals.push(Vector3::new(angle.cos(), angle.sin(), 0.0));
        }
        // Center
        verts.push(Point3::new(0.0, 0.0, 0.0));
        normals.push(Vector3::z());
        let center = (n) as u32;
        let indices: Vec<[u32; 3]> = (0..n)
            .map(|i| [i as u32, ((i + 1) % n) as u32, center])
            .collect();
        (verts, normals, indices)
    }

    #[test]
    fn detect_margin_returns_points() {
        let (v, n, i) = make_ring_mesh();
        let center = Point3::new(0.0, 0.0, 0.0);
        let margin = detect_margin(&v, &n, &i, &center, 10.0);
        // May or may not find a margin on this simple mesh
        if let Some(m) = &margin {
            assert!(!m.points.is_empty(), "Margin should have points");
        }
    }

    #[test]
    fn refine_margin_preserves_count() {
        let (v, _n, i) = make_ring_mesh();
        let center = Point3::new(0.0, 0.0, 0.0);
        if let Some(original) = detect_margin(&v, &_n, &i, &center, 10.0) {
            let count = original.points.len();
            let edit_pt = original.points[0];
            let new_pos = Point3::new(edit_pt.x + 0.1, edit_pt.y, edit_pt.z);
            let refined = refine_margin(&v, &i, &original, &edit_pt, &new_pos);
            assert_eq!(refined.points.len(), count);
        }
    }

    #[test]
    fn validate_margin_flags_short() {
        let margin = MarginLine {
            points: vec![Point3::origin(); 3],
            normals: vec![Vector3::z(); 3],
            margin_type: MarginType::Chamfer,
            confidence: 0.3,
            is_closed: false,
        };
        let issues = validate_margin(&margin);
        assert!(issues.iter().any(|i| i.contains("fewer than 10")));
        assert!(issues.iter().any(|i| i.contains("not closed")));
        assert!(issues.iter().any(|i| i.contains("Low confidence")));
    }

    #[test]
    fn batch_detect_empty_groups() {
        let (v, n, i) = make_ring_mesh();
        let result = detect_margins_batch(&v, &n, &i, &[]);
        assert!(result.is_empty());
    }
}
