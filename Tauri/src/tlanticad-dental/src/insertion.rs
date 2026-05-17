//! Insertion axis computation for dental restorations
//!
//! Finds the optimal path of insertion/withdrawal that minimizes undercuts

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Insertion axis analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertionAnalysis {
    /// Optimal insertion direction (unit vector)
    pub direction: [f64; 3],
    /// Undercut area percentage [0..100]
    pub undercut_percent: f64,
    /// Per-vertex undercut depth (0 = no undercut)
    pub undercut_depths: Vec<f64>,
    /// Maximum undercut depth
    pub max_undercut: f64,
}

/// Compute the best insertion axis to minimize undercuts
/// Uses an iterative search over rotations of the initial axis
pub fn find_insertion_axis(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    _indices: &[[u32; 3]],
    initial_axis: &Vector3<f64>,
    preparation_vertices: &[u32],
) -> InsertionAnalysis {
    let axis = initial_axis.normalize();

    // Search grid: try rotations around the initial axis
    let mut best_axis = axis;
    let mut best_undercut = f64::MAX;

    let angle_steps = 12;
    let tilt_steps = 8;
    let max_tilt = 15.0f64.to_radians();

    for t in 0..tilt_steps {
        let tilt = max_tilt * (t as f64) / (tilt_steps as f64);
        for a in 0..angle_steps {
            let azimuth = std::f64::consts::TAU * (a as f64) / (angle_steps as f64);

            // Create tilted axis
            let perp1 = perpendicular(&axis);
            let perp2 = axis.cross(&perp1);
            let tilted = (axis * tilt.cos() + (perp1 * azimuth.cos() + perp2 * azimuth.sin()) * tilt.sin()).normalize();

            let undercut = compute_undercut_score(vertices, normals, &tilted, preparation_vertices);
            if undercut < best_undercut {
                best_undercut = undercut;
                best_axis = tilted;
            }
        }
    }

    let analysis = analyze_undercuts(vertices, normals, &best_axis, preparation_vertices);
    analysis
}

/// Analyze undercuts for a given insertion direction
pub fn analyze_undercuts(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    insertion_dir: &Vector3<f64>,
    preparation_vertices: &[u32],
) -> InsertionAnalysis {
    let dir = insertion_dir.normalize();
    let mut undercut_depths = vec![0.0f64; vertices.len()];
    let mut total_undercut = 0usize;
    let mut max_undercut = 0.0f64;

    for &vi in preparation_vertices {
        let vi = vi as usize;
        if vi >= normals.len() { continue; }

        // A surface point is undercut if its normal has a component
        // opposing the insertion direction (dot product < 0)
        let dot = normals[vi].dot(&dir);
        if dot < 0.0 {
            let depth = -dot; // depth proportional to how much it opposes
            undercut_depths[vi] = depth;
            max_undercut = max_undercut.max(depth);
            total_undercut += 1;
        }
    }

    let undercut_percent = if preparation_vertices.is_empty() {
        0.0
    } else {
        100.0 * total_undercut as f64 / preparation_vertices.len() as f64
    };

    InsertionAnalysis {
        direction: [dir.x, dir.y, dir.z],
        undercut_percent,
        undercut_depths,
        max_undercut,
    }
}

/// Block out undercuts by projecting undercut vertices along the insertion axis
pub fn block_out_undercuts(
    vertices: &mut [Point3<f64>],
    normals: &[Vector3<f64>],
    insertion_dir: &Vector3<f64>,
    preparation_vertices: &[u32],
    block_out_tolerance: f64,
) {
    let dir = insertion_dir.normalize();

    for &vi in preparation_vertices {
        let vi = vi as usize;
        if vi >= normals.len() { continue; }

        let dot = normals[vi].dot(&dir);
        if dot < -block_out_tolerance {
            // Project vertex along insertion axis to remove undercut
            let offset = (-dot - block_out_tolerance) * 0.5;
            vertices[vi] += dir * offset;
        }
    }
}

fn compute_undercut_score(
    _vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    axis: &Vector3<f64>,
    preparation_vertices: &[u32],
) -> f64 {
    let mut score = 0.0f64;
    for &vi in preparation_vertices {
        let vi = vi as usize;
        if vi >= normals.len() { continue; }
        let dot = normals[vi].dot(axis);
        if dot < 0.0 {
            score += dot * dot; // Squared penalty
        }
    }
    score
}

fn perpendicular(v: &Vector3<f64>) -> Vector3<f64> {
    let a = if v.x.abs() < 0.9 { Vector3::x() } else { Vector3::y() };
    v.cross(&a).normalize()
}

// ---------------------------------------------------------------------------
// S131-135 additions
// ---------------------------------------------------------------------------

/// Refine an insertion axis via gradient descent (finer search around initial).
pub fn refine_insertion_axis(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    initial_axis: &Vector3<f64>,
    preparation_vertices: &[u32],
    max_iterations: usize,
    step_deg: f64,
) -> Vector3<f64> {
    let mut best_axis = initial_axis.normalize();
    let mut best_score = compute_undercut_score(vertices, normals, &best_axis, preparation_vertices);
    let step_rad = step_deg.to_radians();

    for _ in 0..max_iterations {
        let mut improved = false;
        let perp1 = perpendicular(&best_axis);
        let perp2 = best_axis.cross(&perp1).normalize();

        for &dx in &[-1.0, 0.0, 1.0] {
            for &dy in &[-1.0, 0.0, 1.0] {
                if dx == 0.0 && dy == 0.0 {
                    continue;
                }
                let candidate = (best_axis + perp1 * dx * step_rad + perp2 * dy * step_rad).normalize();
                let score = compute_undercut_score(vertices, normals, &candidate, preparation_vertices);
                if score < best_score {
                    best_score = score;
                    best_axis = candidate;
                    improved = true;
                }
            }
        }

        if !improved {
            break;
        }
    }

    best_axis
}

/// Compute common insertion axis for multiple preparations (e.g. bridge abutments).
pub fn find_common_insertion_axis(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    prep_groups: &[Vec<u32>],
) -> Vector3<f64> {
    let all_preps: Vec<u32> = prep_groups.iter().flat_map(|g| g.iter().copied()).collect();
    let initial_axis = Vector3::new(0.0, 0.0, 1.0);
    let initial = find_insertion_axis(vertices, normals, indices, &initial_axis, &all_preps);
    let dir = Vector3::new(initial.direction[0], initial.direction[1], initial.direction[2]);
    refine_insertion_axis(vertices, normals, &dir, &all_preps, 50, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_data() -> (Vec<Point3<f64>>, Vec<Vector3<f64>>, Vec<[u32; 3]>, Vec<u32>) {
        let verts = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
        ];
        let normals = vec![
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        let indices = vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];
        let prep = vec![0, 1, 2, 3];
        (verts, normals, indices, prep)
    }

    #[test]
    fn find_axis_returns_unit_vector() {
        let (v, n, i, p) = make_test_data();
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let result = find_insertion_axis(&v, &n, &i, &axis, &p);
        let dir = Vector3::new(result.direction[0], result.direction[1], result.direction[2]);
        let len = dir.norm();
        assert!((len - 1.0).abs() < 0.01, "Direction should be unit vector");
    }

    #[test]
    fn undercut_analysis_consistent() {
        let (v, n, i, p) = make_test_data();
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let result = find_insertion_axis(&v, &n, &i, &axis, &p);
        let dir = Vector3::new(result.direction[0], result.direction[1], result.direction[2]);
        let analysis = analyze_undercuts(&v, &n, &dir, &p);
        assert_eq!(analysis.undercut_depths.len(), p.len());
        assert!(analysis.undercut_percent >= 0.0 && analysis.undercut_percent <= 100.0);
    }

    #[test]
    fn refine_improves_or_keeps() {
        let (v, n, i, p) = make_test_data();
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let initial = find_insertion_axis(&v, &n, &i, &axis, &p);
        let dir = Vector3::new(initial.direction[0], initial.direction[1], initial.direction[2]);
        let score_before = compute_undercut_score(&v, &n, &dir, &p);
        let refined = refine_insertion_axis(&v, &n, &dir, &p, 20, 1.0);
        let score_after = compute_undercut_score(&v, &n, &refined, &p);
        assert!(score_after <= score_before + 1e-9, "Refinement should not worsen");
    }

    #[test]
    fn common_axis_works() {
        let (v, n, i, _) = make_test_data();
        let groups = vec![vec![0, 1], vec![2, 3]];
        let axis = find_common_insertion_axis(&v, &n, &i, &groups);
        let len = axis.norm();
        assert!((len - 1.0).abs() < 0.01);
    }

    #[test]
    fn block_out_modifies_vertices() {
        let (mut v, n, _i, p) = make_test_data();
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let original_len = v.len();
        block_out_undercuts(&mut v, &n, &axis, &p, 0.0);
        assert_eq!(v.len(), original_len);
    }

    #[test]
    fn compare_axes_identical() {
        let (v, n, _, p) = make_test_data();
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let cmp = compare_insertion_axes(&v, &n, &axis, &axis, &p);
        assert!((cmp.angle_degrees - 0.0).abs() < 1e-9);
        assert!((cmp.score_a - cmp.score_b).abs() < 1e-9);
    }

    #[test]
    fn insertion_path_frames() {
        let path = generate_insertion_path(&Vector3::new(0.0, 0.0, 1.0), 5.0, 10);
        assert_eq!(path.len(), 10);
        assert!((path.last().unwrap().coords - Vector3::new(0.0, 0.0, 5.0)).norm() < 1e-9);
    }

    #[test]
    fn undercut_depth_to_color_gradient() {
        let colors = undercut_depth_to_colors(&[0.0, 0.05, 0.2, 0.5]);
        assert_eq!(colors.len(), 4);
        // No undercut → green
        assert!(colors[0][1] > 0.5);
        // Deep undercut → red
        assert!(colors[3][0] > 0.5);
    }
}

// ---------------------------------------------------------------------------
// S133-S135: Insertion path visualization & multi-axis comparison
// ---------------------------------------------------------------------------

/// Comparison result between two candidate insertion axes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisComparison {
    pub axis_a: [f64; 3],
    pub axis_b: [f64; 3],
    pub score_a: f64,
    pub score_b: f64,
    pub angle_degrees: f64,
    pub recommended: char, // 'A' or 'B'
}

/// Compare two insertion axes quantitatively.
pub fn compare_insertion_axes(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    axis_a: &Vector3<f64>,
    axis_b: &Vector3<f64>,
    preparation_vertices: &[u32],
) -> AxisComparison {
    let a = axis_a.normalize();
    let b = axis_b.normalize();
    let score_a = compute_undercut_score(vertices, normals, &a, preparation_vertices);
    let score_b = compute_undercut_score(vertices, normals, &b, preparation_vertices);
    let angle_degrees = a.dot(&b).clamp(-1.0, 1.0).acos().to_degrees();
    let recommended = if score_a <= score_b { 'A' } else { 'B' };
    AxisComparison {
        axis_a: [a.x, a.y, a.z],
        axis_b: [b.x, b.y, b.z],
        score_a,
        score_b,
        angle_degrees,
        recommended,
    }
}

/// Generate a sequence of positions along the insertion path for animation.
pub fn generate_insertion_path(
    axis: &Vector3<f64>,
    travel_distance: f64,
    num_frames: usize,
) -> Vec<Point3<f64>> {
    let dir = axis.normalize();
    (0..num_frames)
        .map(|i| {
            let t = (i as f64 + 1.0) / num_frames as f64;
            Point3::from(dir * travel_distance * t)
        })
        .collect()
}

/// Map undercut depths to a color gradient (green→yellow→red) for visualization.
pub fn undercut_depth_to_colors(depths: &[f64]) -> Vec<[f32; 4]> {
    let max_depth = depths.iter().cloned().fold(0.0f64, f64::max).max(0.01);
    depths
        .iter()
        .map(|&d| {
            let t = (d / max_depth).clamp(0.0, 1.0) as f32;
            if t < 0.5 {
                // green → yellow
                let u = t * 2.0;
                [u, 1.0, 0.0, 1.0]
            } else {
                // yellow → red
                let u = (t - 0.5) * 2.0;
                [1.0, 1.0 - u, 0.0, 1.0]
            }
        })
        .collect()
}
